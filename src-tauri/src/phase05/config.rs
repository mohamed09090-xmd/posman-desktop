use rusqlite::{params, OptionalExtension, TransactionBehavior};

use super::{
    dto::{
        CompanyProfile, DocumentSequenceView, FiscalPeriodView, FiscalSetup,
        UpdateCompanyProfileRequest, UpdateDocumentSequenceRequest,
        UpdateFiscalSetupRequest,
    },
    error::{Phase05Error, Phase05Result},
    pricing::fiscal_periods,
    state::{audit, new_id, now_iso, trim_optional, trim_required, Phase05Service},
};

impl Phase05Service {
    pub fn get_company_profile(&self) -> Phase05Result<CompanyProfile> {
        let context = self.require_session(Some("company.view"))?;
        self.open()?.query_row(
            r#"
            SELECT c.id, c.code, c.legal_name, c.name_ar, c.name_fr,
                   c.activity_description, c.legal_form, c.trade_register_number,
                   c.tax_identifier, c.statistical_identifier, c.tax_article_number,
                   c.bank_rib, c.social_capital_minor, c.address_text, c.wilaya_code,
                   c.city, c.postal_code, c.phone, c.email,
                   s.default_margin_rate_scaled, s.below_cost_policy,
                   s.session_idle_timeout_minutes, c.row_version
            FROM companies c
            JOIN company_settings s ON s.company_id=c.id
            WHERE c.id=?1
            "#,
            [context.company_id],
            |row| {
                Ok(CompanyProfile {
                    id: row.get(0)?, code: row.get(1)?, legal_name: row.get(2)?,
                    name_ar: row.get(3)?, name_fr: row.get(4)?,
                    activity_description: row.get(5)?, legal_form: row.get(6)?,
                    trade_register_number: row.get(7)?, tax_identifier: row.get(8)?,
                    statistical_identifier: row.get(9)?, tax_article_number: row.get(10)?,
                    bank_rib: row.get(11)?, social_capital_minor: row.get(12)?,
                    address_text: row.get(13)?, wilaya_code: row.get(14)?, city: row.get(15)?,
                    postal_code: row.get(16)?, phone: row.get(17)?, email: row.get(18)?,
                    default_margin_rate_scaled: row.get(19)?, below_cost_policy: row.get(20)?,
                    session_idle_timeout_minutes: row.get(21)?, row_version: row.get(22)?,
                })
            },
        ).map_err(Phase05Error::from)
    }

    pub fn update_company_profile(&self, request: UpdateCompanyProfileRequest) -> Phase05Result<CompanyProfile> {
        if !(0..=1_000_000).contains(&request.default_margin_rate_scaled)
            || !matches!(request.below_cost_policy.as_str(), "BLOCK" | "ADMIN_OVERRIDE" | "WARNING_ONLY")
            || !(5..=120).contains(&request.session_idle_timeout_minutes)
            || request.social_capital_minor.is_some_and(|value| value < 0)
        {
            return Err(Phase05Error::invalid("companyProfile"));
        }
        let context = self.require_session(Some("company.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            r#"
            UPDATE companies SET
                legal_name=?1, name_ar=?2, name_fr=?3, activity_description=?4,
                legal_form=?5, trade_register_number=?6, tax_identifier=?7,
                statistical_identifier=?8, tax_article_number=?9, bank_rib=?10,
                social_capital_minor=?11, address_text=?12, wilaya_code=?13,
                city=?14, postal_code=?15, phone=?16, email=?17,
                updated_at=?18, updated_by=?19, row_version=row_version+1
            WHERE id=?20 AND row_version=?21
            "#,
            params![
                trim_required(&request.legal_name, "legalName")?,
                trim_required(&request.name_ar, "nameAr")?,
                trim_optional(request.name_fr.as_deref()),
                trim_optional(request.activity_description.as_deref()),
                trim_optional(request.legal_form.as_deref()),
                trim_optional(request.trade_register_number.as_deref()),
                trim_optional(request.tax_identifier.as_deref()),
                trim_optional(request.statistical_identifier.as_deref()),
                trim_optional(request.tax_article_number.as_deref()),
                trim_optional(request.bank_rib.as_deref()), request.social_capital_minor,
                trim_required(&request.address_text, "addressText")?,
                trim_required(&request.wilaya_code, "wilayaCode")?,
                trim_optional(request.city.as_deref()), trim_optional(request.postal_code.as_deref()),
                trim_required(&request.phone, "phone")?, trim_optional(request.email.as_deref()),
                now_iso()?, context.user_id, context.company_id, request.row_version
            ],
        )?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        transaction.execute(
            r#"
            UPDATE company_settings SET default_margin_rate_scaled=?1,
                session_idle_timeout_minutes=?2, updated_at=?3, updated_by=?4,
                row_version=row_version+1 WHERE company_id=?5
            "#,
            params![request.default_margin_rate_scaled, request.below_cost_policy,
                request.session_idle_timeout_minutes, now_iso()?, context.user_id, context.company_id],
        )?;
        audit(&transaction, &context, "company.update", "companies", &context.company_id, None)?;
        transaction.commit()?;
        self.get_company_profile()
    }

    pub fn get_fiscal_setup(&self) -> Phase05Result<FiscalSetup> {
        let context = self.require_session(Some("settings.manage"))?;
        let connection = self.open()?;
        let fiscal = connection.query_row(
            "SELECT id, code, starts_on, ends_on, row_version FROM fiscal_years WHERE company_id=?1 ORDER BY starts_on LIMIT 1",
            [context.company_id.as_str()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?)),
        )?;
        let mut statement = connection.prepare(
            "SELECT period_number, name, starts_on, ends_on, status FROM fiscal_periods WHERE company_id=?1 AND fiscal_year_id=?2 ORDER BY period_number",
        )?;
        let periods = statement.query_map(params![context.company_id, fiscal.0], |row| {
            Ok(FiscalPeriodView { period_number: row.get(0)?, name: row.get(1)?, starts_on: row.get(2)?, ends_on: row.get(3)?, status: row.get(4)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        let in_use = operational_data_exists(&connection, &context.company_id)?;
        Ok(FiscalSetup { fiscal_year_id: fiscal.0, code: fiscal.1, starts_on: fiscal.2, ends_on: fiscal.3, periods, row_version: fiscal.4, in_use })
    }

    pub fn update_fiscal_setup(&self, request: UpdateFiscalSetupRequest) -> Phase05Result<FiscalSetup> {
        let periods = fiscal_periods(&request.starts_on, &request.ends_on)?;
        let context = self.require_session(Some("settings.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if operational_data_exists(&transaction, &context.company_id)? {
            return Err(Phase05Error::new("FISCAL_YEAR_ALREADY_IN_USE", "The fiscal year cannot be changed after operational data exists."));
        }
        let fiscal_year_id: String = transaction.query_row(
            "SELECT id FROM fiscal_years WHERE company_id=?1 ORDER BY starts_on LIMIT 1",
            [context.company_id.as_str()], |row| row.get(0))?;
        let changed = transaction.execute(
            r#"
            UPDATE fiscal_years SET code=?1, starts_on=?2, ends_on=?3,
                updated_at=?4, updated_by=?5, row_version=row_version+1
            WHERE id=?6 AND company_id=?7 AND row_version=?8
            "#,
            params![&request.starts_on[..4], request.starts_on, request.ends_on, now_iso()?, context.user_id, fiscal_year_id, context.company_id, request.row_version],
        )?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        transaction.execute("DELETE FROM fiscal_periods WHERE company_id=?1 AND fiscal_year_id=?2", params![context.company_id, fiscal_year_id])?;
        for (index, (starts_on, ends_on)) in periods.iter().enumerate() {
            let number = i64::try_from(index + 1).map_err(|_| Phase05Error::internal())?;
            transaction.execute(
                r#"
                INSERT INTO fiscal_periods (
                    id, company_id, fiscal_year_id, period_number, name, starts_on,
                    ends_on, status, created_at, created_by, updated_at, updated_by
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'OPEN', ?8, ?9, ?8, ?9)
                "#,
                params![new_id(), context.company_id, fiscal_year_id, number, format!("M{number:02}"), starts_on, ends_on, now_iso()?, context.user_id],
            )?;
        }
        audit(&transaction, &context, "settings.fiscal.update", "fiscal_years", &fiscal_year_id, None)?;
        transaction.commit()?;
        self.get_fiscal_setup()
    }

    pub fn list_document_sequences(&self) -> Phase05Result<Vec<DocumentSequenceView>> {
        let context = self.require_session(Some("settings.manage"))?;
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT ds.id, ds.document_type, ds.prefix, ds.next_number,
                   ds.padding_width, fy.code, ds.row_version
            FROM document_sequences ds JOIN fiscal_years fy ON fy.id=ds.fiscal_year_id
            WHERE ds.company_id=?1 AND fy.company_id=?1 ORDER BY ds.document_type
            "#,
        )?;
        let rows = statement.query_map([context.company_id], |row| {
            let prefix: String = row.get(2)?;
            let next_number: i64 = row.get(3)?;
            let padding: i64 = row.get(4)?;
            let year: String = row.get(5)?;
            Ok(DocumentSequenceView { id: row.get(0)?, document_type: row.get(1)?, preview: sequence_number(&prefix, &year, next_number, padding), prefix, next_number, padding_width: padding, row_version: row.get(6)? })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Phase05Error::from)
    }

    pub fn update_document_sequence(&self, request: UpdateDocumentSequenceRequest) -> Phase05Result<DocumentSequenceView> {
        if !(1..=12).contains(&request.padding_width) || request.next_number < 1 { return Err(Phase05Error::invalid("documentSequence")); }
        let prefix = trim_required(&request.prefix, "prefix")?;
        let context = self.require_session(Some("settings.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let current = transaction.query_row(
            "SELECT next_number, prefix, padding_width FROM document_sequences WHERE id=?1 AND company_id=?2",
            params![request.id, context.company_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
        ).optional()?.ok_or_else(|| Phase05Error::new("NOT_FOUND", "The sequence was not found."))?;
        if request.next_number < current.0 { return Err(Phase05Error::new("SEQUENCE_NUMBER_DECREASE_FORBIDDEN", "The next document number cannot be decreased.")); }
        if current.0 > 1 && (prefix != current.1 || request.padding_width != current.2) { return Err(Phase05Error::new("SEQUENCE_FORMAT_LOCKED", "The sequence format cannot change after the first allocation.")); }
        let changed = transaction.execute(
            r#"
            UPDATE document_sequences SET prefix=?1, next_number=?2, padding_width=?3,
                updated_at=?4, updated_by=?5, row_version=row_version+1
            WHERE id=?6 AND company_id=?7 AND row_version=?8
            "#,
            params![prefix, request.next_number, request.padding_width, now_iso()?, context.user_id, request.id, context.company_id, request.row_version],
        )?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        audit(&transaction, &context, "settings.sequence.update", "document_sequences", &request.id, None)?;
        transaction.commit()?;
        self.list_document_sequences()?.into_iter().find(|item| item.id == request.id).ok_or_else(|| Phase05Error::new("NOT_FOUND", "The sequence was not found."))
    }

    #[allow(dead_code)]
    pub(crate) fn allocate_document_number(&self, fiscal_year_id: &str, document_type: &str) -> Phase05Result<String> {
        let context = self.require_session(Some("settings.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let sequence = transaction.query_row(
            r#"
            SELECT ds.id, ds.prefix, ds.next_number, ds.padding_width, fy.code
            FROM document_sequences ds JOIN fiscal_years fy ON fy.id=ds.fiscal_year_id
            WHERE ds.company_id=?1 AND ds.fiscal_year_id=?2 AND ds.document_type=?3 AND fy.company_id=?1
            "#,
            params![context.company_id, fiscal_year_id, document_type],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?)),
        )?;
        let changed = transaction.execute(
            r#"
            UPDATE document_sequences SET next_number=next_number+1,
                updated_at=?1, updated_by=?2, row_version=row_version+1
            WHERE id=?3 AND company_id=?4 AND next_number=?5
            "#,
            params![now_iso()?, context.user_id, sequence.0, context.company_id, sequence.2],
        )?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        let allocated = sequence_number(&sequence.1, &sequence.4, sequence.2, sequence.3);
        transaction.commit()?;
        Ok(allocated)
    }
}

fn sequence_number(prefix: &str, year: &str, number: i64, padding: i64) -> String {
    let width = usize::try_from(padding).unwrap_or(6);
    format!("{prefix}-{year}-{number:0width$}")
}

fn operational_data_exists(connection: &rusqlite::Connection, company_id: &str) -> Phase05Result<bool> {
    let count: i64 = connection.query_row(
        r#"
        SELECT
          (SELECT COUNT(*) FROM commercial_documents WHERE company_id=?1) +
          (SELECT COUNT(*) FROM stock_movements WHERE company_id=?1) +
          (SELECT COUNT(*) FROM stock_reservations WHERE company_id=?1) +
          (SELECT COUNT(*) FROM inventory_counts WHERE company_id=?1) +
          (SELECT COUNT(*) FROM payments WHERE company_id=?1) +
          (SELECT COUNT(*) FROM journal_entries WHERE company_id=?1) +
          (SELECT COUNT(*) FROM posting_attempts WHERE company_id=?1)
        "#,
        [company_id], |row| row.get(0))?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbering_format_has_year_and_six_digit_default_padding() {
        assert_eq!(sequence_number("FAC", "2026", 1, 6), "FAC-2026-000001");
    }
}

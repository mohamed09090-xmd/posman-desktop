use rusqlite::{params, OptionalExtension};

use super::{
    dto::{
        CreatePartnerRequest, Page, PageRequest, PartnerAddressInput, PartnerAddressView,
        PartnerContactInput, PartnerContactView, PartnerView, SetActiveRequest,
        UpdatePartnerRequest,
    },
    error::{Phase05Error, Phase05Result},
    state::{audit, new_id, now_iso, trim_optional, trim_required, Phase05Service},
};

impl Phase05Service {
    pub fn list_partners(&self, request: PageRequest) -> Phase05Result<Page<PartnerView>> {
        let context = self.require_session(Some("partners.view"))?;
        let page = request.page.unwrap_or(1).max(1);
        let page_size = request.page_size.unwrap_or(25).clamp(1, 100);
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let search = request.search.unwrap_or_default().trim().to_lowercase();
        if search.chars().count() > 100 {
            return Err(Phase05Error::invalid("search"));
        }
        let pattern = format!("%{search}%");
        let include_inactive = i64::from(request.include_inactive.unwrap_or(false));
        let connection = self.open()?;
        let total: i64 = connection.query_row(
            r#"SELECT COUNT(*) FROM partners
               WHERE company_id=?1 AND (?2=1 OR is_active=1)
                 AND (?3='' OR lower(code) LIKE ?4 OR lower(legal_name) LIKE ?4
                      OR lower(display_name_ar) LIKE ?4 OR lower(COALESCE(display_name_fr,'')) LIKE ?4)"#,
            params![context.company_id, include_inactive, search, pattern],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            r#"SELECT id,code,legal_name,display_name_ar,display_name_fr,is_customer,is_supplier,is_active,row_version
               FROM partners
               WHERE company_id=?1 AND (?2=1 OR is_active=1)
                 AND (?3='' OR lower(code) LIKE ?4 OR lower(legal_name) LIKE ?4
                      OR lower(display_name_ar) LIKE ?4 OR lower(COALESCE(display_name_fr,'')) LIKE ?4)
               ORDER BY is_active DESC, code LIMIT ?5 OFFSET ?6"#,
        )?;
        let rows = statement.query_map(
            params![
                context.company_id,
                include_inactive,
                search,
                pattern,
                page_size,
                offset
            ],
            map_partner,
        )?;
        Ok(Page {
            items: rows.collect::<Result<Vec<_>, _>>()?,
            page,
            page_size,
            total: u64::try_from(total).map_err(|_| Phase05Error::internal())?,
        })
    }

    pub fn create_partner(&self, request: CreatePartnerRequest) -> Phase05Result<PartnerView> {
        validate_partner_roles(request.is_customer, request.is_supplier)?;
        let context = self.require_session(Some("partners.manage"))?;
        let id = new_id();
        let timestamp = now_iso()?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"INSERT INTO partners (
                id,company_id,code,legal_name,display_name_ar,display_name_fr,is_customer,is_supplier,
                tax_identifier,legal_form,activity_description,trade_register_number,statistical_identifier,
                tax_article_number,payment_term_id,created_at,created_by,updated_at,updated_by
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?16,?17)"#,
            params![
                id, context.company_id, trim_required(&request.code, "code")?,
                trim_required(&request.legal_name, "legalName")?,
                trim_required(&request.display_name_ar, "displayNameAr")?,
                trim_optional(request.display_name_fr.as_deref()),
                i64::from(request.is_customer), i64::from(request.is_supplier),
                trim_optional(request.tax_identifier.as_deref()), trim_optional(request.legal_form.as_deref()),
                trim_optional(request.activity_description.as_deref()), trim_optional(request.trade_register_number.as_deref()),
                trim_optional(request.statistical_identifier.as_deref()), trim_optional(request.tax_article_number.as_deref()),
                request.payment_term_id, timestamp, context.user_id
            ],
        )?;
        audit(
            &transaction,
            &context,
            "partners.create",
            "partners",
            &id,
            None,
        )?;
        transaction.commit()?;
        self.get_partner(&id)
    }

    pub fn update_partner(&self, request: UpdatePartnerRequest) -> Phase05Result<PartnerView> {
        validate_partner_roles(request.is_customer, request.is_supplier)?;
        let context = self.require_session(Some("partners.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            r#"UPDATE partners SET code=?1,legal_name=?2,display_name_ar=?3,display_name_fr=?4,
               is_customer=?5,is_supplier=?6,tax_identifier=?7,legal_form=?8,activity_description=?9,
               trade_register_number=?10,statistical_identifier=?11,tax_article_number=?12,payment_term_id=?13,
               updated_at=?14,updated_by=?15,row_version=row_version+1
               WHERE id=?16 AND company_id=?17 AND row_version=?18"#,
            params![
                trim_required(&request.code, "code")?, trim_required(&request.legal_name, "legalName")?,
                trim_required(&request.display_name_ar, "displayNameAr")?, trim_optional(request.display_name_fr.as_deref()),
                i64::from(request.is_customer), i64::from(request.is_supplier), trim_optional(request.tax_identifier.as_deref()),
                trim_optional(request.legal_form.as_deref()), trim_optional(request.activity_description.as_deref()),
                trim_optional(request.trade_register_number.as_deref()), trim_optional(request.statistical_identifier.as_deref()),
                trim_optional(request.tax_article_number.as_deref()), request.payment_term_id, now_iso()?, context.user_id,
                request.id, context.company_id, request.row_version
            ],
        )?;
        if changed != 1 {
            return Err(Phase05Error::concurrency());
        }
        audit(
            &transaction,
            &context,
            "partners.update",
            "partners",
            &request.id,
            None,
        )?;
        transaction.commit()?;
        self.get_partner(&request.id)
    }

    pub fn set_partner_active(&self, request: SetActiveRequest) -> Phase05Result<PartnerView> {
        let context = self.require_session(Some("partners.manage"))?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE partners SET is_active=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5 AND row_version=?6",
            params![i64::from(request.is_active), now_iso()?, context.user_id, request.id, context.company_id, request.row_version],
        )?;
        if changed != 1 {
            return Err(Phase05Error::concurrency());
        }
        audit(
            &transaction,
            &context,
            "partners.set_active",
            "partners",
            &request.id,
            None,
        )?;
        transaction.commit()?;
        self.get_partner(&request.id)
    }

    pub fn list_partner_addresses(
        &self,
        partner_id: String,
    ) -> Phase05Result<Vec<PartnerAddressView>> {
        let context = self.require_session(Some("partners.view"))?;
        ensure_partner(&self.open()?, &context.company_id, &partner_id)?;
        let connection = self.open()?;
        let mut statement = connection.prepare(
            "SELECT id,partner_id,address_kind,label,address_line_1,address_line_2,city,province,postal_code,is_default,is_active,row_version FROM partner_addresses WHERE company_id=?1 AND partner_id=?2 ORDER BY is_default DESC,address_kind,id",
        )?;
        let rows = statement.query_map(params![context.company_id, partner_id], |row| {
            Ok(PartnerAddressView {
                id: row.get(0)?,
                partner_id: row.get(1)?,
                address_kind: row.get(2)?,
                label: row.get(3)?,
                address_line_1: row.get(4)?,
                address_line_2: row.get(5)?,
                city: row.get(6)?,
                province: row.get(7)?,
                postal_code: row.get(8)?,
                is_default: row.get::<_, i64>(9)? == 1,
                is_active: row.get::<_, i64>(10)? == 1,
                row_version: row.get(11)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(Phase05Error::from)
    }

    pub fn create_partner_address(
        &self,
        input: PartnerAddressInput,
    ) -> Phase05Result<PartnerAddressView> {
        let context = self.require_session(Some("partners.manage"))?;
        let mut connection = self.open()?;
        ensure_partner(&connection, &context.company_id, &input.partner_id)?;
        let transaction = connection.transaction()?;
        if input.is_default {
            transaction.execute("UPDATE partner_addresses SET is_default=0,updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE company_id=?3 AND partner_id=?4 AND is_default=1", params![now_iso()?, context.user_id, context.company_id, input.partner_id])?;
        }
        let id = new_id();
        let timestamp = now_iso()?;
        transaction.execute(
            r#"INSERT INTO partner_addresses (id,company_id,partner_id,address_kind,label,address_line_1,address_line_2,city,province,postal_code,is_default,created_at,created_by,updated_at,updated_by)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?12,?13)"#,
            params![id,context.company_id,input.partner_id,trim_required(&input.address_kind,"addressKind")?,trim_optional(input.label.as_deref()),trim_required(&input.address_line_1,"addressLine1")?,trim_optional(input.address_line_2.as_deref()),trim_optional(input.city.as_deref()),trim_optional(input.province.as_deref()),trim_optional(input.postal_code.as_deref()),i64::from(input.is_default),timestamp,context.user_id],
        )?;
        audit(
            &transaction,
            &context,
            "partners.address.create",
            "partner_addresses",
            &id,
            None,
        )?;
        transaction.commit()?;
        self.get_partner_address(&id)
    }

    pub fn list_partner_contacts(
        &self,
        partner_id: String,
    ) -> Phase05Result<Vec<PartnerContactView>> {
        let context = self.require_session(Some("partners.view"))?;
        let connection = self.open()?;
        ensure_partner(&connection, &context.company_id, &partner_id)?;
        let mut statement = connection.prepare("SELECT id,partner_id,full_name,job_title,phone,email,is_primary,is_active,row_version FROM partner_contacts WHERE company_id=?1 AND partner_id=?2 ORDER BY is_primary DESC,full_name")?;
        let rows = statement.query_map(params![context.company_id, partner_id], |row| {
            Ok(PartnerContactView {
                id: row.get(0)?,
                partner_id: row.get(1)?,
                full_name: row.get(2)?,
                job_title: row.get(3)?,
                phone: row.get(4)?,
                email: row.get(5)?,
                is_primary: row.get::<_, i64>(6)? == 1,
                is_active: row.get::<_, i64>(7)? == 1,
                row_version: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(Phase05Error::from)
    }

    pub fn create_partner_contact(
        &self,
        input: PartnerContactInput,
    ) -> Phase05Result<PartnerContactView> {
        if input
            .phone
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_none()
            && input
                .email
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .is_none()
        {
            return Err(Phase05Error::invalid("contact"));
        }
        let context = self.require_session(Some("partners.manage"))?;
        let mut connection = self.open()?;
        ensure_partner(&connection, &context.company_id, &input.partner_id)?;
        let transaction = connection.transaction()?;
        if input.is_primary {
            transaction.execute("UPDATE partner_contacts SET is_primary=0,updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE company_id=?3 AND partner_id=?4 AND is_primary=1",params![now_iso()?,context.user_id,context.company_id,input.partner_id])?;
        }
        let id = new_id();
        let timestamp = now_iso()?;
        transaction.execute("INSERT INTO partner_contacts (id,company_id,partner_id,full_name,job_title,phone,email,is_primary,created_at,created_by,updated_at,updated_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?9,?10)",params![id,context.company_id,input.partner_id,trim_required(&input.full_name,"fullName")?,trim_optional(input.job_title.as_deref()),trim_optional(input.phone.as_deref()),trim_optional(input.email.as_deref()),i64::from(input.is_primary),timestamp,context.user_id])?;
        audit(
            &transaction,
            &context,
            "partners.contact.create",
            "partner_contacts",
            &id,
            None,
        )?;
        transaction.commit()?;
        self.get_partner_contact(&id)
    }

    fn get_partner(&self, id: &str) -> Phase05Result<PartnerView> {
        let context = self.require_session(Some("partners.view"))?;
        self.open()?.query_row("SELECT id,code,legal_name,display_name_ar,display_name_fr,is_customer,is_supplier,is_active,row_version FROM partners WHERE id=?1 AND company_id=?2",params![id,context.company_id],map_partner).optional()?.ok_or_else(not_found)
    }
    fn get_partner_address(&self, id: &str) -> Phase05Result<PartnerAddressView> {
        let context = self.require_session(Some("partners.view"))?;
        self.open()?.query_row("SELECT id,partner_id,address_kind,label,address_line_1,address_line_2,city,province,postal_code,is_default,is_active,row_version FROM partner_addresses WHERE id=?1 AND company_id=?2",params![id,context.company_id],|row|Ok(PartnerAddressView{id:row.get(0)?,partner_id:row.get(1)?,address_kind:row.get(2)?,label:row.get(3)?,address_line_1:row.get(4)?,address_line_2:row.get(5)?,city:row.get(6)?,province:row.get(7)?,postal_code:row.get(8)?,is_default:row.get::<_,i64>(9)?==1,is_active:row.get::<_,i64>(10)?==1,row_version:row.get(11)?})).optional()?.ok_or_else(not_found)
    }
    fn get_partner_contact(&self, id: &str) -> Phase05Result<PartnerContactView> {
        let context = self.require_session(Some("partners.view"))?;
        self.open()?.query_row("SELECT id,partner_id,full_name,job_title,phone,email,is_primary,is_active,row_version FROM partner_contacts WHERE id=?1 AND company_id=?2",params![id,context.company_id],|row|Ok(PartnerContactView{id:row.get(0)?,partner_id:row.get(1)?,full_name:row.get(2)?,job_title:row.get(3)?,phone:row.get(4)?,email:row.get(5)?,is_primary:row.get::<_,i64>(6)?==1,is_active:row.get::<_,i64>(7)?==1,row_version:row.get(8)?})).optional()?.ok_or_else(not_found)
    }
}

fn map_partner(row: &rusqlite::Row<'_>) -> rusqlite::Result<PartnerView> {
    Ok(PartnerView {
        id: row.get(0)?,
        code: row.get(1)?,
        legal_name: row.get(2)?,
        display_name_ar: row.get(3)?,
        display_name_fr: row.get(4)?,
        is_customer: row.get::<_, i64>(5)? == 1,
        is_supplier: row.get::<_, i64>(6)? == 1,
        is_active: row.get::<_, i64>(7)? == 1,
        row_version: row.get(8)?,
    })
}
fn validate_partner_roles(customer: bool, supplier: bool) -> Phase05Result<()> {
    if customer || supplier {
        Ok(())
    } else {
        Err(Phase05Error::invalid("partnerRoles"))
    }
}
fn ensure_partner(
    connection: &rusqlite::Connection,
    company_id: &str,
    partner_id: &str,
) -> Phase05Result<()> {
    connection
        .query_row(
            "SELECT 1 FROM partners WHERE id=?1 AND company_id=?2",
            params![partner_id, company_id],
            |_| Ok(()),
        )
        .optional()?
        .ok_or_else(not_found)
}
fn not_found() -> Phase05Error {
    Phase05Error::new("NOT_FOUND", "The record was not found.")
}

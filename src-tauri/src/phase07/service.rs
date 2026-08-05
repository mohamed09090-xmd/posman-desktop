use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};

use crate::{
    phase05::{Phase05Service, Phase06AuthContext},
    phase06::{
        audit, authorize_transaction,
        dto::EntityResult,
        entity_result,
        error::{Phase06Error, Phase06Result},
        new_id, now_iso, product_snapshot,
        projections::balance,
        tax_snapshot,
    },
};

use super::{
    dto::{SalesLineInput, TransformLineInput},
    pricing::{allocate_header_discount, base_line, PricedLine},
};

const SALES_CODES: &str =
    "'stock.read','sales_order.confirm','delivery_note.post','sales_invoice.post'";
const OWNER_CODES: &str = "'stock.read','sales_order.confirm','delivery_note.post','sales_invoice.post','pricing.override_below_cost'";
const AUDITOR_CODES: &str = "'stock.read'";

#[derive(Clone)]
pub struct Phase07Service {
    pub(super) phase05: Phase05Service,
}

#[derive(Clone, Debug)]
pub(super) struct PreparedSalesLine {
    pub source_line_id: Option<String>,
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity_scaled: i64,
    pub unit_price_scaled: i64,
    pub unit_cost_scaled: i64,
    pub discount_rate_scaled: i64,
    pub tax_rate_scaled: i64,
    pub product_code: String,
    pub product_name: String,
    pub unit_id: String,
    pub unit_code: String,
    pub tax_code: Option<String>,
    pub priced: PricedLine,
}

impl Phase07Service {
    pub fn new(phase05: Phase05Service) -> Phase06Result<Self> {
        let service = Self { phase05 };
        service.provision_permissions()?;
        Ok(service)
    }

    fn provision_permissions(&self) -> Phase06Result<()> {
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_iso()?;
        grant_company_role(&transaction, "SYSTEM_ADMINISTRATOR", OWNER_CODES, &now)?;
        grant_company_role(&transaction, "OWNER", OWNER_CODES, &now)?;
        grant_company_role(&transaction, "SALES", SALES_CODES, &now)?;
        grant_company_role(
            &transaction,
            "STOCK",
            "'stock.read','delivery_note.post'",
            &now,
        )?;
        grant_company_role(&transaction, "AUDITOR", AUDITOR_CODES, &now)?;
        transaction.commit()?;
        Ok(())
    }

    pub(super) fn context(&self, permission: Option<&str>) -> Phase06Result<Phase06AuthContext> {
        self.phase05
            .phase06_authorize(permission)
            .map_err(Into::into)
    }

    pub(super) fn immediate<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Phase06Result<T>,
    ) -> Phase06Result<T> {
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = operation(&transaction)?;
        transaction.commit()?;
        Ok(result)
    }

    pub(super) fn read<T>(
        &self,
        operation: impl FnOnce(&rusqlite::Connection) -> Phase06Result<T>,
    ) -> Phase06Result<T> {
        let connection = self.phase05.phase06_open()?;
        operation(&connection)
    }
}

fn grant_company_role(
    transaction: &Transaction<'_>,
    role_code: &str,
    permissions: &str,
    timestamp: &str,
) -> Phase06Result<()> {
    let sql = format!(
        "INSERT OR IGNORE INTO role_permissions (
            id, company_id, role_id, permission_id, granted_at, granted_by
         )
         SELECT 'rp-p07-' || role.id || '-' || permission.id,
                role.company_id, role.id, permission.id, ?1, NULL
         FROM roles role CROSS JOIN permissions permission
         WHERE role.company_id IS NOT NULL AND role.is_system=1 AND role.is_active=1
           AND role.code=?2 AND permission.code IN ({permissions})"
    );
    transaction.execute(&sql, params![timestamp, role_code])?;
    Ok(())
}

pub(super) fn ensure_customer(
    transaction: &Transaction<'_>,
    company_id: &str,
    customer_id: &str,
) -> Phase06Result<()> {
    let exists = transaction
        .query_row(
            "SELECT 1 FROM partners WHERE id=?1 AND company_id=?2 AND is_customer=1 AND is_active=1",
            params![customer_id, company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !exists {
        return Err(Phase06Error::new(
            "CUSTOMER_REQUIRED",
            "Select an active customer.",
        ));
    }
    Ok(())
}

pub(super) fn validate_sales_lines(lines: &[SalesLineInput]) -> Phase06Result<()> {
    if lines.is_empty() {
        return Err(Phase06Error::invalid("lines"));
    }
    for line in lines {
        if line.quantity_scaled <= 0
            || line.unit_price_scaled < 0
            || !(0..=1_000_000).contains(&line.discount_rate_scaled)
        {
            return Err(Phase06Error::invalid("salesLine"));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_sales_lines(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    lines: &[SalesLineInput],
    default_warehouse: &str,
    date: &str,
    price_mode: &str,
    header_discount_rate_scaled: i64,
) -> Phase06Result<(Vec<PreparedSalesLine>, (i64, i64, i64, i64))> {
    validate_sales_lines(lines)?;
    let mut prepared = Vec::with_capacity(lines.len());
    for input in lines {
        let warehouse_id = input.warehouse_id.as_deref().unwrap_or(default_warehouse);
        crate::phase06::projections::validate_warehouse_scope(
            transaction,
            &context.company_id,
            warehouse_id,
            None,
        )?;
        let (product_code, product_name, unit_id, unit_code, default_tax_id) =
            product_snapshot(transaction, &context.company_id, &input.product_id)?;
        let (tax_code, tax_rate_scaled) = tax_snapshot(
            transaction,
            &context.company_id,
            input.tax_rate_id.as_deref().or(default_tax_id.as_deref()),
            date,
        )?;
        let cost = balance(
            transaction,
            &context.company_id,
            &input.product_id,
            warehouse_id,
            None,
        )?
        .average_cost;
        let priced = base_line(
            input.quantity_scaled,
            input.unit_price_scaled,
            input.discount_rate_scaled,
            tax_rate_scaled,
            price_mode,
        )?;
        prepared.push(PreparedSalesLine {
            source_line_id: None,
            product_id: input.product_id.clone(),
            warehouse_id: warehouse_id.to_owned(),
            quantity_scaled: input.quantity_scaled,
            unit_price_scaled: input.unit_price_scaled,
            unit_cost_scaled: cost,
            discount_rate_scaled: input.discount_rate_scaled,
            tax_rate_scaled,
            product_code,
            product_name,
            unit_id,
            unit_code,
            tax_code,
            priced,
        });
    }
    let mut priced = prepared
        .iter()
        .map(|line| (line.priced.clone(), line.tax_rate_scaled))
        .collect::<Vec<_>>();
    let totals = allocate_header_discount(&mut priced, header_discount_rate_scaled)?;
    for (line, (priced, _)) in prepared.iter_mut().zip(priced) {
        line.priced = priced;
    }
    Ok((prepared, totals))
}

pub(super) fn insert_prepared_lines(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    document_id: &str,
    lines: &[PreparedSalesLine],
    notes: Option<&str>,
) -> Phase06Result<Vec<String>> {
    let now = now_iso()?;
    let mut ids = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        let id = new_id();
        transaction.execute(
            r#"INSERT INTO commercial_document_lines (
                id, company_id, document_id, product_id, warehouse_id, unit_id,
                line_number, product_code_snapshot, description_snapshot,
                unit_code_snapshot, tax_code_snapshot, quantity_scaled,
                unit_price_scaled, unit_cost_scaled, line_discount_rate_scaled,
                line_discount_minor, allocated_header_discount_minor, tax_rate_scaled,
                line_ht_minor, line_tax_minor, line_ttc_minor, notes,
                created_at, created_by, updated_at, updated_by
            ) VALUES (
                ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,
                ?16,?17,?18,?19,?20,?21,?22,?23,?24,?23,?24
            )"#,
            params![
                id,
                context.company_id,
                document_id,
                line.product_id,
                line.warehouse_id,
                line.unit_id,
                i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?,
                line.product_code,
                line.product_name,
                line.unit_code,
                line.tax_code,
                line.quantity_scaled,
                line.unit_price_scaled,
                line.unit_cost_scaled,
                line.discount_rate_scaled,
                line.priced.line_discount_minor,
                line.priced.allocated_header_discount_minor,
                line.tax_rate_scaled,
                line.priced.taxable_ht_minor,
                line.priced.tax_minor,
                line.priced.ttc_minor,
                notes,
                now,
                context.user_id,
            ],
        )?;
        ids.push(id);
    }
    Ok(ids)
}

pub(super) fn prepare_transformed_lines(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    source_document_id: &str,
    requested: &[TransformLineInput],
) -> Phase06Result<(Vec<PreparedSalesLine>, (i64, i64, i64, i64), String, i64)> {
    if requested.is_empty() {
        return Err(Phase06Error::invalid("lines"));
    }
    let (price_mode, header_rate): (String, i64) = transaction
        .query_row(
            "SELECT price_mode, header_discount_rate_scaled FROM commercial_documents
             WHERE id=?1 AND company_id=?2",
            params![source_document_id, context.company_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)?;
    let mut lines = Vec::with_capacity(requested.len());
    for input in requested {
        if input.quantity_scaled <= 0 {
            return Err(Phase06Error::invalid("quantityScaled"));
        }
        let mut line = transaction
            .query_row(
                r#"SELECT line.product_id, line.warehouse_id, line.unit_id,
                          line.product_code_snapshot, line.description_snapshot,
                          line.unit_code_snapshot, line.tax_code_snapshot,
                          line.unit_price_scaled, line.unit_cost_scaled,
                          line.line_discount_rate_scaled, line.tax_rate_scaled
                   FROM commercial_document_lines line
                   WHERE line.id=?1 AND line.company_id=?2 AND line.document_id=?3"#,
                params![input.source_line_id, context.company_id, source_document_id],
                |row| {
                    Ok(PreparedSalesLine {
                        source_line_id: Some(input.source_line_id.clone()),
                        product_id: row.get(0)?,
                        warehouse_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        quantity_scaled: input.quantity_scaled,
                        unit_id: row.get(2)?,
                        product_code: row.get(3)?,
                        product_name: row.get(4)?,
                        unit_code: row.get(5)?,
                        tax_code: row.get(6)?,
                        unit_price_scaled: row.get(7)?,
                        unit_cost_scaled: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                        discount_rate_scaled: row.get(9)?,
                        tax_rate_scaled: row.get(10)?,
                        priced: PricedLine {
                            line_discount_minor: 0,
                            before_header_ht_minor: 0,
                            allocated_header_discount_minor: 0,
                            taxable_ht_minor: 0,
                            tax_minor: 0,
                            ttc_minor: 0,
                        },
                    })
                },
            )
            .optional()?
            .ok_or_else(Phase06Error::not_found)?;
        if line.warehouse_id.is_empty() {
            return Err(Phase06Error::invalid("warehouseId"));
        }
        line.unit_cost_scaled = balance(
            transaction,
            &context.company_id,
            &line.product_id,
            &line.warehouse_id,
            None,
        )?
        .average_cost;
        line.priced = base_line(
            line.quantity_scaled,
            line.unit_price_scaled,
            line.discount_rate_scaled,
            line.tax_rate_scaled,
            &price_mode,
        )?;
        lines.push(line);
    }
    let mut priced = lines
        .iter()
        .map(|line| (line.priced.clone(), line.tax_rate_scaled))
        .collect::<Vec<_>>();
    let totals = allocate_header_discount(&mut priced, header_rate)?;
    for (line, (priced, _)) in lines.iter_mut().zip(priced) {
        line.priced = priced;
    }
    Ok((lines, totals, price_mode, header_rate))
}

pub(super) fn insert_transform_links(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    lines: &[PreparedSalesLine],
    target_ids: &[String],
    transformation_type: &str,
) -> Phase06Result<()> {
    if lines.len() != target_ids.len() {
        return Err(Phase06Error::internal());
    }
    for (line, target) in lines.iter().zip(target_ids) {
        let source = line
            .source_line_id
            .as_deref()
            .ok_or_else(|| Phase06Error::invalid("sourceLineId"))?;
        let source_quantity: i64 = transaction.query_row(
            "SELECT quantity_scaled FROM commercial_document_lines WHERE id=?1 AND company_id=?2",
            params![source, context.company_id],
            |row| row.get(0),
        )?;
        let transformed: i64 = transaction.query_row(
            "SELECT COALESCE(SUM(transformed_quantity_scaled),0) FROM document_line_links
             WHERE company_id=?1 AND source_line_id=?2 AND transformation_type=?3",
            params![context.company_id, source, transformation_type],
            |row| row.get(0),
        )?;
        let total = transformed
            .checked_add(line.quantity_scaled)
            .ok_or_else(Phase06Error::numeric_overflow)?;
        if total > source_quantity {
            return Err(Phase06Error::over_transformation());
        }
        transaction.execute(
            "INSERT INTO document_line_links (
               id, company_id, source_line_id, target_line_id, transformation_type,
               transformed_quantity_scaled, created_at, created_by
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                new_id(),
                context.company_id,
                source,
                target,
                transformation_type,
                line.quantity_scaled,
                now_iso()?,
                context.user_id
            ],
        )?;
    }
    Ok(())
}

pub(super) fn load_document_priced_lines(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    document_id: &str,
) -> Phase06Result<Vec<PreparedSalesLine>> {
    let mut statement = transaction.prepare(
        r#"SELECT line.id, line.product_id, line.warehouse_id, line.quantity_scaled,
                  line.unit_price_scaled, COALESCE(line.unit_cost_scaled,0),
                  line.line_discount_rate_scaled, line.tax_rate_scaled,
                  line.product_code_snapshot, line.description_snapshot,
                  line.unit_id, line.unit_code_snapshot, line.tax_code_snapshot,
                  line.line_discount_minor + line.line_ht_minor + line.allocated_header_discount_minor,
                  line.allocated_header_discount_minor, line.line_ht_minor,
                  line.line_tax_minor, line.line_ttc_minor
           FROM commercial_document_lines line
           WHERE line.document_id=?1 AND line.company_id=?2 ORDER BY line.line_number"#,
    )?;
    let rows = statement
        .query_map(params![document_id, context.company_id], |row| {
            Ok(PreparedSalesLine {
                source_line_id: Some(row.get(0)?),
                product_id: row.get(1)?,
                warehouse_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                quantity_scaled: row.get(3)?,
                unit_price_scaled: row.get(4)?,
                unit_cost_scaled: row.get(5)?,
                discount_rate_scaled: row.get(6)?,
                tax_rate_scaled: row.get(7)?,
                product_code: row.get(8)?,
                product_name: row.get(9)?,
                unit_id: row.get(10)?,
                unit_code: row.get(11)?,
                tax_code: row.get(12)?,
                priced: PricedLine {
                    line_discount_minor: row.get(13)?,
                    before_header_ht_minor: row.get::<_, i64>(15)? + row.get::<_, i64>(14)?,
                    allocated_header_discount_minor: row.get(14)?,
                    taxable_ht_minor: row.get(15)?,
                    tax_minor: row.get(16)?,
                    ttc_minor: row.get(17)?,
                },
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn apply_document_pricing(
    transaction: &Transaction<'_>,
    company_id: &str,
    document_id: &str,
    price_mode: &str,
    header_rate: i64,
    totals: (i64, i64, i64, i64),
) -> Phase06Result<()> {
    transaction.execute(
        "UPDATE commercial_documents SET price_mode=?1, header_discount_rate_scaled=?2,
         header_discount_minor=?3, total_ht_minor=?4, total_tax_minor=?5, total_ttc_minor=?6
         WHERE id=?7 AND company_id=?8 AND posting_status='DRAFT'",
        params![
            price_mode,
            header_rate,
            totals.0,
            totals.1,
            totals.2,
            totals.3,
            document_id,
            company_id
        ],
    )?;
    Ok(())
}

pub(super) fn enforce_below_cost(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    lines: &[PreparedSalesLine],
    override_reason: Option<&str>,
    action: &str,
    entity_id: &str,
) -> Phase06Result<()> {
    let below = lines
        .iter()
        .filter(|line| {
            crate::phase06::fixed_point::extended_cost_minor(
                line.quantity_scaled,
                line.unit_cost_scaled,
            )
            .map(|cost| line.priced.taxable_ht_minor < cost)
            .unwrap_or(true)
        })
        .count();
    if below == 0 {
        return Ok(());
    }
    let policy: String = transaction.query_row(
        "SELECT below_cost_policy FROM company_settings WHERE company_id=?1",
        params![context.company_id],
        |row| row.get(0),
    )?;
    match policy.as_str() {
        "BLOCK" => {
            return Err(Phase06Error::new(
                "BELOW_COST_BLOCKED",
                "The sale price is below current CUMP.",
            ))
        }
        "ADMIN_OVERRIDE" => {
            let reason = override_reason
                .map(str::trim)
                .filter(|value| value.len() >= 5)
                .ok_or_else(|| {
                    Phase06Error::new(
                        "BELOW_COST_OVERRIDE_REQUIRED",
                        "An authorized reason is required for a below-cost sale.",
                    )
                })?;
            authorize_transaction(transaction, context, "pricing.override_below_cost")?;
            let details = serde_json::json!({
                "reason": reason,
                "belowCostLineCount": below,
                "comparison": "net HT after discounts versus warehouse CUMP"
            })
            .to_string();
            audit(
                transaction,
                context,
                action,
                "commercial_document",
                entity_id,
                Some(&details),
            )?;
        }
        "WARNING_ONLY" => {
            let details =
                serde_json::json!({"belowCostLineCount": below, "policy": "WARNING_ONLY"})
                    .to_string();
            audit(
                transaction,
                context,
                action,
                "commercial_document",
                entity_id,
                Some(&details),
            )?;
        }
        _ => return Err(Phase06Error::internal()),
    }
    Ok(())
}

pub(super) fn insert_status(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    document_id: &str,
    old_status: &str,
    new_status: &str,
    reason: Option<&str>,
) -> Phase06Result<()> {
    let row_version: i64 = transaction.query_row(
        "SELECT row_version FROM commercial_documents WHERE id=?1 AND company_id=?2",
        params![document_id, context.company_id],
        |row| row.get(0),
    )?;
    transaction.execute(
        "INSERT INTO document_status_history (
           id, company_id, document_id, old_status, new_status, reason,
           row_version_snapshot, changed_at, changed_by
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            new_id(),
            context.company_id,
            document_id,
            old_status,
            new_status,
            reason,
            row_version,
            now_iso()?,
            context.user_id
        ],
    )?;
    Ok(())
}

pub(super) fn sales_entity(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    id: &str,
    replayed: bool,
) -> Phase06Result<EntityResult> {
    entity_result(transaction, &context.company_id, id, replayed)
}

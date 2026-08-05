pub(crate) fn validate_commercial_date(value: &str) -> Phase06Result<()> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    time::Date::parse(value, format)
        .map(|_| ())
        .map_err(|_| Phase06Error::invalid("commercialDate"))
}

pub(crate) fn validate_business_date(value: &str) -> Phase06Result<()> {
    validate_commercial_date(value)
}

pub(crate) fn active_fiscal_scope(
    transaction: &Transaction<'_>,
    company_id: &str,
    date: &str,
) -> Phase06Result<(String, String)> {
    validate_commercial_date(date)?;
    transaction
        .query_row(
            r#"
            SELECT fiscal_year.id, fiscal_period.id
            FROM fiscal_years AS fiscal_year
            JOIN fiscal_periods AS fiscal_period
              ON fiscal_period.fiscal_year_id=fiscal_year.id
             AND fiscal_period.company_id=fiscal_year.company_id
            WHERE fiscal_year.company_id=?1
              AND ?2 BETWEEN fiscal_period.starts_on AND fiscal_period.ends_on
              AND fiscal_period.status='OPEN'
              AND fiscal_year.status='OPEN'
            LIMIT 1
            "#,
            params![company_id, date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| {
            Phase06Error::new(
                "FISCAL_PERIOD_UNAVAILABLE",
                "No open fiscal period covers the document date.",
            )
        })
}

pub(crate) fn next_document_number(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    fiscal_year_id: &str,
    document_type: &str,
) -> Phase06Result<String> {
    let existing = transaction
        .query_row(
            r#"
            SELECT id, prefix, next_number, padding_width
            FROM document_sequences
            WHERE company_id=?1 AND fiscal_year_id=?2 AND document_type=?3
            ORDER BY prefix
            LIMIT 1
            "#,
            params![context.company_id, fiscal_year_id, document_type],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;

    let (sequence_id, prefix, next_number, padding_width) =
        if let Some(existing) = existing {
            existing
        } else {
            let sequence_id = new_id();
            let prefix = document_prefix(document_type).to_owned();
            let now = now_iso()?;
            transaction.execute(
                r#"
                INSERT INTO document_sequences (
                    id, company_id, fiscal_year_id, document_type, prefix,
                    next_number, padding_width, created_at, created_by,
                    updated_at, updated_by
                ) VALUES (?1, ?2, ?3, ?4, ?5, 1, 6, ?6, ?7, ?6, ?7)
                "#,
                params![
                    sequence_id,
                    context.company_id,
                    fiscal_year_id,
                    document_type,
                    prefix,
                    now,
                    context.user_id
                ],
            )?;
            (sequence_id, prefix, 1, 6)
        };

    let changed = transaction.execute(
        r#"
        UPDATE document_sequences
        SET next_number=next_number+1, updated_at=?1, updated_by=?2,
            row_version=row_version+1
        WHERE id=?3 AND company_id=?4 AND next_number=?5
        "#,
        params![
            now_iso()?,
            context.user_id,
            sequence_id,
            context.company_id,
            next_number
        ],
    )?;
    if changed != 1 {
        return Err(Phase06Error::conflict());
    }

    let width = usize::try_from(padding_width).unwrap_or(6);
    Ok(format!("{prefix}{next_number:0width$}"))
}

fn document_prefix(document_type: &str) -> &'static str {
    match document_type {
        "OPENING_STOCK" => "OUV-",
        "STOCK_ADJUSTMENT" => "AJU-",
        "STOCK_TRANSFER" => "TRF-",
        "INVENTORY_COUNT" => "INV-",
        "PURCHASE_ORDER" => "BCF-",
        "PURCHASE_RECEIPT" => "BRF-",
        "PURCHASE_INVOICE" => "FAF-",
        "PURCHASE_RETURN" => "RAF-",
        _ => "DOC-",
    }
}

pub(crate) fn ensure_warehouse(
    connection: &rusqlite::Connection,
    company_id: &str,
    warehouse_id: &str,
) -> Phase06Result<()> {
    let exists = connection
        .query_row(
            r#"
            SELECT 1 FROM warehouses
            WHERE id=?1 AND company_id=?2 AND is_active=1
            "#,
            params![warehouse_id, company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !exists {
        return Err(Phase06Error::invalid("warehouseId"));
    }
    Ok(())
}

pub(crate) fn product_snapshot(
    transaction: &Transaction<'_>,
    company_id: &str,
    product_id: &str,
) -> Phase06Result<(String, String, String, String, Option<String>)> {
    transaction
        .query_row(
            r#"
            SELECT product.code,
                   COALESCE(NULLIF(product.name_ar, ''), product.name_fr),
                   unit.id, unit.code, product.default_tax_rate_id
            FROM products AS product
            JOIN units AS unit
              ON unit.id=product.unit_id AND unit.company_id=product.company_id
            WHERE product.id=?1 AND product.company_id=?2
              AND product.is_active=1 AND product.stock_tracked=1
            "#,
            params![product_id, company_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)
}

pub(crate) fn tax_snapshot(
    transaction: &Transaction<'_>,
    company_id: &str,
    tax_id: Option<&str>,
    date: &str,
) -> Phase06Result<(Option<String>, i64)> {
    let Some(tax_id) = tax_id else {
        return Ok((None, 0));
    };
    transaction
        .query_row(
            r#"
            SELECT code, rate_scaled
            FROM tax_rates
            WHERE id=?1 AND company_id=?2 AND is_active=1
              AND valid_from<=?3 AND (valid_to IS NULL OR valid_to>=?3)
            "#,
            params![tax_id, company_id, date],
            |row| Ok((Some(row.get(0)?), row.get(1)?)),
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_document(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    document_type: &str,
    workflow_status: &str,
    posting_status: &str,
    date: &str,
    partner_id: Option<&str>,
    warehouse_id: Option<&str>,
    source_document_id: Option<&str>,
    due_date: Option<&str>,
    notes: Option<&str>,
    idempotency_key: Option<&str>,
    totals: (i64, i64, i64),
) -> Phase06Result<(String, String)> {
    let (fiscal_year_id, fiscal_period_id) =
        active_fiscal_scope(transaction, &context.company_id, date)?;
    if let Some(warehouse_id) = warehouse_id {
        ensure_warehouse(transaction, &context.company_id, warehouse_id)?;
    }
    let document_number = next_document_number(
        transaction,
        context,
        &fiscal_year_id,
        document_type,
    )?;
    let document_id = new_id();
    let now = now_iso()?;

    transaction.execute(
        r#"
        INSERT INTO commercial_documents (
            id, company_id, fiscal_year_id, fiscal_period_id, partner_id,
            warehouse_id, source_document_id, document_type, document_number,
            workflow_status, posting_status, commercial_date, posting_date,
            due_date, total_ht_minor, total_tax_minor, total_ttc_minor, notes,
            idempotency_key, posted_at, posted_by, created_at, created_by,
            updated_at, updated_by
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            CASE WHEN ?11='POSTED' THEN ?12 ELSE NULL END,
            ?13, ?14, ?15, ?16, ?17, ?18,
            CASE WHEN ?11='POSTED' THEN ?19 ELSE NULL END,
            CASE WHEN ?11='POSTED' THEN ?20 ELSE NULL END,
            ?19, ?20, ?19, ?20
        )
        "#,
        params![
            document_id,
            context.company_id,
            fiscal_year_id,
            fiscal_period_id,
            partner_id,
            warehouse_id,
            source_document_id,
            document_type,
            document_number,
            workflow_status,
            posting_status,
            date,
            due_date,
            totals.0,
            totals.1,
            totals.2,
            notes,
            idempotency_key,
            now,
            context.user_id
        ],
    )?;

    Ok((document_id, document_number))
}

pub(crate) fn insert_status_history(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    document_id: &str,
    old_status: Option<&str>,
    new_status: &str,
    reason: Option<&str>,
    row_version: i64,
) -> Phase06Result<()> {
    transaction.execute(
        r#"
        INSERT INTO document_status_history (
            id, company_id, document_id, old_status, new_status, reason,
            row_version_snapshot, changed_at, changed_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
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

pub(crate) fn opening_reviewed(
    transaction: &Transaction<'_>,
    company_id: &str,
    document_id: &str,
) -> Phase06Result<bool> {
    let latest = transaction
        .query_row(
            r#"
            SELECT new_status
            FROM document_status_history
            WHERE company_id=?1 AND document_id=?2
            ORDER BY changed_at DESC, id DESC
            LIMIT 1
            "#,
            params![company_id, document_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(latest.as_deref() == Some("REVIEWED"))
}

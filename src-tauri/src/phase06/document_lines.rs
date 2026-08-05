pub(crate) struct PurchaseLineOptions<'a> {
    pub default_warehouse: Option<&'a str>,
    pub notes: Option<&'a str>,
}

pub(crate) fn insert_purchase_line(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    document_id: &str,
    line_number: i64,
    input: &dto::PurchaseLineInput,
    commercial_date: &str,
    options: PurchaseLineOptions<'_>,
) -> Phase06Result<(String, (i64, i64, i64))> {
    let (product_code, product_name, unit_id, unit_code, default_tax_id) =
        product_snapshot(transaction, &context.company_id, &input.product_id)?;
    let warehouse_id = input.warehouse_id.as_deref().or(options.default_warehouse);
    if let Some(warehouse_id) = warehouse_id {
        ensure_warehouse(transaction, &context.company_id, warehouse_id)?;
    }
    let tax_id = input.tax_rate_id.as_deref().or(default_tax_id.as_deref());
    let (tax_code, tax_rate_scaled) =
        tax_snapshot(transaction, &context.company_id, tax_id, commercial_date)?;
    let (discount_minor, line_ht_minor, line_tax_minor, line_ttc_minor) = line_totals(
        input.quantity_scaled,
        input.unit_price_scaled,
        input.discount_rate_scaled,
        tax_rate_scaled,
    )?;
    let line_id = new_id();
    let now = now_iso()?;

    transaction.execute(
        r#"
        INSERT INTO commercial_document_lines (
            id, company_id, document_id, product_id, warehouse_id, unit_id,
            line_number, product_code_snapshot, description_snapshot,
            unit_code_snapshot, tax_code_snapshot, quantity_scaled,
            unit_price_scaled, unit_cost_scaled, line_discount_rate_scaled,
            line_discount_minor, tax_rate_scaled, line_ht_minor, line_tax_minor,
            line_ttc_minor, notes, created_at, created_by, updated_at, updated_by
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?22, ?23
        )
        "#,
        params![
            line_id,
            context.company_id,
            document_id,
            input.product_id,
            warehouse_id,
            unit_id,
            line_number,
            product_code,
            product_name,
            unit_code,
            tax_code,
            input.quantity_scaled,
            input.unit_price_scaled,
            input.unit_cost_scaled,
            input.discount_rate_scaled,
            discount_minor,
            tax_rate_scaled,
            line_ht_minor,
            line_tax_minor,
            line_ttc_minor,
            options.notes,
            now,
            context.user_id
        ],
    )?;

    Ok((line_id, (line_ht_minor, line_tax_minor, line_ttc_minor)))
}

pub(crate) fn update_document_totals(
    transaction: &Transaction<'_>,
    document_id: &str,
    company_id: &str,
) -> Phase06Result<(i64, i64, i64)> {
    let totals = transaction.query_row(
        r#"
        SELECT COALESCE(SUM(line_ht_minor), 0),
               COALESCE(SUM(line_tax_minor), 0),
               COALESCE(SUM(line_ttc_minor), 0)
        FROM commercial_document_lines
        WHERE document_id=?1 AND company_id=?2
        "#,
        params![document_id, company_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    transaction.execute(
        r#"
        UPDATE commercial_documents
        SET total_ht_minor=?1, total_tax_minor=?2, total_ttc_minor=?3
        WHERE id=?4 AND company_id=?5 AND posting_status='DRAFT'
        "#,
        params![totals.0, totals.1, totals.2, document_id, company_id],
    )?;
    Ok(totals)
}

pub(crate) fn entity_result(
    transaction: &Transaction<'_>,
    company_id: &str,
    document_id: &str,
    replayed: bool,
) -> Phase06Result<EntityResult> {
    transaction
        .query_row(
            r#"
            SELECT document_number,
                   CASE
                     WHEN posting_status='POSTED' THEN 'POSTED'
                     WHEN document_type='OPENING_STOCK' AND EXISTS (
                       SELECT 1 FROM document_status_history history
                       WHERE history.company_id=commercial_documents.company_id
                         AND history.document_id=commercial_documents.id
                         AND history.new_status='REVIEWED'
                         AND NOT EXISTS (
                           SELECT 1 FROM document_status_history later
                           WHERE later.company_id=history.company_id
                             AND later.document_id=history.document_id
                             AND (later.changed_at>history.changed_at
                               OR (later.changed_at=history.changed_at AND later.id>history.id))
                         )
                     ) THEN 'REVIEWED'
                     ELSE workflow_status
                   END,
                   row_version
            FROM commercial_documents
            WHERE id=?1 AND company_id=?2
            "#,
            params![document_id, company_id],
            |row| {
                Ok(EntityResult {
                    id: document_id.to_owned(),
                    document_number: Some(row.get(0)?),
                    status: row.get(1)?,
                    row_version: row.get(2)?,
                    replayed,
                })
            },
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)
}

pub(crate) fn get_document_connection(
    connection: &rusqlite::Connection,
    company_id: &str,
    document_id: &str,
) -> Phase06Result<DocumentView> {
    let mut document = connection
        .query_row(
            r#"
            SELECT id, document_type, document_number, workflow_status,
                   posting_status, commercial_date, partner_id, warehouse_id,
                   source_document_id, total_ht_minor, total_tax_minor,
                   total_ttc_minor, notes, row_version
            FROM commercial_documents
            WHERE id=?1 AND company_id=?2
            "#,
            params![document_id, company_id],
            |row| {
                Ok(DocumentView {
                    id: row.get(0)?,
                    document_type: row.get(1)?,
                    document_number: row.get(2)?,
                    workflow_status: row.get(3)?,
                    posting_status: row.get(4)?,
                    commercial_date: row.get(5)?,
                    partner_id: row.get(6)?,
                    warehouse_id: row.get(7)?,
                    source_document_id: row.get(8)?,
                    total_ht_minor: row.get(9)?,
                    total_tax_minor: row.get(10)?,
                    total_ttc_minor: row.get(11)?,
                    notes: row.get(12)?,
                    row_version: row.get(13)?,
                    lines: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)?;

    let mut statement = connection.prepare(
        r#"
        SELECT line.id, line.product_id, line.product_code_snapshot,
               line.description_snapshot, line.warehouse_id,
               line.quantity_scaled, line.unit_price_scaled, line.unit_cost_scaled,
               line.tax_rate_scaled, line.line_ht_minor, line.line_tax_minor,
               line.line_ttc_minor, line.notes,
               (
                 SELECT link.source_line_id
                 FROM document_line_links AS link
                 WHERE link.target_line_id=line.id
                 ORDER BY link.id
                 LIMIT 1
               )
        FROM commercial_document_lines AS line
        WHERE line.document_id=?1 AND line.company_id=?2
        ORDER BY line.line_number
        "#,
    )?;
    document.lines = statement
        .query_map(params![document_id, company_id], |row| {
            Ok(DocumentLineView {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_code: row.get(2)?,
                description: row.get(3)?,
                warehouse_id: row.get(4)?,
                quantity_scaled: row.get(5)?,
                unit_price_scaled: row.get(6)?,
                unit_cost_scaled: row.get(7)?,
                tax_rate_scaled: row.get(8)?,
                line_ht_minor: row.get(9)?,
                line_tax_minor: row.get(10)?,
                line_ttc_minor: row.get(11)?,
                notes: row.get(12)?,
                source_line_id: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(document)
}

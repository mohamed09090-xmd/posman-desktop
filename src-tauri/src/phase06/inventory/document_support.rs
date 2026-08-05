fn insert_stock_document_lines(
    transaction: &rusqlite::Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    document_id: &str,
    warehouse_id: &str,
    lines: &[StockLineInput],
) -> Phase06Result<()> {
    let now = now_iso()?;
    for (index, line) in lines.iter().enumerate() {
        let (product_code, product_name, unit_id, unit_code, _) =
            product_snapshot(transaction, &context.company_id, &line.product_id)?;
        let quantity = line.quantity_scaled.abs();
        let notes = json!({ "locationId": line.warehouse_location_id }).to_string();
        transaction.execute(
            r#"
            INSERT INTO commercial_document_lines (
                id, company_id, document_id, product_id, warehouse_id, unit_id,
                line_number, product_code_snapshot, description_snapshot,
                unit_code_snapshot, quantity_scaled, unit_price_scaled,
                unit_cost_scaled, notes, created_at, created_by,
                updated_at, updated_by
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0,
                ?12, ?13, ?14, ?15, ?14, ?15
            )
            "#,
            params![
                new_id(),
                context.company_id,
                document_id,
                line.product_id,
                warehouse_id,
                unit_id,
                i64::try_from(index + 1).unwrap_or(i64::MAX),
                product_code,
                product_name,
                unit_code,
                quantity,
                line.unit_cost_scaled,
                notes,
                now,
                context.user_id
            ],
        )?;
    }
    Ok(())
}

pub(crate) fn post_document(
    transaction: &rusqlite::Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    document_id: &str,
    action: &str,
    reason: Option<&str>,
) -> Phase06Result<()> {
    let now = now_iso()?;
    let changed = transaction.execute(
        r#"
        UPDATE commercial_documents
        SET workflow_status='POSTED', posting_status='POSTED',
            posting_date=commercial_date, posted_at=?1, posted_by=?2,
            updated_at=?1, updated_by=?2, row_version=row_version+1
        WHERE id=?3 AND company_id=?4 AND posting_status='DRAFT'
        "#,
        params![now, context.user_id, document_id, context.company_id],
    )?;
    if changed != 1 {
        return Err(Phase06Error::immutable());
    }
    let row_version: i64 = transaction.query_row(
        "SELECT row_version FROM commercial_documents WHERE id=?1 AND company_id=?2",
        params![document_id, context.company_id],
        |row| row.get(0),
    )?;
    insert_status_history(
        transaction,
        context,
        document_id,
        Some("DRAFT"),
        "POSTED",
        reason,
        row_version,
    )?;
    let details = reason.map(|reason| json!({ "reason": reason }).to_string());
    audit(
        transaction,
        context,
        action,
        "commercial_document",
        document_id,
        details.as_deref(),
    )?;
    Ok(())
}

fn post_receipt_lines(
    transaction: &Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    document_id: &str,
    warehouse_id: &str,
    commercial_date: &str,
) -> Phase06Result<()> {
    let mut statement = transaction.prepare(
        "SELECT id, product_id, quantity_scaled, COALESCE(unit_cost_scaled, unit_price_scaled)
         FROM commercial_document_lines
         WHERE document_id = ?1 AND company_id = ?2
         ORDER BY line_number",
    )?;
    let lines = statement
        .query_map(params![document_id, context.company_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);

    for (index, (line_id, product_id, quantity, cost)) in lines.into_iter().enumerate() {
        apply_movement(
            transaction,
            context,
            MovementSpec {
                product_id: &product_id,
                warehouse_id,
                location_id: None,
                source_document_id: Some(document_id),
                source_line_id: Some(&line_id),
                movement_type: "PURCHASE_RECEIPT",
                business_date: commercial_date,
                quantity_delta: quantity,
                inbound_cost: Some(cost),
                recalculate_average: true,
                posting_event_key: &format!("purchase-receipt:{document_id}:{}", index + 1),
                transfer_group_id: None,
                notes: None,
                allow_negative: false,
            },
        )?;
    }
    Ok(())
}

fn update_order_receipt_status(
    transaction: &Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    order_id: &str,
) -> Phase06Result<()> {
    let remaining: i64 = transaction.query_row(
        "SELECT COUNT(*)
         FROM commercial_document_lines source_line
         WHERE source_line.document_id = ?1 AND source_line.company_id = ?2
           AND (
             SELECT COALESCE(SUM(link.transformed_quantity_scaled), 0)
             FROM document_line_links link
             JOIN commercial_document_lines target_line ON target_line.id = link.target_line_id
             JOIN commercial_documents target_document ON target_document.id = target_line.document_id
             WHERE link.company_id = source_line.company_id
               AND link.source_line_id = source_line.id
               AND link.transformation_type = 'PURCHASE_ORDER_TO_RECEIPT'
               AND target_document.posting_status = 'POSTED'
           ) < source_line.quantity_scaled",
        params![order_id, context.company_id],
        |row| row.get(0),
    )?;
    if remaining == 0 {
        let now = now_iso()?;
        transaction.execute(
            "UPDATE commercial_documents
             SET workflow_status = 'CLOSED', updated_at = ?1, updated_by = ?2,
                 row_version = row_version + 1
             WHERE id = ?3 AND company_id = ?4
               AND workflow_status IN ('CONFIRMED', 'ON_HOLD')",
            params![now, context.user_id, order_id, context.company_id],
        )?;
        transaction.execute(
            "INSERT INTO document_status_history (
                id, company_id, document_id, old_status, new_status,
                changed_at, changed_by, reason, row_version_snapshot
             )
             SELECT ?1, ?2, id, 'CONFIRMED', 'CLOSED', ?3, ?4,
                    'All ordered quantities received', row_version
             FROM commercial_documents WHERE id = ?5 AND company_id = ?2",
            params![new_id(), context.company_id, now, context.user_id, order_id],
        )?;
    }
    Ok(())
}

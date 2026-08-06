fn open_balances(
    connection: &Connection,
    company_id: &str,
    document_type: &str,
) -> Phase08Result<Vec<OpenBalanceRow>> {
    let mut statement = connection.prepare(
        "SELECT id,document_number,document_type,commercial_date,due_date,total_ttc_minor FROM commercial_documents WHERE company_id=?1 AND document_type=?2 AND posting_status='POSTED' ORDER BY commercial_date,document_number",
    )?;
    let rows = statement
        .query_map(params![company_id, document_type], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = Vec::new();
    for (id, number, kind, date, due, total) in rows {
        let open = document_open(connection, company_id, &id, total)?;
        if open > 0 {
            result.push(OpenBalanceRow {
                document_id: id,
                document_number: number,
                document_type: kind,
                commercial_date: date,
                due_date: due,
                total_minor: total,
                allocated_minor: total.checked_sub(open).ok_or_else(Phase08Error::internal)?,
                open_minor: open,
            });
        }
    }
    Ok(result)
}

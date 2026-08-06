fn ledger(
    connection: &Connection,
    company_id: &str,
    account_id: Option<&str>,
) -> Phase08Result<Vec<LedgerRow>> {
    let sql = if account_id.is_some() {
        r#"SELECT je.id,je.entry_number,je.entry_date,a.id,a.code,jel.description,jel.debit_minor,jel.credit_minor
           FROM journal_entry_lines jel JOIN journal_entries je ON je.id=jel.journal_entry_id AND je.company_id=jel.company_id
           JOIN accounts a ON a.id=jel.account_id AND a.company_id=jel.company_id
           WHERE jel.company_id=?1 AND je.status='POSTED' AND a.id=?2
           ORDER BY je.entry_date,je.entry_number,jel.line_number"#
    } else {
        r#"SELECT je.id,je.entry_number,je.entry_date,a.id,a.code,jel.description,jel.debit_minor,jel.credit_minor
           FROM journal_entry_lines jel JOIN journal_entries je ON je.id=jel.journal_entry_id AND je.company_id=jel.company_id
           JOIN accounts a ON a.id=jel.account_id AND a.company_id=jel.company_id
           WHERE jel.company_id=?1 AND je.status='POSTED'
           ORDER BY a.code,je.entry_date,je.entry_number,jel.line_number"#
    };
    let mut statement = connection.prepare(sql)?;
    let mut rows = if let Some(account) = account_id {
        statement.query(params![company_id, account])?
    } else {
        statement.query([company_id])?
    };
    let mut running = 0_i64;
    let mut current_account = String::new();
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get(3)?;
        if id != current_account {
            current_account = id.clone();
            running = 0;
        }
        let debit: i64 = row.get(6)?;
        let credit: i64 = row.get(7)?;
        let delta = debit.checked_sub(credit).ok_or_else(Phase08Error::internal)?;
        running = running.checked_add(delta).ok_or_else(Phase08Error::internal)?;
        result.push(LedgerRow {
            journal_entry_id: row.get(0)?,
            entry_number: row.get(1)?,
            entry_date: row.get(2)?,
            account_id: id,
            account_code: row.get(4)?,
            description: row.get(5)?,
            debit_minor: debit,
            credit_minor: credit,
            running_balance_minor: running,
        });
    }
    Ok(result)
}

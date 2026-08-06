fn trial_balance(connection: &Connection, company_id: &str) -> Phase08Result<Vec<TrialBalanceRow>> {
    let mut statement = connection.prepare(
        r#"SELECT a.id,a.code,a.name_ar,
                  COALESCE(SUM(CASE WHEN je.status='POSTED' THEN jel.debit_minor ELSE 0 END),0),
                  COALESCE(SUM(CASE WHEN je.status='POSTED' THEN jel.credit_minor ELSE 0 END),0)
           FROM accounts a
           LEFT JOIN journal_entry_lines jel ON jel.account_id=a.id AND jel.company_id=a.company_id
           LEFT JOIN journal_entries je ON je.id=jel.journal_entry_id AND je.company_id=jel.company_id AND je.status='POSTED'
           WHERE a.company_id=?1
           GROUP BY a.id,a.code,a.name_ar
           HAVING COALESCE(SUM(CASE WHEN je.status='POSTED' THEN jel.debit_minor ELSE 0 END),0)<>0
               OR COALESCE(SUM(CASE WHEN je.status='POSTED' THEN jel.credit_minor ELSE 0 END),0)<>0
           ORDER BY a.code"#,
    )?;
    let rows = statement.query_map([company_id], |row| {
        let debit: i64 = row.get(3)?;
        let credit: i64 = row.get(4)?;
        Ok(TrialBalanceRow {
            account_id: row.get(0)?,
            account_code: row.get(1)?,
            account_name_ar: row.get(2)?,
            debit_minor: debit,
            credit_minor: credit,
            balance_minor: debit.saturating_sub(credit),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

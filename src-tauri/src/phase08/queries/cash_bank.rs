impl Phase08Service {
    pub fn get_cash_bank_register(&self, _: ()) -> Phase08Result<Vec<LedgerRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                r#"SELECT DISTINCT a.id FROM accounts a
                   WHERE a.company_id=?1 AND (
                     EXISTS (SELECT 1 FROM accounting_account_roles ar
                       WHERE ar.company_id=a.company_id AND ar.account_id=a.id AND ar.role_code IN ('CASH','BANK'))
                     OR EXISTS (SELECT 1 FROM payment_method_accounting pma
                       WHERE pma.company_id=a.company_id AND pma.account_id=a.id)
                     OR EXISTS (SELECT 1 FROM payment_method_accounting pma
                       JOIN accounting_account_roles ar ON ar.company_id=pma.company_id AND ar.role_code=pma.account_role_code
                       WHERE pma.company_id=a.company_id AND ar.account_id=a.id)
                   ) ORDER BY a.code"#,
            )?;
            let ids = statement
                .query_map([&context.company_id], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            let mut all = Vec::new();
            for account_id in ids {
                all.extend(ledger(connection, &context.company_id, Some(&account_id))?);
            }
            all.sort_by(|left, right| {
                (&left.entry_date, &left.entry_number).cmp(&(&right.entry_date, &right.entry_number))
            });
            Ok(all)
        })
    }

}

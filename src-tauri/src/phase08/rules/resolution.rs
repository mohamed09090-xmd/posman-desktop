fn resolve_legacy_rule_accounts(
    tx: &Transaction<'_>,
    company: &str,
    input: &PostingRuleInput,
) -> Phase08Result<(String, String)> {
    fn resolve(
        tx: &Transaction<'_>,
        company: &str,
        input: &PostingRuleInput,
        side: &str,
    ) -> Phase08Result<String> {
        let line = input.lines.iter().find(|line| line.side == side).ok_or_else(|| {
            Phase08Error::new(
                "POSTING_RULE_LINES_REQUIRED",
                "A posting rule requires at least one debit and one credit line.",
                false,
            )
        })?;
        let account_id = if let Some(account_id) = line.account_id.as_deref() {
            account_id.to_owned()
        } else {
            tx.query_row(
                "SELECT account_id FROM accounting_account_roles WHERE company_id=?1 AND role_code=?2",
                params![company, line.account_role_code.as_deref().unwrap_or("")],
                |row| row.get::<_, String>(0),
            ).optional()?.ok_or_else(|| Phase08Error::new(
                "ACCOUNT_ROLE_MISSING",
                "Map every account role used by the posting rule.",
                false,
            ))?
        };
        require_active_postable_account(tx, company, &account_id)?;
        Ok(account_id)
    }
    Ok((resolve(tx, company, input, "DEBIT")?, resolve(tx, company, input, "CREDIT")?))
}

impl Phase08Service {
    pub fn get_trial_balance(&self, _: ()) -> Phase08Result<Vec<TrialBalanceRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| trial_balance(connection, &context.company_id))
    }

    pub fn get_general_ledger(&self, _: ()) -> Phase08Result<Vec<LedgerRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| ledger(connection, &context.company_id, None))
    }

    pub fn get_account_ledger(&self, account_id: String) -> Phase08Result<Vec<LedgerRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| ledger(connection, &context.company_id, Some(&account_id)))
    }

}

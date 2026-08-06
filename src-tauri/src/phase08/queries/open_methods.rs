impl Phase08Service {
    pub fn get_open_receivables(&self, _: ()) -> Phase08Result<Vec<OpenBalanceRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| open_balances(connection, &context.company_id, "SALES_INVOICE"))
    }

    pub fn get_open_payables(&self, _: ()) -> Phase08Result<Vec<OpenBalanceRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| open_balances(connection, &context.company_id, "PURCHASE_INVOICE"))
    }
}

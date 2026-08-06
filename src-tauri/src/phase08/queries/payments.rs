impl Phase08Service {
    pub fn list_payments(&self, _: ()) -> Phase08Result<Vec<PaymentResult>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                "SELECT id,journal_entry_id,amount_minor FROM payments WHERE company_id=?1 ORDER BY commercial_date,id",
            )?;
            let mut rows = statement.query([&context.company_id])?;
            let mut result = Vec::new();
            while let Some(row) = rows.next()? {
                let payment_id: String = row.get(0)?;
                let journal_entry_id: Option<String> = row.get(1)?;
                let amount_minor: i64 = row.get(2)?;
                let unallocated = payment_unallocated(
                    connection,
                    &context.company_id,
                    &payment_id,
                    amount_minor,
                )?;
                result.push(PaymentResult {
                    payment_id,
                    journal_entry_id: journal_entry_id.unwrap_or_default(),
                    amount_minor,
                    unallocated_minor: unallocated,
                    replayed: false,
                });
            }
            Ok(result)
        })
    }

}

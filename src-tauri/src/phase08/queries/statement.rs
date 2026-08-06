impl Phase08Service {
    pub fn get_partner_statement(&self, partner_id: String) -> Phase08Result<Vec<StatementRow>> {
        let context = self.context(Some("accounting.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                r#"SELECT je.entry_date,je.source_event_type,je.source_event_id,jel.debit_minor,jel.credit_minor
                   FROM journal_entry_lines jel
                   JOIN journal_entries je ON je.id=jel.journal_entry_id AND je.company_id=jel.company_id
                   WHERE jel.company_id=?1 AND jel.partner_id=?2 AND je.status='POSTED'
                   ORDER BY je.entry_date,je.entry_number,jel.line_number"#,
            )?;
            let mut rows = statement.query(params![context.company_id, partner_id])?;
            let mut running = 0_i64;
            let mut result = Vec::new();
            while let Some(row) = rows.next()? {
                let debit: i64 = row.get(3)?;
                let credit: i64 = row.get(4)?;
                let delta = debit.checked_sub(credit).ok_or_else(Phase08Error::internal)?;
                running = running.checked_add(delta).ok_or_else(Phase08Error::internal)?;
                result.push(StatementRow {
                    event_date: row.get(0)?,
                    source_type: row.get(1)?,
                    source_id: row.get(2)?,
                    debit_minor: debit,
                    credit_minor: credit,
                    running_balance_minor: running,
                });
            }
            Ok(result)
        })
    }

}

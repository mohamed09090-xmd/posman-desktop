impl Phase08Service {
    pub fn save_posting_rule(&self, input: PostingRuleInput) -> Phase08Result<EntityVersion> {
        let context=self.context(Some("accounting.configure"))?;
        validate_rule(&input)?;
        self.immediate(|tx| {
            require_company_row(tx,"accounting_journals",&input.accounting_journal_id,&context.company_id,"JOURNAL_NOT_FOUND")?;
            let now=now_iso()?;
            let id=input.id.clone().unwrap_or_else(new_id);
            let (legacy_debit, legacy_credit) = resolve_legacy_rule_accounts(tx, &context.company_id, &input)?;
            if input.id.is_some() {
                let version=input.row_version.ok_or_else(|| Phase08Error::validation("Posting rule row version is required."))?;
                let used: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM journal_entries WHERE company_id=?1 AND posting_rule_id=?2",
                    params![context.company_id, id],
                    |row| row.get(0),
                )?;
                if used != 0 {
                    return Err(Phase08Error::new(
                        "POSTING_RULE_IMMUTABLE",
                        "A posting rule already used by a journal entry cannot be changed; create a new dated rule.",
                        false,
                    ));
                }
                let changed=tx.execute("UPDATE posting_rules SET accounting_journal_id=?1,debit_account_id=?2,credit_account_id=?3,code=?4,source_event_type=?5,priority=?6,valid_from=?7,valid_to=?8,is_active=?9,updated_at=?10,updated_by=?11,row_version=row_version+1 WHERE id=?12 AND company_id=?13 AND row_version=?14",
                    params![input.accounting_journal_id,legacy_debit,legacy_credit,input.code,input.source_event_type,input.priority,input.valid_from,input.valid_to,boolean(input.is_active),now,context.user_id,id,context.company_id,version])?;
                if changed!=1 { return Err(Phase08Error::new("POSTING_RULE_CONFLICT","The posting rule changed; reload and retry.",true)); }
                tx.execute("DELETE FROM posting_rule_lines WHERE posting_rule_id=?1",[&id])?;
            } else {
                tx.execute("INSERT INTO posting_rules(id,company_id,accounting_journal_id,debit_account_id,credit_account_id,code,source_event_type,condition_expression,priority,valid_from,valid_to,is_active,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,?2,?3,?4,?5,?6,?7,NULL,?8,?9,?10,?11,?12,?13,?12,?13,1)",
                    params![id,context.company_id,input.accounting_journal_id,legacy_debit,legacy_credit,input.code,input.source_event_type,input.priority,input.valid_from,input.valid_to,boolean(input.is_active),now,context.user_id])?;
            }
            for line in &input.lines {
                if let Some(account)=line.account_id.as_deref() { require_active_postable_account(tx,&context.company_id,account)?; }
                tx.execute("INSERT INTO posting_rule_lines(id,company_id,posting_rule_id,line_number,side,account_id,account_role_code,amount_component,description_ar,description_fr,partner_dimension,product_dimension,created_at,created_by,updated_at,updated_by,row_version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?13,?14,1)",
                    params![new_id(),context.company_id,id,line.line_number,line.side,line.account_id,line.account_role_code,line.amount_component,line.description_ar,line.description_fr,boolean(line.partner_dimension),boolean(line.product_dimension),now,context.user_id])?;
            }
            Ok(EntityVersion{id,row_version:input.row_version.unwrap_or(0)+1})
        })
    }

}

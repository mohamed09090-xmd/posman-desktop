impl Phase08Service {
    pub fn validate_posting_configuration(&self, _: ()) -> Phase08Result<Vec<String>> {
        let context=self.context(Some("accounting.read"))?;
        self.read(|c| {
            let mut issues=Vec::new();
            let enabled:i64=c.query_row("SELECT COALESCE((SELECT is_enabled FROM accounting_setups WHERE company_id=?1),0)",[&context.company_id],|r|r.get(0))?;
            if enabled==0 { issues.push("ACCOUNTING_DISABLED".to_owned()); }
            let missing:i64=c.query_row("SELECT COUNT(*) FROM accounting_account_roles r LEFT JOIN accounts a ON a.id=r.account_id AND a.company_id=r.company_id WHERE r.company_id=?1 AND (a.id IS NULL OR a.is_active=0 OR a.allow_posting=0)",[&context.company_id],|r|r.get(0))?;
            if missing>0 { issues.push("INACTIVE_OR_MISSING_ROLE_ACCOUNT".to_owned()); }
            let rules:i64=c.query_row("SELECT COUNT(*) FROM posting_rules pr WHERE pr.company_id=?1 AND pr.is_active=1 AND (SELECT COUNT(*) FROM posting_rule_lines l WHERE l.posting_rule_id=pr.id)<2",[&context.company_id],|r|r.get(0))?;
            if rules>0 { issues.push("INCOMPLETE_POSTING_RULE".to_owned()); }
            for event_type in [
                "SALES_INVOICE", "PURCHASE_INVOICE", "PURCHASE_RECEIVE_INVOICE",
                "DELIVERY_COGS", "DIRECT_SALE", "SALES_RETURN", "PURCHASE_RETURN",
                "CUSTOMER_RECEIPT", "SUPPLIER_PAYMENT", "CUSTOMER_RECEIPT_REVERSAL",
                "SUPPLIER_PAYMENT_REVERSAL",
            ] {
                let configured:i64=c.query_row(
                    "SELECT COUNT(*) FROM posting_rules WHERE company_id=?1 AND source_event_type=?2 AND is_active=1",
                    params![context.company_id,event_type],
                    |r|r.get(0),
                )?;
                if configured==0 { issues.push(format!("{event_type}: POSTING_RULE_MISSING")); }
            }
            Ok(issues)
        })
    }

}

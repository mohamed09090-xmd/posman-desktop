fn balanced_sales_posting() {
    let mut db=fixture(); let result=post(&mut db,&event("SALES_INVOICE","sale-1",&[("DOCUMENT_HT",1000),("DOCUMENT_TAX",190),("DOCUMENT_TTC",1190)])); assert_balanced(&db,&result.journal_entry_id,1190);
}
#[test]
fn balanced_purchase_posting() {
    let mut db=fixture(); let result=post(&mut db,&event("PURCHASE_INVOICE","purchase-1",&[("DOCUMENT_HT",2000),("DOCUMENT_TAX",380),("DOCUMENT_TTC",2380)])); assert_balanced(&db,&result.journal_entry_id,2380);
}
#[test]
fn tax_lines_are_separate() {
    let mut db=fixture(); let result=post(&mut db,&event("SALES_INVOICE","sale-tax",&[("DOCUMENT_HT",1000),("DOCUMENT_TAX",190),("DOCUMENT_TTC",1190)])); let count:i64=db.query_row("SELECT COUNT(*) FROM journal_entry_lines WHERE journal_entry_id=?1 AND credit_minor IN (1000,190)",[result.journal_entry_id],|r|r.get(0)).unwrap(); assert_eq!(count,2);
}
#[test]
fn delivery_cogs_posting() { let mut db=fixture(); let result=post(&mut db,&event("DELIVERY_COGS","delivery-1",&[("STOCK_COST",700)])); assert_balanced(&db,&result.journal_entry_id,700); }
#[test]
fn direct_sale_compound_posting() { let mut db=fixture(); let result=post(&mut db,&event("DIRECT_SALE","direct-1",&[("DOCUMENT_HT",1000),("DOCUMENT_TAX",190),("DOCUMENT_TTC",1190),("STOCK_COST",700)])); assert_balanced(&db,&result.journal_entry_id,1890); }
#[test]
fn purchase_receive_invoice_integration_posting() { let mut db=fixture(); let purchase=post(&mut db,&event("PURCHASE_RECEIVE_INVOICE","receive-invoice-1",&[("DOCUMENT_HT",2000),("DOCUMENT_TAX",380),("DOCUMENT_TTC",2380),("STOCK_COST",2000)])); assert_balanced(&db,&purchase.journal_entry_id,2380); }
#[test]
fn sales_return_credit_compensation() { let mut db=fixture(); let result=post(&mut db,&event("SALES_RETURN","sales-return-1",&[("DOCUMENT_HT",1000),("DOCUMENT_TAX",190),("DOCUMENT_TTC",1190),("STOCK_COST",700)])); assert_balanced(&db,&result.journal_entry_id,1890); }
#[test]
fn purchase_return_compensation() { let mut db=fixture(); let result=post(&mut db,&event("PURCHASE_RETURN","purchase-return-1",&[("DOCUMENT_HT",1000),("DOCUMENT_TAX",190),("DOCUMENT_TTC",1190),("STOCK_COST",1000)])); assert_balanced(&db,&result.journal_entry_id,1190); }
#[test]
fn same_idempotency_key_and_hash_replays_without_duplicate() { let mut db=fixture(); let request=event("SALES_INVOICE","idem-same",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)]); let first=post(&mut db,&request); let second=post(&mut db,&request); assert_eq!(first.journal_entry_id,second.journal_entry_id); assert!(second.replayed); let count:i64=db.query_row("SELECT COUNT(*) FROM journal_entries WHERE source_event_id='idem-same'",[],|r|r.get(0)).unwrap(); assert_eq!(count,1); }
#[test]
fn same_key_different_hash_is_rejected() { let mut db=fixture(); let request=event("SALES_INVOICE","idem-conflict",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)]); post(&mut db,&request); let mut changed=request.clone(); changed.payload.components_minor.insert("DOCUMENT_HT".to_owned(),101); rehash(&mut changed); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&changed).unwrap_err(); assert_eq!(error.code,"ACCOUNTING_IDEMPOTENCY_CONFLICT"); }
#[test]
fn missing_posting_rule_is_actionable() { let mut db=fixture(); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&event("UNKNOWN","missing-rule",&[("X",1)])).unwrap_err(); assert_eq!(error.code,"POSTING_RULE_MISSING"); }
#[test]
fn ambiguous_posting_rules_are_rejected() { let mut db=fixture(); db.execute("INSERT INTO posting_rules(id,company_id,accounting_journal_id,debit_account_id,credit_account_id,code,source_event_type,priority,valid_from,is_active,created_at,updated_at) SELECT 'rule-company-1-SALES-INVOICE-ALT',company_id,accounting_journal_id,debit_account_id,credit_account_id,'SALES-INVOICE-ALT',source_event_type,priority,valid_from,is_active,created_at,updated_at FROM posting_rules WHERE id='rule-company-1-SALES_INVOICE'",[]).unwrap(); db.execute("INSERT INTO posting_rule_lines(id,company_id,posting_rule_id,line_number,side,account_id,account_role_code,amount_component,description_ar,partner_dimension,product_dimension,created_at,updated_at) SELECT 'alt-'||id,company_id,'rule-company-1-SALES-INVOICE-ALT',line_number,side,account_id,account_role_code,amount_component,description_ar,partner_dimension,product_dimension,created_at,updated_at FROM posting_rule_lines WHERE posting_rule_id='rule-company-1-SALES_INVOICE'",[]).unwrap(); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&event("SALES_INVOICE","ambiguous",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)])).unwrap_err(); assert_eq!(error.code,"POSTING_RULE_AMBIGUOUS"); }
#[test]
fn inactive_mapped_account_is_rejected() { let mut db=fixture(); db.execute("UPDATE accounts SET is_active=0 WHERE id='account-company-1-CUSTOMER_RECEIVABLE'",[]).unwrap(); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&event("SALES_INVOICE","inactive",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)])).unwrap_err(); assert_eq!(error.code,"ACCOUNT_INACTIVE"); }
#[test]
fn closed_fiscal_period_is_rejected() { let mut db=fixture(); db.execute("UPDATE fiscal_periods SET status='CLOSED' WHERE id='period-company-1'",[]).unwrap(); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&event("SALES_INVOICE","closed",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)])).unwrap_err(); assert_eq!(error.code,"FISCAL_PERIOD_CLOSED"); }
#[test]
fn unbalanced_generated_entry_is_rejected() { let mut db=fixture(); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&event("SALES_INVOICE","unbalanced",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",18),("DOCUMENT_TTC",119)])).unwrap_err(); assert_eq!(error.code,"UNBALANCED_GENERATED_ENTRY"); }
#[test]
fn mid_posting_failure_rolls_back_header_and_lines() { let mut db=fixture(); let mut request=event("SALES_INVOICE","injected",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)]); request.payload.inject_failure_after_header=true; rehash(&mut request); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&request).unwrap_err(); drop(tx); let entries:i64=db.query_row("SELECT COUNT(*) FROM journal_entries WHERE source_event_id='injected'",[],|r|r.get(0)).unwrap(); let lines:i64=db.query_row("SELECT COUNT(*) FROM journal_entry_lines",[],|r|r.get(0)).unwrap(); assert_eq!((entries,lines),(0,0)); assert_eq!(error.code,"INJECTED_POSTING_FAILURE"); }
#[test]
fn failed_attempt_survives_without_partial_journal() { let mut db=fixture(); let mut request=event("SALES_INVOICE","failed-attempt",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)]); request.payload.inject_failure_after_header=true; rehash(&mut request); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-1"),&request).unwrap_err(); drop(tx); record_failed_attempt_after_rollback(&mut db,&context("company-1"),&request,&error).unwrap(); let failed:i64=db.query_row("SELECT COUNT(*) FROM posting_attempts WHERE source_event_id='failed-attempt' AND status='FAILED'",[],|r|r.get(0)).unwrap(); let journals:i64=db.query_row("SELECT COUNT(*) FROM journal_entries WHERE source_event_id='failed-attempt'",[],|r|r.get(0)).unwrap(); assert_eq!((failed,journals),(1,0)); }
#[test]
fn posted_journal_and_lines_are_immutable() { let mut db=fixture(); let result=post(&mut db,&event("DELIVERY_COGS","immutable",&[("STOCK_COST",700)])); assert!(db.execute("UPDATE journal_entries SET memo='changed' WHERE id=?1",[&result.journal_entry_id]).is_err()); assert!(db.execute("DELETE FROM journal_entry_lines WHERE journal_entry_id=?1",[&result.journal_entry_id]).is_err()); }
#[test]
fn linked_balanced_reversal() {
    let mut db = fixture();
    let original = post(
        &mut db,
        &event(
            "SALES_INVOICE",
            "reverse-original",
            &[("DOCUMENT_HT", 100), ("DOCUMENT_TAX", 19), ("DOCUMENT_TTC", 119)],
        ),
    );
    let tx = db.transaction().unwrap();
    let reversal = reverse_entry_in_tx(
        &tx,
        &context("company-1"),
        &original.journal_entry_id,
        "2026-08-06",
        "Correction with linked reversing entry",
    )
    .unwrap();
    tx.commit().unwrap();
    assert_ne!(original.journal_entry_id, reversal.id);
    assert_balanced(&db, &reversal.id, 119);
    let linked: String = db
        .query_row(
            "SELECT reversal_of_entry_id FROM journal_entries WHERE id=?1",
            [&reversal.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(linked, original.journal_entry_id);
}

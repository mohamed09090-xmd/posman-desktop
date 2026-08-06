fn business_source_stock_and_accounting_failure_roll_back_together() {
    let mut db = fixture();
    let mut request = event(
        "DIRECT_SALE",
        "business-rollback",
        &[
            ("DOCUMENT_HT", 1_000),
            ("DOCUMENT_TAX", 190),
            ("DOCUMENT_TTC", 1_190),
            ("STOCK_COST", 600),
        ],
    );
    request.payload.source_document_id = Some("document-business-rollback".to_owned());
    request.payload.inject_failure_after_header = true;
    rehash(&mut request);

    let transaction = db.transaction_with_behavior(TransactionBehavior::Immediate).unwrap();
    transaction.execute(
        "INSERT INTO commercial_documents(id,company_id,fiscal_year_id,fiscal_period_id,partner_id,warehouse_id,document_type,document_number,workflow_status,posting_status,commercial_date,total_ht_minor,total_tax_minor,total_ttc_minor,created_at,created_by,updated_at,updated_by) VALUES ('document-business-rollback','company-1','fy-company-1','period-company-1','partner-company-1','warehouse-company-1','SALES_INVOICE','FAC-ROLLBACK','DRAFT','DRAFT','2026-08-06',1000,190,1190,?1,'user-company-1',?1,'user-company-1')",
        [NOW],
    ).unwrap();
    transaction.execute(
        "INSERT INTO commercial_document_lines(id,company_id,document_id,product_id,warehouse_id,unit_id,line_number,product_code_snapshot,description_snapshot,unit_code_snapshot,quantity_scaled,unit_cost_scaled,line_ht_minor,line_tax_minor,line_ttc_minor,created_at,created_by,updated_at,updated_by) VALUES ('line-business-rollback','company-1','document-business-rollback','product-company-1','warehouse-company-1','unit-company-1',1,'ITEM','Article','PC',1000000,6000000,1000,190,1190,?1,'user-company-1',?1,'user-company-1')",
        [NOW],
    ).unwrap();
    transaction.execute(
        "INSERT INTO stock_movements(id,company_id,product_id,warehouse_id,source_document_id,source_line_id,movement_type,business_date,occurred_at,quantity_delta_scaled,quantity_before_scaled,quantity_after_scaled,unit_cost_scaled,average_cost_before_scaled,average_cost_after_scaled,extended_cost_minor,posting_event_key,created_by) VALUES ('movement-business-rollback','company-1','product-company-1','warehouse-company-1','document-business-rollback','line-business-rollback','SALES_DELIVERY','2026-08-06',?1,-1000000,1000000,0,6000000,6000000,6000000,600,'movement-business-rollback','user-company-1')",
        [NOW],
    ).unwrap();
    transaction.execute(
        "UPDATE commercial_documents SET workflow_status='POSTED',posting_status='POSTED',posting_date='2026-08-06',posted_at=?1,posted_by='user-company-1',updated_at=?1,row_version=row_version+1 WHERE id='document-business-rollback'",
        [NOW],
    ).unwrap();

    let error = post_source_event_in_tx(&transaction, &context("company-1"), &request).unwrap_err();
    drop(transaction);
    record_failed_attempt_after_rollback(&mut db, &context("company-1"), &request, &error).unwrap();

    let documents: i64 = db.query_row(
        "SELECT COUNT(*) FROM commercial_documents WHERE id='document-business-rollback'",
        [],
        |row| row.get(0),
    ).unwrap();
    let movements: i64 = db.query_row(
        "SELECT COUNT(*) FROM stock_movements WHERE id='movement-business-rollback'",
        [],
        |row| row.get(0),
    ).unwrap();
    let journals: i64 = db.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE source_event_id='business-rollback'",
        [],
        |row| row.get(0),
    ).unwrap();
    let failed: i64 = db.query_row(
        "SELECT COUNT(*) FROM posting_attempts WHERE source_event_id='business-rollback' AND status='FAILED' AND error_message IS NULL",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!((documents, movements, journals, failed), (0, 0, 0, 1));
}

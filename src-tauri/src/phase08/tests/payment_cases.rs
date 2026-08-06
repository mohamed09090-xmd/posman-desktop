fn payment_fixture(db: &mut Connection, amount: i64) -> String {
    let request=envelope(&format!("pay-{amount}"),PaymentInput{partner_id:"partner-company-1".to_owned(),payment_method_id:"method-company-1".to_owned(),commercial_date:"2026-08-06".to_owned(),amount_minor:amount,external_reference:None,notes:None});
    let payment_id=format!("payment-{amount}");
    let source=event("CUSTOMER_RECEIPT",&payment_id,&[("PAYMENT_AMOUNT",amount)]).payload;
    let tx=db.transaction().unwrap();
    post_payment_in_tx(&tx,&context("company-1"),&request,&payment_id,"RECEIPT",true,&source).unwrap();
    tx.commit().unwrap();
    let invoice_id = format!("invoice-{amount}");
    db.execute("INSERT INTO commercial_documents(id,company_id,fiscal_year_id,fiscal_period_id,partner_id,document_type,document_number,workflow_status,posting_status,commercial_date,total_ttc_minor,created_at,updated_at) VALUES (?1,'company-1','fy-company-1','period-company-1','partner-company-1','SALES_INVOICE',?1,'DRAFT','DRAFT','2026-08-06',1000,?2,?2)",params![invoice_id,NOW]).unwrap();
    db.execute("UPDATE commercial_documents SET workflow_status='POSTED',posting_status='POSTED',posted_at=?1,updated_at=?1,row_version=row_version+1 WHERE id=?2",params![NOW,format!("invoice-{amount}")]).unwrap();
    payment_id
}

#[test]
fn partial_payment_allocation() { let mut db=fixture(); let payment=payment_fixture(&mut db,1000); let request=envelope("alloc-partial",AllocationInput{payment_id:payment,document_id:"invoice-1000".to_owned(),amount_minor:400}); let tx=db.transaction().unwrap(); let result=allocate_payment_in_tx(&tx,&context("company-1"),&request).unwrap(); tx.commit().unwrap(); assert_eq!((result.payment_unallocated_minor,result.document_open_minor),(600,600)); }
#[test]
fn full_payment_allocation() { let mut db=fixture(); let payment=payment_fixture(&mut db,1000); let request=envelope("alloc-full",AllocationInput{payment_id:payment,document_id:"invoice-1000".to_owned(),amount_minor:1000}); let tx=db.transaction().unwrap(); let result=allocate_payment_in_tx(&tx,&context("company-1"),&request).unwrap(); tx.commit().unwrap(); assert_eq!((result.payment_unallocated_minor,result.document_open_minor),(0,0)); }
#[test]
fn over_allocation_is_rejected() { let mut db=fixture(); let payment=payment_fixture(&mut db,1000); let request=envelope("alloc-over",AllocationInput{payment_id:payment,document_id:"invoice-1000".to_owned(),amount_minor:1001}); let tx=db.transaction().unwrap(); let error=allocate_payment_in_tx(&tx,&context("company-1"),&request).unwrap_err(); assert_eq!(error.code,"OVER_ALLOCATION"); }
#[test]
fn allocation_reversal_is_compensating_and_append_only() { let mut db=fixture(); let payment=payment_fixture(&mut db,1000); let request=envelope("alloc-reverse",AllocationInput{payment_id:payment,document_id:"invoice-1000".to_owned(),amount_minor:400}); let tx=db.transaction().unwrap(); let allocation=allocate_payment_in_tx(&tx,&context("company-1"),&request).unwrap(); tx.commit().unwrap(); let reverse=envelope("alloc-reversal",ReverseAllocationInput{allocation_id:allocation.allocation_id,reason:"Correction".to_owned()}); let tx=db.transaction().unwrap(); let result=reverse_allocation_in_tx(&tx,&context("company-1"),&reverse).unwrap(); tx.commit().unwrap(); assert_eq!((result.payment_unallocated_minor,result.document_open_minor),(1000,1000)); let count:i64=db.query_row("SELECT COUNT(*) FROM payment_allocations WHERE payment_id=?1",[result.payment_id],|r|r.get(0)).unwrap(); assert_eq!(count,2); }
#[test]
fn company_scope_isolation() { let mut db=fixture(); let tx=db.transaction().unwrap(); let error=post_source_event_in_tx(&tx,&context("company-2"),&event("SALES_INVOICE","other-company",&[("DOCUMENT_HT",100),("DOCUMENT_TAX",19),("DOCUMENT_TTC",119)])).unwrap_err(); assert_eq!(error.code,"POSTING_RULE_MISSING"); let count:i64=tx.query_row("SELECT COUNT(*) FROM journal_entries WHERE company_id='company-1'",[],|r|r.get(0)).unwrap(); assert_eq!(count,0); }

#[test]

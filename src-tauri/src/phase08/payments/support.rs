fn payment_reversal_source(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    payment_id: &str,
    reversal_id: &str,
    reversal_date: &str,
    reason: &str,
) -> Phase08Result<(SourceEventRequest, i64, String, String, String, String, String)> {
    let payment = tx
        .query_row(
            "SELECT amount_minor,payment_kind,partner_id,payment_method_id,fiscal_year_id,fiscal_period_id,status FROM payments WHERE id=?1 AND company_id=?2 AND reversal_of_payment_id IS NULL",
            params![payment_id, context.company_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?, row.get::<_, String>(6)?)),
        )
        .optional()?
        .ok_or_else(|| Phase08Error::new("PAYMENT_NOT_FOUND", "Payment was not found.", false))?;
    if payment.6 != "POSTED" {
        return Err(Phase08Error::new(
            "PAYMENT_NOT_REVERSIBLE",
            "Only a posted payment can be reversed.",
            false,
        ));
    }
    let existing: i64 = tx.query_row(
        "SELECT COUNT(*) FROM payments WHERE reversal_of_payment_id=?1",
        [payment_id],
        |row| row.get(0),
    )?;
    if existing != 0 {
        return Err(Phase08Error::new(
            "PAYMENT_ALREADY_REVERSED",
            "The payment already has a compensating reversal.",
            false,
        ));
    }
    let effective_allocated = effective_payment_allocated(tx, &context.company_id, payment_id)?;
    if effective_allocated != 0 {
        return Err(Phase08Error::new(
            "PAYMENT_HAS_ALLOCATIONS",
            "Reverse payment allocations before reversing the payment.",
            false,
        ));
    }
    let (reversal_year_id, reversal_period_id) =
        super::posting::resolve_open_period(tx, &context.company_id, reversal_date)?;
    let event_type = if payment.1 == "RECEIPT" {
        "CUSTOMER_RECEIPT_REVERSAL"
    } else {
        "SUPPLIER_PAYMENT_REVERSAL"
    };
    let source = SourceEventRequest {
        source_event_type: event_type.to_owned(),
        source_event_id: format!("{event_type}:{payment_id}"),
        source_document_id: None,
        event_date: reversal_date.to_owned(),
        partner_id: Some(payment.2.clone()),
        product_id: None,
        payment_method_id: Some(payment.3.clone()),
        memo: Some(reason.to_owned()),
        components_minor: BTreeMap::from([("PAYMENT_AMOUNT".to_owned(), payment.0)]),
        inject_failure_after_header: false,
    };
    Ok((
        source,
        payment.0,
        payment.1,
        payment.2,
        payment.3,
        reversal_year_id,
        reversal_period_id,
    ))
}

fn authorize_payment_allocation(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    payment_id: &str,
) -> Phase08Result<()> {
    let kind = tx.query_row(
        "SELECT payment_kind FROM payments WHERE id=?1 AND company_id=?2",
        params![payment_id, context.company_id],
        |row| row.get::<_, String>(0),
    ).optional()?.ok_or_else(|| Phase08Error::new("PAYMENT_NOT_FOUND", "Payment was not found.", false))?;
    let permission = if kind == "RECEIPT" {
        "payment.receipt.allocate"
    } else if kind == "DISBURSEMENT" {
        "payment.disbursement.allocate"
    } else {
        return Err(Phase08Error::new("PAYMENT_NOT_ALLOCATABLE", "The payment kind cannot be allocated.", false));
    };
    crate::phase06::authorize_transaction(tx, context, permission)
        .map_err(|_| Phase08Error::permission())?;
    Ok(())
}

fn next_payment_number(
    tx: &Transaction<'_>,
    company_id: &str,
    kind: &str,
) -> Phase08Result<String> {
    let count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM payments WHERE company_id=?1 AND payment_kind=?2",
        params![company_id, kind],
        |row| row.get(0),
    )?;
    let prefix = if kind == "RECEIPT" { "RC" } else { "PY" };
    Ok(format!("{prefix}-{:06}", count + 1))
}

pub(crate) fn effective_payment_allocated(
    tx: &rusqlite::Connection,
    company_id: &str,
    payment_id: &str,
) -> Phase08Result<i64> {
    tx.query_row(
        r#"SELECT COALESCE(SUM(CASE allocation_status WHEN 'ACTIVE' THEN allocated_amount_minor ELSE -allocated_amount_minor END),0)
           FROM payment_allocations WHERE company_id=?1 AND payment_id=?2"#,
        params![company_id, payment_id],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

pub(crate) fn payment_unallocated(
    tx: &rusqlite::Connection,
    company_id: &str,
    payment_id: &str,
    payment_amount: i64,
) -> Phase08Result<i64> {
    payment_amount
        .checked_sub(effective_payment_allocated(tx, company_id, payment_id)?)
        .ok_or_else(Phase08Error::internal)
}

pub(crate) fn document_open(
    tx: &rusqlite::Connection,
    company_id: &str,
    document_id: &str,
    document_total: i64,
) -> Phase08Result<i64> {
    let allocated: i64 = tx.query_row(
        r#"SELECT COALESCE(SUM(CASE pa.allocation_status WHEN 'ACTIVE' THEN pa.allocated_amount_minor ELSE -pa.allocated_amount_minor END),0)
           FROM payment_allocations pa WHERE pa.company_id=?1 AND pa.document_id=?2"#,
        params![company_id, document_id],
        |row| row.get(0),
    )?;
    document_total
        .checked_sub(allocated)
        .ok_or_else(Phase08Error::internal)
}

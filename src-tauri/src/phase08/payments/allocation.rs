pub(crate) fn allocate_payment_in_tx(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    request: &Idempotent<AllocationInput>,
) -> Phase08Result<AllocationResult> {
    super::posting::validate_idempotent(request)?;
    if request.idempotency_key.trim().is_empty() || request.payload.amount_minor <= 0 {
        return Err(Phase08Error::validation(
            "Allocation idempotency key and a positive amount are required.",
        ));
    }
    if let Some(existing) = tx
        .query_row(
            "SELECT id,payment_id,document_id,allocated_amount_minor,request_hash_sha256 FROM payment_allocations WHERE company_id=?1 AND idempotency_key=?2",
            params![context.company_id, request.idempotency_key],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?, row.get::<_, Option<String>>(4)?)),
        )
        .optional()?
    {
        if existing.4.as_deref() != Some(request.request_hash_sha256.as_str()) {
            return Err(Phase08Error::new(
                "IDEMPOTENCY_CONFLICT",
                "The idempotency key was already used with different allocation data.",
                false,
            ));
        }
        authorize_payment_allocation(tx, context, &existing.1)?;
        let payment_amount: i64 = tx.query_row(
            "SELECT amount_minor FROM payments WHERE id=?1 AND company_id=?2",
            params![existing.1, context.company_id],
            |row| row.get(0),
        )?;
        let document_total: i64 = tx.query_row(
            "SELECT total_ttc_minor FROM commercial_documents WHERE id=?1 AND company_id=?2",
            params![existing.2, context.company_id],
            |row| row.get(0),
        )?;
        return Ok(AllocationResult {
            allocation_id: existing.0,
            payment_id: existing.1.clone(),
            document_id: existing.2.clone(),
            amount_minor: existing.3,
            payment_unallocated_minor: payment_unallocated(
                tx,
                &context.company_id,
                &existing.1,
                payment_amount,
            )?,
            document_open_minor: document_open(
                tx,
                &context.company_id,
                &existing.2,
                document_total,
            )?,
        });
    }
    let payment = tx
        .query_row(
            "SELECT partner_id,payment_kind,status,amount_minor FROM payments p WHERE p.id=?1 AND p.company_id=?2 AND p.reversal_of_payment_id IS NULL AND NOT EXISTS (SELECT 1 FROM payments reversal WHERE reversal.company_id=p.company_id AND reversal.reversal_of_payment_id=p.id)",
            params![request.payload.payment_id, context.company_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)),
        )
        .optional()?
        .ok_or_else(|| Phase08Error::new("PAYMENT_NOT_FOUND", "Payment was not found.", false))?;
    authorize_payment_allocation(tx, context, &request.payload.payment_id)?;
    if payment.2 != "POSTED" {
        return Err(Phase08Error::new(
            "PAYMENT_NOT_ALLOCATABLE",
            "Only posted payments can be allocated.",
            false,
        ));
    }
    let document = tx
        .query_row(
            "SELECT partner_id,document_type,posting_status,total_ttc_minor FROM commercial_documents WHERE id=?1 AND company_id=?2",
            params![request.payload.document_id, context.company_id],
            |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)),
        )
        .optional()?
        .ok_or_else(|| Phase08Error::new("DOCUMENT_NOT_FOUND", "Document was not found.", false))?;
    if document.0.as_deref() != Some(payment.0.as_str()) {
        return Err(Phase08Error::new(
            "PARTNER_SCOPE_MISMATCH",
            "The payment and document must belong to the same partner.",
            false,
        ));
    }
    let expected_type = if payment.1 == "RECEIPT" {
        "SALES_INVOICE"
    } else {
        "PURCHASE_INVOICE"
    };
    if document.1 != expected_type || document.2 != "POSTED" {
        return Err(Phase08Error::new(
            "DOCUMENT_NOT_ALLOCATABLE",
            "The selected document cannot receive this payment.",
            false,
        ));
    }
    let payment_remaining = payment_unallocated(
        tx,
        &context.company_id,
        &request.payload.payment_id,
        payment.3,
    )?;
    let document_remaining = document_open(
        tx,
        &context.company_id,
        &request.payload.document_id,
        document.3,
    )?;
    if request.payload.amount_minor > payment_remaining
        || request.payload.amount_minor > document_remaining
    {
        return Err(Phase08Error::new(
            "OVER_ALLOCATION",
            "The allocation exceeds the remaining payment or document balance.",
            false,
        ));
    }
    let id = new_id();
    tx.execute(
        "INSERT INTO payment_allocations(id,company_id,payment_id,document_id,reversal_of_allocation_id,allocated_amount_minor,allocation_status,allocated_at,allocated_by,idempotency_key,request_hash_sha256) VALUES (?1,?2,?3,?4,NULL,?5,'ACTIVE',?6,?7,?8,?9)",
        params![id, context.company_id, request.payload.payment_id, request.payload.document_id, request.payload.amount_minor, now_iso()?, context.user_id, request.idempotency_key, request.request_hash_sha256],
    )?;
    Ok(AllocationResult {
        allocation_id: id,
        payment_id: request.payload.payment_id.clone(),
        document_id: request.payload.document_id.clone(),
        amount_minor: request.payload.amount_minor,
        payment_unallocated_minor: payment_remaining - request.payload.amount_minor,
        document_open_minor: document_remaining - request.payload.amount_minor,
    })
}

pub(crate) fn reverse_allocation_in_tx(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    request: &Idempotent<ReverseAllocationInput>,
) -> Phase08Result<AllocationResult> {
    super::posting::validate_idempotent(request)?;
    if request.idempotency_key.trim().is_empty() || request.payload.reason.trim().is_empty() {
        return Err(Phase08Error::validation(
            "Allocation reversal idempotency key and reason are required.",
        ));
    }
    let original = tx
        .query_row(
            "SELECT payment_id,document_id,allocated_amount_minor,allocation_status FROM payment_allocations WHERE id=?1 AND company_id=?2",
            params![request.payload.allocation_id, context.company_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, String>(3)?)),
        )
        .optional()?
        .ok_or_else(|| Phase08Error::new("ALLOCATION_NOT_FOUND", "Allocation was not found.", false))?;
    authorize_payment_allocation(tx, context, &original.0)?;
    if original.3 != "ACTIVE" {
        return Err(Phase08Error::new(
            "ALLOCATION_ALREADY_REVERSED",
            "Only an active allocation can be reversed.",
            false,
        ));
    }
    let reversed: i64 = tx.query_row(
        "SELECT COUNT(*) FROM payment_allocations WHERE reversal_of_allocation_id=?1",
        [&request.payload.allocation_id],
        |row| row.get(0),
    )?;
    if reversed != 0 {
        return Err(Phase08Error::new(
            "ALLOCATION_ALREADY_REVERSED",
            "The allocation already has a compensating reversal.",
            false,
        ));
    }
    let id = new_id();
    tx.execute(
        "INSERT INTO payment_allocations(id,company_id,payment_id,document_id,reversal_of_allocation_id,allocated_amount_minor,allocation_status,allocated_at,allocated_by,idempotency_key,request_hash_sha256) VALUES (?1,?2,?3,?4,?5,?6,'REVERSED',?7,?8,?9,?10)",
        params![id, context.company_id, original.0, original.1, request.payload.allocation_id, original.2, now_iso()?, context.user_id, request.idempotency_key, request.request_hash_sha256],
    )?;
    let payment_amount: i64 = tx.query_row(
        "SELECT amount_minor FROM payments WHERE id=?1 AND company_id=?2",
        params![original.0, context.company_id],
        |row| row.get(0),
    )?;
    let document_total: i64 = tx.query_row(
        "SELECT total_ttc_minor FROM commercial_documents WHERE id=?1 AND company_id=?2",
        params![original.1, context.company_id],
        |row| row.get(0),
    )?;
    Ok(AllocationResult {
        allocation_id: id,
        payment_id: original.0.clone(),
        document_id: original.1.clone(),
        amount_minor: original.2,
        payment_unallocated_minor: payment_unallocated(
            tx,
            &context.company_id,
            &original.0,
            payment_amount,
        )?,
        document_open_minor: document_open(
            tx,
            &context.company_id,
            &original.1,
            document_total,
        )?,
    })
}

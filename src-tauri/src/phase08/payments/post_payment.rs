fn authorize_allocation_for_payment(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    payment_id: &str,
) -> Phase08Result<()> {
    let kind = tx
        .query_row(
            "SELECT payment_kind FROM payments WHERE id=?1 AND company_id=?2",
            params![payment_id, context.company_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| Phase08Error::new("PAYMENT_NOT_FOUND", "Payment was not found.", false))?;
    let permission = if kind == "RECEIPT" {
        "payment.receipt.allocate"
    } else {
        "payment.disbursement.allocate"
    };
    crate::phase06::authorize_transaction(tx, context, permission)
        .map_err(|_| Phase08Error::permission())
}

fn validate_payment_input(request: &Idempotent<PaymentInput>) -> Phase08Result<()> {
    super::posting::validate_idempotent(request)?;
    if request.idempotency_key.trim().is_empty() {
        return Err(Phase08Error::validation("Idempotency key is required."));
    }
    if request.payload.amount_minor <= 0 {
        return Err(Phase08Error::validation(
            "Payment amount must be greater than zero.",
        ));
    }
    if request.payload.commercial_date.len() != 10 {
        return Err(Phase08Error::validation("Payment date is invalid."));
    }
    Ok(())
}

pub(crate) fn post_payment_in_tx(
    tx: &Transaction<'_>,
    context: &Phase06AuthContext,
    request: &Idempotent<PaymentInput>,
    payment_id: &str,
    kind: &str,
    customer: bool,
    source: &SourceEventRequest,
) -> Phase08Result<PaymentResult> {
    if let Some(existing) = tx
        .query_row(
            "SELECT id,journal_entry_id,amount_minor,request_hash_sha256 FROM payments WHERE company_id=?1 AND idempotency_key=?2",
            params![context.company_id, request.idempotency_key],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?)),
        )
        .optional()?
    {
        if existing.3.as_deref() != Some(request.request_hash_sha256.as_str()) {
            return Err(Phase08Error::new(
                "IDEMPOTENCY_CONFLICT",
                "The idempotency key was already used with different data.",
                false,
            ));
        }
        let journal_entry_id = existing.1.ok_or_else(Phase08Error::internal)?;
        let unallocated = payment_unallocated(tx, &context.company_id, &existing.0, existing.2)?;
        return Ok(PaymentResult {
            payment_id: existing.0,
            journal_entry_id,
            amount_minor: existing.2,
            unallocated_minor: unallocated,
            replayed: true,
        });
    }

    let partner_ok: i64 = tx.query_row(
        if customer {
            "SELECT COUNT(*) FROM partners WHERE id=?1 AND company_id=?2 AND is_active=1 AND is_customer=1"
        } else {
            "SELECT COUNT(*) FROM partners WHERE id=?1 AND company_id=?2 AND is_active=1 AND is_supplier=1"
        },
        params![request.payload.partner_id, context.company_id],
        |row| row.get(0),
    )?;
    if partner_ok != 1 {
        return Err(Phase08Error::new(
            "PARTNER_NOT_ELIGIBLE",
            "The selected partner is not eligible for this payment.",
            false,
        ));
    }
    let reference_required: i64 = tx
        .query_row(
            "SELECT reference_required FROM payment_methods WHERE id=?1 AND company_id=?2 AND is_active=1",
            params![request.payload.payment_method_id, context.company_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| {
            Phase08Error::new(
                "PAYMENT_METHOD_UNAVAILABLE",
                "The payment method is inactive or unavailable.",
                false,
            )
        })?;
    if reference_required == 1
        && request
            .payload
            .external_reference
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(Phase08Error::new(
            "PAYMENT_REFERENCE_REQUIRED",
            "An external payment reference is required.",
            false,
        ));
    }
    let (fiscal_year_id, fiscal_period_id) =
        super::posting::resolve_open_period(tx, &context.company_id, &request.payload.commercial_date)?;
    let posting = post_source_event_in_tx(
        tx,
        context,
        &Idempotent {
            idempotency_key: format!("payment:{}:accounting", request.idempotency_key),
            request_hash_sha256: super::posting::request_hash(source)?,
            payload: source.clone(),
        },
    )?;
    let now = now_iso()?;
    let payment_number = next_payment_number(tx, &context.company_id, kind)?;
    tx.execute(
        r#"INSERT INTO payments(
            id,company_id,fiscal_year_id,fiscal_period_id,partner_id,payment_method_id,
            payment_number,payment_kind,status,commercial_date,posting_date,amount_minor,
            currency_code,external_reference,idempotency_key,notes,created_at,created_by,
            updated_at,updated_by,row_version,journal_entry_id,reversal_of_payment_id,
            request_hash_sha256)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'POSTED',?9,?9,?10,'DZD',?11,?12,?13,
                   ?14,?15,?14,?15,1,?16,NULL,?17)"#,
        params![
            payment_id,
            context.company_id,
            fiscal_year_id,
            fiscal_period_id,
            request.payload.partner_id,
            request.payload.payment_method_id,
            payment_number,
            kind,
            request.payload.commercial_date,
            request.payload.amount_minor,
            request.payload.external_reference,
            request.idempotency_key,
            request.payload.notes,
            now,
            context.user_id,
            posting.journal_entry_id,
            request.request_hash_sha256,
        ],
    )?;
    Ok(PaymentResult {
        payment_id: payment_id.to_owned(),
        journal_entry_id: posting.journal_entry_id,
        amount_minor: request.payload.amount_minor,
        unallocated_minor: request.payload.amount_minor,
        replayed: false,
    })
}

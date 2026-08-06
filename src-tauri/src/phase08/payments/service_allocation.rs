impl Phase08Service {
    pub fn allocate_payment(
        &self,
        request: Idempotent<AllocationInput>,
    ) -> Phase08Result<AllocationResult> {
        let request = super::posting::normalize_idempotent(request)?;
        let context = self.context(None)?;
        if request.payload.amount_minor <= 0 {
            return Err(Phase08Error::validation(
                "Allocation amount must be greater than zero.",
            ));
        }
        self.immediate(|tx| {
            authorize_allocation_for_payment(tx, &context, &request.payload.payment_id)?;
            allocate_payment_in_tx(tx, &context, &request)
        })
    }

    pub fn reverse_payment_allocation(
        &self,
        request: Idempotent<ReverseAllocationInput>,
    ) -> Phase08Result<AllocationResult> {
        let request = super::posting::normalize_idempotent(request)?;
        let context = self.context(None)?;
        if request.payload.reason.trim().is_empty() {
            return Err(Phase08Error::validation("Reversal reason is required."));
        }
        self.immediate(|tx| {
            let payment_id = tx
                .query_row(
                    "SELECT payment_id FROM payment_allocations WHERE id=?1 AND company_id=?2",
                    params![request.payload.allocation_id, context.company_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .ok_or_else(|| {
                    Phase08Error::new(
                        "ALLOCATION_NOT_FOUND",
                        "The payment allocation was not found.",
                        false,
                    )
                })?;
            authorize_allocation_for_payment(tx, &context, &payment_id)?;
            reverse_allocation_in_tx(tx, &context, &request)
        })
    }

    pub fn reverse_payment(
        &self,
        request: Idempotent<ReversePaymentInput>,
    ) -> Phase08Result<PaymentResult> {
        let request = super::posting::normalize_idempotent(request)?;
        let context = self.context(Some("accounting.reverse"))?;
        super::posting::validate_idempotent(&request)?;
        if request.payload.reason.trim().is_empty() {
            return Err(Phase08Error::validation("Reversal reason is required."));
        }
        let mut connection = self
            .phase05
            .phase06_open()
            .map_err(|_| Phase08Error::internal())?;
        if let Some(existing) = connection.query_row(
            "SELECT id,journal_entry_id,amount_minor,request_hash_sha256 FROM payments WHERE company_id=?1 AND idempotency_key=?2 AND reversal_of_payment_id IS NOT NULL",
            params![context.company_id, request.idempotency_key],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, i64>(2)?, row.get::<_, Option<String>>(3)?)),
        ).optional()? {
            if existing.3.as_deref() != Some(request.request_hash_sha256.as_str()) {
                return Err(Phase08Error::new(
                    "IDEMPOTENCY_CONFLICT",
                    "The idempotency key was already used with different data.",
                    false,
                ));
            }
            return Ok(PaymentResult {
                payment_id: existing.0,
                journal_entry_id: existing.1.ok_or_else(Phase08Error::internal)?,
                amount_minor: existing.2,
                unallocated_minor: existing.2,
                replayed: true,
            });
        }
        let reversal_id = new_id();
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let source_result = payment_reversal_source(
            &transaction,
            &context,
            &request.payload.payment_id,
            &reversal_id,
            &request.payload.reversal_date,
            &request.payload.reason,
        );
        let (source, amount, kind, partner_id, method_id, fiscal_year_id, fiscal_period_id) =
            match source_result {
                Ok(value) => value,
                Err(error) => {
                    drop(transaction);
                    return Err(error);
                }
            };
        let posting = post_source_event_in_tx(
            &transaction,
            &context,
            &Idempotent {
                idempotency_key: format!("payment-reversal:{}:accounting", request.idempotency_key),
                request_hash_sha256: super::posting::request_hash(&source)?,
                payload: source.clone(),
            },
        );
        let result = posting.and_then(|posted| {
            let now = now_iso()?;
            let payment_number = next_payment_number(&transaction, &context.company_id, &kind)?;
            transaction.execute(
                r#"INSERT INTO payments(
                    id,company_id,fiscal_year_id,fiscal_period_id,partner_id,payment_method_id,
                    payment_number,payment_kind,status,commercial_date,posting_date,amount_minor,
                    currency_code,external_reference,idempotency_key,notes,created_at,created_by,
                    updated_at,updated_by,row_version,journal_entry_id,reversal_of_payment_id,
                    request_hash_sha256)
                   VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'POSTED',?9,?9,?10,'DZD',NULL,?11,?12,
                           ?13,?14,?13,?14,1,?15,?16,?17)"#,
                params![
                    reversal_id,
                    context.company_id,
                    fiscal_year_id,
                    fiscal_period_id,
                    partner_id,
                    method_id,
                    payment_number,
                    kind,
                    request.payload.reversal_date,
                    amount,
                    request.idempotency_key,
                    request.payload.reason,
                    now,
                    context.user_id,
                    posted.journal_entry_id,
                    request.payload.payment_id,
                    request.request_hash_sha256,
                ],
            )?;
            Ok(PaymentResult {
                payment_id: reversal_id.clone(),
                journal_entry_id: posted.journal_entry_id,
                amount_minor: amount,
                unallocated_minor: amount,
                replayed: posted.replayed,
            })
        });
        match result {
            Ok(value) => {
                transaction.commit()?;
                Ok(value)
            }
            Err(error) => {
                drop(transaction);
                let accounting_request = Idempotent {
                    idempotency_key: format!("payment-reversal:{}:accounting", request.idempotency_key),
                    request_hash_sha256: super::posting::request_hash(&source)?,
                    payload: source.clone(),
                };
                record_failed_attempt_after_rollback(
                    &mut connection,
                    &context,
                    &accounting_request,
                    &error,
                )?;
                Err(error)
            }
        }
    }
}

impl Phase08Service {
    pub fn post_customer_receipt(
        &self,
        request: Idempotent<PaymentInput>,
    ) -> Phase08Result<PaymentResult> {
        self.post_payment(request, "RECEIPT", "CUSTOMER_RECEIPT", true)
    }

    pub fn post_supplier_payment(
        &self,
        request: Idempotent<PaymentInput>,
    ) -> Phase08Result<PaymentResult> {
        self.post_payment(
            request,
            "DISBURSEMENT",
            "SUPPLIER_PAYMENT",
            false,
        )
    }

    fn post_payment(
        &self,
        request: Idempotent<PaymentInput>,
        kind: &str,
        event_type: &str,
        customer: bool,
    ) -> Phase08Result<PaymentResult> {
        let request = super::posting::normalize_idempotent(request)?;
        let permission = if customer {
            "payment.receipt.post"
        } else {
            "payment.disbursement.post"
        };
        let context = self.context(Some(permission))?;
        validate_payment_input(&request)?;
        let payment_id = new_id();
        let source = SourceEventRequest {
            source_event_type: event_type.to_owned(),
            source_event_id: format!("{event_type}:{}", request.idempotency_key),
            source_document_id: None,
            event_date: request.payload.commercial_date.clone(),
            partner_id: Some(request.payload.partner_id.clone()),
            product_id: None,
            payment_method_id: Some(request.payload.payment_method_id.clone()),
            memo: request.payload.notes.clone(),
            components_minor: BTreeMap::from([(
                "PAYMENT_AMOUNT".to_owned(),
                request.payload.amount_minor,
            )]),
            inject_failure_after_header: false,
        };

        let mut connection = self
            .phase05
            .phase06_open()
            .map_err(|_| Phase08Error::internal())?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = post_payment_in_tx(
            &transaction,
            &context,
            &request,
            &payment_id,
            kind,
            customer,
            &source,
        );
        match result {
            Ok(value) => {
                transaction.commit()?;
                Ok(value)
            }
            Err(error) => {
                drop(transaction);
                let accounting_request = Idempotent {
                    idempotency_key: format!("payment:{}:accounting", request.idempotency_key),
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

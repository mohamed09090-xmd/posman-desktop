use super::*;

impl Phase06Service {
    pub fn create_purchase_order(
        &self,
        request: CreatePurchaseOrderRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_order.confirm"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_order.confirm")?;
            super::super::validate_business_date(&request.commercial_date)?;
            ensure_supplier(transaction, &context.company_id, &request.supplier_id)?;
            let (document_id, number) = insert_document(
                transaction,
                &context,
                "PURCHASE_ORDER",
                "DRAFT",
                "DRAFT",
                &request.commercial_date,
                Some(&request.supplier_id),
                None,
                None,
                None,
                request.notes.as_deref(),
                None,
                (0, 0, 0),
            )?;
            for (index, line) in request.lines.iter().enumerate() {
                insert_purchase_line(
                    transaction,
                    &context,
                    &document_id,
                    i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?,
                    line,
                    &request.commercial_date,
                    None,
                    None,
                )?;
            }
            update_document_totals(transaction, &document_id, &context.company_id)?;
            audit(
                transaction,
                &context,
                "purchase_order.create",
                "commercial_document",
                &document_id,
                None,
            )?;
            Ok(EntityResult {
                id: document_id,
                document_number: Some(number),
                status: "DRAFT".to_owned(),
                row_version: 1,
                replayed: false,
            })
        })
    }

    pub fn update_purchase_order(
        &self,
        request: UpdatePurchaseOrderRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_order.confirm"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_order.confirm")?;
            super::super::validate_business_date(&request.commercial_date)?;
            ensure_supplier(transaction, &context.company_id, &request.supplier_id)?;
            let changed = transaction.execute(
                "UPDATE commercial_documents
                 SET partner_id = ?1, commercial_date = ?2, notes = ?3,
                     updated_at = ?4, updated_by = ?5, row_version = row_version + 1
                 WHERE id = ?6 AND company_id = ?7 AND document_type = 'PURCHASE_ORDER'
                   AND workflow_status = 'DRAFT' AND posting_status = 'DRAFT'
                   AND row_version = ?8",
                params![
                    request.supplier_id,
                    request.commercial_date,
                    request.notes,
                    now_iso()?,
                    context.user_id,
                    request.document_id,
                    context.company_id,
                    request.row_version,
                ],
            )?;
            if changed != 1 {
                return Err(Phase06Error::conflict());
            }
            transaction.execute(
                "DELETE FROM commercial_document_lines
                 WHERE document_id = ?1 AND company_id = ?2",
                params![request.document_id, context.company_id],
            )?;
            for (index, line) in request.lines.iter().enumerate() {
                insert_purchase_line(
                    transaction,
                    &context,
                    &request.document_id,
                    i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?,
                    line,
                    &request.commercial_date,
                    None,
                    None,
                )?;
            }
            update_document_totals(transaction, &request.document_id, &context.company_id)?;
            audit(
                transaction,
                &context,
                "purchase_order.update",
                "commercial_document",
                &request.document_id,
                None,
            )?;
            entity_result(transaction, &context.company_id, &request.document_id, false)
        })
    }

    pub fn confirm_purchase_order(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.purchase_order_transition(request, "purchase_order.confirm", "CONFIRMED")
    }

    pub fn cancel_purchase_order(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.purchase_order_transition(request, "purchase_order.cancel", "CANCELLED")
    }

    pub fn hold_purchase_order(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.purchase_order_transition(request, "purchase_order.hold", "ON_HOLD")
    }

    fn purchase_order_transition(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
        namespace: &str,
        target: &str,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_order.confirm"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_order.confirm")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                namespace,
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(transaction, &context.company_id, &id, true);
            }
            let allowed = match target {
                "CONFIRMED" | "CANCELLED" => "DRAFT",
                "ON_HOLD" => "CONFIRMED",
                _ => return Err(Phase06Error::invalid("status")),
            };
            let changed = transaction.execute(
                "UPDATE commercial_documents
                 SET workflow_status = ?1, updated_at = ?2, updated_by = ?3,
                     row_version = row_version + 1
                 WHERE id = ?4 AND company_id = ?5 AND document_type = 'PURCHASE_ORDER'
                   AND workflow_status = ?6 AND posting_status = 'DRAFT' AND row_version = ?7",
                params![
                    target,
                    now_iso()?,
                    context.user_id,
                    request.payload.document_id,
                    context.company_id,
                    allowed,
                    request.payload.row_version,
                ],
            )?;
            if changed != 1 {
                return Err(Phase06Error::conflict());
            }
            transaction.execute(
                "INSERT INTO document_status_history (
                    id, company_id, document_id, old_status, new_status,
                    changed_at, changed_by, reason, row_version_snapshot
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    new_id(),
                    context.company_id,
                    request.payload.document_id,
                    allowed,
                    target,
                    now_iso()?,
                    context.user_id,
                    request.payload.reason,
                    request.payload.row_version + 1,
                ],
            )?;
            audit(
                transaction,
                &context,
                namespace,
                "commercial_document",
                &request.payload.document_id,
                request.payload.reason.as_deref(),
            )?;
            finish_idempotency(
                transaction,
                &context,
                namespace,
                &request.idempotency_key,
                "commercial_document",
                &request.payload.document_id,
            )?;
            entity_result(
                transaction,
                &context.company_id,
                &request.payload.document_id,
                false,
            )
        })
    }
}

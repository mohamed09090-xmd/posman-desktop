use super::*;

impl Phase06Service {
    pub fn create_purchase_receipt(
        &self,
        request: CreateReceiptRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_receipt.post"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_receipt.post")?;
            super::super::validate_business_date(&request.commercial_date)?;
            ensure_supplier(transaction, &context.company_id, &request.supplier_id)?;
            super::super::projections::validate_warehouse_scope(
                transaction,
                &context.company_id,
                &request.warehouse_id,
                None,
            )?;
            if let Some(order_id) = request.purchase_order_id.as_deref() {
                let valid = transaction
                    .query_row(
                        "SELECT 1 FROM commercial_documents
                         WHERE id = ?1 AND company_id = ?2 AND document_type = 'PURCHASE_ORDER'
                           AND workflow_status IN ('CONFIRMED', 'ON_HOLD') AND posting_status = 'DRAFT'
                           AND partner_id = ?3",
                        params![order_id, context.company_id, request.supplier_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()?
                    .is_some();
                if !valid {
                    return Err(Phase06Error::not_found());
                }
            }

            let (document_id, number) = insert_document(
                transaction,
                &context,
                "PURCHASE_RECEIPT",
                "DRAFT",
                "DRAFT",
                &request.commercial_date,
                Some(&request.supplier_id),
                Some(&request.warehouse_id),
                request.purchase_order_id.as_deref(),
                None,
                request.notes.as_deref(),
                None,
                (0, 0, 0),
            )?;
            for (index, line) in request.lines.iter().enumerate() {
                let (target_line_id, _) = insert_purchase_line(
                    transaction,
                    &context,
                    &document_id,
                    i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?,
                    line,
                    &request.commercial_date,
                    Some(&request.warehouse_id),
                    None,
                )?;
                match (
                    request.purchase_order_id.as_deref(),
                    line.source_line_id.as_deref(),
                ) {
                    (Some(order_id), Some(source_line_id)) => {
                        let belongs = transaction
                            .query_row(
                                "SELECT 1 FROM commercial_document_lines
                                 WHERE id = ?1 AND company_id = ?2 AND document_id = ?3
                                   AND product_id = ?4",
                                params![
                                    source_line_id,
                                    context.company_id,
                                    order_id,
                                    line.product_id,
                                ],
                                |row| row.get::<_, i64>(0),
                            )
                            .optional()?
                            .is_some();
                        if !belongs {
                            return Err(Phase06Error::invalid("sourceLineId"));
                        }
                        insert_link(
                            transaction,
                            &context,
                            source_line_id,
                            &target_line_id,
                            "PURCHASE_ORDER_TO_RECEIPT",
                            line.quantity_scaled,
                        )?;
                    }
                    (Some(_), None) => return Err(Phase06Error::invalid("sourceLineId")),
                    (None, Some(_)) => return Err(Phase06Error::invalid("purchaseOrderId")),
                    (None, None) => {}
                }
            }
            update_document_totals(transaction, &document_id, &context.company_id)?;
            audit(
                transaction,
                &context,
                "purchase_receipt.create",
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

    pub fn post_purchase_receipt(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_receipt.post"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_receipt.post")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "purchase_receipt.post",
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(transaction, &context.company_id, &id, true);
            }
            let (date, warehouse_id, source_order_id, row_version, posting_status) = transaction
                .query_row(
                    "SELECT commercial_date, warehouse_id, source_document_id,
                            row_version, posting_status
                     FROM commercial_documents
                     WHERE id = ?1 AND company_id = ?2 AND document_type = 'PURCHASE_RECEIPT'",
                    params![request.payload.document_id, context.company_id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, i64>(3)?,
                            row.get::<_, String>(4)?,
                        ))
                    },
                )
                .optional()?
                .ok_or_else(Phase06Error::not_found)?;
            if posting_status != "DRAFT" {
                return Err(Phase06Error::immutable());
            }
            if row_version != request.payload.row_version {
                return Err(Phase06Error::conflict());
            }
            post_receipt_lines(
                transaction,
                &context,
                &request.payload.document_id,
                &warehouse_id,
                &date,
            )?;
            post_document(
                transaction,
                &context,
                &request.payload.document_id,
                "purchase_receipt.post",
                request.payload.reason.as_deref(),
            )?;
            if let Some(order_id) = source_order_id {
                update_order_receipt_status(transaction, &context, &order_id)?;
            }
            finish_idempotency(
                transaction,
                &context,
                "purchase_receipt.post",
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

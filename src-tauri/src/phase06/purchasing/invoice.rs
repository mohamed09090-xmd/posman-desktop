use super::*;

impl Phase06Service {
    pub fn create_purchase_invoice(
        &self,
        request: CreateInvoiceRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_invoice.post"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_invoice.post")?;
            super::super::validate_business_date(&request.commercial_date)?;
            ensure_supplier(transaction, &context.company_id, &request.supplier_id)?;
            let (document_id, number) = insert_document(
                transaction,
                &context,
                "PURCHASE_INVOICE",
                "DRAFT",
                "DRAFT",
                &request.commercial_date,
                Some(&request.supplier_id),
                None,
                None,
                request.due_date.as_deref(),
                request.notes.as_deref(),
                None,
                (0, 0, 0),
            )?;
            for (index, line) in request.lines.iter().enumerate() {
                let source_line_id = line
                    .source_line_id
                    .as_deref()
                    .ok_or_else(|| Phase06Error::invalid("sourceLineId"))?;
                let (source_supplier, posting_status, source_price, source_product, warehouse_id) =
                    transaction
                        .query_row(
                            "SELECT document.partner_id, document.posting_status,
                                    line.unit_price_scaled, line.product_id, line.warehouse_id
                             FROM commercial_document_lines line
                             JOIN commercial_documents document
                               ON document.id = line.document_id AND document.company_id = line.company_id
                             WHERE line.id = ?1 AND line.company_id = ?2
                               AND document.document_type = 'PURCHASE_RECEIPT'",
                            params![source_line_id, context.company_id],
                            |row| {
                                Ok((
                                    row.get::<_, Option<String>>(0)?,
                                    row.get::<_, String>(1)?,
                                    row.get::<_, i64>(2)?,
                                    row.get::<_, String>(3)?,
                                    row.get::<_, Option<String>>(4)?,
                                ))
                            },
                        )
                        .optional()?
                        .ok_or_else(Phase06Error::not_found)?;
                if source_supplier.as_deref() != Some(&request.supplier_id)
                    || posting_status != "POSTED"
                    || source_product != line.product_id
                {
                    return Err(Phase06Error::invalid("receipt"));
                }
                let variance = line
                    .unit_price_scaled
                    .checked_sub(source_price)
                    .ok_or_else(Phase06Error::numeric_overflow)?;
                let notes = serde_json::json!({
                    "receiptPriceScaled": source_price,
                    "invoicePriceScaled": line.unit_price_scaled,
                    "priceVarianceScaled": variance,
                    "accountingPhase": "PHASE_08"
                })
                .to_string();
                let (target_line_id, _) = insert_purchase_line(
                    transaction,
                    &context,
                    &document_id,
                    i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?,
                    line,
                    &request.commercial_date,
                    PurchaseLineOptions {
                        default_warehouse: line.warehouse_id.as_deref().or(warehouse_id.as_deref()),
                        notes: Some(&notes),
                    },
                )?;
                insert_link(
                    transaction,
                    &context,
                    source_line_id,
                    &target_line_id,
                    "RECEIPT_TO_INVOICE",
                    line.quantity_scaled,
                )?;
            }
            update_document_totals(transaction, &document_id, &context.company_id)?;
            audit(
                transaction,
                &context,
                "purchase_invoice.create",
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

    pub fn post_purchase_invoice(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_invoice.post"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_invoice.post")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "purchase_invoice.post",
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(transaction, &context.company_id, &id, true);
            }
            let row_version = transaction
                .query_row(
                    "SELECT row_version FROM commercial_documents
                     WHERE id = ?1 AND company_id = ?2 AND document_type = 'PURCHASE_INVOICE'
                       AND posting_status = 'DRAFT'",
                    params![request.payload.document_id, context.company_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .ok_or_else(Phase06Error::immutable)?;
            if row_version != request.payload.row_version {
                return Err(Phase06Error::conflict());
            }
            post_document(
                transaction,
                &context,
                &request.payload.document_id,
                "purchase_invoice.post",
                request.payload.reason.as_deref(),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "purchase_invoice.post",
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

    pub fn direct_receive_and_invoice(
        &self,
        request: IdempotentRequest<DirectReceiveInvoiceRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_invoice.post"))?;
        validate_purchase_lines(&request.payload.lines)?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_invoice.post")?;
            authorize_transaction(transaction, &context, "purchase_receipt.post")?;
            super::super::validate_business_date(&request.payload.commercial_date)?;
            ensure_supplier(
                transaction,
                &context.company_id,
                &request.payload.supplier_id,
            )?;
            super::super::projections::validate_warehouse_scope(
                transaction,
                &context.company_id,
                &request.payload.warehouse_id,
                None,
            )?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "purchase.direct_receive_invoice",
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(transaction, &context.company_id, &id, true);
            }

            let (receipt_id, _) = insert_document(
                transaction,
                &context,
                "PURCHASE_RECEIPT",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                Some(&request.payload.supplier_id),
                Some(&request.payload.warehouse_id),
                None,
                None,
                Some("Internal receipt for direct supplier invoice"),
                None,
                (0, 0, 0),
            )?;
            let (invoice_id, invoice_number) = insert_document(
                transaction,
                &context,
                "PURCHASE_INVOICE",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                Some(&request.payload.supplier_id),
                Some(&request.payload.warehouse_id),
                Some(&receipt_id),
                request.payload.due_date.as_deref(),
                request.payload.notes.as_deref(),
                Some(&request.idempotency_key),
                (0, 0, 0),
            )?;
            for (index, line) in request.payload.lines.iter().enumerate() {
                let line_number =
                    i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?;
                let (receipt_line_id, _) = insert_purchase_line(
                    transaction,
                    &context,
                    &receipt_id,
                    line_number,
                    line,
                    &request.payload.commercial_date,
                    PurchaseLineOptions {
                        default_warehouse: Some(&request.payload.warehouse_id),
                        notes: Some("Direct receipt"),
                    },
                )?;
                let (invoice_line_id, _) = insert_purchase_line(
                    transaction,
                    &context,
                    &invoice_id,
                    line_number,
                    line,
                    &request.payload.commercial_date,
                    PurchaseLineOptions {
                        default_warehouse: Some(&request.payload.warehouse_id),
                        notes: Some("Direct receive and invoice"),
                    },
                )?;
                insert_link(
                    transaction,
                    &context,
                    &receipt_line_id,
                    &invoice_line_id,
                    "RECEIPT_TO_INVOICE",
                    line.quantity_scaled,
                )?;
            }
            update_document_totals(transaction, &receipt_id, &context.company_id)?;
            update_document_totals(transaction, &invoice_id, &context.company_id)?;
            post_receipt_lines(
                transaction,
                &context,
                &receipt_id,
                &request.payload.warehouse_id,
                &request.payload.commercial_date,
            )?;
            post_document(
                transaction,
                &context,
                &receipt_id,
                "purchase_receipt.post",
                Some("Direct receive and invoice"),
            )?;
            post_document(
                transaction,
                &context,
                &invoice_id,
                "purchase_invoice.post",
                Some("Direct receive and invoice"),
            )?;
            let details = serde_json::json!({
                "receiptId": receipt_id,
                "invoiceId": invoice_id
            })
            .to_string();
            audit(
                transaction,
                &context,
                "purchase.direct_receive_invoice",
                "commercial_document",
                &invoice_id,
                Some(&details),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "purchase.direct_receive_invoice",
                &request.idempotency_key,
                "commercial_document",
                &invoice_id,
            )?;
            Ok(EntityResult {
                id: invoice_id,
                document_number: Some(invoice_number),
                status: "POSTED".to_owned(),
                row_version: 2,
                replayed: false,
            })
        })
    }
}

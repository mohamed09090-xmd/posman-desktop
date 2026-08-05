use rusqlite::{params, OptionalExtension, Transaction};

use super::inventory::{negative_override_allowed, post_document};
use super::{
    audit, authorize_transaction, begin_idempotency,
    dto::{
        CreateInvoiceRequest, CreatePurchaseOrderRequest, CreateReceiptRequest,
        DirectReceiveInvoiceRequest, DocumentActionRequest, DocumentQuery, DocumentView,
        EntityResult, IdempotentRequest, PurchaseLineInput, PurchaseReturnRequest,
        UpdatePurchaseOrderRequest,
    },
    entity_result,
    error::{Phase06Error, Phase06Result},
    finish_idempotency, get_document_connection, insert_document, insert_purchase_line, new_id,
    now_iso,
    projections::{apply_movement, balance, MovementSpec},
    request_hash, update_document_totals, IdempotencyStart, Phase06Service,
};

fn validate_purchase_lines(lines: &[PurchaseLineInput]) -> Phase06Result<()> {
    if lines.is_empty() {
        return Err(Phase06Error::invalid("lines"));
    }
    for line in lines {
        if line.quantity_scaled <= 0
            || line.unit_price_scaled < 0
            || line.unit_cost_scaled.is_some_and(|value| value < 0)
            || !(0..=1_000_000).contains(&line.discount_rate_scaled)
        {
            return Err(Phase06Error::invalid("purchaseLine"));
        }
    }
    Ok(())
}

fn ensure_supplier(transaction: &Transaction<'_>, company_id: &str, id: &str) -> Phase06Result<()> {
    let exists = transaction
        .query_row(
            "SELECT 1 FROM partners
             WHERE id = ?1 AND company_id = ?2 AND is_supplier = 1 AND is_active = 1",
            params![id, company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !exists {
        return Err(Phase06Error::new(
            "SUPPLIER_REQUIRED",
            "Select an active supplier.",
        ));
    }
    Ok(())
}

fn aggregate_transform_guard(
    transaction: &Transaction<'_>,
    company_id: &str,
    source_line_id: &str,
    transformation_type: &str,
    quantity: i64,
) -> Phase06Result<()> {
    let source_quantity = transaction
        .query_row(
            "SELECT quantity_scaled FROM commercial_document_lines
             WHERE id = ?1 AND company_id = ?2",
            params![source_line_id, company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)?;
    let transformed: i64 = transaction.query_row(
        "SELECT COALESCE(SUM(transformed_quantity_scaled), 0)
         FROM document_line_links
         WHERE company_id = ?1 AND source_line_id = ?2 AND transformation_type = ?3",
        params![company_id, source_line_id, transformation_type],
        |row| row.get(0),
    )?;
    let total = transformed
        .checked_add(quantity)
        .ok_or_else(Phase06Error::numeric_overflow)?;
    if quantity <= 0 || total > source_quantity {
        return Err(Phase06Error::over_transformation());
    }
    Ok(())
}

fn insert_link(
    transaction: &Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    source_line_id: &str,
    target_line_id: &str,
    transformation_type: &str,
    quantity: i64,
) -> Phase06Result<()> {
    aggregate_transform_guard(
        transaction,
        &context.company_id,
        source_line_id,
        transformation_type,
        quantity,
    )?;
    transaction.execute(
        "INSERT INTO document_line_links (
            id, company_id, source_line_id, target_line_id, transformation_type,
            transformed_quantity_scaled, created_at, created_by
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            new_id(),
            context.company_id,
            source_line_id,
            target_line_id,
            transformation_type,
            quantity,
            now_iso()?,
            context.user_id,
        ],
    )?;
    Ok(())
}

impl Phase06Service {
    pub fn create_purchase_order(
        &self,
        request: CreatePurchaseOrderRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_order.confirm"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_order.confirm")?;
            super::validate_business_date(&request.commercial_date)?;
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
            super::validate_business_date(&request.commercial_date)?;
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
    pub fn create_purchase_receipt(
        &self,
        request: CreateReceiptRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_receipt.post"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_receipt.post")?;
            super::validate_business_date(&request.commercial_date)?;
            ensure_supplier(transaction, &context.company_id, &request.supplier_id)?;
            super::projections::validate_warehouse_scope(
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
    pub fn create_purchase_invoice(
        &self,
        request: CreateInvoiceRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_invoice.post"))?;
        validate_purchase_lines(&request.lines)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_invoice.post")?;
            super::validate_business_date(&request.commercial_date)?;
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
                    line.warehouse_id.as_deref().or(warehouse_id.as_deref()),
                    Some(&notes),
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
            super::validate_business_date(&request.payload.commercial_date)?;
            ensure_supplier(
                transaction,
                &context.company_id,
                &request.payload.supplier_id,
            )?;
            super::projections::validate_warehouse_scope(
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
                    Some(&request.payload.warehouse_id),
                    Some("Direct receipt"),
                )?;
                let (invoice_line_id, _) = insert_purchase_line(
                    transaction,
                    &context,
                    &invoice_id,
                    line_number,
                    line,
                    &request.payload.commercial_date,
                    Some(&request.payload.warehouse_id),
                    Some("Direct receive and invoice"),
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
    pub fn post_purchase_return(
        &self,
        request: IdempotentRequest<PurchaseReturnRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("purchase_return.post"))?;
        validate_purchase_lines(&request.payload.lines)?;
        if request.payload.reason.trim().is_empty() {
            return Err(Phase06Error::invalid("reason"));
        }
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "purchase_return.post")?;
            let allow_negative = negative_override_allowed(
                transaction,
                &context,
                request.payload.allow_negative_override,
                Some(&request.payload.reason),
            )?;
            super::validate_business_date(&request.payload.commercial_date)?;
            ensure_supplier(
                transaction,
                &context.company_id,
                &request.payload.supplier_id,
            )?;
            super::projections::validate_warehouse_scope(
                transaction,
                &context.company_id,
                &request.payload.warehouse_id,
                None,
            )?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "purchase_return.post",
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(transaction, &context.company_id, &id, true);
            }

            let source_valid = transaction
                .query_row(
                    "SELECT 1 FROM commercial_documents
                     WHERE id = ?1 AND company_id = ?2
                       AND document_type IN ('PURCHASE_RECEIPT', 'PURCHASE_INVOICE')
                       AND posting_status = 'POSTED' AND partner_id = ?3",
                    params![
                        request.payload.source_document_id,
                        context.company_id,
                        request.payload.supplier_id,
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .is_some();
            if !source_valid {
                return Err(Phase06Error::not_found());
            }

            let (document_id, number) = insert_document(
                transaction,
                &context,
                "PURCHASE_RETURN",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                Some(&request.payload.supplier_id),
                Some(&request.payload.warehouse_id),
                Some(&request.payload.source_document_id),
                None,
                Some(&request.payload.reason),
                Some(&request.idempotency_key),
                (0, 0, 0),
            )?;
            for (index, line) in request.payload.lines.iter().enumerate() {
                let source_line_id = line
                    .source_line_id
                    .as_deref()
                    .ok_or_else(|| Phase06Error::invalid("sourceLineId"))?;
                let (source_product, source_price, source_cost) = transaction
                    .query_row(
                        "SELECT product_id, unit_price_scaled, unit_cost_scaled
                         FROM commercial_document_lines
                         WHERE id = ?1 AND company_id = ?2 AND document_id = ?3",
                        params![
                            source_line_id,
                            context.company_id,
                            request.payload.source_document_id,
                        ],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, Option<i64>>(2)?,
                            ))
                        },
                    )
                    .optional()?
                    .ok_or_else(Phase06Error::not_found)?;
                if source_product != line.product_id {
                    return Err(Phase06Error::invalid("productId"));
                }
                let mut snapshot = line.clone();
                snapshot.unit_price_scaled = source_price;
                snapshot.unit_cost_scaled = source_cost;
                let (target_line_id, _) = insert_purchase_line(
                    transaction,
                    &context,
                    &document_id,
                    i64::try_from(index + 1).map_err(|_| Phase06Error::numeric_overflow())?,
                    &snapshot,
                    &request.payload.commercial_date,
                    Some(&request.payload.warehouse_id),
                    Some(&request.payload.reason),
                )?;
                insert_link(
                    transaction,
                    &context,
                    source_line_id,
                    &target_line_id,
                    "DOCUMENT_TO_RETURN",
                    line.quantity_scaled,
                )?;
                let current = balance(
                    transaction,
                    &context.company_id,
                    &line.product_id,
                    &request.payload.warehouse_id,
                    None,
                )?;
                if current.average_cost <= 0 {
                    return Err(Phase06Error::invalid("averageCost"));
                }
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &line.product_id,
                        warehouse_id: &request.payload.warehouse_id,
                        location_id: None,
                        source_document_id: Some(&document_id),
                        source_line_id: Some(&target_line_id),
                        movement_type: "PURCHASE_RETURN",
                        business_date: &request.payload.commercial_date,
                        quantity_delta: -line.quantity_scaled,
                        inbound_cost: None,
                        recalculate_average: false,
                        posting_event_key: &format!(
                            "purchase-return:{document_id}:{}",
                            index + 1
                        ),
                        transfer_group_id: None,
                        notes: Some(&request.payload.reason),
                        allow_negative,
                    },
                )?;
            }
            update_document_totals(transaction, &document_id, &context.company_id)?;
            post_document(
                transaction,
                &context,
                &document_id,
                "purchase_return.post",
                Some(&request.payload.reason),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "purchase_return.post",
                &request.idempotency_key,
                "commercial_document",
                &document_id,
            )?;
            Ok(EntityResult {
                id: document_id,
                document_number: Some(number),
                status: "POSTED".to_owned(),
                row_version: 2,
                replayed: false,
            })
        })
    }
    pub fn list_purchasing_documents(
        &self,
        query: DocumentQuery,
    ) -> Phase06Result<Vec<DocumentView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                "SELECT id FROM commercial_documents
                 WHERE company_id = ?1
                   AND document_type IN (
                     'PURCHASE_ORDER', 'PURCHASE_RECEIPT', 'PURCHASE_INVOICE', 'PURCHASE_RETURN'
                   )
                   AND (?2 IS NULL OR document_type = ?2)
                   AND (?3 IS NULL OR workflow_status = ?3)
                   AND (?4 IS NULL OR document_number LIKE '%' || ?4 || '%')
                 ORDER BY commercial_date DESC, document_number DESC
                 LIMIT ?5",
            )?;
            let ids = statement
                .query_map(
                    params![
                        context.company_id,
                        query.document_type,
                        query.status,
                        query.search,
                        query.limit.unwrap_or(200).clamp(1, 1000),
                    ],
                    |row| row.get::<_, String>(0),
                )?
                .collect::<Result<Vec<_>, _>>()?;
            ids.iter()
                .map(|id| get_document_connection(connection, &context.company_id, id))
                .collect()
        })
    }

    pub fn get_purchasing_document(&self, id: String) -> Phase06Result<DocumentView> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| get_document_connection(connection, &context.company_id, &id))
    }
}

include!("purchasing/posting_support.rs");

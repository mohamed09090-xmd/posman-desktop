use rusqlite::{params, OptionalExtension};

use crate::phase06::{
    audit, authorize_transaction, begin_idempotency,
    error::{Phase06Error, Phase06Result},
    finish_idempotency, insert_document, new_id, now_iso,
    projections::{balance, set_reserved}, request_hash, IdempotencyStart,
};

use super::{
    dto::{CreateSalesOrderRequest, EntityResult, IdempotentRequest, SalesOrderActionRequest, UpdateSalesOrderRequest},
    service::{
        apply_document_pricing, enforce_below_cost, ensure_customer, insert_prepared_lines,
        insert_status, load_document_priced_lines, prepare_sales_lines, sales_entity,
    },
    Phase07Service,
};

impl Phase07Service {
    pub fn create_sales_order(&self, request: CreateSalesOrderRequest) -> Phase06Result<EntityResult> {
        let context = self.context(Some("sales_order.confirm"))?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "sales_order.confirm")?;
            crate::phase06::validate_business_date(&request.commercial_date)?;
            ensure_customer(transaction, &context.company_id, &request.customer_id)?;
            let (lines, totals) = prepare_sales_lines(
                transaction,
                &context,
                &request.lines,
                &request.warehouse_id,
                &request.commercial_date,
                &request.price_mode,
                request.header_discount_rate_scaled,
            )?;
            let (document_id, number) = insert_document(
                transaction,
                &context,
                "SALES_ORDER",
                "DRAFT",
                "DRAFT",
                &request.commercial_date,
                Some(&request.customer_id),
                Some(&request.warehouse_id),
                None,
                request.due_date.as_deref(),
                request.notes.as_deref(),
                None,
                (0, 0, 0),
            )?;
            insert_prepared_lines(transaction, &context, &document_id, &lines, None)?;
            apply_document_pricing(
                transaction,
                &context.company_id,
                &document_id,
                &request.price_mode,
                request.header_discount_rate_scaled,
                totals,
            )?;
            enforce_below_cost(
                transaction,
                &context,
                &lines,
                request.below_cost_override_reason.as_deref(),
                "sales_order.below_cost",
                &document_id,
            )?;
            audit(transaction, &context, "sales_order.create", "commercial_document", &document_id, None)?;
            Ok(EntityResult {
                id: document_id,
                document_number: Some(number),
                status: "DRAFT".to_owned(),
                row_version: 1,
                replayed: false,
            })
        })
    }

    pub fn update_sales_order(&self, request: UpdateSalesOrderRequest) -> Phase06Result<EntityResult> {
        let context = self.context(Some("sales_order.confirm"))?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "sales_order.confirm")?;
            crate::phase06::validate_business_date(&request.order.commercial_date)?;
            ensure_customer(transaction, &context.company_id, &request.order.customer_id)?;
            let (lines, totals) = prepare_sales_lines(
                transaction,
                &context,
                &request.order.lines,
                &request.order.warehouse_id,
                &request.order.commercial_date,
                &request.order.price_mode,
                request.order.header_discount_rate_scaled,
            )?;
            let changed = transaction.execute(
                "UPDATE commercial_documents SET partner_id=?1,warehouse_id=?2,commercial_date=?3,
                 due_date=?4,notes=?5,updated_at=?6,updated_by=?7,row_version=row_version+1
                 WHERE id=?8 AND company_id=?9 AND document_type='SALES_ORDER'
                   AND workflow_status='DRAFT' AND posting_status='DRAFT' AND row_version=?10",
                params![
                    request.order.customer_id,
                    request.order.warehouse_id,
                    request.order.commercial_date,
                    request.order.due_date,
                    request.order.notes,
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
                "DELETE FROM commercial_document_lines WHERE document_id=?1 AND company_id=?2",
                params![request.document_id, context.company_id],
            )?;
            insert_prepared_lines(transaction, &context, &request.document_id, &lines, None)?;
            apply_document_pricing(
                transaction,
                &context.company_id,
                &request.document_id,
                &request.order.price_mode,
                request.order.header_discount_rate_scaled,
                totals,
            )?;
            enforce_below_cost(
                transaction,
                &context,
                &lines,
                request.order.below_cost_override_reason.as_deref(),
                "sales_order.below_cost",
                &request.document_id,
            )?;
            audit(transaction, &context, "sales_order.update", "commercial_document", &request.document_id, None)?;
            sales_entity(transaction, &context, &request.document_id, false)
        })
    }

    pub fn confirm_sales_order(
        &self,
        request: IdempotentRequest<SalesOrderActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("sales_order.confirm"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "sales_order.confirm")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "sales_order.confirm",
                &request.idempotency_key,
                &hash,
            )? {
                return sales_entity(transaction, &context, &id, true);
            }
            let actual: i64 = transaction
                .query_row(
                    "SELECT row_version FROM commercial_documents WHERE id=?1 AND company_id=?2
                     AND document_type='SALES_ORDER' AND workflow_status='DRAFT' AND posting_status='DRAFT'",
                    params![request.payload.document_id, context.company_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or_else(Phase06Error::immutable)?;
            if actual != request.payload.row_version {
                return Err(Phase06Error::conflict());
            }
            let lines = load_document_priced_lines(transaction, &context, &request.payload.document_id)?;
            enforce_below_cost(
                transaction,
                &context,
                &lines,
                request.payload.below_cost_override_reason.as_deref(),
                "sales_order.confirm_below_cost",
                &request.payload.document_id,
            )?;
            reserve_order_lines(transaction, &context, &request.payload.document_id)?;
            transaction.execute(
                "UPDATE commercial_documents SET workflow_status='CONFIRMED',updated_at=?1,updated_by=?2,
                 row_version=row_version+1 WHERE id=?3 AND company_id=?4 AND row_version=?5",
                params![now_iso()?, context.user_id, request.payload.document_id, context.company_id, actual],
            )?;
            insert_status(transaction, &context, &request.payload.document_id, "DRAFT", "CONFIRMED", request.payload.reason.as_deref())?;
            audit(transaction, &context, "sales_order.confirm", "commercial_document", &request.payload.document_id, None)?;
            finish_idempotency(transaction, &context, "sales_order.confirm", &request.idempotency_key, "commercial_document", &request.payload.document_id)?;
            sales_entity(transaction, &context, &request.payload.document_id, false)
        })
    }

    pub fn hold_sales_order(
        &self,
        request: IdempotentRequest<SalesOrderActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.order_pause_transition(request, "sales_order.hold", "ON_HOLD")
    }

    pub fn cancel_sales_order(
        &self,
        request: IdempotentRequest<SalesOrderActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.order_pause_transition(request, "sales_order.cancel", "CANCELLED")
    }

    pub fn resume_sales_order(
        &self,
        request: IdempotentRequest<SalesOrderActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("sales_order.confirm"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "sales_order.confirm")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(transaction, &context, "sales_order.resume", &request.idempotency_key, &hash)? {
                return sales_entity(transaction, &context, &id, true);
            }
            let actual: i64 = transaction.query_row(
                "SELECT row_version FROM commercial_documents WHERE id=?1 AND company_id=?2
                 AND document_type='SALES_ORDER' AND workflow_status='ON_HOLD' AND posting_status='DRAFT'",
                params![request.payload.document_id, context.company_id], |row| row.get(0),
            ).optional()?.ok_or_else(Phase06Error::immutable)?;
            if actual != request.payload.row_version { return Err(Phase06Error::conflict()); }
            reserve_order_lines(transaction, &context, &request.payload.document_id)?;
            let delivered = delivered_quantity(transaction, &context.company_id, &request.payload.document_id)?;
            let target = if delivered > 0 { "PARTIALLY_DELIVERED" } else { "CONFIRMED" };
            transaction.execute(
                "UPDATE commercial_documents SET workflow_status=?1,updated_at=?2,updated_by=?3,row_version=row_version+1
                 WHERE id=?4 AND company_id=?5 AND row_version=?6",
                params![target, now_iso()?, context.user_id, request.payload.document_id, context.company_id, actual],
            )?;
            insert_status(transaction, &context, &request.payload.document_id, "ON_HOLD", target, request.payload.reason.as_deref())?;
            finish_idempotency(transaction, &context, "sales_order.resume", &request.idempotency_key, "commercial_document", &request.payload.document_id)?;
            sales_entity(transaction, &context, &request.payload.document_id, false)
        })
    }

    fn order_pause_transition(
        &self,
        request: IdempotentRequest<SalesOrderActionRequest>,
        namespace: &str,
        target: &str,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("sales_order.confirm"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "sales_order.confirm")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(transaction, &context, namespace, &request.idempotency_key, &hash)? {
                return sales_entity(transaction, &context, &id, true);
            }
            let (status, actual): (String, i64) = transaction.query_row(
                "SELECT workflow_status,row_version FROM commercial_documents WHERE id=?1 AND company_id=?2
                 AND document_type='SALES_ORDER' AND posting_status='DRAFT'",
                params![request.payload.document_id, context.company_id], |row| Ok((row.get(0)?,row.get(1)?)),
            ).optional()?.ok_or_else(Phase06Error::not_found)?;
            if actual != request.payload.row_version { return Err(Phase06Error::conflict()); }
            if target == "CANCELLED" && delivered_quantity(transaction, &context.company_id, &request.payload.document_id)? > 0 {
                return Err(Phase06Error::new("DELIVERED_ORDER_CANNOT_CANCEL", "Use a return for quantities already delivered."));
            }
            if !matches!(status.as_str(), "DRAFT" | "CONFIRMED" | "PARTIALLY_DELIVERED") {
                return Err(Phase06Error::immutable());
            }
            release_order_reservations(transaction, &context, &request.payload.document_id, target)?;
            transaction.execute(
                "UPDATE commercial_documents SET workflow_status=?1,updated_at=?2,updated_by=?3,row_version=row_version+1
                 WHERE id=?4 AND company_id=?5 AND row_version=?6",
                params![target, now_iso()?, context.user_id, request.payload.document_id, context.company_id, actual],
            )?;
            insert_status(transaction, &context, &request.payload.document_id, &status, target, request.payload.reason.as_deref())?;
            audit(transaction, &context, namespace, "commercial_document", &request.payload.document_id, request.payload.reason.as_deref())?;
            finish_idempotency(transaction, &context, namespace, &request.idempotency_key, "commercial_document", &request.payload.document_id)?;
            sales_entity(transaction, &context, &request.payload.document_id, false)
        })
    }
}

fn reserve_order_lines(
    transaction: &rusqlite::Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    order_id: &str,
) -> Phase06Result<()> {
    let mut statement = transaction.prepare(
        "SELECT line.id,line.product_id,line.warehouse_id,line.quantity_scaled,
          line.quantity_scaled-COALESCE((SELECT SUM(link.transformed_quantity_scaled)
           FROM document_line_links link WHERE link.company_id=line.company_id
             AND link.source_line_id=line.id AND link.transformation_type='ORDER_TO_DELIVERY'),0)
         FROM commercial_document_lines line WHERE line.document_id=?1 AND line.company_id=?2 ORDER BY line.line_number",
    )?;
    let rows = statement.query_map(params![order_id, context.company_id], |row| {
        Ok((row.get::<_, String>(0)?,row.get::<_, String>(1)?,row.get::<_, String>(2)?,row.get::<_, i64>(3)?,row.get::<_, i64>(4)?))
    })?.collect::<Result<Vec<_>,_>>()?;
    drop(statement);
    for (line_id, product_id, warehouse_id, _, remaining) in rows {
        if remaining <= 0 { continue; }
        let current = balance(transaction, &context.company_id, &product_id, &warehouse_id, None)?;
        if current.on_hand - current.reserved < remaining { return Err(Phase06Error::insufficient_stock()); }
        set_reserved(transaction, context, &product_id, &warehouse_id, None, remaining)?;
        let now = now_iso()?;
        transaction.execute(
            "INSERT INTO stock_reservations (id,company_id,product_id,warehouse_id,source_line_id,
             reserved_quantity_scaled,status,created_at,created_by,updated_at,updated_by)
             VALUES (?1,?2,?3,?4,?5,?6,'ACTIVE',?7,?8,?7,?8)",
            params![new_id(),context.company_id,product_id,warehouse_id,line_id,remaining,now,context.user_id],
        )?;
    }
    Ok(())
}

fn release_order_reservations(
    transaction: &rusqlite::Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    order_id: &str,
    target: &str,
) -> Phase06Result<()> {
    let mut statement = transaction.prepare(
        "SELECT reservation.id,reservation.product_id,reservation.warehouse_id,
                reservation.warehouse_location_id,reservation.reserved_quantity_scaled,reservation.row_version
         FROM stock_reservations reservation JOIN commercial_document_lines line ON line.id=reservation.source_line_id
         WHERE line.document_id=?1 AND reservation.company_id=?2 AND reservation.status IN ('ACTIVE','PARTIALLY_CONSUMED')",
    )?;
    let rows = statement.query_map(params![order_id,context.company_id], |row| {
        Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,Option<String>>(3)?,row.get::<_,i64>(4)?,row.get::<_,i64>(5)?))
    })?.collect::<Result<Vec<_>,_>>()?;
    drop(statement);
    for (id,product,warehouse,location,quantity,version) in rows {
        set_reserved(transaction,context,&product,&warehouse,location.as_deref(),-quantity)?;
        transaction.execute(
            "UPDATE stock_reservations SET status=?1,updated_at=?2,updated_by=?3,row_version=row_version+1
             WHERE id=?4 AND company_id=?5 AND row_version=?6",
            params![if target=="CANCELLED" {"CANCELLED"} else {"RELEASED"},now_iso()?,context.user_id,id,context.company_id,version],
        )?;
    }
    Ok(())
}

fn delivered_quantity(
    transaction: &rusqlite::Transaction<'_>,
    company_id: &str,
    order_id: &str,
) -> Phase06Result<i64> {
    Ok(transaction.query_row(
        "SELECT COALESCE(SUM(link.transformed_quantity_scaled),0)
         FROM document_line_links link JOIN commercial_document_lines source ON source.id=link.source_line_id
         WHERE link.company_id=?1 AND source.document_id=?2 AND link.transformation_type='ORDER_TO_DELIVERY'",
        params![company_id,order_id], |row| row.get(0),
    )?)
}

use rusqlite::{params, OptionalExtension};

use crate::phase06::{
    audit, authorize_transaction, begin_idempotency,
    error::{Phase06Error, Phase06Result},
    finish_idempotency, insert_document, new_id, now_iso,
    inventory::post_document,
    projections::{apply_movement, set_reserved, MovementSpec},
    request_hash, IdempotencyStart,
};

use super::{
    dto::{
        DeliverSalesOrderRequest, DirectSaleRequest, EntityResult, IdempotentRequest,
        InvoiceDeliveryRequest, SalesFlowResult,
    },
    service::{
        apply_document_pricing, enforce_below_cost, ensure_customer, insert_prepared_lines,
        insert_status, insert_transform_links, prepare_sales_lines, prepare_transformed_lines,
        sales_entity, PreparedSalesLine,
    },
    Phase07Service,
};

impl Phase07Service {
    pub fn deliver_sales_order(
        &self,
        request: IdempotentRequest<DeliverSalesOrderRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("delivery_note.post"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "delivery_note.post")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(transaction, &context, "sales_order.deliver", &request.idempotency_key, &hash)? {
                return sales_entity(transaction, &context, &id, true);
            }
            crate::phase06::validate_business_date(&request.payload.commercial_date)?;
            crate::phase06::projections::validate_warehouse_scope(transaction, &context.company_id, &request.payload.warehouse_id, None)?;
            let (customer_id, order_status): (String, String) = transaction.query_row(
                "SELECT partner_id,workflow_status FROM commercial_documents
                 WHERE id=?1 AND company_id=?2 AND document_type='SALES_ORDER' AND posting_status='DRAFT'",
                params![request.payload.order_id,context.company_id], |row| Ok((row.get(0)?,row.get(1)?)),
            ).optional()?.ok_or_else(Phase06Error::not_found)?;
            if !matches!(order_status.as_str(), "CONFIRMED" | "PARTIALLY_DELIVERED") {
                return Err(Phase06Error::immutable());
            }
            let (lines, totals, price_mode, header_rate) = prepare_transformed_lines(
                transaction,&context,&request.payload.order_id,&request.payload.lines,
            )?;
            if lines.iter().any(|line| line.warehouse_id != request.payload.warehouse_id) {
                return Err(Phase06Error::invalid("warehouseId"));
            }
            let (delivery_id, number) = insert_document(
                transaction,&context,"DELIVERY_NOTE","DRAFT","DRAFT",
                &request.payload.commercial_date,Some(&customer_id),Some(&request.payload.warehouse_id),
                Some(&request.payload.order_id),None,request.payload.notes.as_deref(),
                Some(&format!("sales-delivery:{}", request.idempotency_key)),(0,0,0),
            )?;
            let target_ids = insert_prepared_lines(transaction,&context,&delivery_id,&lines,request.payload.notes.as_deref())?;
            apply_document_pricing(transaction,&context.company_id,&delivery_id,&price_mode,header_rate,totals)?;
            insert_transform_links(transaction,&context,&lines,&target_ids,"ORDER_TO_DELIVERY")?;
            enforce_below_cost(
                transaction,&context,&lines,request.payload.below_cost_override_reason.as_deref(),
                "sales_delivery.below_cost",&delivery_id,
            )?;
            for (index,(line,target_line_id)) in lines.iter().zip(&target_ids).enumerate() {
                consume_order_reservation(transaction,&context,line)?;
                apply_movement(transaction,&context,MovementSpec {
                    product_id:&line.product_id,warehouse_id:&line.warehouse_id,location_id:None,
                    source_document_id:Some(&delivery_id),source_line_id:Some(target_line_id),
                    movement_type:"SALES_DELIVERY",business_date:&request.payload.commercial_date,
                    quantity_delta:-line.quantity_scaled,inbound_cost:None,recalculate_average:false,
                    posting_event_key:&format!("sales-delivery:{delivery_id}:{}",index+1),
                    transfer_group_id:None,notes:request.payload.notes.as_deref(),allow_negative:false,
                })?;
            }
            post_document(transaction,&context,&delivery_id,"delivery_note.post",request.payload.notes.as_deref())?;
            update_order_delivery_status(transaction,&context,&request.payload.order_id)?;
            finish_idempotency(transaction,&context,"sales_order.deliver",&request.idempotency_key,"commercial_document",&delivery_id)?;
            Ok(EntityResult { id:delivery_id,document_number:Some(number),status:"POSTED".to_owned(),row_version:2,replayed:false })
        })
    }

    pub fn invoice_sales_delivery(
        &self,
        request: IdempotentRequest<InvoiceDeliveryRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("sales_invoice.post"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction,&context,"sales_invoice.post")?;
            if let IdempotencyStart::Replayed(id)=begin_idempotency(transaction,&context,"sales_delivery.invoice",&request.idempotency_key,&hash)? {
                return sales_entity(transaction,&context,&id,true);
            }
            crate::phase06::validate_business_date(&request.payload.commercial_date)?;
            let (customer_id,warehouse_id):(String,String)=transaction.query_row(
                "SELECT partner_id,warehouse_id FROM commercial_documents WHERE id=?1 AND company_id=?2
                 AND document_type='DELIVERY_NOTE' AND posting_status='POSTED' AND workflow_status IN ('POSTED','PARTIALLY_INVOICED')",
                params![request.payload.delivery_id,context.company_id],|row|Ok((row.get(0)?,row.get(1)?)),
            ).optional()?.ok_or_else(Phase06Error::not_found)?;
            let (lines,totals,price_mode,header_rate)=prepare_transformed_lines(transaction,&context,&request.payload.delivery_id,&request.payload.lines)?;
            let (invoice_id,number)=insert_document(
                transaction,&context,"SALES_INVOICE","DRAFT","DRAFT",&request.payload.commercial_date,
                Some(&customer_id),Some(&warehouse_id),Some(&request.payload.delivery_id),request.payload.due_date.as_deref(),
                request.payload.notes.as_deref(),Some(&format!("sales-invoice:{}",request.idempotency_key)),(0,0,0),
            )?;
            let targets=insert_prepared_lines(transaction,&context,&invoice_id,&lines,request.payload.notes.as_deref())?;
            apply_document_pricing(transaction,&context.company_id,&invoice_id,&price_mode,header_rate,totals)?;
            insert_transform_links(transaction,&context,&lines,&targets,"DELIVERY_TO_INVOICE")?;
            post_document(transaction,&context,&invoice_id,"sales_invoice.post",request.payload.notes.as_deref())?;
            update_delivery_invoice_status(transaction,&context,&request.payload.delivery_id)?;
            finish_idempotency(transaction,&context,"sales_delivery.invoice",&request.idempotency_key,"commercial_document",&invoice_id)?;
            Ok(EntityResult{id:invoice_id,document_number:Some(number),status:"POSTED".to_owned(),row_version:2,replayed:false})
        })
    }

    pub fn direct_sale(
        &self,
        request: IdempotentRequest<DirectSaleRequest>,
    ) -> Phase06Result<SalesFlowResult> {
        let context=self.context(Some("sales_invoice.post"))?;
        let hash=request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction,&context,"sales_invoice.post")?;
            authorize_transaction(transaction,&context,"delivery_note.post")?;
            if let IdempotencyStart::Replayed(id)=begin_idempotency(transaction,&context,"sales.direct",&request.idempotency_key,&hash)? {
                let source:Option<String>=transaction.query_row(
                    "SELECT source_document_id FROM commercial_documents WHERE id=?1 AND company_id=?2",
                    params![id,context.company_id],|row|row.get(0),
                ).optional()?.flatten();
                return Ok(SalesFlowResult{primary:sales_entity(transaction,&context,&id,true)?,related_document_ids:source.into_iter().collect()});
            }
            crate::phase06::validate_business_date(&request.payload.commercial_date)?;
            ensure_customer(transaction,&context.company_id,&request.payload.customer_id)?;
            let (lines,totals)=prepare_sales_lines(
                transaction,&context,&request.payload.lines,&request.payload.warehouse_id,
                &request.payload.commercial_date,&request.payload.price_mode,request.payload.header_discount_rate_scaled,
            )?;
            let (delivery_id,_)=insert_document(
                transaction,&context,"DELIVERY_NOTE","DRAFT","DRAFT",&request.payload.commercial_date,
                Some(&request.payload.customer_id),Some(&request.payload.warehouse_id),None,None,
                Some("Internal delivery for direct sale"),None,(0,0,0),
            )?;
            let delivery_lines=insert_prepared_lines(transaction,&context,&delivery_id,&lines,Some("Direct sale"))?;
            apply_document_pricing(transaction,&context.company_id,&delivery_id,&request.payload.price_mode,request.payload.header_discount_rate_scaled,totals)?;
            let (invoice_id,number)=insert_document(
                transaction,&context,"SALES_INVOICE","DRAFT","DRAFT",&request.payload.commercial_date,
                Some(&request.payload.customer_id),Some(&request.payload.warehouse_id),Some(&delivery_id),request.payload.due_date.as_deref(),
                request.payload.notes.as_deref(),Some(&format!("sales-direct:{}",request.idempotency_key)),(0,0,0),
            )?;
            let mut invoice_source=lines.clone();
            for (line,source) in invoice_source.iter_mut().zip(&delivery_lines) { line.source_line_id=Some(source.clone()); }
            let invoice_lines=insert_prepared_lines(transaction,&context,&invoice_id,&invoice_source,request.payload.notes.as_deref())?;
            apply_document_pricing(transaction,&context.company_id,&invoice_id,&request.payload.price_mode,request.payload.header_discount_rate_scaled,totals)?;
            insert_transform_links(transaction,&context,&invoice_source,&invoice_lines,"DELIVERY_TO_INVOICE")?;
            enforce_below_cost(transaction,&context,&lines,request.payload.below_cost_override_reason.as_deref(),"sales_direct.below_cost",&invoice_id)?;
            for (index,(line,line_id)) in lines.iter().zip(&delivery_lines).enumerate() {
                apply_movement(transaction,&context,MovementSpec{
                    product_id:&line.product_id,warehouse_id:&line.warehouse_id,location_id:None,
                    source_document_id:Some(&delivery_id),source_line_id:Some(line_id),movement_type:"SALES_DELIVERY",
                    business_date:&request.payload.commercial_date,quantity_delta:-line.quantity_scaled,
                    inbound_cost:None,recalculate_average:false,posting_event_key:&format!("sales-direct:{delivery_id}:{}",index+1),
                    transfer_group_id:None,notes:request.payload.notes.as_deref(),allow_negative:false,
                })?;
            }
            post_document(transaction,&context,&delivery_id,"delivery_note.post",Some("Direct sale"))?;
            post_document(transaction,&context,&invoice_id,"sales_invoice.post",Some("Direct sale"))?;
            finish_idempotency(transaction,&context,"sales.direct",&request.idempotency_key,"commercial_document",&invoice_id)?;
            Ok(SalesFlowResult{
                primary:EntityResult{id:invoice_id,document_number:Some(number),status:"POSTED".to_owned(),row_version:2,replayed:false},
                related_document_ids:vec![delivery_id],
            })
        })
    }
}

fn consume_order_reservation(
    transaction:&rusqlite::Transaction<'_>,
    context:&crate::phase05::Phase06AuthContext,
    line:&PreparedSalesLine,
) -> Phase06Result<()> {
    let source=line.source_line_id.as_deref().ok_or_else(||Phase06Error::invalid("sourceLineId"))?;
    let row=transaction.query_row(
        "SELECT id,reserved_quantity_scaled,row_version FROM stock_reservations
         WHERE company_id=?1 AND source_line_id=?2 AND status IN ('ACTIVE','PARTIALLY_CONSUMED')
         ORDER BY created_at DESC LIMIT 1",
        params![context.company_id,source],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,i64>(2)?)),
    ).optional()?.ok_or_else(||Phase06Error::new("RESERVATION_REQUIRED","The order line is not reserved."))?;
    if line.quantity_scaled>row.1 {return Err(Phase06Error::insufficient_stock());}
    set_reserved(transaction,context,&line.product_id,&line.warehouse_id,None,-line.quantity_scaled)?;
    let remaining=row.1-line.quantity_scaled;
    transaction.execute(
        "UPDATE stock_reservations SET reserved_quantity_scaled=?1,status=?2,updated_at=?3,updated_by=?4,row_version=row_version+1
         WHERE id=?5 AND company_id=?6 AND row_version=?7",
        params![if remaining==0{row.1}else{remaining},if remaining==0{"CONSUMED"}else{"PARTIALLY_CONSUMED"},now_iso()?,context.user_id,row.0,context.company_id,row.2],
    )?;
    Ok(())
}

fn update_order_delivery_status(
    transaction:&rusqlite::Transaction<'_>,
    context:&crate::phase05::Phase06AuthContext,
    order_id:&str,
) -> Phase06Result<()> {
    let remaining:i64=transaction.query_row(
        "SELECT COUNT(*) FROM commercial_document_lines source
         WHERE source.document_id=?1 AND source.company_id=?2 AND
          COALESCE((SELECT SUM(link.transformed_quantity_scaled) FROM document_line_links link
           WHERE link.company_id=source.company_id AND link.source_line_id=source.id
             AND link.transformation_type='ORDER_TO_DELIVERY'),0)<source.quantity_scaled",
        params![order_id,context.company_id],|row|row.get(0),
    )?;
    let old:String=transaction.query_row("SELECT workflow_status FROM commercial_documents WHERE id=?1 AND company_id=?2",params![order_id,context.company_id],|row|row.get(0))?;
    let target=if remaining==0{"DELIVERED"}else{"PARTIALLY_DELIVERED"};
    if old!=target {
        transaction.execute(
            "UPDATE commercial_documents SET workflow_status=?1,updated_at=?2,updated_by=?3,row_version=row_version+1 WHERE id=?4 AND company_id=?5",
            params![target,now_iso()?,context.user_id,order_id,context.company_id],
        )?;
        insert_status(transaction,context,order_id,&old,target,Some("Delivery posted"))?;
    }
    Ok(())
}

fn update_delivery_invoice_status(
    transaction:&rusqlite::Transaction<'_>,
    context:&crate::phase05::Phase06AuthContext,
    delivery_id:&str,
) -> Phase06Result<()> {
    let remaining:i64=transaction.query_row(
        "SELECT COUNT(*) FROM commercial_document_lines source WHERE source.document_id=?1 AND source.company_id=?2 AND
         COALESCE((SELECT SUM(link.transformed_quantity_scaled) FROM document_line_links link WHERE link.company_id=source.company_id
          AND link.source_line_id=source.id AND link.transformation_type='DELIVERY_TO_INVOICE'),0)<source.quantity_scaled",
        params![delivery_id,context.company_id],|row|row.get(0),
    )?;
    let target=if remaining==0{"INVOICED"}else{"PARTIALLY_INVOICED"};
    transaction.execute(
        "UPDATE commercial_documents SET workflow_status=?1,updated_at=?2,updated_by=?3,row_version=row_version+1
         WHERE id=?4 AND company_id=?5 AND document_type='DELIVERY_NOTE' AND posting_status='POSTED'",
        params![target,now_iso()?,context.user_id,delivery_id,context.company_id],
    )?;
    Ok(())
}

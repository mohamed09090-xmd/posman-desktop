use rusqlite::{params, OptionalExtension};

use crate::phase06::{
    authorize_transaction, begin_idempotency,
    error::{Phase06Error, Phase06Result},
    finish_idempotency, insert_document,
    inventory::post_document,
    projections::{apply_movement, MovementSpec},
    request_hash, IdempotencyStart,
};

use super::{
    dto::{IdempotentRequest, SalesFlowResult, SalesReturnRequest},
    service::{
        apply_document_pricing, insert_prepared_lines, insert_transform_links,
        prepare_transformed_lines, sales_entity,
    },
    Phase07Service,
};

impl Phase07Service {
    pub fn post_sales_return(
        &self,
        request: IdempotentRequest<SalesReturnRequest>,
    ) -> Phase06Result<SalesFlowResult> {
        let context = self.context(Some("sales_invoice.post"))?;
        if request.payload.reason.trim().len() < 5 {
            return Err(Phase06Error::invalid("reason"));
        }
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "sales_invoice.post")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "sales.return_credit",
                &request.idempotency_key,
                &hash,
            )? {
                let source = transaction
                    .query_row(
                        "SELECT source_document_id FROM commercial_documents WHERE id=?1 AND company_id=?2",
                        params![id, context.company_id],
                        |row| row.get::<_, Option<String>>(0),
                    )
                    .optional()?
                    .flatten();
                return Ok(SalesFlowResult {
                    primary: sales_entity(transaction, &context, &id, true)?,
                    related_document_ids: source.into_iter().collect(),
                });
            }
            crate::phase06::validate_business_date(&request.payload.commercial_date)?;
            let source_valid = transaction
                .query_row(
                    "SELECT 1 FROM commercial_documents WHERE id=?1 AND company_id=?2
                     AND document_type IN ('DELIVERY_NOTE','SALES_INVOICE') AND posting_status='POSTED'
                     AND partner_id=?3",
                    params![request.payload.source_document_id, context.company_id, request.payload.customer_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .is_some();
            if !source_valid {
                return Err(Phase06Error::not_found());
            }
            let (lines, totals, price_mode, header_rate) = prepare_transformed_lines(
                transaction,
                &context,
                &request.payload.source_document_id,
                &request.payload.lines,
            )?;
            if lines.iter().any(|line| line.warehouse_id != request.payload.warehouse_id) {
                return Err(Phase06Error::invalid("warehouseId"));
            }
            let (return_id, _) = insert_document(
                transaction,
                &context,
                "SALES_RETURN",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                Some(&request.payload.customer_id),
                Some(&request.payload.warehouse_id),
                Some(&request.payload.source_document_id),
                None,
                Some(&request.payload.reason),
                None,
                (0, 0, 0),
            )?;
            let return_lines = insert_prepared_lines(
                transaction,
                &context,
                &return_id,
                &lines,
                Some(&request.payload.reason),
            )?;
            apply_document_pricing(transaction, &context.company_id, &return_id, &price_mode, header_rate, totals)?;
            insert_transform_links(transaction, &context, &lines, &return_lines, "DOCUMENT_TO_RETURN")?;

            let (credit_id, credit_number) = insert_document(
                transaction,
                &context,
                "SALES_CREDIT_NOTE",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                Some(&request.payload.customer_id),
                Some(&request.payload.warehouse_id),
                Some(&return_id),
                None,
                Some(&request.payload.reason),
                Some(&format!("sales-credit:{}", request.idempotency_key)),
                (0, 0, 0),
            )?;
            let mut credit_source = lines.clone();
            for (line, source_id) in credit_source.iter_mut().zip(&return_lines) {
                line.source_line_id = Some(source_id.clone());
            }
            let credit_lines = insert_prepared_lines(
                transaction,
                &context,
                &credit_id,
                &credit_source,
                Some(&request.payload.reason),
            )?;
            apply_document_pricing(transaction, &context.company_id, &credit_id, &price_mode, header_rate, totals)?;
            insert_transform_links(transaction, &context, &credit_source, &credit_lines, "DOCUMENT_TO_CREDIT")?;

            for (index, (line, line_id)) in lines.iter().zip(&return_lines).enumerate() {
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &line.product_id,
                        warehouse_id: &line.warehouse_id,
                        location_id: None,
                        source_document_id: Some(&return_id),
                        source_line_id: Some(line_id),
                        movement_type: "SALES_RETURN",
                        business_date: &request.payload.commercial_date,
                        quantity_delta: line.quantity_scaled,
                        inbound_cost: Some(line.unit_cost_scaled),
                        recalculate_average: false,
                        posting_event_key: &format!("sales-return:{return_id}:{}", index + 1),
                        transfer_group_id: None,
                        notes: Some(&request.payload.reason),
                        allow_negative: false,
                    },
                )?;
            }
            post_document(transaction, &context, &return_id, "sales_return.post", Some(&request.payload.reason))?;
            post_document(transaction, &context, &credit_id, "sales_credit_note.post", Some(&request.payload.reason))?;
            finish_idempotency(
                transaction,
                &context,
                "sales.return_credit",
                &request.idempotency_key,
                "commercial_document",
                &credit_id,
            )?;
            Ok(SalesFlowResult {
                primary: crate::phase06::dto::EntityResult {
                    id: credit_id,
                    document_number: Some(credit_number),
                    status: "POSTED".to_owned(),
                    row_version: 2,
                    replayed: false,
                },
                related_document_ids: vec![return_id],
            })
        })
    }
}

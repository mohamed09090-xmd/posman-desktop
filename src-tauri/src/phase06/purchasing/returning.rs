use super::*;

impl Phase06Service {
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
                        posting_event_key: &format!("purchase-return:{document_id}:{}", index + 1),
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
}

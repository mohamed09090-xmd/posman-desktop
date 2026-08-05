use super::*;

impl Phase06Service {
    pub fn post_transfer(
        &self,
        request: IdempotentRequest<TransferRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.transfer"))?;
        validate_commercial_date(&request.payload.commercial_date)?;
        if request.payload.lines.is_empty()
            || request
                .payload
                .lines
                .iter()
                .any(|line| line.quantity_scaled <= 0)
        {
            return Err(Phase06Error::invalid("lines"));
        }
        if request.payload.source_warehouse_id == request.payload.destination_warehouse_id
            && request.payload.source_location_id == request.payload.destination_location_id
        {
            return Err(Phase06Error::invalid("destination"));
        }
        let hash = request_hash(&request.payload)?;

        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.transfer")?;
            let allow_negative = negative_override_allowed(
                transaction,
                &context,
                request.payload.allow_negative_override,
                request.payload.reason.as_deref(),
            )?;
            if let IdempotencyStart::Replayed(document_id) = begin_idempotency(
                transaction,
                &context,
                "stock.transfer.post",
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(transaction, &context.company_id, &document_id, true);
            }

            let transfer_group_id = new_id();
            let document_notes = json!({
                "destinationWarehouseId": request.payload.destination_warehouse_id,
                "sourceLocationId": request.payload.source_location_id,
                "destinationLocationId": request.payload.destination_location_id,
                "reason": request.payload.reason,
                "transferGroupId": transfer_group_id,
            })
            .to_string();
            let document_key =
                document_idempotency_key("stock.transfer.post", &request.idempotency_key)?;
            let (document_id, document_number) = insert_document(
                transaction,
                &context,
                "STOCK_TRANSFER",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                None,
                Some(&request.payload.source_warehouse_id),
                None,
                None,
                Some(&document_notes),
                Some(&document_key),
                (0, 0, 0),
            )?;
            let stock_lines = request
                .payload
                .lines
                .iter()
                .map(|line| StockLineInput {
                    product_id: line.product_id.clone(),
                    warehouse_location_id: request.payload.source_location_id.clone(),
                    quantity_scaled: line.quantity_scaled,
                    unit_cost_scaled: None,
                })
                .collect::<Vec<_>>();
            insert_stock_document_lines(
                transaction,
                &context,
                &document_id,
                &request.payload.source_warehouse_id,
                &stock_lines,
            )?;

            let mut statement = transaction.prepare(
                r#"
                SELECT id, product_id, quantity_scaled
                FROM commercial_document_lines
                WHERE document_id=?1 AND company_id=?2
                ORDER BY line_number
                "#,
            )?;
            let lines = statement
                .query_map(params![document_id, context.company_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            drop(statement);

            for (index, (line_id, product_id, quantity)) in lines.into_iter().enumerate() {
                let source_balance = balance(
                    transaction,
                    &context.company_id,
                    &product_id,
                    &request.payload.source_warehouse_id,
                    None,
                )?;
                let carry_cost = source_balance.average_cost;
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &product_id,
                        warehouse_id: &request.payload.source_warehouse_id,
                        location_id: request.payload.source_location_id.as_deref(),
                        source_document_id: Some(&document_id),
                        source_line_id: Some(&line_id),
                        movement_type: "TRANSFER_OUT",
                        business_date: &request.payload.commercial_date,
                        quantity_delta: -quantity,
                        inbound_cost: None,
                        recalculate_average: false,
                        posting_event_key: &format!("transfer:{document_id}:{}:out", index + 1),
                        transfer_group_id: Some(&transfer_group_id),
                        notes: request.payload.reason.as_deref(),
                        allow_negative,
                    },
                )?;
                let cross_warehouse =
                    request.payload.source_warehouse_id != request.payload.destination_warehouse_id;
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &product_id,
                        warehouse_id: &request.payload.destination_warehouse_id,
                        location_id: request.payload.destination_location_id.as_deref(),
                        source_document_id: Some(&document_id),
                        source_line_id: Some(&line_id),
                        movement_type: "TRANSFER_IN",
                        business_date: &request.payload.commercial_date,
                        quantity_delta: quantity,
                        inbound_cost: Some(carry_cost),
                        recalculate_average: cross_warehouse,
                        posting_event_key: &format!("transfer:{document_id}:{}:in", index + 1),
                        transfer_group_id: Some(&transfer_group_id),
                        notes: request.payload.reason.as_deref(),
                        allow_negative: false,
                    },
                )?;
            }

            post_document(
                transaction,
                &context,
                &document_id,
                "stock.transfer.post",
                request.payload.reason.as_deref(),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "stock.transfer.post",
                &request.idempotency_key,
                "commercial_document",
                &document_id,
            )?;
            Ok(EntityResult {
                id: document_id,
                document_number: Some(document_number),
                status: "POSTED".to_owned(),
                row_version: 2,
                replayed: false,
            })
        })
    }
}

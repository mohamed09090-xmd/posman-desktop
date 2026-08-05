    pub fn post_adjustment(
        &self,
        request: IdempotentRequest<AdjustmentRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.adjust"))?;
        validate_commercial_date(&request.payload.commercial_date)?;
        validate_stock_lines(&request.payload.lines, true)?;
        if request.payload.reason.trim().is_empty() {
            return Err(Phase06Error::invalid("reason"));
        }
        let hash = request_hash(&request.payload)?;

        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.adjust")?;
            let allow_negative = negative_override_allowed(
                transaction,
                &context,
                request.payload.allow_negative_override,
                Some(&request.payload.reason),
            )?;
            if let IdempotencyStart::Replayed(document_id) = begin_idempotency(
                transaction,
                &context,
                "stock.adjust.post",
                &request.idempotency_key,
                &hash,
            )? {
                return entity_result(
                    transaction,
                    &context.company_id,
                    &document_id,
                    true,
                );
            }

            let document_key =
                document_idempotency_key("stock.adjust.post", &request.idempotency_key)?;
            let (document_id, document_number) = insert_document(
                transaction,
                &context,
                "STOCK_ADJUSTMENT",
                "DRAFT",
                "DRAFT",
                &request.payload.commercial_date,
                None,
                Some(&request.payload.warehouse_id),
                None,
                None,
                Some(&request.payload.reason),
                Some(&document_key),
                (0, 0, 0),
            )?;
            insert_stock_document_lines(
                transaction,
                &context,
                &document_id,
                &request.payload.warehouse_id,
                &request.payload.lines,
            )?;

            let mut statement = transaction.prepare(
                r#"
                SELECT id, product_id, unit_cost_scaled, notes
                FROM commercial_document_lines
                WHERE document_id=?1 AND company_id=?2
                ORDER BY line_number
                "#,
            )?;
            let persisted_lines = statement
                .query_map(params![document_id, context.company_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            drop(statement);

            for (index, (line_id, product_id, explicit_cost, notes)) in
                persisted_lines.into_iter().enumerate()
            {
                let signed_quantity = request.payload.lines[index].quantity_scaled;
                let movement_type = if signed_quantity > 0 {
                    "ADJUSTMENT_IN"
                } else {
                    "ADJUSTMENT_OUT"
                };
                let current = balance(
                    transaction,
                    &context.company_id,
                    &product_id,
                    &request.payload.warehouse_id,
                    None,
                )?;
                let cost = if signed_quantity > 0 {
                    explicit_cost
                        .or_else(|| {
                            (current.average_cost > 0).then_some(current.average_cost)
                        })
                        .ok_or_else(|| Phase06Error::invalid("unitCostScaled"))?
                } else {
                    current.average_cost
                };
                let location_id = line_location(notes);
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &product_id,
                        warehouse_id: &request.payload.warehouse_id,
                        location_id: location_id.as_deref(),
                        source_document_id: Some(&document_id),
                        source_line_id: Some(&line_id),
                        movement_type,
                        business_date: &request.payload.commercial_date,
                        quantity_delta: signed_quantity,
                        inbound_cost: Some(cost),
                        recalculate_average: signed_quantity > 0,
                        posting_event_key: &format!(
                            "adjustment:{document_id}:{}",
                            index + 1
                        ),
                        transfer_group_id: None,
                        notes: Some(&request.payload.reason),
                        allow_negative,
                    },
                )?;
            }

            post_document(
                transaction,
                &context,
                &document_id,
                "stock.adjust.post",
                Some(&request.payload.reason),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "stock.adjust.post",
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

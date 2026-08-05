    pub fn create_opening_stock(
        &self,
        request: OpeningDraftRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.opening.post"))?;
        validate_commercial_date(&request.commercial_date)?;
        validate_stock_lines(&request.lines, false)?;
        if request
            .lines
            .iter()
            .any(|line| line.unit_cost_scaled.is_none())
        {
            return Err(Phase06Error::invalid("unitCostScaled"));
        }

        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.opening.post")?;
            let (document_id, document_number) = insert_document(
                transaction,
                &context,
                "OPENING_STOCK",
                "DRAFT",
                "DRAFT",
                &request.commercial_date,
                None,
                Some(&request.warehouse_id),
                None,
                None,
                request.notes.as_deref(),
                None,
                (0, 0, 0),
            )?;
            insert_stock_document_lines(
                transaction,
                &context,
                &document_id,
                &request.warehouse_id,
                &request.lines,
            )?;
            audit(
                transaction,
                &context,
                "stock.opening.create",
                "commercial_document",
                &document_id,
                None,
            )?;
            Ok(EntityResult {
                id: document_id,
                document_number: Some(document_number),
                status: "DRAFT".to_owned(),
                row_version: 1,
                replayed: false,
            })
        })
    }

    pub fn review_opening_stock(
        &self,
        request: DocumentActionRequest,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.opening.post"))?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.opening.post")?;
            if opening_reviewed(transaction, &context.company_id, &request.document_id)? {
                return Err(Phase06Error::conflict());
            }
            let changed = transaction.execute(
                r#"
                UPDATE commercial_documents
                SET updated_at=?1, updated_by=?2, row_version=row_version+1
                WHERE id=?3 AND company_id=?4 AND document_type='OPENING_STOCK'
                  AND workflow_status='DRAFT' AND posting_status='DRAFT'
                  AND row_version=?5
                "#,
                params![
                    now_iso()?,
                    context.user_id,
                    request.document_id,
                    context.company_id,
                    request.row_version
                ],
            )?;
            if changed != 1 {
                return Err(Phase06Error::conflict());
            }
            insert_status_history(
                transaction,
                &context,
                &request.document_id,
                Some("DRAFT"),
                "REVIEWED",
                request.reason.as_deref(),
                request.row_version + 1,
            )?;
            audit(
                transaction,
                &context,
                "stock.opening.review",
                "commercial_document",
                &request.document_id,
                None,
            )?;
            entity_result(
                transaction,
                &context.company_id,
                &request.document_id,
                false,
            )
        })
    }

    pub fn post_opening_stock(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.opening.post"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.opening.post")?;
            match begin_idempotency(
                transaction,
                &context,
                "stock.opening.post",
                &request.idempotency_key,
                &hash,
            )? {
                IdempotencyStart::Replayed(document_id) => {
                    return entity_result(
                        transaction,
                        &context.company_id,
                        &document_id,
                        true,
                    );
                }
                IdempotencyStart::New => {}
            }

            let (commercial_date, warehouse_id, posting_status, row_version) =
                transaction
                    .query_row(
                        r#"
                        SELECT commercial_date, warehouse_id, posting_status, row_version
                        FROM commercial_documents
                        WHERE id=?1 AND company_id=?2
                          AND document_type='OPENING_STOCK'
                        "#,
                        params![request.payload.document_id, context.company_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, i64>(3)?,
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
            if !opening_reviewed(
                transaction,
                &context.company_id,
                &request.payload.document_id,
            )? {
                return Err(Phase06Error::new(
                    "REVIEW_REQUIRED",
                    "Opening stock must be reviewed before posting.",
                ));
            }

            let mut statement = transaction.prepare(
                r#"
                SELECT id, product_id, quantity_scaled, unit_cost_scaled, notes
                FROM commercial_document_lines
                WHERE document_id=?1 AND company_id=?2
                ORDER BY line_number
                "#,
            )?;
            let lines = statement
                .query_map(
                    params![request.payload.document_id, context.company_id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, Option<i64>>(3)?,
                            row.get::<_, Option<String>>(4)?,
                        ))
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;
            drop(statement);

            for (_, product_id, _, _, _) in &lines {
                let existing_activity: i64 = transaction.query_row(
                    r#"
                    SELECT COUNT(*) FROM stock_movements
                    WHERE company_id=?1 AND product_id=?2 AND warehouse_id=?3
                    "#,
                    params![context.company_id, product_id, warehouse_id],
                    |row| row.get(0),
                )?;
                if existing_activity > 0 {
                    return Err(Phase06Error::new(
                        "OPENING_ACTIVITY_EXISTS",
                        "Opening stock is allowed only before warehouse activity begins for the product.",
                    ));
                }
            }

            for (index, (line_id, product_id, quantity, cost, notes)) in
                lines.into_iter().enumerate()
            {
                let location_id = line_location(notes);
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &product_id,
                        warehouse_id: &warehouse_id,
                        location_id: location_id.as_deref(),
                        source_document_id: Some(&request.payload.document_id),
                        source_line_id: Some(&line_id),
                        movement_type: "OPENING",
                        business_date: &commercial_date,
                        quantity_delta: quantity,
                        inbound_cost: cost,
                        recalculate_average: true,
                        posting_event_key: &format!(
                            "opening:{}:{}",
                            request.payload.document_id,
                            index + 1
                        ),
                        transfer_group_id: None,
                        notes: request.payload.reason.as_deref(),
                        allow_negative: false,
                    },
                )?;
            }

            post_document(
                transaction,
                &context,
                &request.payload.document_id,
                "stock.opening.post",
                request.payload.reason.as_deref(),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "stock.opening.post",
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

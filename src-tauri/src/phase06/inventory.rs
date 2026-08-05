use rusqlite::{params, OptionalExtension};
use serde_json::json;

use super::{
    audit, authorize_transaction, begin_idempotency, document_idempotency_key,
    dto::{
        AdjustmentRequest, DocumentActionRequest, EntityResult, IdempotentRequest, MovementView,
        OpeningDraftRequest, StockBalanceView, StockLineInput, StockQuery, TransferRequest,
    },
    entity_result,
    error::{Phase06Error, Phase06Result},
    finish_idempotency, insert_document, insert_status_history, new_id, now_iso, opening_reviewed,
    product_snapshot,
    projections::{apply_movement, balance, MovementSpec},
    request_hash, validate_commercial_date, IdempotencyStart, Phase06Service,
};

fn validate_stock_lines(lines: &[StockLineInput], allow_signed: bool) -> Phase06Result<()> {
    if lines.is_empty() {
        return Err(Phase06Error::invalid("lines"));
    }
    for line in lines {
        if (allow_signed && line.quantity_scaled == 0)
            || (!allow_signed && line.quantity_scaled <= 0)
            || line.unit_cost_scaled.is_some_and(|value| value < 0)
        {
            return Err(Phase06Error::invalid("stockLine"));
        }
    }
    Ok(())
}

fn line_location(notes: Option<String>) -> Option<String> {
    notes
        .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
        .and_then(|value| {
            value
                .get("locationId")
                .and_then(|location| location.as_str())
                .map(str::to_owned)
        })
}

pub(crate) fn negative_override_allowed(
    transaction: &rusqlite::Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    requested: bool,
    reason: Option<&str>,
) -> Phase06Result<bool> {
    if !requested {
        return Ok(false);
    }
    if reason.is_none_or(|value| value.trim().is_empty()) {
        return Err(Phase06Error::override_required());
    }
    authorize_transaction(transaction, context, "stock.negative.override")?;
    let policy = transaction
        .query_row(
            "SELECT negative_stock_policy FROM company_settings WHERE company_id=?1",
            [&context.company_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)?;
    if policy != "PRIVILEGED_OVERRIDE" {
        return Err(Phase06Error::override_required());
    }
    Ok(true)
}

impl Phase06Service {
    pub fn list_stock_balances(
        &self,
        query: StockQuery,
    ) -> Phase06Result<Vec<StockBalanceView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                r#"
                SELECT balance.product_id, product.code,
                       COALESCE(NULLIF(product.name_ar, ''), product.name_fr),
                       balance.warehouse_id,
                       COALESCE(NULLIF(warehouse.name_ar, ''), warehouse.name_fr),
                       balance.warehouse_location_id,
                       CASE WHEN location.id IS NULL THEN NULL
                            ELSE COALESCE(NULLIF(location.name_ar, ''), location.name_fr)
                       END,
                       balance.on_hand_scaled, balance.reserved_scaled,
                       balance.available_scaled, balance.average_cost_scaled,
                       balance.row_version
                FROM stock_balances AS balance
                JOIN products AS product
                  ON product.id=balance.product_id
                 AND product.company_id=balance.company_id
                JOIN warehouses AS warehouse
                  ON warehouse.id=balance.warehouse_id
                 AND warehouse.company_id=balance.company_id
                LEFT JOIN warehouse_locations AS location
                  ON location.id=balance.warehouse_location_id
                 AND location.company_id=balance.company_id
                WHERE balance.company_id=?1
                  AND (?2 IS NULL OR balance.product_id=?2)
                  AND (?3 IS NULL OR balance.warehouse_id=?3)
                  AND (?4 IS NULL OR ifnull(balance.warehouse_location_id, '')=ifnull(?4, ''))
                ORDER BY product.code, warehouse.code, ifnull(location.code, '')
                LIMIT ?5
                "#,
            )?;
            let raw = statement
                .query_map(
                    params![
                        context.company_id,
                        query.product_id,
                        query.warehouse_id,
                        query.warehouse_location_id,
                        query.limit.unwrap_or(500).clamp(1, 2_000)
                    ],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, Option<String>>(6)?,
                            row.get::<_, i64>(7)?,
                            row.get::<_, i64>(8)?,
                            row.get::<_, i64>(9)?,
                            row.get::<_, i64>(10)?,
                            row.get::<_, i64>(11)?,
                        ))
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;

            raw.into_iter()
                .map(
                    |(
                        product_id,
                        product_code,
                        product_name,
                        warehouse_id,
                        warehouse_name,
                        warehouse_location_id,
                        location_name,
                        on_hand_scaled,
                        reserved_scaled,
                        available_scaled,
                        average_cost_scaled,
                        row_version,
                    )| {
                        let absolute = super::fixed_point::extended_cost_minor(
                            on_hand_scaled,
                            average_cost_scaled,
                        )?;
                        let inventory_value_minor = if on_hand_scaled < 0 {
                            -absolute
                        } else {
                            absolute
                        };
                        Ok(StockBalanceView {
                            product_id,
                            product_code,
                            product_name,
                            warehouse_id,
                            warehouse_name,
                            warehouse_location_id,
                            location_name,
                            on_hand_scaled,
                            reserved_scaled,
                            available_scaled,
                            average_cost_scaled,
                            inventory_value_minor,
                            row_version,
                        })
                    },
                )
                .collect()
        })
    }

    pub fn list_stock_movements(
        &self,
        query: StockQuery,
    ) -> Phase06Result<Vec<MovementView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                r#"
                SELECT id, product_id, warehouse_id, warehouse_location_id,
                       source_document_id, movement_type, business_date,
                       quantity_delta_scaled, quantity_after_scaled,
                       unit_cost_scaled, average_cost_after_scaled,
                       extended_cost_minor, notes
                FROM stock_movements
                WHERE company_id=?1
                  AND (?2 IS NULL OR product_id=?2)
                  AND (?3 IS NULL OR warehouse_id=?3)
                  AND (?4 IS NULL OR ifnull(warehouse_location_id, '')=ifnull(?4, ''))
                ORDER BY occurred_at DESC, id DESC
                LIMIT ?5
                "#,
            )?;
            let rows = statement
                .query_map(
                    params![
                        context.company_id,
                        query.product_id,
                        query.warehouse_id,
                        query.warehouse_location_id,
                        query.limit.unwrap_or(500).clamp(1, 2_000)
                    ],
                    |row| {
                        Ok(MovementView {
                            id: row.get(0)?,
                            product_id: row.get(1)?,
                            warehouse_id: row.get(2)?,
                            warehouse_location_id: row.get(3)?,
                            source_document_id: row.get(4)?,
                            movement_type: row.get(5)?,
                            business_date: row.get(6)?,
                            quantity_delta_scaled: row.get(7)?,
                            quantity_after_scaled: row.get(8)?,
                            unit_cost_scaled: row.get(9)?,
                            average_cost_after_scaled: row.get(10)?,
                            extended_cost_minor: row.get(11)?,
                            notes: row.get(12)?,
                        })
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
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
        if request.payload.source_warehouse_id
            == request.payload.destination_warehouse_id
            && request.payload.source_location_id
                == request.payload.destination_location_id
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
                return entity_result(
                    transaction,
                    &context.company_id,
                    &document_id,
                    true,
                );
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

            for (index, (line_id, product_id, quantity)) in
                lines.into_iter().enumerate()
            {
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
                        posting_event_key: &format!(
                            "transfer:{document_id}:{}:out",
                            index + 1
                        ),
                        transfer_group_id: Some(&transfer_group_id),
                        notes: request.payload.reason.as_deref(),
                        allow_negative,
                    },
                )?;
                let cross_warehouse = request.payload.source_warehouse_id
                    != request.payload.destination_warehouse_id;
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &product_id,
                        warehouse_id: &request.payload.destination_warehouse_id,
                        location_id: request
                            .payload
                            .destination_location_id
                            .as_deref(),
                        source_document_id: Some(&document_id),
                        source_line_id: Some(&line_id),
                        movement_type: "TRANSFER_IN",
                        business_date: &request.payload.commercial_date,
                        quantity_delta: quantity,
                        inbound_cost: Some(carry_cost),
                        recalculate_average: cross_warehouse,
                        posting_event_key: &format!(
                            "transfer:{document_id}:{}:in",
                            index + 1
                        ),
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

include!("inventory/document_support.rs");

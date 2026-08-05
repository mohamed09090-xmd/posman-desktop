use rusqlite::{params, Connection, OptionalExtension, Transaction};

use super::{
    audit, authorize_transaction, begin_idempotency,
    dto::{
        CountLineView, CountView, CreateCountRequest, DocumentActionRequest, EntityResult,
        IdempotentRequest, UpdateCountRequest,
    },
    error::{Phase06Error, Phase06Result},
    finish_idempotency, insert_document,
    inventory::post_document,
    new_id, now_iso,
    projections::{apply_movement, balance, MovementSpec},
    request_hash, IdempotencyStart, Phase06Service,
};

impl Phase06Service {
    pub fn create_inventory_count(&self, request: CreateCountRequest) -> Phase06Result<CountView> {
        let context = self.context(Some("stock.count"))?;
        validate_count_lines(&request.lines)?;
        if request.count_number.trim().is_empty() {
            return Err(Phase06Error::invalid("countNumber"));
        }

        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.count")?;
            super::validate_business_date(&request.commercial_date)?;
            super::projections::validate_warehouse_scope(
                transaction,
                &context.company_id,
                &request.warehouse_id,
                None,
            )?;

            let count_id = new_id();
            let now = now_iso()?;
            transaction.execute(
                "INSERT INTO inventory_counts (
                    id, company_id, warehouse_id, count_number, commercial_date,
                    status, notes, created_at, created_by, updated_at, updated_by
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'DRAFT', ?6, ?7, ?8, ?7, ?8)",
                params![
                    count_id,
                    context.company_id,
                    request.warehouse_id,
                    request.count_number.trim(),
                    request.commercial_date,
                    request.notes,
                    now,
                    context.user_id,
                ],
            )?;

            for line in &request.lines {
                super::projections::validate_product_scope(
                    transaction,
                    &context.company_id,
                    &line.product_id,
                )?;
                super::projections::validate_warehouse_scope(
                    transaction,
                    &context.company_id,
                    &request.warehouse_id,
                    line.warehouse_location_id.as_deref(),
                )?;
                let snapshot = balance(
                    transaction,
                    &context.company_id,
                    &line.product_id,
                    &request.warehouse_id,
                    line.warehouse_location_id.as_deref(),
                )?;
                transaction.execute(
                    "INSERT INTO inventory_count_lines (
                        id, company_id, inventory_count_id, product_id, warehouse_location_id,
                        system_quantity_scaled, counted_quantity_scaled, variance_quantity_scaled,
                        unit_cost_scaled, created_at, created_by, updated_at, updated_by
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7 - ?6, ?8, ?9, ?10, ?9, ?10)",
                    params![
                        new_id(),
                        context.company_id,
                        count_id,
                        line.product_id,
                        line.warehouse_location_id,
                        snapshot.on_hand,
                        line.counted_quantity_scaled,
                        line.unit_cost_scaled,
                        now,
                        context.user_id,
                    ],
                )?;
            }

            audit(
                transaction,
                &context,
                "stock.count.create",
                "inventory_count",
                &count_id,
                None,
            )?;
            load_count(transaction, &context.company_id, &count_id)
        })
    }

    pub fn update_inventory_count(&self, request: UpdateCountRequest) -> Phase06Result<CountView> {
        let context = self.context(Some("stock.count"))?;
        validate_count_lines(&request.lines)?;

        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.count")?;
            let (warehouse_id, status) = transaction
                .query_row(
                    "SELECT warehouse_id, status
                     FROM inventory_counts
                     WHERE id = ?1 AND company_id = ?2 AND row_version = ?3",
                    params![request.count_id, context.company_id, request.row_version],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?
                .ok_or_else(Phase06Error::conflict)?;
            if !matches!(status.as_str(), "DRAFT" | "COUNTING") {
                return Err(Phase06Error::immutable());
            }

            for line in &request.lines {
                super::projections::validate_warehouse_scope(
                    transaction,
                    &context.company_id,
                    &warehouse_id,
                    line.warehouse_location_id.as_deref(),
                )?;
                let changed = transaction.execute(
                    "UPDATE inventory_count_lines
                     SET counted_quantity_scaled = ?1,
                         variance_quantity_scaled = ?1 - system_quantity_scaled,
                         unit_cost_scaled = ?2,
                         updated_at = ?3,
                         updated_by = ?4,
                         row_version = row_version + 1
                     WHERE inventory_count_id = ?5
                       AND company_id = ?6
                       AND product_id = ?7
                       AND ifnull(warehouse_location_id, '') = ifnull(?8, '')",
                    params![
                        line.counted_quantity_scaled,
                        line.unit_cost_scaled,
                        now_iso()?,
                        context.user_id,
                        request.count_id,
                        context.company_id,
                        line.product_id,
                        line.warehouse_location_id,
                    ],
                )?;
                if changed != 1 {
                    return Err(Phase06Error::not_found());
                }
            }

            let changed = transaction.execute(
                "UPDATE inventory_counts
                 SET status = 'COUNTING', updated_at = ?1, updated_by = ?2,
                     row_version = row_version + 1
                 WHERE id = ?3 AND company_id = ?4 AND row_version = ?5",
                params![
                    now_iso()?,
                    context.user_id,
                    request.count_id,
                    context.company_id,
                    request.row_version,
                ],
            )?;
            if changed != 1 {
                return Err(Phase06Error::conflict());
            }
            audit(
                transaction,
                &context,
                "stock.count.update",
                "inventory_count",
                &request.count_id,
                None,
            )?;
            load_count(transaction, &context.company_id, &request.count_id)
        })
    }

    pub fn review_inventory_count(
        &self,
        request: DocumentActionRequest,
    ) -> Phase06Result<CountView> {
        let context = self.context(Some("stock.count"))?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.count")?;
            let changed = transaction.execute(
                "UPDATE inventory_counts
                 SET status = 'REVIEWED', updated_at = ?1, updated_by = ?2,
                     row_version = row_version + 1
                 WHERE id = ?3 AND company_id = ?4
                   AND status IN ('DRAFT', 'COUNTING') AND row_version = ?5",
                params![
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
            audit(
                transaction,
                &context,
                "stock.count.review",
                "inventory_count",
                &request.document_id,
                request.reason.as_deref(),
            )?;
            load_count(transaction, &context.company_id, &request.document_id)
        })
    }

    pub fn post_inventory_count(
        &self,
        request: IdempotentRequest<DocumentActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.count"))?;
        let hash = request_hash(&request.payload)?;

        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.count")?;
            if let IdempotencyStart::Replayed(count_id) = begin_idempotency(
                transaction,
                &context,
                "stock.count.post",
                &request.idempotency_key,
                &hash,
            )? {
                let count = load_count(transaction, &context.company_id, &count_id)?;
                return Ok(EntityResult {
                    id: count.id,
                    document_number: None,
                    status: count.status,
                    row_version: count.row_version,
                    replayed: true,
                });
            }

            let (warehouse_id, commercial_date, status, row_version) = transaction
                .query_row(
                    "SELECT warehouse_id, commercial_date, status, row_version
                     FROM inventory_counts WHERE id = ?1 AND company_id = ?2",
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
            if status != "REVIEWED" {
                return Err(Phase06Error::new(
                    "REVIEW_REQUIRED",
                    "The inventory count must be reviewed before posting.",
                ));
            }
            if row_version != request.payload.row_version {
                return Err(Phase06Error::conflict());
            }

            let lines = load_count_posting_lines(
                transaction,
                &context.company_id,
                &request.payload.document_id,
            )?;
            for line in &lines {
                let live = balance(
                    transaction,
                    &context.company_id,
                    &line.product_id,
                    &warehouse_id,
                    line.location_id.as_deref(),
                )?;
                if live.on_hand != line.system_quantity {
                    return Err(Phase06Error::stale_count());
                }
            }

            let (document_id, document_number) = insert_document(
                transaction,
                &context,
                "STOCK_ADJUSTMENT",
                "DRAFT",
                "DRAFT",
                &commercial_date,
                None,
                Some(&warehouse_id),
                None,
                None,
                Some("Physical count variance"),
                Some(&request.idempotency_key),
                (0, 0, 0),
            )?;

            for (index, line) in lines.iter().enumerate() {
                if line.variance == 0 {
                    continue;
                }
                let warehouse_balance = balance(
                    transaction,
                    &context.company_id,
                    &line.product_id,
                    &warehouse_id,
                    None,
                )?;
                let cost = if line.variance > 0 {
                    line.explicit_cost
                        .filter(|value| *value > 0)
                        .or_else(|| {
                            (warehouse_balance.average_cost > 0)
                                .then_some(warehouse_balance.average_cost)
                        })
                        .ok_or_else(|| Phase06Error::invalid("unitCostScaled"))?
                } else {
                    warehouse_balance.average_cost
                };
                apply_movement(
                    transaction,
                    &context,
                    MovementSpec {
                        product_id: &line.product_id,
                        warehouse_id: &warehouse_id,
                        location_id: line.location_id.as_deref(),
                        source_document_id: Some(&document_id),
                        source_line_id: None,
                        movement_type: "COUNT_VARIANCE",
                        business_date: &commercial_date,
                        quantity_delta: line.variance,
                        inbound_cost: Some(cost),
                        recalculate_average: line.variance > 0,
                        posting_event_key: &format!(
                            "count:{}:{}",
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
                &document_id,
                "stock.count.post",
                request.payload.reason.as_deref(),
            )?;
            let changed = transaction.execute(
                "UPDATE inventory_counts
                 SET status = 'POSTED', adjustment_document_id = ?1,
                     updated_at = ?2, updated_by = ?3, row_version = row_version + 1
                 WHERE id = ?4 AND company_id = ?5
                   AND status = 'REVIEWED' AND row_version = ?6",
                params![
                    document_id,
                    now_iso()?,
                    context.user_id,
                    request.payload.document_id,
                    context.company_id,
                    row_version,
                ],
            )?;
            if changed != 1 {
                return Err(Phase06Error::conflict());
            }
            audit(
                transaction,
                &context,
                "stock.count.post",
                "inventory_count",
                &request.payload.document_id,
                Some(&document_id),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "stock.count.post",
                &request.idempotency_key,
                "inventory_count",
                &request.payload.document_id,
            )?;
            Ok(EntityResult {
                id: request.payload.document_id.clone(),
                document_number: Some(document_number),
                status: "POSTED".to_owned(),
                row_version: row_version + 1,
                replayed: false,
            })
        })
    }

    pub fn get_inventory_count(&self, id: String) -> Phase06Result<CountView> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| load_count(connection, &context.company_id, &id))
    }
}

#[derive(Debug)]
struct CountPostingLine {
    product_id: String,
    location_id: Option<String>,
    system_quantity: i64,
    variance: i64,
    explicit_cost: Option<i64>,
}

fn validate_count_lines(lines: &[super::dto::CountLineInput]) -> Phase06Result<()> {
    if lines.is_empty() || lines.iter().any(|line| line.counted_quantity_scaled < 0) {
        return Err(Phase06Error::invalid("lines"));
    }
    let mut scopes = std::collections::BTreeSet::new();
    for line in lines {
        if !scopes.insert((line.product_id.clone(), line.warehouse_location_id.clone())) {
            return Err(Phase06Error::invalid("duplicateCountScope"));
        }
    }
    Ok(())
}

fn load_count_posting_lines(
    transaction: &Transaction<'_>,
    company_id: &str,
    count_id: &str,
) -> Phase06Result<Vec<CountPostingLine>> {
    let mut statement = transaction.prepare(
        "SELECT product_id, warehouse_location_id, system_quantity_scaled,
                variance_quantity_scaled, unit_cost_scaled
         FROM inventory_count_lines
         WHERE inventory_count_id = ?1 AND company_id = ?2
         ORDER BY id",
    )?;
    let rows = statement
        .query_map(params![count_id, company_id], |row| {
            Ok(CountPostingLine {
                product_id: row.get(0)?,
                location_id: row.get(1)?,
                system_quantity: row.get(2)?,
                variance: row.get(3)?,
                explicit_cost: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn load_count(connection: &Connection, company_id: &str, id: &str) -> Phase06Result<CountView> {
    let mut view = connection
        .query_row(
            "SELECT id, warehouse_id, count_number, commercial_date, status, row_version
             FROM inventory_counts WHERE id = ?1 AND company_id = ?2",
            params![id, company_id],
            |row| {
                Ok(CountView {
                    id: row.get(0)?,
                    warehouse_id: row.get(1)?,
                    count_number: row.get(2)?,
                    commercial_date: row.get(3)?,
                    status: row.get(4)?,
                    row_version: row.get(5)?,
                    lines: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)?;
    let mut statement = connection.prepare(
        "SELECT id, product_id, warehouse_location_id, system_quantity_scaled,
                counted_quantity_scaled, variance_quantity_scaled, unit_cost_scaled, row_version
         FROM inventory_count_lines
         WHERE inventory_count_id = ?1 AND company_id = ?2
         ORDER BY id",
    )?;
    view.lines = statement
        .query_map(params![id, company_id], |row| {
            Ok(CountLineView {
                id: row.get(0)?,
                product_id: row.get(1)?,
                warehouse_location_id: row.get(2)?,
                system_quantity_scaled: row.get(3)?,
                counted_quantity_scaled: row.get(4)?,
                variance_quantity_scaled: row.get(5)?,
                unit_cost_scaled: row.get(6)?,
                row_version: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(view)
}

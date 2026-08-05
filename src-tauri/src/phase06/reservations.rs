use rusqlite::{params, Connection, OptionalExtension, Transaction};

use super::{
    audit, authorize_transaction, begin_idempotency,
    dto::{
        EntityResult, IdempotentRequest, ReservationActionRequest, ReservationRequest,
        ReservationView,
    },
    error::{Phase06Error, Phase06Result},
    finish_idempotency, new_id, now_iso,
    projections::{balance, set_reserved},
    request_hash, IdempotencyStart, Phase06Service,
};

impl Phase06Service {
    pub fn list_active_reservations(&self) -> Phase06Result<Vec<ReservationView>> {
        let context = self.context(Some("stock.read"))?;
        self.read(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, source_line_id, product_id, warehouse_id, warehouse_location_id,
                        reserved_quantity_scaled, status, row_version
                 FROM stock_reservations
                 WHERE company_id = ?1 AND status IN ('ACTIVE', 'PARTIALLY_CONSUMED')
                 ORDER BY created_at, id",
            )?;
            Ok(statement
                .query_map([context.company_id], |row| {
                    Ok(ReservationView {
                        id: row.get(0)?,
                        source_line_id: row.get(1)?,
                        product_id: row.get(2)?,
                        warehouse_id: row.get(3)?,
                        warehouse_location_id: row.get(4)?,
                        reserved_quantity_scaled: row.get(5)?,
                        status: row.get(6)?,
                        row_version: row.get(7)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn create_reservation(
        &self,
        request: IdempotentRequest<ReservationRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.reservation.manage"))?;
        if request.payload.quantity_scaled <= 0 {
            return Err(Phase06Error::invalid("quantityScaled"));
        }
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.reservation.manage")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "stock.reservation.create",
                &request.idempotency_key,
                &hash,
            )? {
                return reservation_result(transaction, &context.company_id, &id, true);
            }

            let source_line_version = transaction
                .query_row(
                    "SELECT l.row_version
                     FROM commercial_document_lines l
                     JOIN commercial_documents d
                       ON d.id = l.document_id AND d.company_id = l.company_id
                     WHERE l.id = ?1 AND l.company_id = ?2 AND l.product_id = ?3",
                    params![
                        request.payload.source_line_id,
                        context.company_id,
                        request.payload.product_id,
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .ok_or_else(Phase06Error::not_found)?;
            if request
                .payload
                .row_version
                .is_some_and(|expected| expected != source_line_version)
            {
                return Err(Phase06Error::conflict());
            }

            super::projections::validate_product_scope(
                transaction,
                &context.company_id,
                &request.payload.product_id,
            )?;
            super::projections::validate_warehouse_scope(
                transaction,
                &context.company_id,
                &request.payload.warehouse_id,
                request.payload.warehouse_location_id.as_deref(),
            )?;
            let current = balance(
                transaction,
                &context.company_id,
                &request.payload.product_id,
                &request.payload.warehouse_id,
                request.payload.warehouse_location_id.as_deref(),
            )?;
            if request.payload.quantity_scaled > current.on_hand - current.reserved {
                return Err(Phase06Error::insufficient_stock());
            }

            set_reserved(
                transaction,
                &context,
                &request.payload.product_id,
                &request.payload.warehouse_id,
                request.payload.warehouse_location_id.as_deref(),
                request.payload.quantity_scaled,
            )?;
            let id = new_id();
            let now = now_iso()?;
            transaction.execute(
                "INSERT INTO stock_reservations (
                    id, company_id, product_id, warehouse_id, warehouse_location_id,
                    source_line_id, reserved_quantity_scaled, status,
                    created_at, created_by, updated_at, updated_by
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ACTIVE', ?8, ?9, ?8, ?9)",
                params![
                    id,
                    context.company_id,
                    request.payload.product_id,
                    request.payload.warehouse_id,
                    request.payload.warehouse_location_id,
                    request.payload.source_line_id,
                    request.payload.quantity_scaled,
                    now,
                    context.user_id,
                ],
            )?;
            audit(
                transaction,
                &context,
                "stock.reservation.create",
                "stock_reservation",
                &id,
                None,
            )?;
            finish_idempotency(
                transaction,
                &context,
                "stock.reservation.create",
                &request.idempotency_key,
                "stock_reservation",
                &id,
            )?;
            reservation_result(transaction, &context.company_id, &id, false)
        })
    }

    pub fn release_reservation(
        &self,
        request: IdempotentRequest<ReservationActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.release_or_cancel(request, "stock.reservation.release", false)
    }

    pub fn cancel_reservation(
        &self,
        request: IdempotentRequest<ReservationActionRequest>,
    ) -> Phase06Result<EntityResult> {
        self.release_or_cancel(request, "stock.reservation.cancel", true)
    }

    fn release_or_cancel(
        &self,
        request: IdempotentRequest<ReservationActionRequest>,
        namespace: &str,
        cancel: bool,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.reservation.manage"))?;
        let hash = request_hash(&request.payload)?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.reservation.manage")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                namespace,
                &request.idempotency_key,
                &hash,
            )? {
                return reservation_result(transaction, &context.company_id, &id, true);
            }

            let row = load_reservation(transaction, &context.company_id, &request.payload.reservation_id)?;
            validate_active_reservation(&row.status, row.row_version, request.payload.row_version)?;
            let quantity = request.payload.quantity_scaled.unwrap_or(row.remaining_quantity);
            if quantity <= 0 || quantity > row.remaining_quantity {
                return Err(Phase06Error::invalid("quantityScaled"));
            }
            if cancel && quantity != row.remaining_quantity {
                return Err(Phase06Error::invalid("cancelRequiresFullRemainingQuantity"));
            }

            set_reserved(
                transaction,
                &context,
                &row.product_id,
                &row.warehouse_id,
                row.location_id.as_deref(),
                -quantity,
            )?;
            let new_remaining = row.remaining_quantity - quantity;
            let new_status = if new_remaining == 0 {
                if cancel { "CANCELLED" } else { "RELEASED" }
            } else {
                row.status.as_str()
            };
            update_reservation_state(
                transaction,
                &context,
                &request.payload.reservation_id,
                row.row_version,
                new_remaining,
                row.remaining_quantity,
                new_status,
            )?;
            let details = serde_json::json!({"releasedQuantityScaled": quantity}).to_string();
            audit(
                transaction,
                &context,
                namespace,
                "stock_reservation",
                &request.payload.reservation_id,
                Some(&details),
            )?;
            finish_idempotency(
                transaction,
                &context,
                namespace,
                &request.idempotency_key,
                "stock_reservation",
                &request.payload.reservation_id,
            )?;
            reservation_result(
                transaction,
                &context.company_id,
                &request.payload.reservation_id,
                false,
            )
        })
    }

    pub fn consume_reservation(
        &self,
        request: IdempotentRequest<ReservationActionRequest>,
    ) -> Phase06Result<EntityResult> {
        let context = self.context(Some("stock.reservation.manage"))?;
        let hash = request_hash(&request.payload)?;
        let movement_id = request
            .payload
            .movement_id
            .as_deref()
            .ok_or_else(|| Phase06Error::invalid("movementId"))?;
        self.immediate(|transaction| {
            authorize_transaction(transaction, &context, "stock.reservation.manage")?;
            if let IdempotencyStart::Replayed(id) = begin_idempotency(
                transaction,
                &context,
                "stock.reservation.consume",
                &request.idempotency_key,
                &hash,
            )? {
                return reservation_result(transaction, &context.company_id, &id, true);
            }

            let row = load_reservation(transaction, &context.company_id, &request.payload.reservation_id)?;
            validate_active_reservation(&row.status, row.row_version, request.payload.row_version)?;
            let movement_quantity = transaction
                .query_row(
                    "SELECT -quantity_delta_scaled
                     FROM stock_movements
                     WHERE id = ?1 AND company_id = ?2 AND product_id = ?3
                       AND warehouse_id = ?4
                       AND ifnull(warehouse_location_id, '') = ifnull(?5, '')
                       AND quantity_delta_scaled < 0",
                    params![
                        movement_id,
                        context.company_id,
                        row.product_id,
                        row.warehouse_id,
                        row.location_id,
                    ],
                    |result| result.get::<_, i64>(0),
                )
                .optional()?
                .ok_or_else(|| Phase06Error::invalid("movementId"))?;
            let already_consumed: i64 = transaction.query_row(
                "SELECT COALESCE(SUM(CAST(json_extract(details_json, '$.consumedQuantityScaled') AS INTEGER)), 0)
                 FROM audit_logs
                 WHERE company_id = ?1 AND action_code = 'stock.reservation.consume'
                   AND entity_type = 'stock_reservation' AND entity_id = ?2
                   AND json_extract(details_json, '$.movementId') = ?3",
                params![context.company_id, request.payload.reservation_id, movement_id],
                |result| result.get(0),
            )?;
            let quantity = request.payload.quantity_scaled.unwrap_or(row.remaining_quantity);
            if quantity <= 0 || quantity > row.remaining_quantity {
                return Err(Phase06Error::invalid("quantityScaled"));
            }
            if already_consumed
                .checked_add(quantity)
                .ok_or_else(Phase06Error::numeric_overflow)?
                > movement_quantity
            {
                return Err(Phase06Error::new(
                    "MOVEMENT_CONSUMPTION_EXCEEDED",
                    "Reservation consumption exceeds the linked stock movement.",
                ));
            }

            set_reserved(
                transaction,
                &context,
                &row.product_id,
                &row.warehouse_id,
                row.location_id.as_deref(),
                -quantity,
            )?;
            let new_remaining = row.remaining_quantity - quantity;
            let new_status = if new_remaining == 0 {
                "CONSUMED"
            } else {
                "PARTIALLY_CONSUMED"
            };
            update_reservation_state(
                transaction,
                &context,
                &request.payload.reservation_id,
                row.row_version,
                new_remaining,
                row.remaining_quantity,
                new_status,
            )?;
            let details = serde_json::json!({
                "movementId": movement_id,
                "consumedQuantityScaled": quantity
            })
            .to_string();
            audit(
                transaction,
                &context,
                "stock.reservation.consume",
                "stock_reservation",
                &request.payload.reservation_id,
                Some(&details),
            )?;
            finish_idempotency(
                transaction,
                &context,
                "stock.reservation.consume",
                &request.idempotency_key,
                "stock_reservation",
                &request.payload.reservation_id,
            )?;
            reservation_result(
                transaction,
                &context.company_id,
                &request.payload.reservation_id,
                false,
            )
        })
    }
}

#[derive(Debug)]
struct ReservationRow {
    product_id: String,
    warehouse_id: String,
    location_id: Option<String>,
    remaining_quantity: i64,
    status: String,
    row_version: i64,
}

fn validate_active_reservation(status: &str, actual_version: i64, expected_version: i64) -> Phase06Result<()> {
    if !matches!(status, "ACTIVE" | "PARTIALLY_CONSUMED") {
        return Err(Phase06Error::immutable());
    }
    if actual_version != expected_version {
        return Err(Phase06Error::conflict());
    }
    Ok(())
}

fn update_reservation_state(
    transaction: &Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    reservation_id: &str,
    row_version: i64,
    new_remaining: i64,
    prior_remaining: i64,
    status: &str,
) -> Phase06Result<()> {
    let persisted_quantity = if new_remaining == 0 {
        prior_remaining
    } else {
        new_remaining
    };
    let changed = transaction.execute(
        "UPDATE stock_reservations
         SET reserved_quantity_scaled = ?1, status = ?2, updated_at = ?3, updated_by = ?4,
             row_version = row_version + 1
         WHERE id = ?5 AND company_id = ?6 AND row_version = ?7",
        params![
            persisted_quantity,
            status,
            now_iso()?,
            context.user_id,
            reservation_id,
            context.company_id,
            row_version,
        ],
    )?;
    if changed != 1 {
        return Err(Phase06Error::conflict());
    }
    Ok(())
}

fn load_reservation(
    transaction: &Transaction<'_>,
    company_id: &str,
    id: &str,
) -> Phase06Result<ReservationRow> {
    transaction
        .query_row(
            "SELECT product_id, warehouse_id, warehouse_location_id,
                    reserved_quantity_scaled, status, row_version
             FROM stock_reservations WHERE id = ?1 AND company_id = ?2",
            params![id, company_id],
            |row| {
                Ok(ReservationRow {
                    product_id: row.get(0)?,
                    warehouse_id: row.get(1)?,
                    location_id: row.get(2)?,
                    remaining_quantity: row.get(3)?,
                    status: row.get(4)?,
                    row_version: row.get(5)?,
                })
            },
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)
}

fn reservation_result(
    connection: &Connection,
    company_id: &str,
    id: &str,
    replayed: bool,
) -> Phase06Result<EntityResult> {
    connection
        .query_row(
            "SELECT status, row_version FROM stock_reservations
             WHERE id = ?1 AND company_id = ?2",
            params![id, company_id],
            |row| {
                Ok(EntityResult {
                    id: id.to_owned(),
                    document_number: None,
                    status: row.get(0)?,
                    row_version: row.get(1)?,
                    replayed,
                })
            },
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)
}

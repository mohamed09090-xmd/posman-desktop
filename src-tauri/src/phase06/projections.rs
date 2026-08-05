use rusqlite::{params, OptionalExtension, Transaction};

use crate::phase05::Phase06AuthContext;

use super::{
    ensure_warehouse,
    error::{Phase06Error, Phase06Result},
    fixed_point::{extended_cost_minor, weighted_average_cost},
    new_id, now_iso,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BalanceState {
    pub on_hand: i64,
    pub reserved: i64,
    pub average_cost: i64,
    pub row_version: i64,
    pub last_movement_id: Option<String>,
}

impl BalanceState {
    #[cfg(test)]
    pub fn available(&self) -> Phase06Result<i64> {
        self.on_hand
            .checked_sub(self.reserved)
            .ok_or_else(Phase06Error::numeric_overflow)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MovementSpec<'a> {
    pub product_id: &'a str,
    pub warehouse_id: &'a str,
    pub location_id: Option<&'a str>,
    pub source_document_id: Option<&'a str>,
    pub source_line_id: Option<&'a str>,
    pub movement_type: &'a str,
    pub business_date: &'a str,
    pub quantity_delta: i64,
    pub inbound_cost: Option<i64>,
    pub recalculate_average: bool,
    pub posting_event_key: &'a str,
    pub transfer_group_id: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub allow_negative: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MovementApplied {
    pub movement_id: String,
    pub quantity_before: i64,
    pub quantity_after: i64,
    pub average_cost_before: i64,
    pub average_cost_after: i64,
    pub unit_cost: i64,
}

pub(crate) fn balance(
    connection: &rusqlite::Connection,
    company_id: &str,
    product_id: &str,
    warehouse_id: &str,
    location_id: Option<&str>,
) -> Phase06Result<BalanceState> {
    Ok(connection
        .query_row(
            r#"
            SELECT on_hand_scaled, reserved_scaled, average_cost_scaled,
                   row_version, last_movement_id
            FROM stock_balances
            WHERE company_id=?1 AND product_id=?2 AND warehouse_id=?3
              AND ifnull(warehouse_location_id, '')=ifnull(?4, '')
            "#,
            params![company_id, product_id, warehouse_id, location_id],
            |row| {
                Ok(BalanceState {
                    on_hand: row.get(0)?,
                    reserved: row.get(1)?,
                    average_cost: row.get(2)?,
                    row_version: row.get(3)?,
                    last_movement_id: row.get(4)?,
                })
            },
        )
        .optional()?
        .unwrap_or(BalanceState {
            on_hand: 0,
            reserved: 0,
            average_cost: 0,
            row_version: 0,
            last_movement_id: None,
        }))
}

pub(crate) fn validate_product_scope(
    transaction: &Transaction<'_>,
    company_id: &str,
    product_id: &str,
) -> Phase06Result<()> {
    super::product_snapshot(transaction, company_id, product_id).map(|_| ())
}

pub(crate) fn validate_warehouse_scope(
    transaction: &Transaction<'_>,
    company_id: &str,
    warehouse_id: &str,
    location_id: Option<&str>,
) -> Phase06Result<()> {
    validate_location(transaction, company_id, warehouse_id, location_id)
}

fn validate_location(
    transaction: &Transaction<'_>,
    company_id: &str,
    warehouse_id: &str,
    location_id: Option<&str>,
) -> Phase06Result<()> {
    ensure_warehouse(transaction, company_id, warehouse_id)?;
    if let Some(location_id) = location_id {
        let exists = transaction
            .query_row(
                r#"
                SELECT 1 FROM warehouse_locations
                WHERE id=?1 AND company_id=?2 AND warehouse_id=?3 AND is_active=1
                "#,
                params![location_id, company_id, warehouse_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(Phase06Error::invalid("warehouseLocationId"));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn upsert_balance(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    product_id: &str,
    warehouse_id: &str,
    location_id: Option<&str>,
    movement_id: &str,
    on_hand: i64,
    reserved: i64,
    average_cost: i64,
) -> Phase06Result<()> {
    let available = on_hand
        .checked_sub(reserved)
        .ok_or_else(Phase06Error::numeric_overflow)?;
    transaction.execute(
        r#"
        INSERT INTO stock_balances (
            id, company_id, product_id, warehouse_id, warehouse_location_id,
            last_movement_id, on_hand_scaled, reserved_scaled, available_scaled,
            average_cost_scaled, rebuilt_at, row_version
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)
        ON CONFLICT DO UPDATE SET
            last_movement_id=excluded.last_movement_id,
            on_hand_scaled=excluded.on_hand_scaled,
            reserved_scaled=excluded.reserved_scaled,
            available_scaled=excluded.available_scaled,
            average_cost_scaled=excluded.average_cost_scaled,
            rebuilt_at=excluded.rebuilt_at,
            row_version=stock_balances.row_version+1
        "#,
        params![
            new_id(),
            context.company_id,
            product_id,
            warehouse_id,
            location_id,
            movement_id,
            on_hand,
            reserved,
            available,
            average_cost,
            now_iso()?
        ],
    )?;
    Ok(())
}

pub(crate) fn apply_movement(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    specification: MovementSpec<'_>,
) -> Phase06Result<MovementApplied> {
    if specification.quantity_delta == 0 {
        return Err(Phase06Error::invalid("quantityScaled"));
    }
    validate_location(
        transaction,
        &context.company_id,
        specification.warehouse_id,
        specification.location_id,
    )?;
    super::product_snapshot(transaction, &context.company_id, specification.product_id)?;

    let aggregate = balance(
        transaction,
        &context.company_id,
        specification.product_id,
        specification.warehouse_id,
        None,
    )?;
    let quantity_after = aggregate
        .on_hand
        .checked_add(specification.quantity_delta)
        .ok_or_else(Phase06Error::numeric_overflow)?;
    if quantity_after < 0 && !specification.allow_negative {
        return Err(Phase06Error::insufficient_stock());
    }
    if specification.quantity_delta < 0
        && aggregate.reserved > 0
        && quantity_after < aggregate.reserved
    {
        return Err(Phase06Error::reserved_stock_conflict());
    }

    let average_before = aggregate.average_cost;
    let unit_cost = if specification.quantity_delta < 0 {
        average_before
    } else {
        specification
            .inbound_cost
            .ok_or_else(|| Phase06Error::invalid("unitCostScaled"))?
    };
    if unit_cost < 0 {
        return Err(Phase06Error::invalid("unitCostScaled"));
    }

    let average_after = if specification.quantity_delta > 0 && specification.recalculate_average {
        weighted_average_cost(
            aggregate.on_hand,
            average_before,
            specification.quantity_delta,
            unit_cost,
        )?
    } else {
        average_before
    };

    let location_state = if let Some(location_id) = specification.location_id {
        let local = balance(
            transaction,
            &context.company_id,
            specification.product_id,
            specification.warehouse_id,
            Some(location_id),
        )?;
        let local_after = local
            .on_hand
            .checked_add(specification.quantity_delta)
            .ok_or_else(Phase06Error::numeric_overflow)?;
        if local_after < 0 && !specification.allow_negative {
            return Err(Phase06Error::insufficient_stock());
        }
        if specification.quantity_delta < 0 && local.reserved > 0 && local_after < local.reserved {
            return Err(Phase06Error::reserved_stock_conflict());
        }
        Some((location_id, local, local_after))
    } else {
        None
    };

    let movement_id = new_id();
    let extended_cost = extended_cost_minor(specification.quantity_delta, unit_cost)?;
    transaction.execute(
        r#"
        INSERT INTO stock_movements (
            id, company_id, product_id, warehouse_id, warehouse_location_id,
            source_document_id, source_line_id, movement_type, business_date,
            occurred_at, quantity_delta_scaled, quantity_before_scaled,
            quantity_after_scaled, unit_cost_scaled, average_cost_before_scaled,
            average_cost_after_scaled, extended_cost_minor, posting_event_key,
            transfer_group_id, notes, created_by
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21
        )
        "#,
        params![
            movement_id,
            context.company_id,
            specification.product_id,
            specification.warehouse_id,
            specification.location_id,
            specification.source_document_id,
            specification.source_line_id,
            specification.movement_type,
            specification.business_date,
            now_iso()?,
            specification.quantity_delta,
            aggregate.on_hand,
            quantity_after,
            unit_cost,
            average_before,
            average_after,
            extended_cost,
            specification.posting_event_key,
            specification.transfer_group_id,
            specification.notes,
            context.user_id
        ],
    )?;

    upsert_balance(
        transaction,
        context,
        specification.product_id,
        specification.warehouse_id,
        None,
        &movement_id,
        quantity_after,
        aggregate.reserved,
        average_after,
    )?;

    if let Some((location_id, local, local_after)) = location_state {
        upsert_balance(
            transaction,
            context,
            specification.product_id,
            specification.warehouse_id,
            Some(location_id),
            &movement_id,
            local_after,
            local.reserved,
            average_after,
        )?;
    }

    transaction.execute(
        r#"
        UPDATE stock_balances
        SET average_cost_scaled=?1, rebuilt_at=?2, row_version=row_version+1
        WHERE company_id=?3 AND product_id=?4 AND warehouse_id=?5
          AND warehouse_location_id IS NOT NULL
          AND average_cost_scaled<>?1
        "#,
        params![
            average_after,
            now_iso()?,
            context.company_id,
            specification.product_id,
            specification.warehouse_id
        ],
    )?;

    Ok(MovementApplied {
        movement_id,
        quantity_before: aggregate.on_hand,
        quantity_after,
        average_cost_before: average_before,
        average_cost_after: average_after,
        unit_cost,
    })
}

pub(crate) fn set_reserved(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    product_id: &str,
    warehouse_id: &str,
    location_id: Option<&str>,
    delta: i64,
) -> Phase06Result<()> {
    if delta == 0 {
        return Err(Phase06Error::invalid("reservedQuantityScaled"));
    }
    validate_location(transaction, &context.company_id, warehouse_id, location_id)?;

    let aggregate = balance(
        transaction,
        &context.company_id,
        product_id,
        warehouse_id,
        None,
    )?;
    let movement_id = aggregate
        .last_movement_id
        .as_deref()
        .ok_or_else(Phase06Error::insufficient_stock)?;
    let aggregate_reserved = aggregate
        .reserved
        .checked_add(delta)
        .ok_or_else(Phase06Error::numeric_overflow)?;
    if aggregate_reserved < 0 || aggregate_reserved > aggregate.on_hand {
        return Err(Phase06Error::insufficient_stock());
    }

    let local_update = if let Some(location_id) = location_id {
        let local = balance(
            transaction,
            &context.company_id,
            product_id,
            warehouse_id,
            Some(location_id),
        )?;
        let local_reserved = local
            .reserved
            .checked_add(delta)
            .ok_or_else(Phase06Error::numeric_overflow)?;
        if local_reserved < 0 || local_reserved > local.on_hand {
            return Err(Phase06Error::insufficient_stock());
        }
        Some((location_id, local, local_reserved))
    } else {
        None
    };

    upsert_balance(
        transaction,
        context,
        product_id,
        warehouse_id,
        None,
        movement_id,
        aggregate.on_hand,
        aggregate_reserved,
        aggregate.average_cost,
    )?;

    if let Some((location_id, local, local_reserved)) = local_update {
        upsert_balance(
            transaction,
            context,
            product_id,
            warehouse_id,
            Some(location_id),
            movement_id,
            local.on_hand,
            local_reserved,
            aggregate.average_cost,
        )?;
    }
    Ok(())
}

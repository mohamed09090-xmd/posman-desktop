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

fn negative_override_allowed(
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
    include!("inventory/query.rs");
    include!("inventory/opening.rs");
    include!("inventory/adjustment.rs");
    include!("inventory/transfer.rs");
}

include!("inventory/document_support.rs");

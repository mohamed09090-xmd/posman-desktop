use rusqlite::{params, OptionalExtension, Transaction};

use super::inventory::{negative_override_allowed, post_document};
use super::{
    audit, authorize_transaction, begin_idempotency,
    dto::{
        CreateInvoiceRequest, CreatePurchaseOrderRequest, CreateReceiptRequest,
        DirectReceiveInvoiceRequest, DocumentActionRequest, DocumentQuery, DocumentView,
        EntityResult, IdempotentRequest, PurchaseLineInput, PurchaseReturnRequest,
        UpdatePurchaseOrderRequest,
    },
    entity_result,
    error::{Phase06Error, Phase06Result},
    finish_idempotency, get_document_connection, insert_document, insert_purchase_line, new_id,
    now_iso,
    projections::{apply_movement, balance, MovementSpec},
    request_hash, update_document_totals, IdempotencyStart, Phase06Service,
};

fn validate_purchase_lines(lines: &[PurchaseLineInput]) -> Phase06Result<()> {
    if lines.is_empty() {
        return Err(Phase06Error::invalid("lines"));
    }
    for line in lines {
        if line.quantity_scaled <= 0
            || line.unit_price_scaled < 0
            || line.unit_cost_scaled.is_some_and(|value| value < 0)
            || !(0..=1_000_000).contains(&line.discount_rate_scaled)
        {
            return Err(Phase06Error::invalid("purchaseLine"));
        }
    }
    Ok(())
}

fn ensure_supplier(transaction: &Transaction<'_>, company_id: &str, id: &str) -> Phase06Result<()> {
    let exists = transaction
        .query_row(
            "SELECT 1 FROM partners
             WHERE id = ?1 AND company_id = ?2 AND is_supplier = 1 AND is_active = 1",
            params![id, company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !exists {
        return Err(Phase06Error::new(
            "SUPPLIER_REQUIRED",
            "Select an active supplier.",
        ));
    }
    Ok(())
}

fn aggregate_transform_guard(
    transaction: &Transaction<'_>,
    company_id: &str,
    source_line_id: &str,
    transformation_type: &str,
    quantity: i64,
) -> Phase06Result<()> {
    let source_quantity = transaction
        .query_row(
            "SELECT quantity_scaled FROM commercial_document_lines
             WHERE id = ?1 AND company_id = ?2",
            params![source_line_id, company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or_else(Phase06Error::not_found)?;
    let transformed: i64 = transaction.query_row(
        "SELECT COALESCE(SUM(transformed_quantity_scaled), 0)
         FROM document_line_links
         WHERE company_id = ?1 AND source_line_id = ?2 AND transformation_type = ?3",
        params![company_id, source_line_id, transformation_type],
        |row| row.get(0),
    )?;
    let total = transformed
        .checked_add(quantity)
        .ok_or_else(Phase06Error::numeric_overflow)?;
    if quantity <= 0 || total > source_quantity {
        return Err(Phase06Error::over_transformation());
    }
    Ok(())
}

fn insert_link(
    transaction: &Transaction<'_>,
    context: &crate::phase05::Phase06AuthContext,
    source_line_id: &str,
    target_line_id: &str,
    transformation_type: &str,
    quantity: i64,
) -> Phase06Result<()> {
    aggregate_transform_guard(
        transaction,
        &context.company_id,
        source_line_id,
        transformation_type,
        quantity,
    )?;
    transaction.execute(
        "INSERT INTO document_line_links (
            id, company_id, source_line_id, target_line_id, transformation_type,
            transformed_quantity_scaled, created_at, created_by
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            new_id(),
            context.company_id,
            source_line_id,
            target_line_id,
            transformation_type,
            quantity,
            now_iso()?,
            context.user_id,
        ],
    )?;
    Ok(())
}

impl Phase06Service {
    include!("purchasing/order.rs");
    include!("purchasing/receipt.rs");
    include!("purchasing/invoice.rs");
    include!("purchasing/returning.rs");
    include!("purchasing/query.rs");
}

include!("purchasing/posting_support.rs");

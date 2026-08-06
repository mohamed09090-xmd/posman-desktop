use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::{
    phase05::Phase06AuthContext,
    phase06::{error::Phase06Error, fixed_point::extended_cost_minor},
};

use super::{
    dto::{Idempotent, SourceEventRequest},
    error::{Phase08Error, Phase08Result},
};

#[derive(Clone, Debug)]
pub(crate) struct FailedPostingAttempt {
    pub context: Phase06AuthContext,
    pub request: Idempotent<SourceEventRequest>,
    pub error: Phase08Error,
}

pub(crate) fn record_failed_posting_attempt(
    connection: &mut Connection,
    failure: &FailedPostingAttempt,
) -> Phase08Result<()> {
    super::posting::record_failed_attempt_after_rollback(
        connection,
        &failure.context,
        &failure.request,
        &failure.error,
    )?;
    Ok(())
}

pub(crate) fn accounting_enabled_in_tx(
    transaction: &Transaction<'_>,
    company_id: &str,
) -> Phase08Result<bool> {
    transaction
        .query_row(
            "SELECT is_enabled FROM accounting_setups WHERE company_id=?1",
            [company_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|value| value.unwrap_or(0) == 1)
        .map_err(Into::into)
}

pub(crate) fn commercial_event_plan(namespace: &str) -> Option<(&'static str, bool)> {
    match namespace {
        "purchase_invoice.post" => Some(("PURCHASE_INVOICE", false)),
        "purchase.direct_receive_invoice" => Some(("PURCHASE_RECEIVE_INVOICE", true)),
        "purchase_return.post" => Some(("PURCHASE_RETURN", true)),
        "sales_order.deliver" => Some(("DELIVERY_COGS", true)),
        "sales_delivery.invoice" => Some(("SALES_INVOICE", false)),
        "sales.direct" => Some(("DIRECT_SALE", true)),
        "sales.return_credit" => Some(("SALES_RETURN", true)),
        _ => None,
    }
}

pub(crate) fn document_source_event_in_tx(
    transaction: &Transaction<'_>,
    company_id: &str,
    document_id: &str,
    source_event_type: &str,
    source_event_id: &str,
    event_date: &str,
    include_stock_cost: bool,
) -> Phase08Result<SourceEventRequest> {
    let document = transaction
        .query_row(
            r#"SELECT partner_id,total_ht_minor,total_tax_minor,total_ttc_minor
               FROM commercial_documents WHERE id=?1 AND company_id=?2"#,
            params![document_id, company_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| {
            Phase08Error::new(
                "SOURCE_DOCUMENT_NOT_FOUND",
                "The accounting source document was not found.",
                false,
            )
        })?;
    let mut components = BTreeMap::from([
        ("DOCUMENT_HT".to_owned(), document.1),
        ("DOCUMENT_TAX".to_owned(), document.2),
        ("DOCUMENT_TTC".to_owned(), document.3),
    ]);
    if include_stock_cost {
        let mut statement = transaction.prepare(
            "SELECT quantity_scaled,COALESCE(unit_cost_scaled,0)
             FROM commercial_document_lines WHERE company_id=?1 AND document_id=?2",
        )?;
        let rows = statement.query_map(params![company_id, document_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut total_cost = 0_i64;
        for row in rows {
            let (quantity, cost) = row?;
            total_cost = total_cost
                .checked_add(extended_cost_minor(quantity, cost).map_err(|_| {
                    Phase08Error::new(
                        "ACCOUNTING_NUMERIC_OVERFLOW",
                        "The stock cost exceeds the supported fixed-point range.",
                        false,
                    )
                })?)
                .ok_or_else(Phase08Error::internal)?;
        }
        components.insert("STOCK_COST".to_owned(), total_cost);
    }
    Ok(SourceEventRequest {
        source_event_type: source_event_type.to_owned(),
        source_event_id: source_event_id.to_owned(),
        source_document_id: Some(document_id.to_owned()),
        event_date: event_date.to_owned(),
        partner_id: document.0,
        product_id: None,
        payment_method_id: None,
        memo: None,
        components_minor: components,
        inject_failure_after_header: false,
    })
}

pub(crate) fn phase06_error(
    error: Phase08Error,
    context: &Phase06AuthContext,
    request: Idempotent<SourceEventRequest>,
) -> Phase06Error {
    Phase06Error::new(&error.code, &error.message).with_accounting_failure(FailedPostingAttempt {
        context: context.clone(),
        request,
        error,
    })
}

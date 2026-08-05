use serde::{Deserialize, Serialize};

pub use crate::phase06::dto::{DocumentQuery, DocumentView, EntityResult, IdempotentRequest};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesLineInput {
    pub product_id: String,
    pub warehouse_id: Option<String>,
    pub quantity_scaled: i64,
    pub unit_price_scaled: i64,
    pub discount_rate_scaled: i64,
    pub tax_rate_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSalesOrderRequest {
    pub customer_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub price_mode: String,
    pub header_discount_rate_scaled: i64,
    pub below_cost_override_reason: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<SalesLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSalesOrderRequest {
    pub document_id: String,
    pub row_version: i64,
    #[serde(flatten)]
    pub order: CreateSalesOrderRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesOrderActionRequest {
    pub document_id: String,
    pub row_version: i64,
    pub reason: Option<String>,
    pub below_cost_override_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformLineInput {
    pub source_line_id: String,
    pub quantity_scaled: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverSalesOrderRequest {
    pub order_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub below_cost_override_reason: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<TransformLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceDeliveryRequest {
    pub delivery_id: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<TransformLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectSaleRequest {
    pub customer_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub price_mode: String,
    pub header_discount_rate_scaled: i64,
    pub below_cost_override_reason: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<SalesLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesReturnRequest {
    pub source_document_id: String,
    pub customer_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub reason: String,
    pub lines: Vec<TransformLineInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SalesFlowResult {
    pub primary: EntityResult,
    pub related_document_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SalesSummary {
    pub draft_orders: i64,
    pub confirmed_orders: i64,
    pub partial_orders: i64,
    pub uninvoiced_deliveries: i64,
    pub posted_invoices: i64,
    pub below_cost_overrides: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SalesLineAvailability {
    pub source_line_id: String,
    pub product_id: String,
    pub original_quantity_scaled: i64,
    pub delivered_quantity_scaled: i64,
    pub invoiced_quantity_scaled: i64,
    pub returned_quantity_scaled: i64,
    pub remaining_quantity_scaled: i64,
}

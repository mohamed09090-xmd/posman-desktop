use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdempotentRequest<T> {
    pub idempotency_key: String,
    pub payload: T,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockLineInput {
    pub product_id: String,
    pub warehouse_location_id: Option<String>,
    pub quantity_scaled: i64,
    pub unit_cost_scaled: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningDraftRequest {
    pub warehouse_id: String,
    pub commercial_date: String,
    pub notes: Option<String>,
    pub lines: Vec<StockLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentActionRequest {
    pub document_id: String,
    pub row_version: i64,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustmentRequest {
    pub warehouse_id: String,
    pub commercial_date: String,
    pub reason: String,
    pub allow_negative_override: bool,
    pub lines: Vec<StockLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferLineInput {
    pub product_id: String,
    pub quantity_scaled: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    pub source_warehouse_id: String,
    pub source_location_id: Option<String>,
    pub destination_warehouse_id: String,
    pub destination_location_id: Option<String>,
    pub commercial_date: String,
    pub reason: Option<String>,
    pub allow_negative_override: bool,
    pub lines: Vec<TransferLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountLineInput {
    pub product_id: String,
    pub warehouse_location_id: Option<String>,
    pub counted_quantity_scaled: i64,
    pub unit_cost_scaled: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCountRequest {
    pub warehouse_id: String,
    pub count_number: String,
    pub commercial_date: String,
    pub notes: Option<String>,
    pub lines: Vec<CountLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCountRequest {
    pub count_id: String,
    pub row_version: i64,
    pub lines: Vec<CountLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationRequest {
    pub source_line_id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub warehouse_location_id: Option<String>,
    pub quantity_scaled: i64,
    pub row_version: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationActionRequest {
    pub reservation_id: String,
    pub quantity_scaled: Option<i64>,
    pub row_version: i64,
    pub movement_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseLineInput {
    pub source_line_id: Option<String>,
    pub product_id: String,
    pub warehouse_id: Option<String>,
    pub quantity_scaled: i64,
    pub unit_price_scaled: i64,
    pub unit_cost_scaled: Option<i64>,
    pub discount_rate_scaled: i64,
    pub tax_rate_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePurchaseOrderRequest {
    pub supplier_id: String,
    pub commercial_date: String,
    pub notes: Option<String>,
    pub lines: Vec<PurchaseLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePurchaseOrderRequest {
    pub document_id: String,
    pub row_version: i64,
    pub supplier_id: String,
    pub commercial_date: String,
    pub notes: Option<String>,
    pub lines: Vec<PurchaseLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReceiptRequest {
    pub purchase_order_id: Option<String>,
    pub supplier_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub notes: Option<String>,
    pub lines: Vec<PurchaseLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceRequest {
    pub supplier_id: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<PurchaseLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectReceiveInvoiceRequest {
    pub supplier_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<PurchaseLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnRequest {
    pub source_document_id: String,
    pub supplier_id: String,
    pub warehouse_id: String,
    pub commercial_date: String,
    pub reason: String,
    pub allow_negative_override: bool,
    pub lines: Vec<PurchaseLineInput>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentQuery {
    pub document_type: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockQuery {
    pub product_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub warehouse_location_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntityResult {
    pub id: String,
    pub document_number: Option<String>,
    pub status: String,
    pub row_version: i64,
    pub replayed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StockBalanceView {
    pub product_id: String,
    pub product_code: String,
    pub product_name: String,
    pub warehouse_id: String,
    pub warehouse_name: String,
    pub warehouse_location_id: Option<String>,
    pub location_name: Option<String>,
    pub on_hand_scaled: i64,
    pub reserved_scaled: i64,
    pub available_scaled: i64,
    pub average_cost_scaled: i64,
    pub inventory_value_minor: i64,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MovementView {
    pub id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub warehouse_location_id: Option<String>,
    pub source_document_id: Option<String>,
    pub movement_type: String,
    pub business_date: String,
    pub quantity_delta_scaled: i64,
    pub quantity_after_scaled: i64,
    pub unit_cost_scaled: Option<i64>,
    pub average_cost_after_scaled: Option<i64>,
    pub extended_cost_minor: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReservationView {
    pub id: String,
    pub source_line_id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub warehouse_location_id: Option<String>,
    pub reserved_quantity_scaled: i64,
    pub status: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLineView {
    pub id: String,
    pub source_line_id: Option<String>,
    pub product_id: String,
    pub product_code: String,
    pub description: String,
    pub warehouse_id: Option<String>,
    pub quantity_scaled: i64,
    pub unit_price_scaled: i64,
    pub unit_cost_scaled: Option<i64>,
    pub tax_rate_scaled: i64,
    pub line_ht_minor: i64,
    pub line_tax_minor: i64,
    pub line_ttc_minor: i64,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub id: String,
    pub document_type: String,
    pub document_number: String,
    pub workflow_status: String,
    pub posting_status: String,
    pub commercial_date: String,
    pub partner_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub source_document_id: Option<String>,
    pub total_ht_minor: i64,
    pub total_tax_minor: i64,
    pub total_ttc_minor: i64,
    pub notes: Option<String>,
    pub row_version: i64,
    pub lines: Vec<DocumentLineView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CountView {
    pub id: String,
    pub warehouse_id: String,
    pub count_number: String,
    pub commercial_date: String,
    pub status: String,
    pub row_version: i64,
    pub lines: Vec<CountLineView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CountLineView {
    pub id: String,
    pub product_id: String,
    pub warehouse_location_id: Option<String>,
    pub system_quantity_scaled: i64,
    pub counted_quantity_scaled: i64,
    pub variance_quantity_scaled: i64,
    pub unit_cost_scaled: Option<i64>,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationRow {
    pub product_id: String,
    pub warehouse_id: String,
    pub warehouse_location_id: Option<String>,
    pub projection_on_hand_scaled: i64,
    pub rebuilt_on_hand_scaled: i64,
    pub projection_reserved_scaled: i64,
    pub rebuilt_reserved_scaled: i64,
    pub projection_average_cost_scaled: i64,
    pub rebuilt_average_cost_scaled: i64,
    pub matches: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationView {
    pub rows: Vec<ReconciliationRow>,
    pub mismatch_count: usize,
    pub rebuilt: bool,
}

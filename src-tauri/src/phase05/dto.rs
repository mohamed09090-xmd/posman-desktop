use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    pub setup_required: bool,
    pub has_draft: bool,
    pub schema_version: String,
    pub default_fiscal_starts_on: String,
    pub default_fiscal_ends_on: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSetupDraftRequest {
    pub draft_schema_version: i64,
    pub data: Value,
    pub row_version: Option<i64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupDraft {
    pub draft_schema_version: i64,
    pub data: Value,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxSetup {
    pub code: String,
    pub name_ar: String,
    pub name_fr: String,
    pub rate_scaled: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialSetupRequest {
    pub idempotency_key: String,
    pub company_code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub legal_name: String,
    pub activity_description: String,
    pub legal_form: Option<String>,
    pub trade_register_number: Option<String>,
    pub tax_identifier: Option<String>,
    pub statistical_identifier: Option<String>,
    pub tax_article_number: Option<String>,
    pub bank_rib: Option<String>,
    pub social_capital_minor: Option<i64>,
    pub address_text: String,
    pub wilaya_code: String,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub language: String,
    pub fiscal_starts_on: String,
    pub fiscal_ends_on: String,
    pub default_margin_rate_scaled: i64,
    pub below_cost_policy: Option<String>,
    pub session_idle_timeout_minutes: i64,
    pub taxes: Vec<TaxSetup>,
    pub default_tax_code: Option<String>,
    pub warehouse_code: String,
    pub warehouse_name_ar: String,
    pub warehouse_name_fr: Option<String>,
    pub administrator_username: String,
    pub administrator_display_name: String,
    pub administrator_password: String,
    pub administrator_password_confirmation: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteSetupResult {
    pub company_id: String,
    pub administrator_user_id: String,
    pub recovery_code: Option<String>,
    pub already_completed: bool,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub company_id: String,
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub preferred_language: String,
    pub permissions: Vec<String>,
    pub locked: bool,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockSessionRequest {
    pub password: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirmation: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverPasswordRequest {
    pub username: String,
    pub recovery_code: String,
    pub new_password: String,
    pub new_password_confirmation: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCodeResult {
    pub recovery_code: String,
}

#[derive(Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    pub search: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub include_inactive: Option<bool>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub id: String,
    pub code: String,
    pub legal_name: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub activity_description: Option<String>,
    pub legal_form: Option<String>,
    pub trade_register_number: Option<String>,
    pub tax_identifier: Option<String>,
    pub statistical_identifier: Option<String>,
    pub tax_article_number: Option<String>,
    pub bank_rib: Option<String>,
    pub social_capital_minor: Option<i64>,
    pub address_text: Option<String>,
    pub wilaya_code: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub default_margin_rate_scaled: i64,
    pub below_cost_policy: String,
    pub session_idle_timeout_minutes: i64,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyProfileRequest {
    pub legal_name: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub activity_description: Option<String>,
    pub legal_form: Option<String>,
    pub trade_register_number: Option<String>,
    pub tax_identifier: Option<String>,
    pub statistical_identifier: Option<String>,
    pub tax_article_number: Option<String>,
    pub bank_rib: Option<String>,
    pub social_capital_minor: Option<i64>,
    pub address_text: String,
    pub wilaya_code: String,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub default_margin_rate_scaled: i64,
    pub below_cost_policy: String,
    pub session_idle_timeout_minutes: i64,
    pub row_version: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalSetup {
    pub fiscal_year_id: String,
    pub code: String,
    pub starts_on: String,
    pub ends_on: String,
    pub periods: Vec<FiscalPeriodView>,
    pub row_version: i64,
    pub in_use: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalPeriodView {
    pub period_number: i64,
    pub name: String,
    pub starts_on: String,
    pub ends_on: String,
    pub status: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFiscalSetupRequest {
    pub starts_on: String,
    pub ends_on: String,
    pub row_version: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceView {
    pub id: String,
    pub document_type: String,
    pub prefix: String,
    pub next_number: i64,
    pub padding_width: i64,
    pub preview: String,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentSequenceRequest {
    pub id: String,
    pub prefix: String,
    pub next_number: i64,
    pub padding_width: i64,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceInput {
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub numeric_value: Option<i64>,
    pub kind: Option<String>,
    pub parent_id: Option<String>,
    pub related_id: Option<String>,
    pub address_text: Option<String>,
    pub flag: Option<bool>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceUpdate {
    pub id: String,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub numeric_value: Option<i64>,
    pub kind: Option<String>,
    pub parent_id: Option<String>,
    pub related_id: Option<String>,
    pub address_text: Option<String>,
    pub flag: Option<bool>,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveRequest {
    pub id: String,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceRecord {
    pub id: String,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub is_active: bool,
    pub row_version: i64,
    pub details: Value,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub barcode: Option<String>,
    pub unit_id: String,
    pub product_family_id: Option<String>,
    pub default_tax_rate_id: Option<String>,
    pub default_purchase_price_scaled: i64,
    pub manual_sale_price_scaled: Option<i64>,
    pub below_cost_override_reason: Option<String>,
    pub margin_rate_scaled: Option<i64>,
    pub minimum_stock_scaled: i64,
    pub product_kind: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    pub id: String,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub barcode: Option<String>,
    pub unit_id: String,
    pub product_family_id: Option<String>,
    pub default_tax_rate_id: Option<String>,
    pub default_purchase_price_scaled: i64,
    pub manual_sale_price_scaled: Option<i64>,
    pub below_cost_override_reason: Option<String>,
    pub margin_rate_scaled: Option<i64>,
    pub minimum_stock_scaled: i64,
    pub product_kind: String,
    pub row_version: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductView {
    pub id: String,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub unit_id: String,
    pub product_family_id: Option<String>,
    pub tax_rate_id: Option<String>,
    pub purchase_price_scaled: i64,
    pub sale_price_scaled: i64,
    pub suggested_sale_price_scaled: i64,
    pub pricing_warning: Option<String>,
    pub below_cost_policy: String,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPriceInput {
    pub product_id: String,
    pub price_list_id: String,
    pub unit_price_scaled: i64,
    pub valid_from: String,
    pub below_cost_override_reason: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartnerRequest {
    pub code: String,
    pub legal_name: String,
    pub display_name_ar: String,
    pub display_name_fr: Option<String>,
    pub is_customer: bool,
    pub is_supplier: bool,
    pub legal_form: Option<String>,
    pub activity_description: Option<String>,
    pub tax_identifier: Option<String>,
    pub trade_register_number: Option<String>,
    pub statistical_identifier: Option<String>,
    pub tax_article_number: Option<String>,
    pub payment_term_id: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePartnerRequest {
    pub id: String,
    pub code: String,
    pub legal_name: String,
    pub display_name_ar: String,
    pub display_name_fr: Option<String>,
    pub is_customer: bool,
    pub is_supplier: bool,
    pub legal_form: Option<String>,
    pub activity_description: Option<String>,
    pub tax_identifier: Option<String>,
    pub trade_register_number: Option<String>,
    pub statistical_identifier: Option<String>,
    pub tax_article_number: Option<String>,
    pub payment_term_id: Option<String>,
    pub row_version: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerView {
    pub id: String,
    pub code: String,
    pub legal_name: String,
    pub display_name_ar: String,
    pub display_name_fr: Option<String>,
    pub is_customer: bool,
    pub is_supplier: bool,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerAddressInput {
    pub partner_id: String,
    pub address_kind: String,
    pub label: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub is_default: bool,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerContactInput {
    pub partner_id: String,
    pub full_name: String,
    pub job_title: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerAddressView {
    pub id: String,
    pub partner_id: String,
    pub address_kind: String,
    pub label: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePartnerAddressRequest {
    pub id: String,
    pub partner_id: String,
    pub address_kind: String,
    pub label: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub is_default: bool,
    pub row_version: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerContactView {
    pub id: String,
    pub partner_id: String,
    pub full_name: String,
    pub job_title: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePartnerContactRequest {
    pub id: String,
    pub partner_id: String,
    pub full_name: String,
    pub job_title: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub preferred_language: String,
    pub password: String,
    pub password_confirmation: String,
    pub role_ids: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub id: String,
    pub display_name: String,
    pub preferred_language: String,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUserRolesRequest {
    pub user_id: String,
    pub role_ids: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetUserPasswordRequest {
    pub user_id: String,
    pub new_password: String,
    pub new_password_confirmation: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub preferred_language: String,
    pub is_active: bool,
    pub failed_login_count: i64,
    pub locked_until: Option<String>,
    pub role_ids: Vec<String>,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub code: String,
    pub name_ar: String,
    pub name_fr: String,
    pub permission_ids: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub id: String,
    pub name_ar: String,
    pub name_fr: String,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRolePermissionsRequest {
    pub role_id: String,
    pub permission_ids: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleView {
    pub id: String,
    pub code: String,
    pub name_ar: String,
    pub name_fr: String,
    pub is_system: bool,
    pub is_active: bool,
    pub permission_ids: Vec<String>,
    pub row_version: i64,
}

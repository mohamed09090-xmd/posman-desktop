use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfiguration {
    pub document_title_ar: String,
    pub document_title_fr: String,
    pub show_logo: bool,
    pub show_company_identity: bool,
    pub show_trade_register: bool,
    pub show_tax_identifier: bool,
    pub show_partner_address: bool,
    pub show_payment_information: bool,
    pub footer_text_ar: String,
    pub footer_text_fr: String,
    pub spacing: String,
    pub orientation: String,
    pub enabled_sections: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub template_id: String,
    pub document_type: String,
    pub locale: String,
    pub display_name: String,
    pub active_version_id: Option<String>,
    pub active_version_number: Option<i64>,
    pub active_content_sha256: Option<String>,
    pub draft_id: Option<String>,
    pub draft_row_version: Option<i64>,
    pub state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVersionView {
    pub version_id: String,
    pub version_number: i64,
    pub locale: String,
    pub content_sha256: String,
    pub status: String,
    pub published_at: String,
    pub published_by: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDraftView {
    pub draft_id: String,
    pub template_id: String,
    pub document_type: String,
    pub locale: String,
    pub display_name: String,
    pub configuration: TemplateConfiguration,
    pub base_template_version_id: Option<String>,
    pub row_version: i64,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDetail {
    pub summary: TemplateSummary,
    pub draft: Option<TemplateDraftView>,
    pub versions: Vec<TemplateVersionView>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateKeyRequest {
    pub document_type: String,
    pub locale: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateDraftRequest {
    pub document_type: String,
    pub locale: String,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplateDraftRequest {
    pub draft_id: String,
    pub expected_row_version: i64,
    pub display_name: String,
    pub configuration: TemplateConfiguration,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublishTemplateRequest {
    pub draft_id: String,
    pub expected_row_version: i64,
    pub confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetireTemplateRequest {
    pub template_version_id: String,
    pub expected_row_version: i64,
    pub confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRequest {
    pub document_type: String,
    pub source_document_id: String,
    pub locale: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderedDocumentKeyRequest {
    pub render_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderedDocumentsRequest {
    pub document_type: Option<String>,
    pub source_document_id: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinePayload {
    pub line_number: i64,
    pub product_code: String,
    pub description: String,
    pub unit_code: String,
    pub quantity_scaled: i64,
    pub unit_price_scaled: i64,
    pub discount_rate_scaled: i64,
    pub discount_minor: i64,
    pub tax_rate_scaled: i64,
    pub ht_minor: i64,
    pub tax_minor: i64,
    pub ttc_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalDocumentPayload {
    pub company_name: String,
    pub company_legal_name: String,
    pub company_address: Option<String>,
    pub company_trade_register: Option<String>,
    pub company_tax_identifier: Option<String>,
    pub company_phone: Option<String>,
    pub company_email: Option<String>,
    pub partner_name: Option<String>,
    pub partner_address: Option<String>,
    pub partner_tax_identifier: Option<String>,
    pub document_type: String,
    pub document_number: String,
    pub document_status: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub currency_code: String,
    pub total_ht_minor: i64,
    pub total_tax_minor: i64,
    pub total_ttc_minor: i64,
    pub payment_information: Option<String>,
    pub references: Vec<String>,
    pub notes: Option<String>,
    pub lines: Vec<DocumentLinePayload>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub preview_id: String,
    pub document_type: String,
    pub source_document_id: String,
    pub locale: String,
    pub integrity_state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewContent {
    pub preview_id: String,
    pub locale: String,
    pub direction: String,
    pub html: String,
    pub css: String,
    pub content_sha256: String,
    pub integrity_state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderedDocumentView {
    pub render_id: String,
    pub document_type: String,
    pub source_document_id: String,
    pub source_document_number: String,
    pub source_document_status: String,
    pub template_id: String,
    pub template_version_id: String,
    pub locale: String,
    pub content_sha256: String,
    pub pdf_relative_path: String,
    pub pdf_sha256: String,
    pub pdf_size_bytes: i64,
    pub rendered_at: String,
    pub rendered_by: String,
    pub integrity_state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Paged<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportDescriptor {
    pub report_id: String,
    pub name_ar: String,
    pub name_fr: String,
    pub supports_date_range: bool,
    pub supports_warehouse: bool,
    pub supports_partner: bool,
    pub supports_product: bool,
    pub supports_status: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportRequest {
    pub report_id: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub warehouse_id: Option<String>,
    pub partner_id: Option<String>,
    pub product_id: Option<String>,
    pub status: Option<String>,
    pub sort_field: Option<String>,
    pub sort_direction: Option<String>,
    pub page: i64,
    pub page_size: i64,
    pub locale: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportColumn {
    pub key: String,
    pub label_ar: String,
    pub label_fr: String,
    pub kind: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ReportValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
    Null,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportRow {
    pub values: BTreeMap<String, ReportValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportPage {
    pub report_id: String,
    pub columns: Vec<ReportColumn>,
    pub rows: Vec<ReportRow>,
    pub page: i64,
    pub page_size: i64,
    pub total_rows: i64,
    pub generated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditRequest {
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub user_id: Option<String>,
    pub domain: Option<String>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub outcome: Option<String>,
    pub sensitive_only: Option<bool>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditEventView {
    pub id: String,
    pub actor_user_id: Option<String>,
    pub actor_display_name: Option<String>,
    pub action_code: String,
    pub domain: String,
    pub entity_type: String,
    pub entity_id: String,
    pub occurred_at: String,
    pub outcome: String,
    pub sensitive: bool,
    pub details: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettingsView {
    pub automatic_enabled: bool,
    pub weekly_enabled: bool,
    pub timezone_name: String,
    pub last_attempt_local_date: Option<String>,
    pub last_success_local_date: Option<String>,
    pub last_warning_code: Option<String>,
    pub row_version: i64,
    pub encryption_status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBackupSettingsRequest {
    pub automatic_enabled: bool,
    pub weekly_enabled: bool,
    pub expected_row_version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    pub backup_kind: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupKeyRequest {
    pub backup_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupListRequest {
    pub backup_kind: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupView {
    pub backup_id: String,
    pub backup_kind: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub application_version: String,
    pub schema_version: String,
    pub migration_ledger_digest: String,
    pub database_size_bytes: i64,
    pub sha256: String,
    pub relative_path: String,
    pub integrity_status: String,
    pub foreign_key_status: String,
    pub verification_status: String,
    pub failure_reason: Option<String>,
    pub selected_for_restore: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBackupRequest {
    pub backup_id: String,
    pub current_password: String,
    pub confirmation_text: String,
    pub confirmed: bool,
}

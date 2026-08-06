use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Idempotent<T> {
    pub idempotency_key: String,
    pub request_hash_sha256: String,
    pub payload: T,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInput {
    pub id: Option<String>,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub account_type: String,
    pub normal_side: String,
    pub parent_account_id: Option<String>,
    pub allow_posting: bool,
    pub is_active: bool,
    pub row_version: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub account_type: String,
    pub normal_side: String,
    pub allow_posting: bool,
    pub is_active: bool,
    pub row_version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalInput {
    pub id: Option<String>,
    pub code: String,
    pub name_ar: String,
    pub name_fr: Option<String>,
    pub journal_type: String,
    pub is_active: bool,
    pub row_version: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntityVersion {
    pub id: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRoleInput {
    pub role_code: String,
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethodAccountingInput {
    pub payment_method_id: String,
    pub account_id: Option<String>,
    pub account_role_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallAccountingTemplateRequest {
    pub enabled: bool,
    pub current_fiscal_year_id: Option<String>,
    pub roles: Vec<AccountRoleInput>,
    #[serde(default)]
    pub payment_methods: Vec<PaymentMethodAccountingInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingRuleLineInput {
    pub line_number: i64,
    pub side: String,
    pub account_id: Option<String>,
    pub account_role_code: Option<String>,
    pub amount_component: String,
    pub description_ar: String,
    pub description_fr: Option<String>,
    pub partner_dimension: bool,
    pub product_dimension: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingRuleInput {
    pub id: Option<String>,
    pub code: String,
    pub source_event_type: String,
    pub accounting_journal_id: String,
    pub priority: i64,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub is_active: bool,
    pub row_version: Option<i64>,
    pub lines: Vec<PostingRuleLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceEventRequest {
    pub source_event_type: String,
    pub source_event_id: String,
    pub source_document_id: Option<String>,
    pub event_date: String,
    pub partner_id: Option<String>,
    pub product_id: Option<String>,
    pub payment_method_id: Option<String>,
    pub memo: Option<String>,
    pub components_minor: BTreeMap<String, i64>,
    #[serde(default)]
    pub inject_failure_after_header: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostingResult {
    pub journal_entry_id: String,
    pub posting_attempt_id: String,
    pub replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualJournalLineInput {
    pub account_id: String,
    pub partner_id: Option<String>,
    pub product_id: Option<String>,
    pub description: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualJournalInput {
    pub id: Option<String>,
    pub accounting_journal_id: String,
    pub entry_date: String,
    pub memo: Option<String>,
    pub row_version: Option<i64>,
    pub lines: Vec<ManualJournalLineInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseJournalRequest {
    pub journal_entry_id: String,
    pub reversal_date: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInput {
    pub partner_id: String,
    pub payment_method_id: String,
    pub commercial_date: String,
    pub amount_minor: i64,
    pub external_reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentResult {
    pub payment_id: String,
    pub journal_entry_id: String,
    pub amount_minor: i64,
    pub unallocated_minor: i64,
    pub replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationInput {
    pub payment_id: String,
    pub document_id: String,
    pub amount_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AllocationResult {
    pub allocation_id: String,
    pub payment_id: String,
    pub document_id: String,
    pub amount_minor: i64,
    pub payment_unallocated_minor: i64,
    pub document_open_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseAllocationInput {
    pub allocation_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReversePaymentInput {
    pub payment_id: String,
    pub reversal_date: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalLineView {
    pub account_id: String,
    pub account_code: String,
    pub description: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryView {
    pub id: String,
    pub entry_number: String,
    pub entry_date: String,
    pub status: String,
    pub source_event_type: String,
    pub source_event_id: String,
    pub reversal_of_entry_id: Option<String>,
    pub memo: Option<String>,
    pub debit_total_minor: i64,
    pub credit_total_minor: i64,
    pub lines: Vec<JournalLineView>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrialBalanceRow {
    pub account_id: String,
    pub account_code: String,
    pub account_name_ar: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
    pub balance_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LedgerRow {
    pub journal_entry_id: String,
    pub entry_number: String,
    pub entry_date: String,
    pub account_id: String,
    pub account_code: String,
    pub description: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
    pub running_balance_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatementRow {
    pub event_date: String,
    pub source_type: String,
    pub source_id: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
    pub running_balance_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenBalanceRow {
    pub document_id: String,
    pub document_number: String,
    pub document_type: String,
    pub commercial_date: String,
    pub due_date: Option<String>,
    pub total_minor: i64,
    pub allocated_minor: i64,
    pub open_minor: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FiscalPeriodView {
    pub id: String,
    pub fiscal_year_id: String,
    pub period_number: i64,
    pub name: String,
    pub starts_on: String,
    pub ends_on: String,
    pub status: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodActionInput {
    pub fiscal_period_id: String,
    pub row_version: i64,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostingAttemptView {
    pub id: String,
    pub source_event_type: String,
    pub source_event_id: String,
    pub attempt_number: i64,
    pub status: String,
    pub error_code: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

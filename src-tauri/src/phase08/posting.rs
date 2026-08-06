use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::phase05::Phase06AuthContext;

use super::{
    dto::{
        EntityVersion, Idempotent, JournalEntryView, JournalLineView, ManualJournalInput,
        PostingAttemptView, PostingResult, ReverseJournalRequest, SourceEventRequest,
    },
    error::{Phase08Error, Phase08Result},
    service::{new_id, now_iso, require_active_postable_account, require_company_row},
    Phase08Service,
};

#[derive(Clone, Debug)]
struct RuleLine {
    line_number: i64,
    side: String,
    account_id: String,
    component: String,
    description: String,
    partner_dimension: bool,
    product_dimension: bool,
}

#[derive(Clone, Debug)]
struct SelectedRule {
    id: String,
    journal_id: String,
    lines: Vec<RuleLine>,
}

include!("posting/source.rs");
include!("posting/service_methods.rs");
include!("posting/reversal_rules.rs");
include!("posting/validation_queries.rs");

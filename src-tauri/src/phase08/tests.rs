use std::collections::BTreeMap;

use rusqlite::{params, Connection, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::phase05::Phase06AuthContext;

use super::{
    dto::{
        AllocationInput, Idempotent, PaymentInput, ReverseAllocationInput, SourceEventRequest,
    },
    payments::{allocate_payment_in_tx, post_payment_in_tx, reverse_allocation_in_tx},
    posting::{post_source_event_in_tx, record_failed_attempt_after_rollback, request_hash, reverse_entry_in_tx},
};

const NOW: &str = "2026-08-06T00:00:00Z";

const TEST_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_system_company_security", include_str!("../../../database/migrations/0001_system_company_security.sql")),
    ("0002_reference_catalog_partners", include_str!("../../../database/migrations/0002_reference_catalog_partners.sql")),
    ("0003_commerce_inventory", include_str!("../../../database/migrations/0003_commerce_inventory.sql")),
    ("0004_accounting_documents_audit", include_str!("../../../database/migrations/0004_accounting_documents_audit.sql")),
    ("0005_setup_security_reference_data", include_str!("../../../database/migrations/0005_setup_security_reference_data.sql")),
    ("0006_accounting_payments_hardening", include_str!("../../../database/migrations/0006_accounting_payments_hardening.sql")),
];


include!("tests/fixture.rs");
include!("tests/assertions.rs");
include!("tests/posting_cases.rs");
include!("tests/payment_cases.rs");
include!("tests/rollback_case.rs");

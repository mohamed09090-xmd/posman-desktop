use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};

use super::{
    dto::{
        AllocationInput, AllocationResult, Idempotent, PaymentInput, PaymentResult,
        ReverseAllocationInput, ReversePaymentInput, SourceEventRequest,
    },
    error::{Phase08Error, Phase08Result},
    posting::{post_source_event_in_tx, record_failed_attempt_after_rollback},
    service::{new_id, now_iso},
    Phase08Service,
};
use crate::phase05::Phase06AuthContext;

include!("payments/service_post.rs");
include!("payments/service_allocation.rs");
include!("payments/post_payment.rs");
include!("payments/allocation.rs");
include!("payments/support.rs");

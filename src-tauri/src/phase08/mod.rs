pub mod dto;
pub mod error;
pub(crate) mod integration;
mod accounts;
mod periods;
mod rules;
mod payments;
mod posting;
mod queries;
mod service;

#[cfg(test)]
mod tests;

use crate::phase05::Phase05Service;

#[derive(Clone)]
pub struct Phase08Service {
    phase05: Phase05Service,
}

pub(crate) use integration::{
    accounting_enabled_in_tx, commercial_event_plan, document_source_event_in_tx, phase06_error,
    record_failed_posting_attempt,
    record_failed_posting_attempt_at_path,
};
pub(crate) use posting::{post_source_event_in_tx, record_failed_attempt_after_rollback, request_hash};

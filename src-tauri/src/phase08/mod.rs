mod accounts;
pub mod dto;
pub mod error;
pub(crate) mod integration;
mod payments;
mod periods;
mod posting;
mod queries;
mod rules;
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
};
pub(crate) use posting::{post_source_event_in_tx, request_hash};

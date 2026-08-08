//! PHASE 09 bounded service surface.
//!
//! Command handlers depend on [`Phase09Service`]. Domain implementation remains
//! split by workstream in templates, documents, output, reports, audit, backup,
//! and restore modules; no business logic belongs in Tauri handlers.

pub type Phase09CommandService = super::Phase09Service;

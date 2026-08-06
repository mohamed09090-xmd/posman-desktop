use rusqlite::{params, OptionalExtension, Transaction};

use super::{
    dto::{EntityVersion, PostingRuleInput},
    error::{Phase08Error, Phase08Result},
    service::{boolean, new_id, now_iso, require_active_postable_account, require_company_row},
    Phase08Service,
};

include!("rules/save.rs");
include!("rules/list.rs");
include!("rules/validate_configuration.rs");
include!("rules/resolution.rs");
include!("rules/validation.rs");

use rusqlite::params;

use super::{
    dto::{
        AccountInput, AccountView, EntityVersion, InstallAccountingTemplateRequest, JournalInput,
    },
    error::{Phase08Error, Phase08Result},
    service::{boolean, new_id, now_iso, require_active_postable_account, require_company_row},
    Phase08Service,
};

include!("accounts/setup.rs");
include!("accounts/accounts.rs");
include!("accounts/journals.rs");
include!("accounts/validation.rs");

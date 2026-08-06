use rusqlite::{params, Connection};

use super::{
    dto::{
        LedgerRow, OpenBalanceRow, PaymentResult, StatementRow, TrialBalanceRow,
    },
    error::{Phase08Error, Phase08Result},
    payments::{document_open, payment_unallocated},
    Phase08Service,
};

include!("queries/payments.rs");
include!("queries/ledger_methods.rs");
include!("queries/statement.rs");
include!("queries/cash_bank.rs");
include!("queries/open_methods.rs");
include!("queries/trial_balance.rs");
include!("queries/ledger.rs");
include!("queries/open_balances.rs");

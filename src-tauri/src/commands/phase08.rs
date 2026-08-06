use tauri::State;

use crate::phase08::{
    dto::*,
    error::{Phase08Error, Phase08Result},
    Phase08Service,
};

async fn run<T: Send + 'static>(
    service: Phase08Service,
    operation: impl FnOnce(Phase08Service) -> Phase08Result<T> + Send + 'static,
) -> Phase08Result<T> {
    tauri::async_runtime::spawn_blocking(move || operation(service))
        .await
        .map_err(|_| Phase08Error::internal())?
}

macro_rules! command {
    ($name:ident,$request:ty,$result:ty,$method:ident) => {
        #[tauri::command]
        pub async fn $name(
            state: State<'_, Phase08Service>,
            request: $request,
        ) -> Phase08Result<$result> {
            run(state.inner().clone(), move |service| {
                service.$method(request)
            })
            .await
        }
    };
}

macro_rules! query {
    ($name:ident,$result:ty,$method:ident) => {
        #[tauri::command]
        pub async fn $name(state: State<'_, Phase08Service>) -> Phase08Result<$result> {
            run(state.inner().clone(), move |service| service.$method(())).await
        }
    };
}

command!(
    install_accounting_template,
    InstallAccountingTemplateRequest,
    EntityVersion,
    install_accounting_template
);
query!(list_accounts, Vec<AccountView>, list_accounts);
command!(create_account, AccountInput, EntityVersion, create_account);
command!(update_account, AccountInput, EntityVersion, update_account);
query!(
    list_accounting_journals,
    Vec<EntityVersion>,
    list_accounting_journals
);
command!(
    create_accounting_journal,
    JournalInput,
    EntityVersion,
    create_accounting_journal
);
command!(
    update_accounting_journal,
    JournalInput,
    EntityVersion,
    update_accounting_journal
);
query!(list_posting_rules, Vec<EntityVersion>, list_posting_rules);
command!(
    save_posting_rule,
    PostingRuleInput,
    EntityVersion,
    save_posting_rule
);
query!(
    validate_posting_configuration,
    Vec<String>,
    validate_posting_configuration
);
query!(
    list_accounting_posting_queue,
    Vec<PostingAttemptView>,
    list_accounting_posting_queue
);
command!(
    post_source_event,
    Idempotent<SourceEventRequest>,
    PostingResult,
    post_source_event
);
command!(
    retry_posting_attempt,
    Idempotent<SourceEventRequest>,
    PostingResult,
    retry_posting_attempt
);
query!(
    list_journal_entries,
    Vec<JournalEntryView>,
    list_journal_entries
);
command!(
    get_journal_entry,
    String,
    JournalEntryView,
    get_journal_entry
);
command!(
    create_manual_journal_entry,
    ManualJournalInput,
    EntityVersion,
    create_manual_journal_entry
);
command!(
    update_manual_journal_entry,
    ManualJournalInput,
    EntityVersion,
    update_manual_journal_entry
);
command!(
    post_manual_journal_entry,
    String,
    EntityVersion,
    post_manual_journal_entry
);
command!(
    reverse_journal_entry,
    ReverseJournalRequest,
    EntityVersion,
    reverse_journal_entry
);
command!(
    post_customer_receipt,
    Idempotent<PaymentInput>,
    PaymentResult,
    post_customer_receipt
);
command!(
    post_supplier_payment,
    Idempotent<PaymentInput>,
    PaymentResult,
    post_supplier_payment
);
command!(
    allocate_payment,
    Idempotent<AllocationInput>,
    AllocationResult,
    allocate_payment
);
command!(
    reverse_payment_allocation,
    Idempotent<ReverseAllocationInput>,
    AllocationResult,
    reverse_payment_allocation
);
command!(
    reverse_payment,
    Idempotent<ReversePaymentInput>,
    PaymentResult,
    reverse_payment
);
query!(list_payments, Vec<PaymentResult>, list_payments);
command!(
    get_partner_statement,
    String,
    Vec<StatementRow>,
    get_partner_statement
);
query!(
    get_cash_bank_register,
    Vec<LedgerRow>,
    get_cash_bank_register
);
query!(get_trial_balance, Vec<TrialBalanceRow>, get_trial_balance);
query!(get_general_ledger, Vec<LedgerRow>, get_general_ledger);
command!(
    get_account_ledger,
    String,
    Vec<LedgerRow>,
    get_account_ledger
);
query!(
    get_open_receivables,
    Vec<OpenBalanceRow>,
    get_open_receivables
);
query!(get_open_payables, Vec<OpenBalanceRow>, get_open_payables);
query!(
    list_fiscal_periods,
    Vec<FiscalPeriodView>,
    list_fiscal_periods
);
command!(
    close_fiscal_period,
    PeriodActionInput,
    EntityVersion,
    close_fiscal_period
);
command!(
    reopen_fiscal_period,
    PeriodActionInput,
    EntityVersion,
    reopen_fiscal_period
);

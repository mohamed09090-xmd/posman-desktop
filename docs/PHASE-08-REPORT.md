# POSMAN PHASE 08 Report — Automatic Accounting and Payments

## Coordinates

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Accepted baseline: `ae133cea9c3b6760a5fd22b38d3169aa2f976dc6`
- Branch: `phase/08-accounting-payments`
- Pull Request: `#12` — `[Phase 08] POSMAN automatic accounting and payments`
- Initial ownership lease: `90097176a03bdc503463d37494d1f2ff4ab83d30`
- Migration: `database/migrations/0006_accounting_payments_hardening.sql`
- Migration SHA-256 at this revision: `08763076ce7cbd77e585bf06b10bc856e7b8f02193484b1db974db95143cebd0`

## Delivered implementation

PHASE 08 supplies company-scoped accounting setup, semantic account-role mapping, configurable multi-line posting rules, automatic source posting, manual journals, compensating reversals, customer receipts, supplier payments, partial/full allocations, compensating allocation reversal, statements, open balances, trial balance, ledgers, fiscal-period controls, and failed-attempt retry history.

The source transaction, stock effects, accounting entry, success attempt, audit, and idempotency completion share one SQLite `BEGIN IMMEDIATE` transaction. A failed transaction rolls back completely; a separate short transaction appends only safe FAILED-attempt metadata.

See `docs/architecture/phase-08-accounting-payments.md` for the complete topology and contracts.

## Typed commands

35 commands are registered through one typed Tauri boundary:

1. `install_accounting_template`
2. `list_accounts`
3. `create_account`
4. `update_account`
5. `list_accounting_journals`
6. `create_accounting_journal`
7. `update_accounting_journal`
8. `list_posting_rules`
9. `save_posting_rule`
10. `validate_posting_configuration`
11. `list_accounting_posting_queue`
12. `post_source_event`
13. `retry_posting_attempt`
14. `list_journal_entries`
15. `get_journal_entry`
16. `create_manual_journal_entry`
17. `update_manual_journal_entry`
18. `post_manual_journal_entry`
19. `reverse_journal_entry`
20. `post_customer_receipt`
21. `post_supplier_payment`
22. `allocate_payment`
23. `reverse_payment_allocation`
24. `reverse_payment`
25. `list_payments`
26. `get_partner_statement`
27. `get_cash_bank_register`
28. `get_trial_balance`
29. `get_general_ledger`
30. `get_account_ledger`
31. `get_open_receivables`
32. `get_open_payables`
33. `list_fiscal_periods`
34. `close_fiscal_period`
35. `reopen_fiscal_period`

## Real SQLite test cases

`src-tauri/src/phase08/tests.rs` contains real in-memory SQLite fixtures against the generated accepted schema. The suite covers balanced sales and purchase posting, separated tax lines, delivery COGS, direct-sale compound posting, purchase receipt/invoice integration, sales and purchase return compensation, idempotent replay and hash conflict, missing/ambiguous rules, inactive accounts, closed periods, unbalanced-entry rejection, injected mid-posting failure, complete source/stock/journal rollback, persisted safe FAILED attempt, posted-journal immutability, balanced linked reversal, partial/full/over allocation, compensating allocation reversal, and company isolation.

A real `tauri::test::get_ipc_response` test invokes the PHASE 08 command boundary and verifies unauthenticated rejection without bypassing Tauri IPC.

## Browser evidence scenarios

`tests/e2e/run_phase08.py` defines six operational scenarios:

1. Arabic accounting setup and posting-rule configuration at `1280×800`.
2. French automatic sales posting with source-to-journal trace at `1280×800`.
3. Arabic purchase posting and supplier payment at `1280×800`.
4. French manual journal posting and reversal at `1024×640`.
5. Arabic customer receipt with partial then full allocation at `1024×640`.
6. French missing-rule failure, correction, and successful retry at `1280×800`.

Each scenario checks directionality, page overflow, primary-label clipping, browser/console errors, Axe violations, and unresolved critical/serious incomplete findings, then writes a screenshot and Axe JSON.

## Validation matrix

The final results table is populated from one stable final head after all required GitHub Actions jobs complete successfully.

| Command / gate | Final result |
|---|---|
| `python scripts/verify_schema.py` | Pending final CI |
| `python scripts/verify_phase06.py` | Pending final CI |
| `python scripts/verify_phase07.py` | Pending final CI |
| `python scripts/verify_phase08.py` | Pending final CI |
| `npm ci` | Pending final CI |
| `npm run typecheck` | Pending final CI |
| `npm run build` | Pending final CI |
| `npm run test:ui` | Pending final CI |
| `npm run test:integration` | Pending final CI |
| `npm run test:e2e` | Pending final CI |
| Rust 1.85 `cargo fmt --check` | Pending final CI |
| Rust 1.85 locked `cargo check` | Pending final CI |
| Rust 1.85 locked Clippy `-D warnings` | Pending final CI |
| Rust 1.85 locked `cargo test -- --nocapture` | Pending final CI |
| `npm run desktop:check` | Pending final CI |
| Windows application manifest | Pending final CI |
| `git diff --check` and clean tree | Pending final CI |

Local pre-push validation established that `verify_schema.py` and `verify_phase06.py` pass with six migrations, 57 tables, 47 integrity triggers, 134 schema checks, and frozen SHA-256 values for migrations `0001–0005`. Rust and Node product validation remain authoritative in GitHub Actions because the local execution environment lacks the required Rust toolchain and its package mirror does not serve the locked Vite archive.

## Workflow and artifact evidence

Final values are recorded after the green final-head run:

- PHASE 08 workflow run URL: Pending.
- Ubuntu/Windows job states: Pending.
- `phase-08-ui-evidence`: ID, size, digest, expiry, and contents pending.
- `phase-08-integration-evidence`: ID, size, digest, expiry, and contents pending.

## Risks and limits

- No printing/PDF engine, backup, installer, updater, cloud service, telemetry, or PHASE 09 feature is included.
- Accepted migrations `0001–0005` are untouched.
- Posting rules and account mappings are mandatory; the runtime deliberately fails closed instead of guessing accounts or tax rates.
- A failed attempt stores a safe code only; operators use the UI configuration state and retry flow rather than raw database diagnostics.
- Final reviewer acceptance and merge remain outside the implementation engineer’s authority.

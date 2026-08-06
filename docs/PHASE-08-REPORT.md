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

The implementation evidence head is `fc976100c0c30e8df52ab61d9b75f4fc8ce415b0`. Product source is unchanged by its documentation/workflow-only successors. The delivery head is accepted only after the same required workflows complete successfully; its exact run links are recorded in PR #12 before merge.

| Command / gate | Final result |
|---|---|
| `python scripts/verify_schema.py` | PASS — 6 migrations, schema `0006`, 57 tables, 47 triggers, 134 checks |
| `python scripts/verify_phase06.py` | PASS — accepted PHASE 06 boundary preserved |
| `python scripts/verify_phase07.py` | PASS — accepted PHASE 07 boundary preserved |
| `python scripts/verify_phase08.py` | PASS — 35 typed commands and real SQLite coverage |
| `npm ci` | PASS — locked install |
| `npm run typecheck` | PASS |
| `npm run build` | PASS |
| `npm run test:ui` | PASS — 17 tests |
| `npm run test:integration` | PASS — 55 tests |
| `npm run test:e2e` | PASS — 6 named PHASE 08 scenarios |
| Rust 1.85 `cargo fmt --check` | PASS |
| Rust 1.85 locked `cargo check` | PASS |
| Rust 1.85 locked Clippy `-D warnings` | PASS |
| Rust 1.85 locked `cargo test -- --nocapture` | PASS — 64 tests |
| `npm run desktop:check` | PASS — Ubuntu and Windows |
| Windows test/application manifests | PASS — Common Controls v6 retained |
| `git diff --check` and clean tree | PASS |

Migrations `0001–0005` retain their accepted SHA-256 values. The generated schema matches all six ordered migrations, `foreign_key_check` and `integrity_check` are clean, and application money columns remain fixed-point integers.

All six PHASE 08 browser scenarios reported zero Axe violations, zero incomplete results, and zero unresolved critical/serious incomplete findings. They also passed RTL/LTR, page-level overflow, primary-label clipping, console-error, and page-error assertions. The six screenshots were downloaded and reviewed visually before acceptance.

## Workflow and artifact evidence

Green source-evidence workflow runs:

- PHASE 08 accounting/payments: `31104573103` — success.
- Frontend Runtime Integration: `31104573328` — success.
- PHASE 05 validation: `31104573483` — success.
- PHASE 06 inventory/purchasing: `31104573420` — success.
- PHASE 07 sales cycle: `31104573623` — success.
- Runtime CI: `31104573372` — Ubuntu, Windows, and Rust 1.85 success.
- SQLite schema verification: `31104573139` — Ubuntu and Windows success.

The Desktop Bootstrap source run `31104573055` passed Ubuntu and completed all Windows Rust/native checks, then exposed an evidence guard that still searched for the pre-refactor test module path. The guard was corrected to the real `ipc_tests::application_setup_builds_with_mock_runtime` path without weakening either manifest assertion. The corrected delivery head must pass Desktop Bootstrap on both platforms before merge.

Final source-evidence artifacts from PHASE 08 run `31104573103`:

- `phase-08-ui-evidence`: ID `8969013242`, size `2,595,645` bytes, digest `sha256:2199b7d0ff8fb11f172484c776fc073fa0b51d241e289da46a238738b1e6c03b`, expires `2026-09-05T13:11:27Z`.
- `phase-08-integration-evidence`: ID `8969226912`, size `31,742` bytes, digest `sha256:6c6e114e5b99ac474da94b30ec4377017bcb7e044bbc0990d148e070c2d0bea0`, expires `2026-09-05T13:18:07Z`.

The final documentation/workflow-only delivery-head run IDs and replacement artifact metadata are recorded in the PR description before the guarded squash merge.

## Risks and limits

- No printing/PDF engine, backup, installer, updater, cloud service, telemetry, or PHASE 09 feature is included.
- Accepted migrations `0001–0005` are untouched.
- Posting rules and account mappings are mandatory; the runtime deliberately fails closed instead of guessing accounts or tax rates.
- A failed attempt stores a safe code only; operators use the UI configuration state and retry flow rather than raw database diagnostics.
- PHASE 09 remains outside this delivery and must not begin before the guarded PHASE 08 squash merge completes.

# PHASE 08 — Automatic Accounting and Payments

## Scope

PHASE 08 adds the offline accounting and settlement layer to the accepted POSMAN desktop runtime. The implementation remains local to bundled SQLite, uses integer fixed-point amounts, preserves Arabic RTL and French LTR operation, and does not add network, telemetry, cloud identity, or an external database.

Accepted migrations `0001` through `0005` remain byte-for-byte frozen. The unaccepted migration `0006_accounting_payments_hardening.sql` is the only schema addition for this phase.

## Transaction topology

A newly posted commercial event is atomic across the source domain and accounting:

```text
public Phase 06/07 service wrapper
  → BEGIN IMMEDIATE
  → authorization and idempotency start
  → source document mutation in_tx
  → stock movement/projection in_tx when applicable
  → finish_idempotency in_tx
      → document_source_event_in_tx
      → post_source_event_in_tx
      → balanced journal header + lines
      → SUCCEEDED posting attempt
  → source audit and idempotency completion
  → COMMIT
```

`phase06::finish_idempotency` is the shared transaction hook used by accepted purchasing and sales services. It maps the accepted namespaces to accounting events:

| Source namespace | Accounting event | Stock-cost component |
|---|---|---:|
| `purchase_invoice.post` | `PURCHASE_INVOICE` | No |
| `purchase.direct_receive_invoice` | `PURCHASE_RECEIVE_INVOICE` | Yes |
| `purchase_return.post` | `PURCHASE_RETURN` | Yes |
| `sales_order.deliver` | `DELIVERY_COGS` | Yes |
| `sales_delivery.invoice` | `SALES_INVOICE` | No |
| `sales.direct` | `DIRECT_SALE` | Yes |
| `sales.return_credit` | `SALES_RETURN` | Yes |

No successful source path opens a second accounting transaction. A failure in source mutation, stock movement, rule resolution, fiscal-period validation, journal construction, audit, or idempotency completion prevents the business transaction from committing.

## Failed posting attempts

A failed journal attempt is intentionally separated from the failed business transaction:

1. The source, stock, journal draft, journal lines, success attempt, audit, and idempotency completion run in the business transaction.
2. Any failure returns a safe `Phase08Error`; the business transaction is rolled back in full.
3. After rollback, a short independent `BEGIN IMMEDIATE` transaction appends one `posting_attempts` row with status `FAILED`.
4. The row contains company, source-event type and ID, idempotency key, request SHA-256, attempt number, retry lineage, safe error code, and timestamps.
5. `error_message` is deliberately `NULL`; SQL, stack traces, paths, and secrets are never persisted.
6. Attempts are append-only. A retry creates a later attempt number and never mutates the failed row into success.

The SQLite tests verify that an injected mid-posting failure leaves zero source documents, zero stock movements, zero journal headers/lines, and one safe failed attempt.

## Accounting setup and account mapping

`accounting_setups` enables accounting per company and links the active fiscal year. `accounting_account_roles` maps semantic roles to active postable accounts, avoiding hardcoded account numbers. Supported roles include customer receivable, supplier payable, cash, bank, sales revenue, collected/output tax, recoverable/input tax, inventory, and COGS.

`payment_method_accounting` maps each accepted PHASE 05 payment method to either a direct account or a semantic account role. Company-scope triggers reject cross-company account, fiscal-year, payment-method, journal, rule, payment, and allocation references on insert and update.

## Posting-rule selection

Posting rules are company-scoped and date-effective. Each rule contains ordered `posting_rule_lines`, and every line defines:

- debit or credit side;
- direct account or semantic account role;
- amount component such as `DOCUMENT_HT`, `DOCUMENT_TAX`, `DOCUMENT_TTC`, `STOCK_COST`, or `PAYMENT_AMOUNT`;
- partner and product dimension flags;
- Arabic line description.

Selection filters by company, source event, active status, and validity date, then chooses the highest priority. More than one eligible rule at the winning priority is rejected as `POSTING_RULE_AMBIGUOUS`. Missing rules, missing account roles, inactive/non-postable accounts, missing payment-method accounting, and closed periods return safe actionable codes before any posted journal survives.

## Fixed-point calculations

All monetary values use signed 64-bit minor units. Quantities and unit costs use the accepted scaled-integer rules. Extended stock cost uses checked `i128` intermediates and the accepted half-up rounding before conversion to `i64`. Floating-point storage and calculations are absent from the accounting runtime and schema.

Generated journal lines are validated before posting:

- at least two lines;
- exactly one positive side per line;
- checked debit and credit accumulation;
- debit total equals credit total;
- all accounts are company-scoped, active, and postable;
- the fiscal period is open.

## Idempotency

Source posting is unique by company, source-event type, and source-event ID. The stored request SHA-256 must match for a replay. The same identity and hash returns the existing journal without duplication; the same identity or idempotency key with a different hash returns `ACCOUNTING_IDEMPOTENCY_CONFLICT`/`IDEMPOTENCY_CONFLICT`.

Payments and allocations carry their own idempotency keys and request hashes. Allocation retries return the existing result; conflicting payloads are rejected.

## Manual journals and reversals

A manual journal is created as `DRAFT`, may be edited using `row_version`, requires two or more balanced lines, and is validated against the period and accounts. Posting changes it once to `POSTED`. Existing migration-0004 immutability triggers reject subsequent changes to the posted header or lines.

Journal reversal creates a new balanced posted entry linked by `reversal_of_entry_id`; the original entry is never mutated. Payment reversal similarly creates a compensating payment and journal linked to the original payment. The reversal date resolves its own open fiscal year and period.

## Payments and allocations

Customer receipts and supplier disbursements are posted with their accounting journal in one SQLite transaction. SALES receives customer-receipt rights only; PURCHASING receives supplier-disbursement rights only. Payment-method reference requirements are enforced before posting.

Allocations support partial and full settlement. Effective allocation is the sum of active rows minus compensating reversal rows. Over-allocation is rejected against both payment unallocated balance and document open balance. Historical allocation rows are never updated or deleted. A payment cannot be reversed while effective allocations remain.

## Queries

The read model exposes:

- journal list and details;
- trial balance from posted entries only;
- general ledger and account ledger;
- cash/bank register;
- customer/supplier statements;
- open receivables and payables;
- payment list and unallocated balances;
- fiscal periods and status history;
- posting-attempt queue.

All queries filter by the authenticated company.

## Typed Tauri boundary

PHASE 08 registers 35 typed commands through `src-tauri/src/commands/phase08.rs` and `src-tauri/src/lib.rs`:

`install_accounting_template`, `list_accounts`, `create_account`, `update_account`, `list_accounting_journals`, `create_accounting_journal`, `update_accounting_journal`, `list_posting_rules`, `save_posting_rule`, `validate_posting_configuration`, `list_accounting_posting_queue`, `post_source_event`, `retry_posting_attempt`, `list_journal_entries`, `get_journal_entry`, `create_manual_journal_entry`, `update_manual_journal_entry`, `post_manual_journal_entry`, `reverse_journal_entry`, `post_customer_receipt`, `post_supplier_payment`, `allocate_payment`, `reverse_payment_allocation`, `reverse_payment`, `list_payments`, `get_partner_statement`, `get_cash_bank_register`, `get_trial_balance`, `get_general_ledger`, `get_account_ledger`, `get_open_receivables`, `get_open_payables`, `list_fiscal_periods`, `close_fiscal_period`, and `reopen_fiscal_period`.

The TypeScript gateway owns all `invoke` calls, validates runtime response shapes, normalizes safe error codes, and supports abort/stale-response handling. React contains no SQL and no HTTP/runtime network primitive.

## Accounting workspace

The Operations Ledger adds a PHASE 08 workspace with:

- account and role setup;
- posting-rule configuration;
- journal list, source trace, manual draft/post/reversal;
- customer receipts, supplier payments, and allocations;
- partner statements and open balances;
- trial balance and ledger;
- fiscal-period operations;
- failed-attempt correction and retry.

Arabic `ar-DZ` is the default RTL locale and French `fr-DZ` uses LTR. Money, dates, and numbers use the existing `Intl`-backed i18n provider. The workspace has loading, empty, error, and retry states; duplicate-submit guards; confirmation before posting, reversal, and period changes; internal table scrolling; reduced-motion handling; and no page-level horizontal overflow or primary-text ellipsis.

## Validation evidence

`phase08-ci.yml` runs permanent read-only gates for:

- schema, PHASE 06, PHASE 07, and PHASE 08 verifiers on Ubuntu and Windows;
- Node 24 typecheck, build, UI tests, integration tests, Playwright, and Axe;
- Rust 1.85 formatting, locked check, Clippy with warnings denied, SQLite tests, real Tauri IPC, and native desktop build on Ubuntu and Windows;
- Windows embedded application-manifest checks;
- ownership, whitespace, and clean-tree evidence.

The workflow emits `phase-08-ui-evidence` and `phase-08-integration-evidence`, each with metadata, content inventory, and SHA-256 files. IDs, final head, run URLs, sizes, server-provided digests, and expiry are recorded in `docs/PHASE-08-REPORT.md` after the final green run.

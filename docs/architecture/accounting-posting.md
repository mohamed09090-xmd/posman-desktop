# POSMAN accounting posting design

## Objective

Posting converts one validated business event into one immutable balanced journal entry using company-configured accounts and rules. No account number is permanent program logic.

## Source events

Examples include sales/purchase invoice posting, delivery cost recognition, returns, payments, inventory adjustments, and reversals. A request supplies `company_id`, event type/id, commercial and posting dates, fiscal context, optional source document, calculated monetary components, and a stable idempotency key.

## Rule lookup

The future Rust service queries active `posting_rules` by company, source event, effective date, and optional structured condition. Priority resolves eligible rules; zero or multiple equally valid results are posting failures, not silent guesses.

## Transactional path

One SQLite transaction performs:

1. acquire/create the namespaced idempotency record;
2. verify event eligibility and absence of a prior successful result;
3. resolve fiscal year and an open period;
4. resolve exactly one configured rule set;
5. insert a `DRAFT` journal entry;
6. insert journal lines;
7. recalculate debit and credit totals in Rust using integer minor units;
8. transition the entry to `POSTED`;
9. store posting-attempt and source linkage;
10. commit.

The database independently rejects direct `POSTED` inserts and rejects the transition when the period is closed/mismatched, line count is below two, total is not positive, or debit and credit differ.

## Idempotency

`journal_entries` has a unique `(company_id, idempotency_key)` constraint. `idempotency_keys` coordinates duplicate commands across domains. `posting_attempts` records attempts and retries. Repeating the same command returns/references the existing result instead of creating a second entry.

## Failure and retry

A failed attempt records a clear error outside any rolled-back partial journal transaction. Retry uses the same business idempotency key and a new attempt linked with `retry_of_attempt_id`. No partial posted entry remains.

## Reversal and immutability

Posted entries and lines cannot be changed or deleted. A correction creates a new balanced entry with `reversal_of_entry_id`; any corrected posting is separate. The original accounting and source evidence remains visible.

## Fiscal-period lock

Posting requires an `OPEN` period matching company, fiscal year, and entry date. `CLOSED` and `LOCKED` periods reject the documented transition. Authorization to close/reopen periods and its audit event belong to the future service.

## Traceability

Entries retain source event type/id, optional source document, journal, fiscal period, idempotency key, timestamps, actor, and reversal link. Lines retain account and optional partner/product dimensions. Sensitive attempts, reversals, closes, and restores are written to append-only audit history.

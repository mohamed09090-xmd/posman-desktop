# POSMAN accounting posting design

## Objective

Accounting posting converts a validated business event into one immutable, balanced journal entry using company-configured accounts and rules. No account number is permanent program logic.

## Source events

Examples include sales invoice posting, purchase invoice posting, goods delivery cost recognition, returns, payments, inventory adjustments, and reversals. Each event supplies:

- `company_id`;
- event type and event ID;
- commercial/posting date;
- fiscal context;
- source document when applicable;
- monetary components already calculated with decimal arithmetic;
- a stable idempotency key.

## Rule lookup

The future Rust posting service queries active `posting_rules` for the company, source event, effective date, and optional structured conditions. Rules identify the journal and configurable debit/credit accounts. Priority resolves multiple matching rules; ambiguity is a posting failure, not a silent guess.

Illustrative mappings may resemble customer-to-sales or cost-of-sales-to-inventory entries, but account codes are chosen by the merchant/accountant and stored in `accounts` and `posting_rules`.

## Posting transaction

The documented posting path is one SQLite transaction:

1. acquire or create the namespaced idempotency record;
2. verify that the source event is eligible and not already posted;
3. resolve the fiscal year and open fiscal period;
4. resolve exactly one valid posting rule set;
5. create a `DRAFT` journal entry;
6. insert journal lines;
7. recalculate debit and credit totals in Rust using integer minor units;
8. transition the entry to `POSTED`;
9. record the successful posting attempt and source linkage;
10. commit.

The database trigger independently rejects a `POSTED` transition when the period is closed/mismatched, the line count is below two, the total is not positive, or debit and credit differ.

## Idempotency

`journal_entries` has a unique `(company_id, idempotency_key)` constraint. `idempotency_keys` provides a cross-domain namespace, while `posting_attempts` records retries and failures. Repeating the same command must return or reference the existing result rather than creating a second entry.

## Failure and retry

A failed attempt records a clear failure outside any rolled-back partial journal transaction. Retry uses the same business idempotency key and a new attempt record linked by `retry_of_attempt_id`. No partially posted entry may remain.

## Immutability and reversal

After posting, entry headers and lines cannot be updated or deleted. A correction creates a new balanced reversal entry with `reversal_of_entry_id`, then posts any corrected entry separately. The original source and accounting history remain traceable.

## Fiscal-period lock

Posting requires an `OPEN` period whose company, fiscal year, and date range match the journal entry. `CLOSED` and `LOCKED` periods reject the database transition. Reopening policy, authorization, and audit logging belong to the future accounting service.

## Traceability

A posted entry carries source event type, source event ID, optional source document ID, idempotency key, timestamps, and actor fields. Journal lines retain account and optional partner/product dimensions. Audit records capture sensitive posting, retry, reversal, period-close, and restore actions.

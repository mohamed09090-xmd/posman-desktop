# POSMAN database decisions

## Scope and authority

This document records Phase 01 SQLite decisions derived from `docs/spec/POSMAN-Blueprint-v1.md`. Ordered files in `database/migrations/` are executable authority. The future Rust application layer owns workflow orchestration, authorization, decimal calculations, and multi-row rules that SQLite cannot enforce safely without hidden business logic.

## Connection contract

Every SQLite connection must execute:

```sql
PRAGMA foreign_keys = ON;
```

The installed writable database should normally evaluate `PRAGMA journal_mode = WAL`, use a bounded busy timeout, and fall back deliberately for read-only, recovery, backup-verification, or unsupported filesystem cases. WAL is runtime database state and is not treated as a migration-only guarantee.

## Names and identifiers

- Tables and columns use English `snake_case`; table names are plural.
- Primary keys are `id`; foreign keys are `<entity>_id`.
- Business identifiers are UUID-compatible `TEXT` values declared explicitly `NOT NULL` and constrained so trimmed identifiers cannot be blank.
- Human document and journal numbers are separate scoped fields.
- The future Rust service should generate UUIDv7 values for new operational rows using a maintained library; UUIDv4 remains acceptable for imports. No SQLite extension is required.

## Company scope

Version 1 installs one company, but every business table carries `company_id`. `app_migrations` is installation-scoped. Global reference role templates, permissions, and their grants intentionally use nullable `company_id`; no company or administrator is seeded.

Globally unique UUIDs and repository-layer checks prevent accidental cross-company joins in v1. A future approved multi-company phase may add composite company-scoped foreign keys through new migrations.

## Fixed-point numbers

SQLite `REAL` is prohibited for application columns.

| Category | Convention | Scale | Example |
|---|---|---:|---|
| Final monetary totals | `*_minor` | 2 | DZD 1,234.56 → `123456` |
| Unit prices and costs | `*_scaled` | 4 | DZD 12.3456 → `123456` |
| Quantities | `*_scaled` | 6 | 8.125000 → `8125000` |
| Percentage rates | `*_rate_scaled` / `rate_scaled` | 4 percentage points | 19.0000% → `190000` |

The Rust layer must use decimal arithmetic and explicit rounding. SQLite validates signs/ranges but does not calculate document totals, taxes, discounts, or CUMP.

## Dates and timestamps

Commercial, posting, due, fiscal, and validity dates are ISO `YYYY-MM-DD` text. Record/event timestamps are ISO 8601 UTC text, for example `2026-07-28T10:00:00Z`. The company timezone is stored separately.

## Migrations and snapshot

Migrations are applied in filename order and recorded in `app_migrations` with SHA-256 checksums. `database/schema.sql` is generated review output and must match the migrations byte-for-byte through:

```bash
python scripts/verify_schema.py --write-schema
python scripts/verify_schema.py
```

Released migrations are immutable; corrections are new roll-forward migrations.

## Commercial documents and conversion

A unified header/line model supports sales, purchases, inventory documents, direct invoices, returns, and credits. `document_line_links` is the quantity-level conversion source of truth. It supports one source line to multiple targets and compatible sources into one target.

A link may be created from an already posted source document into a draft downstream target; otherwise posted delivery-to-invoice workflows would be impossible. Once the target is posted, new inbound lineage is rejected. Existing links touching a posted source or target cannot be updated or deleted. Commercial line-update protection also inspects both the old and new document parent, so a draft line cannot be reparented into a posted document.

Aggregate transformed quantity must not exceed source quantity. This requires a `SUM` across sibling links and must be checked inside the future Rust conversion transaction. Verification deliberately creates an over-conversion under a savepoint and proves the detector finds it; no misleading row CHECK or workflow trigger is used.

## Inventory

`stock_movements` is append-only and authoritative. Corrections are new movements, optionally linked through `reversal_of_movement_id`. `stock_balances` is a rebuildable projection with the row-level identity `available = on_hand - reserved`.

CUMP, negative-stock authorization, reservations, transfer pairing validation, and projection updates belong to Rust application services. Movement rows retain before/after quantity and average-cost snapshots for auditability.

## Accounting

Account codes, journals, and posting mappings are company configuration, never hardcoded legal rules. Journal entries must be inserted as `DRAFT`. The database validates the documented transition to `POSTED`: matching open fiscal period, at least two lines, positive total, and equal debit/credit.

Each journal line has exactly one positive side. Posted entries and lines are immutable; line-update triggers inspect both the old and new journal parent, closing reparenting into a posted entry. Reversal is a new balanced entry. Company-scoped idempotency keys prevent duplicate posting results.

## Trigger policy

The schema contains 25 triggers limited to append-only protection, posted-data immutability, forcing the documented draft-to-posted path, and validating journal posting. Triggers do not calculate CUMP, document totals, taxes, stock balances, general workflow transitions, or posting-rule selection.

## Controlled vocabularies

Stable states and categories use CHECK-constrained text. This prevents obvious invalid values while detailed transition authorization remains in future Rust domain services.

## Security and privacy

Passwords and session tokens are represented only as hashes. SQL contains no real company, person, password, tax rate, account number, token, binary attachment, telemetry, or network dependency.

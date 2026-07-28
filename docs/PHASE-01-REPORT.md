# PHASE 01 implementation report

## Delivery identity

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Branch: `phase/01-data-foundation`
- Draft Pull Request: `https://github.com/mohamed09090-xmd/posman-desktop/pull/1`
- Pull Request title: `[Phase 01] POSMAN SQLite data foundation`
- Active execution pack: `POSMAN-Execution-Pack-01-GitHub-Data-Foundation.md`
- Architecture source: `docs/spec/POSMAN-Blueprint-v1.md`
- Active correction: `POSMAN-Patch-Prompt-01A-SQLite-Integrity.md`
- Blueprint SHA-256: `d932aa0b36099d5ad5dbbb873abc39c957393349af7e1dd6565af06f08be8a84`

Both source documents were read completely before implementation. The execution pack contains 1,015 lines and the Blueprint contains 1,014 lines.

## Scope delivered

PHASE 01 delivers the SQLite data foundation only:

- repository policy and developer guidance;
- four ordered SQLite migrations;
- safe deterministic global reference seed data;
- generated schema snapshot;
- schema/invariant verification script and SQL assertions;
- Linux and Windows GitHub Actions verification;
- ERD, data dictionary, database decisions, migration policy, and accounting-posting design;
- preserved authoritative Blueprint;
- Draft Pull Request and this phase report.

No Tauri application, Rust service implementation, React UI, PDF runtime, installer, cloud service, telemetry, online account, subscription, or PHASE 02 work was added.

## Database result

- Ordered migrations: **4**
- SQLite tables: **49**
- SQLite triggers: **25**
- Seeded system role templates: **6**
- Seeded permissions: **22**
- Application columns declared as `REAL`: **0**

The migration files are authoritative. `database/schema.sql` is generated review output and is verified against the migration concatenation.

### Migration boundaries

1. `0001_system_company_security.sql`
   - migration ledger, company/fiscal structure, sequences, users, roles, permissions, assignments, and sessions.
2. `0002_reference_catalog_partners.sql`
   - units, taxes, payment terms/methods, warehouses/locations, product families/products/prices, partners, addresses, and contacts.
3. `0003_commerce_inventory.sql`
   - commercial document headers/lines, quantity-level conversion lineage, status history, payments/allocations, append-only stock movements, projections, reservations, and inventory counts.
4. `0004_accounting_documents_audit.sql`
   - accounts, journals, configurable posting rules, journal entries/lines, posting attempts, document templates/versions, rendered documents, attachments, audit, idempotency, and backup history.

## Fixed-point and data decisions

- Final monetary amounts use integer minor units, scale 2.
- Unit prices and unit costs use integer scale 4.
- Quantities use integer scale 6.
- Percentage rates use integer percentage-point scale 4; `19.0000%` is stored as `190000`.
- Business identifiers are UUID-compatible `TEXT`, explicitly non-null and nonblank; human document numbers remain separate.
- Commercial dates are date-only ISO text; record/event timestamps are UTC ISO 8601 text.
- Every future SQLite connection must enable `PRAGMA foreign_keys = ON`.
- WAL is the preferred runtime mode for writable local databases, but is not encoded as a migration invariant.
- `.gitattributes` enforces LF for deterministic Blueprint and schema hashes on Windows and Linux.

## Integrity and immutability decisions

- `stock_movements` is the append-only inventory source of truth.
- `stock_balances` is a rebuildable projection and is not authoritative.
- Posted commercial document headers and lines cannot be updated or deleted; child-line update protection checks both old and new parent identities.
- Status history, stock movements, audit logs, template versions, and rendered documents are append-only/immutable.
- Posted journal entries and lines cannot be mutated or deleted; child-line update protection checks both old and new parent identities.
- Journal posting requires a matching open fiscal period, at least two lines, a positive total, and equal debit/credit totals.
- Corrections use reversal, return, credit, or compensating records instead of destructive edits.
- Account numbers and posting mappings are configurable data; no account code is permanent application logic.
- No company, administrator, password, tax rate, account code, secret, customer record, or production database is seeded.

## Partial-document lineage

`document_line_links` records source line, target line, transformation type, transformed quantity, actor, and timestamp. It supports order to delivery/receipt to invoice lineage and partial conversion.

The aggregate rule “sum of transformed quantities must not exceed the source-line quantity” is intentionally a future Rust application-service invariant. SQLite cannot express the multi-row sum as a plain `CHECK`, and a hidden aggregate trigger would create fragile workflow logic. The verifier inserts an over-conversion inside a savepoint, proves the violation is detectable, and rolls it back.

## Validation performed

### Local validation

Command:

```text
python scripts/verify_schema.py
```

Result:

```text
POSMAN SQLite verification: PASS
migrations: 4
tables: 49
triggers: 25
passed checks: 67
pending application invariants: 1
application-service invariant: aggregate transformed quantity must not exceed source quantity
```

Additional commands:

```text
git diff --check
```

Result: success, no output.

```text
git status --short
```

Result: success, no output; worktree clean.

The verifier also confirms:

- all four migrations apply to a fresh temporary SQLite database;
- seed data applies twice without duplication or destructive change;
- `PRAGMA foreign_keys = ON` is active;
- `PRAGMA foreign_key_check` returns no violations;
- all expected tables/triggers exist;
- no application schema column declares `REAL`;
- all 48 built-schema `TEXT` business primary keys report explicit `NOT NULL`;
- null and whitespace-only business identifiers are rejected;
- draft commercial and journal child lines cannot be reparented into posted parents;
- rejected reparenting leaves posted line counts, commercial totals, and journal balance unchanged;
- schema snapshot matches the ordered migrations;
- Blueprint SHA-256 matches the preserved source;
- positive commercial, inventory, lineage, and balanced-accounting fixtures succeed;
- negative constraints and immutability triggers reject invalid writes;
- Mermaid blocks are structurally present;
- no secret, environment, or database artifact is tracked.

### GitHub Actions

Permanent workflow: `.github/workflows/schema-ci.yml`

Successful matrix run:

- Run: `30366975982`
- URL: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30366975982`
- `ubuntu-latest`: **success**
- `windows-latest`: **success**

Each matrix job successfully executed:

```text
python scripts/verify_schema.py
git diff --check
python -c "... git status --short ..."
```

An earlier Windows materialization check exposed CRLF conversion affecting deterministic byte hashes. Commit `420089e2f97ebdee3b6af3a6b1ef305548ef535b` added `.gitattributes`; the permanent Linux/Windows matrix then passed completely.

## Files delivered

### Repository and CI

- `.editorconfig`
- `.gitattributes`
- `.gitignore`
- `.github/pull_request_template.md`
- `.github/workflows/schema-ci.yml`
- `AGENTS.md`
- `README.md`

### Database

- `database/migrations/0001_system_company_security.sql`
- `database/migrations/0002_reference_catalog_partners.sql`
- `database/migrations/0003_commerce_inventory.sql`
- `database/migrations/0004_accounting_documents_audit.sql`
- `database/seed/reference_data.sql`
- `database/schema.sql`
- `database/tests/invariants.sql`
- `scripts/verify_schema.py`

### Documentation

- `docs/spec/POSMAN-Blueprint-v1.md`
- `docs/architecture/accounting-posting.md`
- `docs/architecture/data-dictionary.md`
- `docs/architecture/database-decisions.md`
- `docs/architecture/erd.md`
- `docs/architecture/migration-policy.md`
- `docs/PHASE-01-REPORT.md`

The one-line bootstrap instruction file `agent.md` was removed as its content explicitly instructed deletion at project start.

## Commit history and transport note

The exact implementation history before this report is recorded in the Draft Pull Request comment marked `phase-01-exact-commit-log`. The core implementation commits are:

- `1f00935a8d73b7d8995901e213eb78ad3acb8bf5` — `docs: preserve POSMAN blueprint and data architecture`
- `1ecac540640d83e82d155531d5ba763947870018` — `feat(db): add ordered SQLite schema migrations`
- `f9ae891fef4ece8f8260e1355a579a8104ce95de` — `test(db): add schema and invariant verification`
- `420089e2f97ebdee3b6af3a6b1ef305548ef535b` — `chore: enforce deterministic LF line endings`
- `dcb3d426ca59adc77c8b478a6ef99ce34015ab3d` — `ci: verify SQLite schema on Linux and Windows`

Because the execution container could not resolve `github.com` for Git CLI push, a SHA-256-verified compressed source bundle and temporary PR workflow were used to materialize the large source files through GitHub Actions. Those transport artifacts and temporary workflows were deleted from the final tree. The operational transport commits remain visible in immutable Git history; they contain no secrets, credentials, customer data, production database, or executable product feature.

## Risks and limitations

- Aggregate transformed-quantity enforcement remains a documented Rust transaction invariant; only its detection is implemented in this phase.
- CUMP calculation, negative-stock authorization, reservation consumption, stock-balance projection, tax rounding, document-total calculation, posting-rule selection, and workflow authorization belong to future application services.
- SQLite trigger enforcement is intentionally limited to immutability, append-only evidence, and journal-posting aggregate validation.
- POSMAN is not yet a runnable commercial desktop application; this phase is only its validated data foundation.
- The separate architect/reviewer must accept or reject the phase. This report does not self-approve it.

## Delivery state

- Branch pushed: **yes**
- Required local validation: **passed**
- Linux CI: **passed**
- Windows CI: **passed**
- Draft Pull Request opened: **yes**
- Pull Request merged: **no**
- PHASE 02 started: **no**

## Review Patch 01A — SQLite integrity hardening

The independent architectural review reproduced two SQLite integrity defects in the unreleased Phase 01 schema. Patch 01A corrects the existing migrations rather than adding a false `0005` migration.

### Corrected identifier contract

All 48 business tables with `TEXT` primary keys now declare `id` explicitly `NOT NULL` and reject identifiers whose trimmed length is zero. `app_migrations.id INTEGER PRIMARY KEY` is unchanged. Future Rust UUIDv7 generation remains the application policy; SQLite does not generate UUIDs.

The verifier inspects the schema built by SQLite through `PRAGMA table_info` for every expected table, requires all 48 text primary keys to report explicit nullability protection, and executes negative inserts proving both `NULL` and whitespace-only company identifiers are rejected.

### Closed posted-child reparenting bypasses

`trg_commercial_lines_posted_no_update` now checks both `OLD.document_id` and `NEW.document_id`. `trg_journal_lines_posted_no_update` now checks both `OLD.journal_entry_id` and `NEW.journal_entry_id`.

Negative fixtures create draft child lines and attempt to move them into already posted parents. SQLite must reject both updates. Follow-up assertions verify that the posted invoice retains its original line count and header totals, while the posted journal entry retains its original line count and remains balanced.

### Patch validation

The final verifier output and final Linux/Windows CI run are recorded in the Draft Pull Request Review Patch 01A section. The only pending application invariant remains aggregate transformed-quantity enforcement in the future Rust conversion transaction.

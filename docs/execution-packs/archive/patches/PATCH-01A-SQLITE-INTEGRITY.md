# POSMAN — Review Patch Prompt 01A

## PHASE 01 SQLite integrity hardening

Status of the architectural review:

```text
CHANGES REQUIRED
```

This is a bounded correction pass for PHASE 01. It is not PHASE 02.

---

# EXECUTOR PROMPT — COPY FROM HERE

You are correcting the existing POSMAN PHASE 01 implementation after an
independent architectural review found two reproducible database-integrity
defects.

## 1. Repository and existing Pull Request

Repository:

```text
mohamed09090-xmd/posman-desktop
```

Continue on the existing branch:

```text
phase/01-data-foundation
```

Update the existing Draft Pull Request:

```text
https://github.com/mohamed09090-xmd/posman-desktop/pull/1
```

Do not:

- Create a replacement Pull Request.
- Merge the Pull Request.
- Commit to `main`.
- Force-push or rewrite existing history.
- Begin PHASE 02.
- Add Tauri, Rust application code, React, UI, or installer work.

The four migrations are still unreleased because PHASE 01 has not been
accepted or merged. Therefore, correct the existing PHASE 01 migration files
and regenerate `database/schema.sql`. Do not add a fake `0005` migration merely
to avoid correcting an unreleased schema.

## 2. Read before editing

Read completely:

```text
AGENTS.md
docs/spec/POSMAN-Blueprint-v1.md
docs/PHASE-01-REPORT.md
docs/architecture/database-decisions.md
docs/architecture/data-dictionary.md
database/migrations/0001_system_company_security.sql
database/migrations/0002_reference_catalog_partners.sql
database/migrations/0003_commerce_inventory.sql
database/migrations/0004_accounting_documents_audit.sql
scripts/verify_schema.py
database/tests/invariants.sql
```

Inspect the current branch, clean-worktree state, PR head, and current CI before
editing.

## 3. Review finding A — nullable business primary keys

### Reproduced defect

The schema declares business identifiers using:

```sql
id TEXT PRIMARY KEY
```

In ordinary SQLite rowid tables, a non-`INTEGER` primary key does not
automatically receive the same `NOT NULL` behavior expected from standard SQL.
The reviewed schema exposes 48 `TEXT` primary-key columns with
`PRAGMA table_info(...).notnull = 0`.

The following invalid operations were reproduced successfully against a fresh
database built from the four migrations:

```text
INSERT company row with id = NULL  -> ALLOWED
INSERT a second company with id = NULL -> ALLOWED
```

This violates the UUID-compatible identifier requirement and permits ambiguous
business records.

### Mandatory correction

For every business table whose primary key is a text identifier, change the
identifier declaration to explicitly reject null and blank identifiers.

Use one consistent form such as:

```sql
id TEXT NOT NULL PRIMARY KEY CHECK (length(trim(id)) > 0)
```

Requirements:

- Apply this to all 48 affected `TEXT` primary keys across migrations
  `0001` through `0004`.
- Do not change `app_migrations.id INTEGER PRIMARY KEY`.
- Do not introduce an SQLite UUID extension.
- Do not require a database-generated UUID.
- Keep the documented future Rust UUIDv7 generation policy.
- Existing symbolic test fixture identifiers may remain; the database contract
  in this phase must guarantee non-null, nonblank text identifiers.

### Mandatory automated verification

Extend `scripts/verify_schema.py` so it:

1. Inspects every expected application table through `PRAGMA table_info`.
2. Verifies that every text primary key named `id` is explicitly `NOT NULL`.
3. Fails with the table name if any nullable text primary key is found.
4. Includes a negative operation proving that a null business identifier is
   rejected.
5. Includes a negative operation proving that a blank business identifier is
   rejected.

Do not implement this as a source-text regex alone. The verifier must inspect
the schema SQLite actually built.

## 4. Review finding B — posted-child reparenting bypass

### Reproduced commercial-document defect

The current trigger:

```text
trg_commercial_lines_posted_no_update
```

checks only `OLD.document_id`.

The following sequence was reproduced successfully:

1. Create a Draft commercial document.
2. Add a line to the Draft document.
3. Update that line so `document_id` points to an already Posted invoice.
4. The update succeeds.

This adds a new line to a posted commercial document through an `UPDATE`,
bypassing both the posted-line update trigger and posted-line insert trigger.

### Reproduced accounting defect

The current trigger:

```text
trg_journal_lines_posted_no_update
```

checks only `OLD.journal_entry_id`.

The following sequence was reproduced successfully:

1. Create a Draft journal entry.
2. Add a line to the Draft entry.
3. Update that line so `journal_entry_id` points to an already Posted entry.
4. The update succeeds.

This mutates the effective lines of a posted accounting entry.

### Mandatory correction

Update the two affected `BEFORE UPDATE` triggers so protection applies when
either the old parent or the new parent is posted.

The commercial-line trigger must reject an update when a posted document is
found for either:

```text
OLD.document_id
NEW.document_id
```

The journal-line trigger must reject an update when a posted journal entry is
found for either:

```text
OLD.journal_entry_id
NEW.journal_entry_id
```

Using an `IN (OLD..., NEW...)` parent lookup is acceptable. Keeping child-parent
identifiers immutable at the application layer is not sufficient; the required
database protection must not be bypassable by an ordinary SQL update.

Do not weaken the existing protection for:

- Updating a line already belonging to a posted parent.
- Deleting a line belonging to a posted parent.
- Inserting a line directly into a posted parent.

### Mandatory automated verification

Add negative tests that:

1. Create a Draft commercial document and line.
2. Attempt to reparent the Draft line into an already Posted commercial
   document.
3. Confirm SQLite rejects the operation.
4. Confirm the posted document's line count and totals fixture state were not
   mutated.
5. Create a Draft journal entry and line.
6. Attempt to reparent the Draft line into an already Posted journal entry.
7. Confirm SQLite rejects the operation.
8. Confirm the posted entry retains the original line count and remains
   balanced.

The tests must fail against the current reviewed triggers and pass only after
the triggers are corrected.

## 5. Documentation consistency correction

In:

```text
docs/architecture/data-dictionary.md
```

the `Numeric columns by table` appendix currently lists
`rendered_documents` twice. Remove the duplicate.

Also update the relevant documentation so it explicitly states:

- Text business primary keys are non-null and nonblank.
- Posted child-line update protection checks both old and new parent
  identities.
- The new verifier coverage exists.

Update:

```text
docs/architecture/database-decisions.md
docs/architecture/data-dictionary.md
docs/PHASE-01-REPORT.md
README.md
```

Only add information relevant to this review patch.

## 6. Generated schema

`database/schema.sql` is generated review output.

After correcting the unreleased migrations, regenerate it with:

```text
python scripts/verify_schema.py --write-schema
```

Then verify it again without the write flag:

```text
python scripts/verify_schema.py
```

Do not manually maintain a schema snapshot that differs from the ordered
migrations.

## 7. Required validation

Run and report the exact output of:

```text
python scripts/verify_schema.py
git diff --check
git status --short
```

The verifier must still prove all original PHASE 01 scenarios, including:

- Four ordered migrations.
- 49 expected tables.
- No application `REAL` columns.
- Idempotent safe seed data.
- Foreign-key integrity.
- Commercial partial lineage fixtures.
- Append-only stock and audit ledgers.
- Posted commercial document immutability.
- Balanced journal posting.
- Closed-period rejection.
- Posted journal immutability.
- The single documented pending Rust aggregate over-conversion invariant.

It must additionally prove:

- Every `TEXT` business primary key is explicitly non-null.
- Null and blank business identifiers are rejected.
- Commercial-line reparenting into a posted document is rejected.
- Journal-line reparenting into a posted entry is rejected.

The allowed pending invariant remains:

```text
application-service invariant: aggregate transformed quantity must not exceed source quantity
```

Do not add either review defect to the pending list. Both must be enforced and
tested in SQLite.

## 8. CI and Pull Request

Push ordinary new commits to:

```text
phase/01-data-foundation
```

Suggested commit structure:

```text
fix(db): enforce non-null business identifiers
fix(db): close posted-line reparenting bypasses
test(db): cover reviewed SQLite integrity defects
docs: record phase 01 integrity review patch
```

Combining closely related commits is acceptable, but commit messages must remain
clear.

Wait for the permanent workflow to run on:

```text
ubuntu-latest
windows-latest
```

Both jobs must pass on the final head commit.

Keep Pull Request #1 as Draft. Update its body with a short `Review Patch 01A`
section containing:

- Both corrected defects.
- New verifier checks.
- Final test output.
- Final CI run link.

Do not merge it.

## 9. Stop conditions

Stop and ask the user before proceeding if:

- Repository access becomes read-only.
- The current PR or branch no longer matches the specified target.
- Fixing the defects would require destructive history rewriting.
- The Blueprint conflicts with a mandatory review correction.
- A required test cannot be made deterministic on both Windows and Ubuntu.

Do not stop for ordinary implementation choices already resolved above.

## 10. Definition of done

Patch 01A is complete only when:

- All 48 text primary keys explicitly reject null identifiers.
- Blank identifiers are rejected consistently.
- The verifier inspects built-schema PK nullability.
- The commercial posted-line reparenting bypass is closed and tested.
- The journal posted-line reparenting bypass is closed and tested.
- Existing invariant tests still pass.
- `database/schema.sql` exactly matches ordered migrations.
- Documentation and Phase report describe the corrected behavior.
- CI succeeds on Ubuntu and Windows at the final head.
- New commits are pushed without force-push.
- Pull Request #1 remains open, Draft, and unmerged.
- PHASE 02 has not started.

## 11. Final handoff format

Return:

1. Final branch head SHA.
2. New commits and messages.
3. Exact files changed.
4. Exact `python scripts/verify_schema.py` output.
5. Evidence that nullable/blank IDs are rejected.
6. Evidence that both reparenting bypasses are rejected.
7. Ubuntu and Windows CI job results and run URL.
8. Updated Draft PR URL.
9. Confirmation that no force-push was used.
10. Confirmation that the PR was not merged.
11. Confirmation that PHASE 02 was not started.

# END OF EXECUTOR PROMPT

---

## What to return to the architect

Send the complete final handoff, the new head SHA, and the CI run URL back to
the external architect/reviewer. PHASE 01 remains rejected until the new head
is independently reviewed.

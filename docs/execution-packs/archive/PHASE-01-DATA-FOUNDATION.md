# POSMAN — Execution Pack 01

## GitHub Repository & SQLite Data Foundation

> Executor: ChatGPT Thinking SOL 5.6 High
> Execution surface: GitHub
> Reviewer/Architect: the planner in the original POSMAN conversation
> Status: Ready to send
> Scope type: bounded architecture and database task

---

## تعليمات للمستخدم قبل إرسال الـPrompt

1. حمّل وأرفق للمنفذ الملف:
   - `POSMAN-Blueprint-v1.md`
2. أرسل له النص الموجود تحت عنوان **EXECUTOR PROMPT — COPY FROM HERE** كاملًا.
3. لا تطلب منه تنفيذ الواجهة أو برنامج Tauri في هذه المرحلة.
4. عند انتهائه، أرسل للمراجع:
   - رابط المستودع.
   - رابط Draft Pull Request.
   - تقرير المرحلة.
   - مخرجات الاختبارات.
   - أي ملاحظات أو أسئلة كتبها المنفذ.

### الافتراض الافتراضي للمستودع

- الاسم: `posman-desktop`
- الخصوصية: Private أثناء التطوير
- الفرع: `phase/01-data-foundation`
- لا يتم الدمج إلى `main` قبل المراجعة.

إذا كان المستودع موجودًا، يجب على المنفذ استعماله بدل إنشاء مستودع جديد.

---

# EXECUTOR PROMPT — COPY FROM HERE

You are the implementation agent for **POSMAN**, a Windows desktop commercial
management application for Algerian merchants.

You are working under a strict architect–executor–reviewer workflow:

- The attached `POSMAN-Blueprint-v1.md` is the authoritative product
  specification.
- You are the executor for one bounded phase only.
- Do not expand scope.
- Do not merge your own Pull Request.
- The work will be reviewed by a separate architect after you finish.

## 1. Mission

Create the GitHub data-foundation package for POSMAN:

1. Initialize or orient the GitHub repository.
2. Preserve the supplied Blueprint unchanged.
3. Design the detailed SQLite relational model.
4. Create ordered SQLite migrations.
5. Add an ERD and data dictionary.
6. Implement automated schema and invariant verification.
7. Open a Draft Pull Request for review.

This phase is successful only when a clean temporary SQLite database can be
created from the migrations and the required invariants are demonstrated by
automated tests.

## 2. Repository rule

Preferred repository:

```text
posman-desktop
```

Preferred visibility during development:

```text
Private
```

If a repository URL is supplied, use that repository.

If no repository exists and your GitHub connection allows repository creation,
create a private repository named `posman-desktop`.

If you cannot create the repository:

1. Stop before doing local-only implementation.
2. Ask the user to create an empty private GitHub repository named
   `posman-desktop`.
3. Continue only after repository access is confirmed.

Never create a public repository without explicit user approval.

## 3. Branch and Pull Request

Create and use:

```text
phase/01-data-foundation
```

Open a Draft Pull Request titled:

```text
[Phase 01] POSMAN SQLite data foundation
```

Do not merge it.

Use intentional commits. A recommended sequence is:

```text
docs: add POSMAN product blueprint and architecture rules
feat(db): add ordered SQLite schema migrations
test(db): add schema and invariant verification
docs: add phase 01 implementation report
```

Do not rewrite unrelated history or force-push unless a repository-specific
instruction explicitly requires it.

## 4. Inspect before editing

Before creating or modifying files:

1. Inspect the repository root.
2. Read `AGENTS.md`, `README.md`, repository instructions and existing
   architecture documents if present.
3. Check the current branch and working tree.
4. Preserve all unrelated user changes.
5. Report conflicts instead of overwriting them.
6. Confirm whether this is an empty repository or an existing project.

Write a short execution plan before editing.

## 5. Authoritative requirements

The attached `POSMAN-Blueprint-v1.md` is authoritative.

Key decisions that must not be changed:

- Product name: POSMAN.
- Product type: general desktop software for Algerian merchants.
- Windows 10/11 64-bit first.
- Offline and local-first.
- One company per installation in v1, but every business table must still
  carry `company_id` for future expansion.
- Arabic first, French supported.
- Currency: DZD in v1.
- Database: embedded SQLite.
- No external database service.
- No cloud dependency.
- No subscription or activation system in v1.
- Moving weighted-average inventory costing: CUMP/CMUP.
- Negative inventory blocked by default.
- Order → partial/full delivery → invoice workflow.
- Automatic configurable accounting posting.
- Posted documents and entries are immutable.
- Corrections use reversal, return or credit documents.
- Stock movement ledger is the inventory source of truth.

If the Blueprint conflicts with a suggestion in this prompt, the Blueprint
wins unless the conflict would make the database invalid. In that case,
document the conflict and stop for clarification.

## 6. Scope boundaries

### In scope

- Repository orientation and documentation.
- SQLite database architecture.
- Migrations.
- ERD.
- Data dictionary.
- Accounting-posting design documentation.
- Seeded reference roles and permissions.
- Automated database verification.
- GitHub Actions for database verification.
- Draft Pull Request.

### Explicitly out of scope

- Tauri application scaffold.
- React UI.
- Rust application services.
- Login screens.
- Business feature implementation.
- PDF generation.
- Installer creation.
- Cloud sync.
- Multi-computer networking.
- Mobile applications.
- License or activation servers.
- Real company data.
- Real credentials or secrets.

Do not add placeholder UI or a demo web application.

## 7. Required repository structure

Create or adapt this structure:

```text
.
├── .github/
│   ├── pull_request_template.md
│   └── workflows/
│       └── schema-ci.yml
├── database/
│   ├── migrations/
│   │   ├── 0001_system_company_security.sql
│   │   ├── 0002_reference_catalog_partners.sql
│   │   ├── 0003_commerce_inventory.sql
│   │   └── 0004_accounting_documents_audit.sql
│   ├── seed/
│   │   └── reference_data.sql
│   ├── tests/
│   │   └── invariants.sql
│   └── schema.sql
├── docs/
│   ├── spec/
│   │   └── POSMAN-Blueprint-v1.md
│   ├── architecture/
│   │   ├── database-decisions.md
│   │   ├── data-dictionary.md
│   │   ├── erd.md
│   │   ├── accounting-posting.md
│   │   └── migration-policy.md
│   └── PHASE-01-REPORT.md
├── scripts/
│   └── verify_schema.py
├── .editorconfig
├── .gitignore
├── AGENTS.md
└── README.md
```

You may make small justified naming adjustments, but document every deviation
in `docs/PHASE-01-REPORT.md`.

Do not add a LICENSE in this phase because the project license has not been
selected.

## 8. Blueprint preservation

Copy the supplied `POSMAN-Blueprint-v1.md` to:

```text
docs/spec/POSMAN-Blueprint-v1.md
```

Preserve its content. You may only normalize final line endings or add a short
front-matter note stating that it is the authoritative v1 specification.

Do not silently rewrite product requirements.

## 9. SQLite architecture rules

### 9.1 Connection configuration

Document runtime connection requirements:

```sql
PRAGMA foreign_keys = ON;
```

Evaluate and document WAL mode for the future application runtime. Do not
assume that a migration alone configures every future connection.

Use a migration ledger table so the application can identify the installed
schema version.

### 9.2 Naming

- English `snake_case` table and column names.
- Singular or plural naming must be consistent across the complete schema.
- Primary keys named `id`.
- Foreign keys named `<entity>_id`.
- Human document numbers are separate from internal UUIDs.

### 9.3 Identifiers

- Use UUID-compatible text identifiers for business entities.
- Do not use a displayed invoice/order number as a primary key.
- Document how UUIDs will be generated by the future Rust application.
- Do not add an SQLite extension dependency only for UUID generation.

### 9.4 Fixed-point numbers

Never store money, prices, costs, tax rates, discounts or quantities in a
SQLite `REAL` column.

Use fixed-point integers:

- Final monetary totals: scale 2, stored as integer minor units.
- Unit prices and unit costs: scale 4.
- Quantities: scale 6.
- Percentage rates: explicit fixed scale documented in the data dictionary.

Use names that expose the scale where ambiguity is dangerous, or define the
scale centrally and consistently.

Examples of acceptable names:

```text
total_ttc_minor
unit_price_scaled
quantity_scaled
tax_rate_basis
```

The data dictionary must document the scale and unit for every numeric field.

### 9.5 Dates and times

- Store business dates separately from record timestamps.
- Use a documented ISO 8601 UTC format for timestamps.
- Preserve a local commercial date for invoices and stock documents.
- Do not mix a date-only value with a timestamp in the same column.

### 9.6 Audit columns

Business records should use consistent fields where appropriate:

```text
created_at
created_by
updated_at
updated_by
row_version
company_id
```

Do not add meaningless `updated_at` fields to append-only ledgers if they can
never be updated.

### 9.7 Soft deletion

- Master data may use archival flags such as `is_active`.
- Posted transactional data must never rely on soft deletion to simulate
  accounting correction.
- Posted documents, journal entries, stock movements and audit records must not
  be deleted.

## 10. Required domain groups

The schema must cover these groups.

### 10.1 System, company and security

At minimum:

```text
companies
company_settings
fiscal_years
fiscal_periods
document_sequences
users
roles
permissions
user_roles
role_permissions
sessions
app_migrations
```

No default administrator password may be seeded. The first-run application
will create the administrator later.

### 10.2 Reference data and catalog

At minimum:

```text
units
tax_rates
payment_terms
payment_methods
warehouses
warehouse_locations
product_families
products
price_lists
product_prices
partners
partner_addresses
partner_contacts
```

Requirements:

- A partner can be a customer, supplier or both.
- Product commercial data must not be mixed with stock balance.
- Tax rates require an effective validity period.
- Do not hardcode a current Algerian VAT percentage as permanent logic.
- Product code and barcode uniqueness must be scoped intentionally.

### 10.3 Commercial documents

At minimum:

```text
commercial_documents
commercial_document_lines
document_line_links
document_status_history
payments
payment_allocations
```

The design must support:

- Sales and purchases.
- Order, delivery/receipt, invoice, return and credit document categories.
- Draft and posted states.
- Direct invoices.
- One source line converted into several target lines.
- Partial delivery and partial invoicing.
- Quantity remaining to convert.
- Snapshot product code, description, unit, unit price, unit cost, tax and
  discount data on document lines.
- Header and line discounts.
- HT, tax and TTC totals.
- Separate commercial date and posting date.
- Human-readable sequence number.
- Idempotent posting.

`document_line_links` must contain the transformed quantity and preserve the
source/target line relationship.

Do not model document conversion using only one `source_document_id`.

### 10.4 Inventory

At minimum:

```text
stock_movements
stock_balances
stock_reservations
inventory_counts
inventory_count_lines
```

Requirements:

- `stock_movements` is the authoritative append-only ledger.
- Corrections are new movements.
- `stock_balances` is a rebuildable projection.
- Track on-hand, reserved and available semantics.
- Opening stock is an `OPENING` movement.
- Store cost snapshots required for CUMP/CMUP.
- Prevent duplicate movement generation for the same posting event.
- Support warehouse transfers as paired movements.
- Do not update stock only by editing a balance column.

### 10.5 Accounting

At minimum:

```text
accounts
accounting_journals
posting_rules
journal_entries
journal_entry_lines
posting_attempts
```

Requirements:

- Configurable chart of accounts.
- Configurable posting mappings.
- Balanced debit and credit before an entry becomes posted.
- One posting result per source event/idempotency key.
- Posted entry immutability.
- Reversal by a new entry.
- Fiscal-period lock.
- Traceability from entry to source document and event.
- Do not hardcode Algerian account numbers as immutable program logic.

### 10.6 Documents and audit

At minimum:

```text
document_templates
document_template_versions
rendered_documents
attachments
audit_logs
idempotency_keys
backup_history
```

Requirements:

- Versioned print templates.
- Rendered document traceability.
- Append-only audit records for sensitive actions.
- No secret or binary file content embedded directly in SQL migration files.

## 11. Controlled vocabularies

Do not scatter undocumented free-form status strings.

Choose and document one consistent approach:

1. Lookup/reference tables, or
2. CHECK-constrained text values where migration cost is acceptable.

The design must make invalid document states difficult to store.

At minimum, document the allowed states for:

- Sales order.
- Delivery note.
- Sales invoice.
- Purchase receipt.
- Purchase invoice.
- Journal entry.
- Payment allocation.

State-transition enforcement may remain in the future Rust domain layer, but
the database must still reject obviously invalid terminal-state mutations.

## 12. Database invariants

Implement database-level constraints or triggers where SQLite can safely and
clearly enforce them.

At minimum verify:

1. Foreign keys are valid.
2. No `REAL` columns exist in the application schema.
3. Fixed-point amounts and quantities have non-negative constraints where
   appropriate.
4. Document line quantities are greater than zero.
5. A source line cannot link to itself.
6. A transformed quantity is greater than zero.
7. Human document number uniqueness is scoped by company, fiscal context and
   document type.
8. Duplicate posting/idempotency keys are rejected.
9. A posted commercial document cannot be deleted.
10. Lines of a posted commercial document cannot be inserted, updated or
    deleted.
11. A stock movement cannot be updated or deleted.
12. An audit record cannot be updated or deleted.
13. A posted journal entry cannot be updated or deleted.
14. Lines of a posted journal entry cannot be inserted, updated or deleted.
15. An unbalanced journal entry cannot transition to `POSTED`.
16. A journal line cannot have both a positive debit and positive credit.
17. A journal line cannot have neither debit nor credit.
18. A closed fiscal period prevents posting through the documented posting
    path.

If a rule cannot be safely enforced in SQLite without fragile or misleading
triggers:

- Document why.
- Add an automated negative test describing the expected future application
  rule.
- Mark it as an application-service invariant in
  `database-decisions.md`.

Do not pretend that a plain row CHECK constraint can aggregate child rows.

## 13. Trigger policy

Triggers are allowed only for:

- Immutability protection.
- Append-only enforcement.
- Validating the transition to a posted state.
- Maintaining schema-level invariants that cannot be bypassed by ordinary
  writes.

Do not use hidden trigger logic as the primary implementation for:

- CUMP calculation.
- Document totals.
- General workflow orchestration.
- Complex accounting-rule selection.
- Stock-balance business logic.

Those responsibilities belong to future Rust application services.

Every trigger must be documented in the data dictionary and tested.

## 14. Seed data

`database/seed/reference_data.sql` may contain only safe reference data:

- Standard POSMAN role names.
- Permission identifiers.
- Document-type identifiers.
- Status identifiers or allowed transition metadata, if that design is chosen.
- Application metadata that is not company-specific.

Do not seed:

- A real company.
- A real person.
- A password.
- API keys.
- Tokens.
- A permanent Algerian tax rate presented as universal truth.
- Real account numbers as mandatory mappings.

Seed scripts must be deterministic and safe for a fresh database.

## 15. `database/schema.sql`

Provide a reviewable complete schema file representing the result of applying
all migrations in order.

Choose one documented source-of-truth policy:

- Migrations are authoritative and `schema.sql` is reproducibly generated, or
- `schema.sql` is authoritative and migrations are validated against it.

The recommended policy is:

```text
Ordered migrations are authoritative.
schema.sql is a generated/review snapshot.
```

If generated, include the generation/check command in README.

## 16. ERD

Create:

```text
docs/architecture/erd.md
```

It must contain:

1. A readable Mermaid `erDiagram`.
2. Domain-group explanations.
3. Notes on relationships that Mermaid cannot express fully.
4. A smaller commercial-document lineage diagram showing:

```text
Order line → Delivery line(s) → Invoice line(s)
```

Keep the primary ERD readable. Split it into domain diagrams if one diagram
becomes too dense.

## 17. Data dictionary

Create:

```text
docs/architecture/data-dictionary.md
```

For every table document:

- Purpose.
- Primary key.
- Foreign keys.
- Important columns.
- Numeric scale and unit.
- Uniqueness rules.
- Mutability policy.
- Archival/deletion policy.
- Related triggers.
- Source-of-truth status.

## 18. Accounting posting document

Create:

```text
docs/architecture/accounting-posting.md
```

Document:

- Source events.
- Posting-rule lookup.
- Journal-entry creation.
- Balance validation.
- Idempotency.
- Failure and retry.
- Reversal.
- Fiscal-period lock.
- Traceability.

Account numbers must remain configurable.

Use examples only as illustrations, never as hardcoded legal/accounting
requirements.

## 19. Migration policy

Create:

```text
docs/architecture/migration-policy.md
```

Include:

- Ordered immutable migrations.
- Naming convention.
- Transaction expectations.
- Roll-forward correction policy.
- Development reset policy.
- Production backup requirement before migration.
- Schema-version compatibility.
- What happens if a migration fails.
- Prohibition on silently editing an already released migration.

## 20. AGENTS.md

Create repository guidance for future implementation agents.

It must include:

- Read the Blueprint before changing domain logic.
- Keep the app Windows-first, offline and local.
- Never use floating-point money.
- Never update stock balances without ledger movements.
- Never mutate posted documents or entries.
- Never hardcode tax rates or accounting mappings.
- Never add secrets.
- Preserve Arabic RTL and French support.
- Keep UI original and avoid generic dashboard templates.
- Use small scoped branches and Draft PRs.
- Run required validation before claiming success.
- Do not merge without reviewer approval.

## 21. Verification script

Create:

```text
scripts/verify_schema.py
```

Use Python standard library only unless a dependency is strictly necessary and
justified.

The script must:

1. Create a temporary SQLite database.
2. Enable foreign keys.
3. Apply migrations in filename order.
4. Apply deterministic reference seed data.
5. Confirm the expected tables exist.
6. Run `PRAGMA foreign_key_check`.
7. Scan application tables and fail if any column uses declared type `REAL`.
8. Execute positive fixture operations.
9. Execute negative invariant tests and confirm they fail.
10. Run `database/tests/invariants.sql`.
11. Produce a concise pass/fail summary.
12. Exit nonzero on any failure.
13. Delete temporary output on success and failure.

Do not hide exceptions or convert failures into warnings.

## 22. Minimum verification scenarios

Automate at least these scenarios:

### Positive

1. Create a company and fiscal year.
2. Create a warehouse, family, unit and product.
3. Create a customer and supplier.
4. Create an opening-stock document and movement.
5. Create an order with one line.
6. Create two delivery lines linked to the order line.
7. Create invoice lines linked to delivered quantities.
8. Create a balanced journal entry and post it.

### Negative

1. Duplicate scoped product code where prohibited.
2. Duplicate scoped human document number.
3. Document line with zero quantity.
4. Self-linking document line.
5. Duplicate idempotency key.
6. Update or delete a stock movement.
7. Update or delete an audit record.
8. Modify a posted document.
9. Add a line to a posted document.
10. Post an unbalanced journal entry.
11. Give one journal line debit and credit simultaneously.
12. Modify a posted journal entry.
13. Delete a posted journal line.
14. Post through the documented path into a closed fiscal period.

If partial-quantity over-conversion is enforced only by a future application
service, add a clearly marked pending application-invariant test and explain
why it is not falsely enforced by a fragile trigger.

## 23. GitHub Actions

Create:

```text
.github/workflows/schema-ci.yml
```

Run verification on:

- `ubuntu-latest`
- `windows-latest`

Use a maintained Python version available on both runners.

The workflow must:

1. Check out the repository.
2. Set up Python.
3. Run `scripts/verify_schema.py`.
4. Fail on any schema or invariant error.

Do not add secrets or external services.

## 24. README

The root README must explain:

- What POSMAN is.
- Phase 01 scope.
- Authoritative Blueprint path.
- How to run schema verification.
- Database source-of-truth policy.
- Fixed-point numeric rules.
- Current non-goals.
- That application code is intentionally not part of this phase.

Do not claim that POSMAN is finished or production-ready.

## 25. Pull Request template

Create a practical template containing:

- Scope.
- Linked phase.
- Changed files.
- Architecture decisions.
- Validation performed.
- Test output.
- Screenshots, if applicable.
- Security/data considerations.
- Known limitations.
- Reviewer checklist.

## 26. Phase report

Create:

```text
docs/PHASE-01-REPORT.md
```

It must include:

1. Executive summary.
2. Repository and branch.
3. Files added/modified/deleted.
4. Final table count.
5. Migration list.
6. Trigger list and purpose.
7. Numeric scale decisions.
8. Test scenarios executed.
9. Exact validation commands.
10. Actual results.
11. CI status and links.
12. Deviations from this prompt.
13. Open questions.
14. Risks for the Tauri/Rust phase.
15. Explicit statement that no secrets or real credentials were added.

Do not report a test as passed unless it was actually run.

## 27. Validation commands

At minimum run:

```bash
python scripts/verify_schema.py
git diff --check
git status --short
```

Also validate Mermaid code structurally if an available tool supports it.

If a validation cannot run:

- State what did not run.
- State why.
- State the remaining risk.
- Do not call the phase complete.

## 28. Security and privacy

- No `.env` with real values.
- No access tokens.
- No credentials.
- No real customer/company data.
- No production database.
- No analytics or telemetry.
- No external network service required for tests.

Check the final branch for secret-like files before reporting completion.

## 29. Stop conditions

Stop and ask the user before proceeding if:

- The repository target is ambiguous.
- You lack GitHub write access.
- Existing repository instructions conflict with the Blueprint.
- The working tree contains overlapping uncommitted user changes.
- A requested schema rule contradicts SQLite capabilities.
- A legal/accounting assumption would need to be invented.
- You would need to make the repository public.
- You would need to add a real secret.

Do not bypass permission or access restrictions.

## 30. Definition of done

Phase 01 is done only when:

- The Blueprint is preserved in the repository.
- The relational model covers every required domain group.
- Ordered migrations build a fresh database.
- Foreign-key checks pass.
- No application column declares `REAL`.
- Fixed-point scales are documented.
- Required immutability protections exist and are tested.
- Journal balance protection is tested.
- Partial document lineage is modeled.
- Verification passes locally.
- CI configuration exists for Windows and Ubuntu.
- The report contains actual evidence.
- Changes are committed to `phase/01-data-foundation`.
- A Draft Pull Request is open.
- The PR is not merged.

## 31. Final response to the user

Return a concise handoff containing:

1. Repository URL.
2. Branch name.
3. Draft PR URL.
4. Commit SHAs and messages.
5. Validation command results.
6. CI status.
7. Files changed.
8. Open questions and risks.
9. A request to send the PR and report to the external architect for review.

Do not begin Phase 02.

# END OF EXECUTOR PROMPT

---

## ماذا تعيد للمراجع بعد التنفيذ؟

أرسل للمراجع رسالة بهذا الشكل:

```text
نفّذ الوكيل POSMAN Execution Pack 01.

Repository:
<URL>

Branch:
phase/01-data-foundation

Draft PR:
<URL>

أرفقت:
- docs/PHASE-01-REPORT.md
- مخرجات python scripts/verify_schema.py
- مخرجات GitHub Actions

أريد مراجعة كاملة وإصدار:
1. قرار قبول أو رفض المرحلة.
2. قائمة المشاكل حسب الخطورة.
3. Patch Prompt جاهز للمنفذ إذا لزم.
```

---

## معايير مراجعة المعماري

لن تُقبل المرحلة إذا:

- تم البدء في UI أو Tauri قبل اعتماد قاعدة البيانات.
- استُعمل `REAL` للأموال أو الكميات.
- تم تخزين الرصيد دون سجل حركات مرجعي.
- لا يدعم النموذج التسليم الجزئي.
- المستند المرحّل قابل للتعديل.
- القيد غير المتوازن يمكن ترحيله.
- تم تثبيت نسب ضريبة أو حسابات محاسبية داخل الكود.
- الاختبارات شكلية أو غير منفذة.
- تم الدمج إلى `main` قبل المراجعة.

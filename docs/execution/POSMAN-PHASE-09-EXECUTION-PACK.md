# POSMAN PHASE 09 Execution Pack

**Status:** ACTIVE implementation pack

**Authorized phase:** PHASE 09 only

**Branch:** `phase/09-documents-reports-audit-backup`

**Draft Pull Request title:** `[Phase 09] POSMAN documents, reports, audit, and backup`

**Repository:** `https://github.com/mohamed09090-xmd/posman-desktop`
**Minimum accepted baseline:** `0abaff289758fd2e5597faef834f9b70156d54e1`

This document persists the Product Owner's active PHASE 09 authorization. It is an implementation contract, not acceptance evidence. The implementation engineer must not self-approve, merge the Pull Request, mark the phase accepted, advance continuity to Checkpoint 09, or start PHASE 10.

## 1. Execution and continuation contract

Before editing, resolve live `main`, all open Pull Requests, all PHASE 09 branches, repository instructions, continuity documents, accepted PHASE 05–08 implementations, current schema, current worktree/branch state, and GitHub Actions state. Branch from live `main` when no continuation branch exists. If the exact branch or Draft Pull Request exists, continue it idempotently, preserving valid unrelated work. If the exact Pull Request is already merged or closed as completed, stop rather than duplicate it.

The Draft Pull Request body is the cross-conversation work ledger. It must record baseline, branch, current head SHA, migration decision and hash, completed/incomplete checkpoints, exact failures/blockers, workflow links/states, evidence artifact IDs/digests, risks, confirmation that PHASE 10 was not started, and confirmation that the Pull Request was not merged.

## 2. Scope

Implement all five bounded workstreams:

1. Versioned structured document templates.
2. Historical document rendering, local preview, A4 PDF generation, Windows printing, and immutable historical reprint.
3. Typed operational reports with safe CSV/PDF export.
4. Permission-controlled read-only audit-log presentation and redacted export.
5. Manual and automatic SQLite backup with verification, retention, import, destructive restore safeguards, rollback, and session invalidation.

All PHASE 09 work remains Windows-first, offline, local to the customer device, company-scoped, permission-controlled, fixed-point, Arabic-first/RTL, French-capable/LTR, and compatible with accepted PHASE 01–08 architecture.

## 3. Explicit non-goals

Do not implement PHASE 10, installer production work, signing, application version `1.0.0`, cloud synchronization, remote backup, email delivery, telemetry, analytics, licensing/subscriptions, online accounts, external database servers, HTTP APIs, arbitrary plugins, arbitrary SQL report builders, arbitrary JavaScript/templates, user-authored raw HTML/CSS, XLSX export, thermal/80 mm layouts, barcode-label printing, encrypted backups, homemade encryption, OCR, global navigation redesign, or unrelated refactoring of PHASE 05–08. A4 is the v1 document/report boundary.

## 4. Database contract

Create exactly one additive ordered migration:

`database/migrations/0007_phase09_documents_reports_audit_backup.sql`

Migrations `0001`–`0006` are immutable. Migration `0007` may add minimally sufficient PHASE 09 columns, protections, indexes, backup policy/settings, document-output metadata, permissions, default structured templates, and immutable-history triggers. It must not drop/rename accepted tables or columns, rewrite accepted business history, introduce floating-point business truth, weaken foreign keys/triggers, remove company scope/immutability, or silently repair invalid production data.

Regenerate `database/schema.sql`; update schema verification and invariants. Preserve frozen hashes for migrations `0001`–`0006`.

## 5. Permissions and authorization

Seed and enforce the established naming convention for:

- `documents.templates.view`
- `documents.templates.manage`
- `documents.render`
- `documents.print`
- `documents.export`
- `reports.view`
- `reports.export`
- `audit.view`
- `audit.export`
- `backup.view`
- `backup.create`
- `backup.restore`
- `backup.manage`

`backup.restore` and `audit.export` are sensitive. Template publishing requires `documents.templates.manage`. Every command must require an active session, derive company scope from that session, reject cross-company identifiers, enforce permissions in Rust, use safe errors, and audit sensitive operations. UI visibility is not authorization.

## 6. Structured template model

Users edit controlled fields only: display name; Arabic/French title and footer; visibility of logo/company/trade register/tax ID/partner address/payment information; spacing; supported orientation; and allowlisted optional sections. Rust converts typed configuration to safe local HTML/CSS. No raw HTML/CSS editor is permitted.

States are `DRAFT`, `PUBLISHED`, and `RETIRED`.

- Drafts use optimistic concurrency.
- Publishing validates the complete template.
- Version numbers are monotonic per company/document type/locale.
- Published versions are immutable and cannot be deleted.
- Retirement prevents new renders only and never changes history.
- Editing after publication creates a new draft/version.
- Published content receives a SHA-256 hash.
- Rendering fails closed without a valid published version.

Reject `<script>`, `<iframe>`, `<object>`, `<embed>`, inline event attributes, JavaScript URLs, remote HTTP/HTTPS resources, remote fonts/stylesheets, arbitrary local paths, unsanitized HTML, executable SVG, browser storage dependencies, and runtime network calls. Company logos are loaded by Rust from managed attachments and embedded as controlled local data.

Provide safe default Arabic and French A4 templates for:

- `SALES_ORDER`
- `DELIVERY_NOTE`
- `SALES_INVOICE`
- `SALES_CREDIT_NOTE`
- `PURCHASE_ORDER`
- `GOODS_RECEIPT`
- `SUPPLIER_INVOICE`
- `PURCHASE_RETURN`
- `CUSTOMER_RECEIPT`
- `SUPPLIER_PAYMENT`

Templates/rendering use accepted PHASE 06–08 source records and lineage; do not duplicate commercial/accounting truth in editable tables.

## 7. Historical rendering and artifact storage

The first finalized render creates an immutable record containing or referencing:

`company_id`, `document_type`, `source_document_id`, `source_document_number`, `source_document_status`, `template_id`, `template_version_id`, `locale`, `canonical_payload_json`, `rendered_html`, `rendered_css`, `content_sha256`, `pdf_relative_path`, `pdf_sha256`, `pdf_size_bytes`, `rendered_at`, and `rendered_by`.

The canonical payload stores the display values used at render time: company/partner identities, addresses, document number/dates, line descriptions, quantities, prices, discounts, taxes, HT/TVA/TTC, payment information, and relevant references. Reprint must never reconstruct from current mutable names or settings.

Reprint/re-export uses the original stored PDF artifact. Missing or hash-mismatched historical PDFs fail with a safe integrity error, preserve evidence, create an audit event, and are not silently regenerated or overwritten. A requested new render creates a separate immutable record.

Store generated PDFs below the application-owned document root using a deterministic hierarchy such as:

`documents/<company-id>/<document-type>/<year>/<month>/<render-id>.pdf`

Prevent traversal; sanitize components; write through temporary files; flush; hash; verify; atomically rename where supported; coordinate file/database success; remove temporary artifacts on failure; never overwrite; and never commit generated PDFs.

## 8. Windows output engine and preview security

Keep the pinned Tauri version. Inspect resolved Tauri/Wry/WebView2 dependencies before adding any direct Windows COM dependency and pin only an exact compatible version. Use a business abstraction comparable to:

```rust
trait DocumentOutputEngine {
    fn generate_pdf(&self, request: PdfOutputRequest) -> Result<PdfArtifact>;
    fn show_print_ui(&self, request: PrintOutputRequest) -> Result<()>;
}
```

Production Windows output uses Tauri platform webview access and `WebviewWindow::with_webview` with compatible WebView2 PDF/printing APIs. Serialize output jobs and return typed `OUTPUT_BUSY` for overlap. Use application-owned temporary paths; only move verified output to final storage after success. Platform code must not leak into business services.

Ubuntu must compile, run pure rendering/validation/UI preview tests, and return an explicit typed platform-unsupported result for native Windows PDF/printer operations. No fake Linux PDF implementation is allowed. PHASE 09 cannot be reported complete without legitimate Windows-native PDF/printing evidence.

The preview window loads local content only, forbids arbitrary navigation/remote URLs, uses existing or stricter CSP, receives only a render identifier, obtains snapshot content through a typed Rust command, escapes displayed text, sets `lang`/`dir`, disables unnecessary browser interactions, shows integrity state, and receives no general filesystem access.

## 9. Filesystem/dialog boundary

React receives no unrestricted filesystem commands or general filesystem API. Rust-side dialog integration chooses report/document export destinations, backup import files, and backup export destinations. Prefer official Rust-side integration without broad frontend dialog/filesystem permissions. Any capability is narrowly scoped to the necessary window and named commands only; no wildcard permissions or broad merging into the main window.

## 10. Reports

Implement typed report identifiers:

- `SALES_SUMMARY`
- `SALES_BY_PRODUCT`
- `SALES_BY_CUSTOMER`
- `PURCHASES_SUMMARY`
- `PURCHASES_BY_SUPPLIER`
- `STOCK_ON_HAND`
- `STOCK_VALUATION`
- `STOCK_MOVEMENTS`
- `LOW_STOCK`
- `OPEN_RECEIVABLES`
- `OPEN_PAYABLES`
- `CASH_BANK_REGISTER`
- `TRIAL_BALANCE`

Reuse accepted PHASE 06–08 business/query logic where available; do not duplicate accounting formulas. React cannot submit raw SQL. Typed requests contain report ID, applicable date/warehouse/partner/product/status filters, allowlisted sort field/direction, page, page size, and locale. Company scope is session-derived.

UI page size defaults to 50 and is capped at 200. CSV exports cap at 100,000 rows. PDF exports cap at 5,000 rows and must return a clear CSV-directed error when exceeded. Export incrementally/stream where practical.

CSV uses UTF-8 BOM, one documented delimiter, correct quoting, normalized line endings, fixed-point display formatting, metadata (report/filters/time/user where appropriate), disallowed-control-character removal, and spreadsheet-formula-injection prevention. Cells whose first non-whitespace character is `=`, `+`, `-`, or `@` must be neutralized. Do not export secrets, hashes, recovery material, credentials, or internal SQL.

## 11. Audit workspace

Implement read-only company-scoped, permission-controlled, Rust-paginated audit presentation with filters for start/end time, user, domain, action, entity type/ID, success/failure, and sensitive-event indicator. Default to a recent bounded period; cap page size at 200. CSV export requires `audit.export`. Audit rows are never updated/deleted.

Redact before serialization from Rust: `password`, `password_hash`, `token`, `token_hash`, `recovery_code`, `secret`, `credential`, `private_key`, and values identified by accepted audit conventions. The frontend must never receive hidden sensitive values.

## 12. Backup engine

Keep `rusqlite = "=0.32.1"` and enable its `backup` feature alongside `bundled`. Use SQLite Online Backup API. Never back up active WAL state by copying only the primary database; do not shell out to copy/cp/PowerShell/external SQLite as the primary mechanism.

Backup kinds:

- `MANUAL`
- `AUTOMATIC_DAILY`
- `AUTOMATIC_WEEKLY`
- `PRE_RESTORE`

Successful history records include/reference backup ID, company, kind, creation time/user, application/schema versions, migration-ledger digest, database size, SHA-256, relative path, integrity/foreign-key/verification states, and failure reason. Success requires reopening and verification with `PRAGMA integrity_check`, `PRAGMA foreign_key_check`, expected application metadata, migration continuity/checksums, supported schema, required tables, valid hash, and non-zero size.

When automatic backup is enabled, attempt after the first successful authenticated startup of each local calendar day; create at most one successful daily automatic backup per company/day; use `Africa/Algiers` and configured company timezone rules; do not block login indefinitely; warn and audit failures.

Retention:

- daily automatic: 7 successful backups;
- weekly automatic: 4 successful backups;
- manual: never auto-delete;
- pre-restore: latest 3 successful backups.

Never delete the last known valid backup, the backup selected for restore, or failed backups as retention candidates. Delete the database record only after successful file deletion or explicitly record deletion failure. Prevent traversal/deletion outside the backup root. Backups are local and unencrypted; encryption is outside PHASE 09.

## 13. Restore and maintenance gate

Restore requires active session, `backup.restore`, current-user password re-authentication, explicit confirmation, and exact typed text `RESTORE`.

Required sequence:

1. Acquire an application-wide exclusive maintenance gate.
2. Reject new business writes while restore is active.
3. Stage the selected backup under an application-owned directory.
4. Verify size and SHA-256.
5. Open the staged database separately.
6. Run integrity check.
7. Run foreign-key check.
8. Verify migration continuity/checksums.
9. Reject schemas newer than the running application.
10. Reject missing required migrations/tables.
11. Create and verify a `PRE_RESTORE` backup of the current database.
12. Ensure no active database operation remains.
13. Replace the active database with a safe same-filesystem temp/rename strategy.
14. Reopen through normal runtime initialization.
15. Rerun integrity, foreign-key, and migration verification.
16. Roll back from the verified pre-restore backup if post-replacement verification fails.
17. Audit the complete outcome.
18. Invalidate the prior frontend session.
19. Return to login/recovery state.

The active database is not replaced before steps 1–11 succeed. Abort if the safety backup cannot be created/verified. Never claim successful restore before reopen/revalidation succeeds.

The Rust-owned maintenance gate must allow current normal concurrency, give Online Backup the required access, give restore exclusive access, safely reject new commands during restore, avoid frontend booleans as locks, handle poisoning/failure, always release on error, and preserve accepted transaction/idempotency behavior.

## 14. Required module and command surface

Create an explicit bounded Rust module under `src-tauri/src/phase09/` with files comparable to `mod.rs`, `models.rs`, `permissions.rs`, `templates.rs`, `documents.rs`, `rendering.rs`, `output.rs`, `reports.rs`, `audit.rs`, `backup.rs`, and `restore.rs`. Create `src-tauri/src/commands/phase09.rs`; register commands explicitly in `src-tauri/src/lib.rs`; keep business logic out of handlers.

One-to-one typed behavior must be identifiable for:

### Templates
`phase09_list_templates`, `phase09_get_template`, `phase09_create_template_draft`, `phase09_update_template_draft`, `phase09_publish_template`, `phase09_retire_template`.

### Documents
`phase09_preview_document`, `phase09_render_document`, `phase09_list_rendered_documents`, `phase09_get_rendered_document`, `phase09_verify_rendered_document`, `phase09_export_rendered_pdf`, `phase09_print_rendered_document`.

### Reports
`phase09_list_reports`, `phase09_run_report`, `phase09_export_report_csv`, `phase09_export_report_pdf`.

### Audit
`phase09_list_audit_events`, `phase09_export_audit_csv`.

### Backup
`phase09_get_backup_settings`, `phase09_update_backup_settings`, `phase09_create_backup`, `phase09_list_backups`, `phase09_verify_backup`, `phase09_export_backup`, `phase09_import_backup`, `phase09_restore_backup`, `phase09_delete_backup`.

Deletion enforces retention/protected-backup rules. Use accepted safe-error/response conventions.

## 15. TypeScript gateway and UI

Create `src/platform/tauri/phase09/` with typed contracts and bounded template/document/report/audit/backup gateways. No SQL, `fetch`, `XMLHttpRequest`, `WebSocket`, remote URLs, or arbitrary filesystem authority. Validate responses, normalize safe errors, suppress stale responses, protect unmounted components, and preserve StrictMode.

Create `src/features/phase09/` with workspace sections: Documents, Templates, Reports, Audit, Backup and Restore. Arabic is default and French translations are mandatory. Support RTL/LTR, keyboard operation, visible focus, reduced motion, `1280×800`, `1024×640`, loading/empty/safe-failure/permission-denied states, progress, integrity warnings, and destructive confirmation. Integrate with existing navigation minimally; do not redesign the shell.

Template administration must support document type/locale selection, active published version inspection, draft creation from active version, structured editing, safe preview, explicit publishing confirmation, retirement of older active versions after replacement, version history, and hashes with clear Draft/Published/Retired distinction.

## 16. Testing and evidence

Add database/migration, Rust, gateway, UI, E2E, policy, and CI coverage for every applicable contract, including authorization/company isolation, safe errors, optimistic concurrency, immutable publishing/rendering, forbidden content/resources, canonical snapshots/fixed-point formatting, artifact atomicity/hashes/history, reports/limits/totals, CSV injection, audit redaction, WAL backup/verification/corruption/incompatibility, restore safety/rollback/session invalidation, retention/protected deletion/path traversal, maintenance gate, idempotent retry, RTL/LTR/accessibility/layout/error states, and exact command names.

Required named E2E scenarios:

- `phase09_ar_template_publish_and_historical_reprint`
- `phase09_fr_sales_invoice_preview_and_pdf`
- `phase09_ar_reports_csv_and_pdf`
- `phase09_fr_audit_filter_and_redacted_export`
- `phase09_ar_manual_backup_and_verification`
- `phase09_fr_corrupted_backup_rejected`
- `phase09_ar_restore_requires_verified_safety_backup`
- `phase09_fr_restore_success_returns_to_login`

Browser evidence records screenshots, console/page errors, Axe results, overflow/clipping, locale, viewport, and outcome. Windows-native evidence includes a real generated PDF and metadata with synthetic data only. Generated evidence artifacts are not committed.

Create `scripts/verify_phase09.py` and `scripts/phase09_policy.py`. Policy rejects changes to migrations `0001`–`0006`, runtime databases/WAL/SHM/journals, backups/PDFs/real exports/private documents/secrets/credentials/private keys/signing material/`.env`, external runtime URLs/assets, template JavaScript, unrestricted frontend filesystem access, write-capable workflows, out-of-scope files, and oversized artifacts while allowing only required integration files.

Create `.github/workflows/phase09-ci.yml` with read-only permissions and relevant path triggers. Run cross-platform schema/fresh/upgrade/invariant jobs, frontend type/build/UI/integration/E2E jobs, Ubuntu/Windows Rust fmt/check/clippy/test/desktop checks, Rust 1.85 MSRV, and a legitimate Windows output job for WebView2 PDF/printing behavior using synthetic data. If hosted Windows cannot execute native output, document the exact blocker and keep the phase incomplete rather than substituting a fake test. Upload synthetic evidence with 30-day retention.

## 17. Required validation commands

Before completion, execute locally or obtain successful GitHub Actions evidence for:

```text
python scripts/verify_schema.py
python scripts/verify_phase06.py
python scripts/verify_phase07.py
python scripts/verify_phase08.py
python scripts/verify_phase09.py
python scripts/phase09_policy.py
npm ci
npm run typecheck
npm run build
npm run test:ui
npm run test:integration
npm run test:e2e
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:check
git diff --check
git status --short --untracked-files=all
```

Classify each as locally executed, GitHub Actions executed, not executed, blocked, or failed. Never report a pass that was not observed.

## 18. Acceptance gates

A complete implementation candidate requires all of the following:

1. Additive migration `0007`; frozen `0001`–`0006`.
2. Safe Arabic/French defaults for all required types.
3. No arbitrary JS/HTML/CSS/remote asset/path authority.
4. Draft concurrency and published immutability.
5. Immutable historical snapshots and verified initial PDFs.
6. Original stored PDF used for reprint; missing/tampered artifacts fail closed.
7. Readable RTL Arabic and correct LTR French A4; multipage output without essential clipping.
8. Functional Windows print UI and legitimate Windows evidence.
9. Typed, company-scoped reports with accepted totals and export limits.
10. CSV formula-injection protection.
11. Read-only permission-controlled audit with Rust-side redaction.
12. Consistent WAL backup and reopening/verification of every successful backup.
13. Rejection of corrupt/incompatible backups.
14. Restore re-authentication and exact confirmation.
15. Verified pre-restore safety backup before replacement.
16. Tested rollback after failed post-restore verification.
17. Previous-session invalidation after restore.
18. Retention cannot remove the last valid/protected backup.
19. React has no unrestricted filesystem access and runtime remains offline.
20. Arabic/French, Axe, overflow, clipping, console, page-error, Ubuntu, Windows, Rust/Tauri, and required CI gates pass.
21. Architecture and candidate report are committed.
22. Pull Request remains Draft/unmerged; PHASE 10 remains unstarted.

## 19. Documentation and delivery boundaries

Create:

- `docs/architecture/phase-09-documents-reports-audit-backup.md`
- `docs/PHASE-09-REPORT.md`

The architecture document covers template/snapshot/output/report/audit/backup/restore/maintenance/capability/storage/failure/security/non-Windows behavior. The report must state exactly:

`Status: Candidate implementation in Draft Pull Request — not independently accepted`

Do not mark PHASE 09 accepted, change Continuity Checkpoint 08 to 09, or claim roadmap acceptance. Acceptance/continuity advancement belongs to the independent reviewer after merge.

Use small intentional commits where real changes exist:

- `docs(phase09): add active execution pack`
- `db(phase09): add documents reports audit and backup migration`
- `feat(phase09): add template and historical document services`
- `feat(phase09): add Windows PDF and printing output`
- `feat(phase09): add reports and audit workspace`
- `feat(phase09): add verified backup and restore`
- `test(phase09): add validation CI and evidence`
- `docs(phase09): add candidate delivery report`

Do not create empty commits, squash locally, force-push, merge, self-approve, or begin PHASE 10.

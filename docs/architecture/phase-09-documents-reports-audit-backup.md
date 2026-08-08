# PHASE 09 — Documents, Reports, Audit, and Backup Architecture

## Status and boundary

This document describes the PHASE 09 candidate implementation on `phase/09-documents-reports-audit-backup`. It is not an acceptance record. PHASE 09 remains Windows-first, offline, company-scoped, permission-controlled, Arabic-first with RTL, French-capable with LTR, and based on the local bundled SQLite database accepted through PHASE 08.

A4 is the only v1 document/report output boundary. Cloud synchronization, remote backup, email delivery, telemetry, arbitrary report SQL, raw user HTML/CSS/JavaScript, XLSX, thermal layouts, label printing, backup encryption, OCR, signing, installer production, and PHASE 10 are excluded.

## Structured template model

Users edit typed configuration only:

- bilingual document titles and footers;
- logo/company/trade-register/tax-identifier visibility;
- partner address and payment information visibility;
- normal or compact spacing;
- supported A4 orientation;
- allowlisted optional sections.

The storage model uses:

- `document_templates` for the company/document-type master;
- `document_template_versions` for immutable numbered published versions and SHA-256 content hashes;
- `phase09_template_drafts` for optimistic-concurrency drafts;
- `phase09_template_version_configs` for the canonical structured configuration of a published version;
- `phase09_template_retirements` for append-only retirement events.

Database locale values remain the migration-authorized `ar` and `fr`; the Rust/TypeScript boundary exposes `ar-DZ` and `fr-DZ`. Publishing converts structured configuration to canonical JSON, validates every field, assigns the next monotonic version, and hashes the complete typed content. Published rows are protected by migration triggers. Retirement requires another non-retired published version and never mutates historical output.

The renderer escapes every displayed value and emits local HTML/CSS with a restrictive CSP. It rejects executable tags, inline event handlers, JavaScript URLs, remote resources, browser storage dependencies, executable SVG, and arbitrary file paths. There is no raw HTML or CSS editor.

## Historical snapshot model

The render service reads accepted PHASE 06–08 source records and constructs a canonical display payload containing the values used at render time: company and partner identities, addresses, document number/status/dates, line descriptions, scaled quantities and prices, discounts, taxes, HT/TVA/TTC, payment information, references, and notes.

A finalized render stores an immutable row in `phase09_rendered_documents` containing:

- company, source document, type, number, and status;
- template and template-version identifiers;
- locale;
- canonical payload JSON;
- rendered HTML and CSS;
- content SHA-256;
- relative PDF path, PDF SHA-256, and size;
- rendering time and user.

Historical reprint and re-export read the originally stored PDF only. They do not reconstruct from current company, partner, product, or template data. A missing file, invalid PDF header, size mismatch, or digest mismatch returns a safe integrity error, records an audit failure, preserves the database row, and does not regenerate or overwrite evidence. A deliberately requested new render creates a separate immutable row and artifact.

## Local document storage

Generated PDFs are stored below the application-owned document root:

`documents/<company>/<document-type>/<year>/<month>/<render-id>.pdf`

Every component is sanitized. Absolute paths, traversal, backslashes, and non-normal path components are rejected. Output is written to an application-owned temporary path, flushed, verified for a `%PDF-` header, hashed, moved into final storage, and then recorded in SQLite as one coordinated success path. Existing historical artifacts are never overwritten. Generated PDFs are excluded from Git.

## PDF and printing boundary

Business services depend on a `DocumentOutputEngine` abstraction. The Windows implementation uses Tauri platform-webview access and the resolved WebView2 COM graph. Output jobs are serialized with a Rust-owned mutex and concurrent attempts receive `OUTPUT_BUSY`.

The Windows implementation:

1. opens a dedicated hidden local WebView2 window;
2. writes only the Rust-generated local snapshot;
3. applies the structured renderer CSS;
4. uses WebView2 PDF/print APIs;
5. verifies the generated PDF before final storage;
6. opens the system print UI for a previously verified historical artifact.

Non-Windows builds retain rendering, validation, gateway, and preview coverage but return `PLATFORM_UNSUPPORTED` for native PDF or printer operations. No fake Linux PDF generator is provided.

The final candidate still requires exact-head Windows-native evidence for Arabic, French, multi-page A4 output, original-artifact reprint, and print integration.

## Preview security

The preview window receives a render/preview identifier, not arbitrary frontend HTML. Snapshot content is held by the Rust service and returned through a typed command. The preview uses local application content only, sets `lang` and `dir`, escapes displayed values, forbids remote navigation/resources, and exposes an explicit integrity state. The frontend receives no general filesystem capability.

## Typed report engine

The report engine accepts one of thirteen fixed identifiers and typed filters only. Company scope is derived from the authenticated session. React cannot submit SQL, table names, columns, or arbitrary expressions. Each report maps to a fixed Rust-owned query and an allowlisted sort field/direction.

Report values are serialized as text, integer, boolean, or null. SQLite `REAL` and blob values are rejected at the report boundary so fixed-point business truth is not silently converted to floating point. UI pages default to 50 and are capped at 200. CSV exports are capped at 100,000 rows; PDF exports are capped at 5,000 and direct oversized requests to CSV.

CSV uses UTF-8 BOM, semicolon delimiters, CRLF line endings, correct quoting, fixed-point integer preservation, metadata rows, control-character filtering, and formula-injection neutralization for cells beginning after whitespace with `=`, `+`, `-`, or `@`.

## Audit presentation and redaction

The audit workspace is read-only and Rust-paginated. It defaults to a bounded recent period and supports time, user, domain, action, entity type/id, outcome, and sensitive-event filters. Company scope and `audit.view`/`audit.export` are enforced in Rust.

Before serialization or CSV writing, Rust recursively redacts keys matching accepted sensitive conventions, including password, password hash, token, token hash, recovery code, secret, credential, and private key. The frontend never receives an unredacted hidden value. Audit rows are never updated or deleted.

## Backup engine

`rusqlite = 0.32.1` uses the bundled SQLite and Online Backup features. The implementation does not copy the primary file of a running WAL database and does not shell out to operating-system copy commands or an external SQLite executable.

A backup is successful only after:

- Online Backup completes;
- the file is non-empty and SHA-256 is calculated;
- the backup is reopened read-only;
- `PRAGMA integrity_check` returns `ok`;
- `PRAGMA foreign_key_check` is empty;
- required POSMAN tables exist;
- every migration ID, version, name, and SHA-256 matches the embedded `0001`–`0007` catalog exactly;
- the schema is supported and not newer than the running application.

History records include kind, application/schema versions, migration digest, size, SHA-256, relative path, verification states, and failure information. Import stages a user-selected file inside an application-owned directory before verification. Export and import dialogs are Rust-owned; React receives only operation results, not general path authority.

Retention preserves seven successful daily automatic backups, four weekly backups, all manual backups, and the latest three pre-restore backups. Failed backups are not candidates. The last valid backup and a backup protected for restore cannot be deleted. File deletion occurs before history deletion; failure is recorded without losing the history row.

After a successful login, a detached bounded task checks the company-local calendar date (`Africa/Algiers`). It creates at most one verified daily backup per company/day and, on the configured weekday, at most one verified weekly backup. An in-process mutex prevents duplicate concurrent startup attempts. The login response is not blocked by backup I/O; failures are audited and persisted as a warning in backup settings. Interactive backup creation is restricted to `MANUAL`, so callers cannot forge scheduler-owned history kinds.

Backups are local and unencrypted. This is an explicit PHASE 09 security limitation; homemade encryption is prohibited.

## Validated restore sequence

Restore requires an active session, `backup.restore`, current-password re-authentication, an explicit confirmation, and the exact text `RESTORE`.

The sequence is:

1. verify the selected backup and its recorded digest;
2. acquire the application-wide exclusive maintenance gate;
3. stage and reverify the selected backup;
4. record the restore attempt;
5. create and verify a protected `PRE_RESTORE` Online Backup of the active database;
6. abort before replacement if the safety backup fails;
7. stage the incoming database on the active database filesystem;
8. remove stale SQLite sidecars only while the exclusive gate is held;
9. rename the active database aside and move the verified incoming file into place;
10. reopen and rerun integrity, foreign-key, required-table, migration, size, and digest verification;
11. restore the verified pre-restore artifact if post-replacement verification fails;
12. record success or rollback outcome and audit it;
13. invalidate the previous session and return the frontend to authentication/recovery state.

The active database is never replaced before selection verification, maintenance-gate acquisition, staging verification, and the verified safety backup complete.

## Maintenance gate

The Rust-owned maintenance gate tracks every ordinary database connection used by PHASE 05–09 and restore exclusivity. Normal operations can proceed under existing transaction rules while their connection-owned permit is alive. Backup uses ordinary guarded access compatible with SQLite Online Backup. Restore rejects new operations, waits for active operations to drain, and uses a narrowly scoped raw connection path only while its exclusive permit is held. Session invalidation occurs inside that exclusive interval. RAII releases every permit on success/error; no frontend boolean is authoritative.

## Tauri capability and filesystem boundary

PHASE 09 commands are registered explicitly. React uses typed gateways under `src/platform/tauri/phase09/`. It has no SQL, `fetch`, `XMLHttpRequest`, `WebSocket`, remote URL, or general filesystem API. Native save/open dialogs are implemented on the Rust side. Final workflow permissions are read-only.

## Failure behavior

The implementation fails closed for:

- missing published templates;
- stale template drafts;
- cross-company identifiers;
- unsupported locale/type/report/sort/filter values;
- concurrent output;
- oversized report exports;
- missing or modified historical PDFs;
- corrupt, truncated, incompatible, or incomplete backups;
- missing verified pre-restore safety backup;
- maintenance-mode conflicts;
- unsupported native output platforms.

Errors use typed safe codes and avoid exposing SQL, paths, credentials, hashes that are not intended for presentation, or hidden sensitive audit values.

## Remaining evidence boundary

This architecture is a candidate implementation description. Independent acceptance requires all exact-head database, Node 24, Rust 1.85/stable, Ubuntu/Windows, Tauri, WebView2, backup/restore runtime, browser accessibility/layout, and synthetic evidence jobs to complete successfully. The Draft Pull Request remains unmerged and PHASE 10 remains unstarted.

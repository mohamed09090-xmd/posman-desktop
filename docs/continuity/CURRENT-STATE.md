# POSMAN Continuity Checkpoint 03

> Snapshot verified on 2026-07-30 against the accepted product baseline. This file is a recovery aid, not a replacement for the live repository, merged Pull Requests, the Blueprint, or accepted architecture documents.

## 1. Recovery coordinates

| Item | Accepted value |
| --- | --- |
| Repository | `https://github.com/mohamed09090-xmd/posman-desktop` |
| Default branch | `main` |
| Last accepted product baseline | `f4cda85b24f9d69ebb0442c02f8a037da8ba9baf` |
| Latest accepted product phase | `PHASE 03 — Original UI Foundation` |
| Next candidate phase | `PHASE 04 — Frontend–Runtime Integration Gate` |
| PHASE 04 status | Not started and not authorized by this checkpoint |

Always fetch and verify the live `main` SHA, merged PRs, open PRs, and CI before relying on this snapshot. If the live repository conflicts with this file, stop and reconcile the difference from Git history and accepted review evidence.

## 2. Product intent

POSMAN is a Windows-first, offline, local-first desktop commercial-management application inspired by the scope of Sage 100 Gestion Commerciale without copying its interface. It is intended to install and run without a separately installed database server or cloud account.

The target product includes company setup, catalogue and families, customers and suppliers, initial stock, inventory movement tracking, purchases, direct sales, the order-to-delivery-to-invoice cycle, customizable business documents, and automatic accounting posting. Arabic (`ar-DZ`, RTL) is the default language and French (`fr-DZ`, LTR) is supported.

The authoritative product specification is [POSMAN Blueprint v1](../spec/POSMAN-Blueprint-v1.md).

## 3. Collaboration and authority contract

The normal collaboration model is:

- The user owns product decisions and acceptance.
- The primary AI acts as software architect, planner, prompt author, and independent reviewer.
- A separate implementation agent may execute an approved phase package and return an evidence-based report.
- The primary AI must not treat an implementer's report as proof by itself; it verifies the branch, diff, CI, artifacts, and scope.
- No phase is self-accepted. A phase becomes accepted only after explicit review and an authorized merge.
- The primary AI must not implement, push, merge, mark a PR ready, or start a later phase unless the user explicitly authorizes that action.
- Work uses small scoped branches and Draft Pull Requests. No force-push, history rewrite, direct commit to `main`, auto-merge, or unapproved merge.

Authority order when recovering context:

1. Live `main`, Git history, merged PR metadata, and completed CI evidence.
2. Repository instructions in [AGENTS.md](../../AGENTS.md).
3. The Blueprint and accepted architecture/phase reports.
4. This checkpoint and conversation summaries.
5. Unmerged branch reports or agent claims.

An active execution pack may narrow the scope further, but it cannot silently override accepted architecture, repository instructions, or user decisions.

## 4. Accepted delivery ledger

| Gate | PR | Accepted squash on `main` | Result |
| --- | ---: | --- | --- |
| PHASE 01 — SQLite Data Foundation | [#1](https://github.com/mohamed09090-xmd/posman-desktop/pull/1) | `0c72eb75eb5db916a51d1ee42fec47f21328ad28` | Accepted |
| Bootstrap Gate — Tauri Desktop Shell | [#2](https://github.com/mohamed09090-xmd/posman-desktop/pull/2) | `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9` | Accepted |
| PHASE 02 — Local Runtime Foundation | [#3](https://github.com/mohamed09090-xmd/posman-desktop/pull/3) | `7112e7f029a6419c7e58f89947f66ccad8bb69e4` | Accepted |
| PHASE 03 — Original UI Foundation | [#4](https://github.com/mohamed09090-xmd/posman-desktop/pull/4) | `f4cda85b24f9d69ebb0442c02f8a037da8ba9baf` | Accepted |

Supporting reports:

- [PHASE 01 report](../PHASE-01-REPORT.md)
- [Bootstrap Gate report](../BOOTSTRAP-GATE-02-REPORT.md)
- [PHASE 02 report](../PHASE-02-REPORT.md)
- [PHASE 03 report](../PHASE-03-REPORT.md)

No PHASE 04 implementation was accepted or started at the time of this checkpoint.

## 5. Implemented architecture

### Desktop shell

- Tauri 2 desktop shell with Rust, React, TypeScript, and Vite.
- Offline runtime with no HTTP client, cloud service, telemetry, sidecar, subscription, or external database.
- Explicit local CSP and a minimal Tauri capability associated with the main window.
- Windows/MSVC uses a shared embedded Common Controls v6 manifest while preserving Tauri application icons and version resources.

See [desktop shell architecture](../architecture/desktop-shell.md).

### Data foundation

- Four ordered SQLite migrations define 49 tables and 25 integrity triggers.
- SQLite is the authoritative local store.
- Monetary and quantitative values use fixed-point integers; application business columns do not use `REAL`.
- Business text identifiers are non-null, non-blank primary keys. Future application-created identifiers are intended to use UUIDv7 generated in Rust.
- `stock_movements` is the append-only inventory source of truth; `stock_balances` is a rebuildable projection.
- Posted commercial documents, posted journal entries, stock movements, rendered historical documents, and audit records are immutable.
- Corrections use returns, reversals, credit documents, or compensating entries.
- Accounting posting requires an open period, at least two lines, positive totals, and equal debit and credit.
- Accepted migrations must never be edited; corrections require a new ordered migration.

Fixed-point storage scales:

| Value | Integer scale |
| --- | ---: |
| Final money amounts | 2 decimal places |
| Unit prices and unit costs | 4 decimal places |
| Quantities | 6 decimal places |
| Percentage points | 4 decimal places; `19.0000%` is stored as `190000` |

See [database decisions](../architecture/database-decisions.md), [data dictionary](../architecture/data-dictionary.md), [ERD](../architecture/erd.md), and [accounting posting](../architecture/accounting-posting.md).

### Local runtime

- `rusqlite` with bundled SQLite; no database server, ORM, Tauri SQL plugin, or global `rusqlite::Connection`.
- Production data root is resolved from Tauri `local_data_dir()` under `POSMAN`.
- Runtime creates local `data`, `backups`, `documents`, `templates`, and `logs` directories and uses `data/posman.sqlite3`.
- Every connection enables foreign keys, sets a 5000 ms busy timeout, and requests WAL while reporting the actual journal mode.
- Migrations and deterministic seed data are embedded from the accepted `database/**` sources.
- Migration ledger validation checks a contiguous prefix and exact version, name, and SHA-256 values. Gaps, checksum changes, metadata mismatches, and databases newer than the application are rejected without destructive reset or downgrade.
- Readiness verifies the migration count, current schema version, table count, and `foreign_key_check`.
- Tauri initializes the runtime during `setup` and exposes only the read-only, sanitized `get_runtime_status` command.

See [runtime database architecture](../architecture/runtime-database.md) and [runtime command contracts](../architecture/runtime-command-contracts.md).

### UI foundation

- Original visual direction: **Contemporary Operations Ledger**.
- The interface is operations-led rather than a generic admin dashboard: numbered workspace rail, document canvas, process strip, status stamps, action dock, operational grids, and detail drawers.
- Arabic is the default and switches correctly between RTL Arabic and LTR French without reload.
- Typed dictionaries share identical keys and use `Intl` for DZD, dates, quantities, and numbers.
- Central CSS tokens and logical properties support both directions and desktop viewports.
- Accessibility evidence for the accepted representative screens reported zero Axe violations and zero incomplete checks, with keyboard-visible focus, semantic labels, reduced-motion support, and no page-level horizontal overflow.
- UI actions and data remain fixtures only. They do not claim persistence or business processing.
- No component framework, heavy router, state-management framework, or runtime network dependency was introduced.

See [UI foundation](../design/ui-foundation.md), [component inventory](../design/component-inventory.md), and [design direction study](../design/direction-study.md).

## 6. What is deliberately not implemented

The accepted baseline does **not** yet include:

- Frontend calls to Tauri commands.
- Frontend consumption of `get_runtime_status`.
- Real company setup, authentication, roles, or permissions.
- Catalogue, partner, purchasing, sales, stock, or accounting CRUD services.
- CUMP calculation, negative-stock policy enforcement, reservation consumption, business-total calculation, tax rounding, or accounting-rule selection.
- Document transformation services or enforcement of aggregate transformed quantity.
- PDF generation, printing, template designer, report engine, backup/restore workflows, installer, signing, packaged-release validation, cloud sync, telemetry, or online accounts.

Do not describe gallery fixtures, buttons, or sample documents as working business features.

## 7. Preserved invariants and deferred work

The following must remain explicit in future phase packages:

- The sum of quantities transformed from a source document line must not exceed the source quantity. This is the single known PHASE 01 invariant intentionally deferred to a Rust application transaction.
- Backup-before-future-migration and full backup/restore are documented but not implemented.
- Data-grid virtualization remains deferred until operational data volumes require it.
- IBM Plex licensing is present, but bundled WOFF2 font files and exact typography remain deferred; the accepted UI uses offline system fallbacks.
- Installer, signing, and packaged Windows validation remain outside the accepted baseline.

## 8. Next candidate: PHASE 04 integration gate

PHASE 04 is a planning candidate only. It must not begin without explicit user authorization and an approved execution package.

Its recommended narrow goal is to connect the accepted frontend shell to the accepted read-only runtime status safely:

- Add a typed frontend adapter around the Tauri invocation boundary.
- Consume only the existing `get_runtime_status` command.
- Represent initialization, ready, and sanitized failure states in the application shell.
- Preserve browser-based UI tests through a controlled mock and add real Tauri integration evidence.
- Keep the app offline and preserve Arabic/RTL, French/LTR, accessibility, and the Operations Ledger direction.
- Remove or clearly replace any demonstration-only runtime indicator that would conflict with the real status.

PHASE 04 should exclude company writes, authentication, general CRUD, inventory calculations, accounting posting, printing, backup/restore, installer work, and all other business modules.

Before authoring its execution pack, inspect the live source and define an ownership contract that protects accepted PHASE 01–03 files while identifying the smallest necessary shared integration points.

## 9. Recovery procedure

When continuing from a new account, lost conversation, or new AI session:

1. Open the repository and read [AGENTS.md](../../AGENTS.md).
2. Read this checkpoint and [RECOVERY-PROMPT.md](RECOVERY-PROMPT.md).
3. Verify the live `main` SHA and compare it with the accepted product baseline above.
4. Inspect merged and open PRs created after this checkpoint.
5. Read the Blueprint and all accepted reports/architecture documents relevant to the next task.
6. Run or inspect the repository's existing validation workflows before claiming the baseline is healthy.
7. Produce a short recovery report listing verified facts, differences since this snapshot, unresolved risks, and the proposed next action.
8. Do not modify code, open a branch, or start a phase until the user explicitly authorizes it.

If repository access is unavailable, ask the user to attach this file, `AGENTS.md`, the Blueprint, and the latest accepted phase reports. Do not reconstruct critical state from memory.

## 10. Checkpoint maintenance

After each newly accepted merge:

1. Verify the exact new `main` SHA, squash title, PR state, changed files, CI results, and phase boundaries.
2. Update the accepted delivery ledger and implemented/not-implemented sections.
3. Move the next-candidate section only after separating accepted facts from proposed scope.
4. Update `RECOVERY-PROMPT.md` if the role contract, recovery sequence, or authoritative files change.
5. Submit continuity changes in a docs-only Draft PR; do not mix them into business implementation.
6. Never claim acceptance from an unmerged branch or an implementer's report alone.


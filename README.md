# POSMAN

POSMAN is a Windows-first, offline desktop commercial-management application for Algerian merchants. Version 1 targets Windows 10/11 64-bit, Arabic-first operation with French support, local embedded SQLite, and DZD.

## Current repository status

The accepted product baseline includes PHASE 01–09 through commit:

```text
ebc256cd1503867558ebc9cdaac285066c8466d9
```

| Delivery | Status | Main capability |
| --- | --- | --- |
| PHASE 01 | Accepted | SQLite data foundation and invariants |
| Bootstrap Gate | Accepted | Tauri 2 desktop shell |
| PHASE 02 | Accepted | Local runtime and embedded migrations |
| PHASE 03 | Accepted | Arabic/French UI foundation |
| PHASE 04 | Accepted | Typed frontend/runtime integration |
| PHASE 05 | Accepted | Setup, authentication, users, permissions, catalogue, and partners |
| PHASE 06 | Accepted | Inventory, CUMP/CMUP, purchasing, reservations, and reconciliation |
| PHASE 07 | Accepted | Sales, delivery/invoice transformation, direct sale, and returns |
| PHASE 08 | Accepted | Accounting posting, payments, allocations, ledgers, and period controls |
| PHASE 09 | Accepted | Documents, PDF/printing, reports, audit presentation, and backup/restore |
| PHASE 10 | In delivery | Distribution, hardening, offline installer, and v1.0.0 release evidence |

POSMAN contains the complete approved business capability baseline. PHASE 10 is
now preparing and validating the offline Windows v1.0.0 distribution; it must
not be treated as a published production release until its Draft PR is accepted
and its final installer is signed under the documented release policy.

## Implemented product capabilities

The accepted source contains:

- atomic first-run company setup, fiscal configuration, users, roles, permissions, local login, sessions, inactivity lock, and Argon2id password handling;
- configurable products, families, units, taxes, prices, warehouses, locations, customers, suppliers, payment methods, and payment terms;
- append-only inventory movements, stock projections, opening stock, adjustments, transfers, counts, reservations, negative-stock controls, moving CUMP/CMUP, reconciliation, and rebuild;
- purchase orders, receipts, supplier invoices, direct receive-and-invoice, and purchase returns;
- sales orders, reservations, partial/full delivery, delivery-backed invoicing, direct sale, returns/credit documents, lineage, fixed-point totals, and below-cost policy;
- configurable chart of accounts, journals and posting rules, automatic source posting, manual journals, reversals, customer receipts, supplier payments, allocations, statements, trial balance, general ledger, account ledger, open balances, and fiscal-period controls;
- typed Tauri command gateways, company scoping, authorization, audit, idempotency, safe error normalization, Arabic RTL, French LTR, and permanent CI coverage.

See the phase reports under `docs/PHASE-01-REPORT.md` through
`docs/PHASE-09-REPORT.md` for detailed scope and validation evidence.

## Technology

- Tauri 2
- React 19
- TypeScript
- Vite
- Rust 1.85 minimum supported toolchain
- `rusqlite` with bundled SQLite
- Local/offline operation with no external database server

## Development prerequisites

Use Node.js 24 LTS with npm 11 and Rust 1.85 or newer. Windows development also requires Microsoft C++ Build Tools and WebView2 prerequisites required by Tauri.

## Development and validation commands

```bash
npm ci
npm run typecheck
npm run build
npm run test:ui
npm run test:integration
npm run test:e2e
python scripts/verify_schema.py
python scripts/verify_phase06.py
python scripts/verify_phase07.py
python scripts/verify_phase08.py
python scripts/verify_phase09.py
python scripts/verify_phase10.py
python scripts/phase10_performance.py
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked
npm run desktop:check
```

For interactive development:

```bash
npm run dev
npm run desktop:dev
```

`npm run desktop:check` compiles a debug Tauri application without producing a
published installer. On Windows, `npm run release:windows` creates the offline
NSIS bundle after all PHASE 10 gates have passed.

## Authoritative specification and continuity

The product specification is:

```text
docs/spec/POSMAN-Blueprint-v1.md
```

Before continuing project work, read:

```text
docs/continuity/PROJECT-MEMORY-INDEX.md
```

The continuity package records accepted phases, repository coordinates, architecture decisions, recovery procedure, and the next authorized boundary.

## Database source of truth

Ordered files in `database/migrations/` are authoritative. `database/schema.sql` is a generated review snapshot and must match those migrations exactly.

The accepted schema currently contains:

- seven ordered migrations through `0007`;
- 64 tables;
- 63 triggers;
- fixed-point integer storage for business truth;
- append-only and immutable-history protections.

Accepted migrations must never be edited. Corrections are roll-forward migrations.

Every SQLite connection must enforce foreign keys. The runtime also uses a bounded busy timeout and requests WAL mode.

## Fixed-point numeric rules

No application column uses SQLite `REAL` for business truth.

| Value | Storage | Scale |
| --- | --- | ---: |
| Final monetary amounts | `INTEGER` minor units | 2 |
| Unit prices and unit costs | `INTEGER` | 4 |
| Quantities | `INTEGER` | 6 |
| Percentage rates | `INTEGER` percentage points | 4 |

For percentage rates, `19.0000%` is stored as `190000`.

## Windows v1 distribution

PHASE 10 produces `POSMAN-Setup-Offline.exe` for Windows 10/11 x64. It embeds
WebView2, installs the application under Program Files, and preserves the local
database, backups, documents, templates, exports, and logs under
`%LOCALAPPDATA%\POSMAN` during upgrade and uninstall. See
`docs/release/WINDOWS-INSTALLATION.md` and
`docs/release/POSMAN-1.0.0-RELEASE-NOTES.md`.

Cloud synchronization, telemetry, subscriptions, an automatic updater, and
mandatory online activation remain outside the approved v1 boundary.

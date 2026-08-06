# POSMAN

POSMAN is a Windows-first, offline desktop commercial-management application for Algerian merchants. Version 1 targets Windows 10/11 64-bit, Arabic-first operation with French support, local embedded SQLite, and DZD.

## Current repository status

The accepted `main` baseline currently includes PHASE 01–08 through commit:

```text
5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69
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
| PHASE 09 | Not started | Documents, printing, reports, audit presentation, and backup/restore |
| PHASE 10 | Not started | Distribution, hardening, installer, and v1 release |

POSMAN is therefore a substantial working product baseline, but it is **not yet production-ready or distributable as v1.0.0** because PHASE 09 and PHASE 10 remain incomplete.

## Implemented product capabilities

The accepted source contains:

- atomic first-run company setup, fiscal configuration, users, roles, permissions, local login, sessions, inactivity lock, and Argon2id password handling;
- configurable products, families, units, taxes, prices, warehouses, locations, customers, suppliers, payment methods, and payment terms;
- append-only inventory movements, stock projections, opening stock, adjustments, transfers, counts, reservations, negative-stock controls, moving CUMP/CMUP, reconciliation, and rebuild;
- purchase orders, receipts, supplier invoices, direct receive-and-invoice, and purchase returns;
- sales orders, reservations, partial/full delivery, delivery-backed invoicing, direct sale, returns/credit documents, lineage, fixed-point totals, and below-cost policy;
- configurable chart of accounts, journals and posting rules, automatic source posting, manual journals, reversals, customer receipts, supplier payments, allocations, statements, trial balance, general ledger, account ledger, open balances, and fiscal-period controls;
- typed Tauri command gateways, company scoping, authorization, audit, idempotency, safe error normalization, Arabic RTL, French LTR, and permanent CI coverage.

See the phase reports under `docs/PHASE-01-REPORT.md` through `docs/PHASE-08-REPORT.md` for detailed scope and validation evidence.

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

`npm run desktop:check` compiles a debug Tauri application without producing a published installer.

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

- six ordered migrations through `0006`;
- 57 tables;
- 47 triggers;
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

## Remaining production work

The repository does not yet contain the completed PHASE 09/10 delivery:

- validated historical document rendering and PDF/printing;
- complete operational report/export workspace;
- audit-log presentation;
- safe manual/automatic backup and validated restore;
- production Windows installer, signing strategy, clean-machine upgrade/uninstall evidence, and v1 release artifacts.

Cloud synchronization, telemetry, subscriptions, and mandatory online activation remain outside the approved v1 boundary.

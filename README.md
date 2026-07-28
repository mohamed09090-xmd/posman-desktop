# POSMAN

POSMAN is a Windows-first, offline desktop commercial-management application for Algerian merchants. Version 1 targets Windows 10/11 64-bit, Arabic-first operation with French support, local embedded SQLite, and DZD.

## Phase 01 scope

PHASE 01 established the SQLite data foundation:

- ordered relational migrations with explicitly non-null, nonblank text business identifiers;
- deterministic safe reference seed data;
- a generated complete schema snapshot;
- commercial-document lineage for partial delivery and invoicing;
- append-only inventory, audit, and historical-document protections, including old/new parent checks for posted child-line updates;
- configurable accounting structures and balanced-posting protection;
- ERD, data dictionary, migration policy, and accounting-posting documentation;
- automated schema and invariant verification on Linux and Windows CI runners.

PHASE 01 did not implement application services, business workflows, PDF generation, installation, or operational UI.

## Desktop shell bootstrap

The Bootstrap Gate adds one shared Tauri 2 + React + TypeScript + Vite desktop-project structure before PHASE 02 and PHASE 03 run in parallel. It contains only a minimal Arabic/RTL render proof and native shell bootstrap. It does not contain runtime database services, business commands, authentication, reports, or a design system.

### Development prerequisites

Use Node.js 24 LTS with npm and Rust stable. Windows development also requires the current Microsoft C++ build tools and WebView2 prerequisites documented by Tauri. Linux CI installs the official Tauri 2 Debian/Ubuntu packages.

### Development commands

```bash
npm ci
npm run dev
npm run typecheck
npm run build
npm run desktop:dev
npm run desktop:check
```

`npm run desktop:check` compiles a debug Tauri application without producing a bundle or published installer.

## Authoritative specification

The preserved product specification is:

```text
docs/spec/POSMAN-Blueprint-v1.md
```

Domain behavior must remain consistent with that file.

## Verify the schema

Python standard library is sufficient:

```bash
python scripts/verify_schema.py
```

The script creates a temporary SQLite database, enables foreign keys, applies all migrations and seed data, verifies the expected 49 tables, rejects `REAL` declarations, inspects built-schema text-primary-key nullability, proves null/blank identifiers are rejected, proves draft lines cannot be reparented into posted commercial or journal parents, runs positive and negative fixture scenarios, executes `database/tests/invariants.sql`, and removes temporary output.

Regenerate the review snapshot after intentionally changing an unreleased migration:

```bash
python scripts/verify_schema.py --write-schema
python scripts/verify_schema.py
```

## Database source of truth

Ordered files in `database/migrations/` are authoritative. `database/schema.sql` is a generated, reviewable snapshot and must match those migrations exactly. Released migrations are immutable; corrections are roll-forward migrations.

Every future SQLite connection must execute:

```sql
PRAGMA foreign_keys = ON;
```

WAL is the preferred normal runtime mode for the local application database, but it is a per-database/runtime concern and is not assumed to be permanently configured by migrations.

## Fixed-point numeric rules

No application column uses SQLite `REAL`:

| Value | Storage | Scale |
|---|---|---:|
| Final monetary amounts | `INTEGER` minor units | 2 |
| Unit prices and unit costs | `INTEGER` | 4 |
| Quantities | `INTEGER` | 6 |
| Percentage rates | `INTEGER` percentage points | 4 |

For percentage rates, `19.0000%` is stored as `190000`. Future Rust services must use decimal arithmetic and explicit rounding rules.

## Current non-goals

This repository is not yet a finished or production-ready application. It does not contain operational authentication screens, runtime posting services, CUMP calculation services, stock-balance projection logic, PDF rendering, backups, installer/release output, cloud synchronization, telemetry, licensing, subscriptions, or real business data.

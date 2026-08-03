# PHASE 05 — Setup, Security, and Reference Data Architecture

## Scope

PHASE 05 establishes the first operational configuration boundary for POSMAN. It covers initial company setup, local authentication and authorization, company and fiscal settings, reference-data CRUD, fixed-point product pricing, document sequences, and the Arabic/French desktop workspace required to operate those capabilities.

The phase remains offline and Windows-first. React does not open SQLite, execute SQL, or access a network service. All persistence and authorization flow through typed Tauri commands into local Rust services backed by the bundled SQLite database.

Explicitly excluded are stock posting, purchasing, sales, payments, accounting posting, printing/PDF generation, backup/restore, installer work, cloud services, telemetry, HTTP APIs, and every PHASE 06 capability.

## Additive database boundary

Migration `0005_setup_security_reference_data.sql` is additive. Accepted migrations `0001`–`0004` remain byte-for-byte frozen. The verifier proves both paths:

- a fresh database applies ordered migrations `0001` through `0005`;
- a real database at accepted schema `0004` upgrades to `0005` without foreign-key violations.

The schema uses fixed-point integers for money and rates, local text identifiers for business entities, explicit foreign keys, row-version columns for optimistic concurrency, and append-only audit records. Company-scoped uniqueness prevents one company from colliding with another company's users, products, partners, references, or sequences.

## Atomic initial setup

Initial setup is a single immediate SQLite transaction. It creates the company, company settings, fiscal year, fiscal periods, local System Administrator, role assignments, recovery material, document sequences, and required default references. The transaction either commits the complete usable tenant or rolls back without leaving a partially configured installation.

A request ledger records an idempotency key and request hash. Repeating the same request returns the prior result; reusing a key with different content is rejected. Setup drafts contain only typed non-secret JSON and are constrained to one active draft.

## Local security model

Passwords are hashed with Argon2id. Plaintext passwords and recovery codes are never persisted. Authentication is local and creates bounded sessions tied to a company and user. The service enforces login, logout, inactivity timeout, active-user state, and session revocation.

Recovery uses a one-time code whose hash is stored locally. A used or revoked code cannot be reused, and rotation permits only one active recovery code per user.

Roles contain explicit permissions. System Administrator protection prevents removal or deactivation of the last active administrator. Authorization is checked in Rust before data access; the frontend cannot grant itself a permission by changing presentation state.

## Service boundary and error contract

`src-tauri/src/phase05` separates setup, authentication, users/roles, company configuration, partners, products/pricing, and simple reference data. Commands registered in `src-tauri/src/lib.rs` accept typed request DTOs and return typed views. SQL remains inside Rust services.

Every mutation applies:

- authenticated permission checks;
- company scoping;
- input validation and fixed-point bounds;
- optimistic concurrency through `row_version` where records are editable;
- audit creation in the same transaction;
- structured error codes.

The TypeScript gateway in `src/platform/tauri/phase05.ts` is the only frontend IPC boundary. It wraps request DTOs under the expected command argument, invokes no-argument commands without synthetic payloads, and maps unknown or sensitive backend failures to stable safe codes. Raw SQL, paths, stack traces, hashes, and secret material are not exposed to the UI.

## Company, fiscal, and sequence configuration

The company profile includes Algerian legal/contact fields, Wilaya and postal validation, social capital in minor units, default margin rate, configurable below-cost policy, and inactivity timeout.

The fiscal year produces deterministic monthly periods. Once operational data exists, the service rejects destructive fiscal-boundary changes. Document sequences are unique per company, fiscal year, and document type. Sequence numbers cannot decrease, and format changes are locked after allocation begins.

The company-settings update persists `default_margin_rate_scaled`, `below_cost_policy`, `session_idle_timeout_minutes`, audit metadata, and the incremented row version as one SQL statement. A regression test executes that statement on SQLite and verifies all values, including the below-cost policy.

## Reference data and partners

The phase supplies company-scoped CRUD for product families, units, taxes, payment terms, payment methods, warehouses, warehouse locations, customers, suppliers, partner addresses, and partner contacts. Customer and supplier roles share a partner identity while retaining explicit role flags. Primary/default address and contact constraints are enforced by the service and database contract.

No reference endpoint performs stock movement, commercial posting, or accounting work.

## Products and fixed-point pricing

Products use integer fixed-point purchase and sale prices. The service derives the suggested sale price from purchase cost and the configured margin rate. It classifies prices as below cost, zero margin, or acceptable without floating-point arithmetic.

Below-cost behavior is configurable through the company policy (`BLOCK`, `ADMIN_OVERRIDE`, or `WARNING_ONLY`). The backend remains authoritative: presentation warnings do not bypass policy checks. The UI displays the warning state and suggested price in localized DZD formatting.

## Arabic/French operational UI

Arabic (`ar-DZ`) is the default locale with `dir="rtl"`; French (`fr-DZ`) uses LTR. The PHASE 05 workspace is connected to the typed Tauri gateway for setup, login, company configuration, security administration, partners, products, and references. Browser test fixtures replace only the invoker during deterministic E2E execution; production desktop code resolves the real Tauri gateway.

The interface avoids generic dashboard patterns and keeps navigation, tables, forms, warnings, and operational feedback compact. Keyboard-focus visibility, native controls, semantic labels, reduced-motion support, responsive layout, no horizontal overflow, and no clipping of primary text are validation requirements.

## Accessibility evidence

The E2E harness executes Arabic and French at `1280×800` and `1024×640`. It records screenshots, full Axe JSON, console/page errors, overflow measurements, and clipping checks.

A detected Axe `color-contrast` violation affected the first product-table header: foreground `#5f6965` on background `#e7e3d7` measured `4.42:1`. The source color was changed to `#56605c`, producing approximately `5.08:1`. No Axe rule, impact level, threshold, or assertion was disabled or weakened.

When Axe fails, diagnostics include rule ID, impact, help text, help URL, CSS targets, a bounded HTML snippet, and the failure summary. Evidence upload uses `if: always()` so a failing run retains its reports.

## CI and evidence policy

The permanent `.github/workflows/phase05-ci.yml` has only `contents: read`. It validates:

- schema and migrations on Ubuntu and Windows;
- secret and runtime-database-artifact scanning;
- TypeScript, production build, UI and integration tests;
- Arabic/French Playwright and Axe evidence;
- Rust formatting, locked compilation, Clippy with warnings denied, and tests on Rust 1.85;
- native Tauri compilation and the Windows embedded manifest;
- clean worktrees and absence of write-capable helpers, payloads, or transport files.

Compatibility workflows for the desktop shell, runtime, UI, and frontend-runtime integration are reconciled to accept the additive PHASE 05 ownership boundary while retaining their previous checks. No permanent workflow may commit or push.

## Architectural decisions

1. Keep SQLite and every business invariant inside the Rust desktop process; React remains a typed client of local commands.
2. Make initial setup transactional and idempotent rather than a sequence of independently committed screens.
3. Use Argon2id and one-time hashed recovery material for local credentials.
4. Enforce company scope, permission checks, audit, and optimistic concurrency in the service layer for every mutation.
5. Use integer fixed-point pricing and a persisted configurable below-cost policy.
6. Preserve Arabic RTL as the default while treating French LTR as an equal operational path.
7. Treat screenshots and full accessibility JSON as build evidence, not manual-only artifacts.
8. Keep all PHASE 06 domains outside this phase.

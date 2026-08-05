# PHASE 06 REPORT — Inventory and Purchasing

## Delivery identity

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Accepted baseline: `ccf2263104455681cc07ecceda2569c4f7ce0de9`
- Baseline title: `feat(setup): establish POSMAN setup security and reference data (Phase 05)`
- Branch: `phase/06-inventory-purchasing`
- Draft PR title: `[Phase 06] POSMAN inventory and purchasing`
- Merge state required at handoff: open, Draft, and unmerged
- Migration decision: no `0006`; migrations `0001`–`0005` remain frozen

## Implemented scope

The phase implements the inventory ledger and projection, moving CUMP, opening stock, adjustment, transfer, physical count and stale-snapshot protection, reservation lifecycle, negative-stock policy, purchase orders, partial/full receipts, receipt-backed invoices, atomic direct receive-and-invoice, purchase returns, balance reconciliation/rebuild, authenticated typed Tauri IPC, TypeScript gateway, Arabic/French operations UI, database/Rust/TypeScript/integration/E2E/Axe validation, permanent CI, and architecture documentation.

The implementation is source-visible in the PR. No chunks, Base64 payload source, archive, transport workflow, temporary write workflow, external database, HTTP server, cloud service, or telemetry is used.

## Architecture decisions

1. `stock_movements` is the append-only source of truth; `stock_balances` is rebuildable.
2. Warehouse CUMP is shared by aggregate and location projection rows.
3. Every posting uses one SQLite `IMMEDIATE` transaction.
4. Idempotency is company-scoped and binds a key to a stable SHA-256 request hash.
5. Fixed-point values use quantity scale 6, price/cost scale 4, DZD scale 2, percentage-points scale 4, checked integers, `i128`, and deterministic half-up division.
6. Negative stock requires policy `PRIVILEGED_OVERRIDE`, sensitive permission, and a non-empty audited reason.
7. Purchase aggregate transformation is checked transactionally for order→receipt, receipt→invoice, and document→return.
8. No accepted migration was modified and no migration was added.
9. Phase 05 authentication, company scope, permissions, audit, and safe-error conventions are reused rather than duplicated.
10. React uses one typed Tauri gateway and integrates PHASE 06 inside the authenticated PHASE 05 workspace; no alternate route or authentication bypass is introduced.

## Document workflows

- Opening: `DRAFT → REVIEWED → POSTED`.
- Transfer: paired `TRANSFER_OUT + TRANSFER_IN`, one group, one transaction.
- Adjustment: reason required; positive and negative cost/policy rules applied.
- Count: snapshot → counted quantity → review → stale check → variance posting.
- Reservation: active → partial/full release or consume → terminal state; no on-hand change on reservation.
- Purchase order: draft/edit → confirmed/on-hold/cancelled/completed.
- Receipt: optional confirmed order source → partial/full links → posted movements/CUMP.
- Invoice: one/many compatible receipt lines → quantity cap → posted without duplicate stock.
- Direct invoice: internal receipt + invoice + links + movements atomically.
- Return: posted source + aggregate cap → outbound current-CUMP movement; original unchanged.

## Exact typed Tauri commands

`list_stock_balances`, `list_stock_movements`, `create_opening_stock`, `review_opening_stock`, `post_opening_stock`, `post_stock_adjustment`, `post_stock_transfer`, `create_inventory_count`, `update_inventory_count`, `review_inventory_count`, `post_inventory_count`, `get_inventory_count`, `create_stock_reservation`, `release_stock_reservation`, `consume_stock_reservation`, `cancel_stock_reservation`, `list_active_stock_reservations`, `reconcile_stock_balances`, `rebuild_stock_balances`, `create_purchase_order`, `update_purchase_order`, `confirm_purchase_order`, `cancel_purchase_order`, `hold_purchase_order`, `create_purchase_receipt`, `post_purchase_receipt`, `create_purchase_invoice`, `post_purchase_invoice`, `direct_receive_and_invoice`, `post_purchase_return`, `list_purchasing_documents`, and `get_purchasing_document`.

## Local validation record

| Command | Result | Exit code |
|---|---:|---:|
| `python scripts/verify_schema.py` | PASS — 5 migrations, schema `0005`, 52 tables, 25 triggers, 147 checks, one PHASE 07 pending invariant | 0 |
| `python scripts/verify_phase06.py` | PASS — baseline, frozen migrations, 12 permissions, 32 commands, and three purchase transformation paths verified | 0 |
| `npm run test:ui` | PASS — 7 tests | 0 |
| `node --experimental-strip-types --test tests/integration/phase06-request-gate.test.ts` | PASS — 3 tests | 0 |
| `python -m py_compile tests/e2e/run_phase06.py` | PASS | 0 |
| `git diff --check` | PASS | 0 |

The local execution image does not provide Rust and cannot install the repository's locked Node 24/npm 11 dependency set from its internal registry. Therefore local Rust compilation, TypeScript typecheck/build, the gateway integration suite, native Tauri validation, and browser E2E/Axe are not claimed locally. The permanent GitHub Actions workflow runs those commands on Ubuntu and Windows with Rust 1.85 and Node 24; final-head results replace this provisional record in the Draft PR description after completion.

## Evidence and final-head rule

The permanent workflow emits:

- `phase-06-ui-evidence`: screenshots, complete Axe JSON, and SHA-256 manifest.
- `phase-06-integration-evidence`: database, frontend, Rust Ubuntu/Windows logs, final head, run URL, size, and SHA-256 manifest.

Artifact IDs, sizes, digests, run URLs, final SHA, and per-job conclusions belong in the Draft PR description after the final unified-head run. They are not added through a documentation-only commit after CI.

## Permissions

The phase provisions `stock.read`, `stock.opening.post`, `stock.adjust`, `stock.transfer`, `stock.count`, `stock.reservation.manage`, `stock.reconcile`, `stock.negative.override`, `purchase_order.confirm`, `purchase_receipt.post`, `purchase_invoice.post`, and `purchase_return.post`. Existing companies receive system-template availability without setup repetition. Custom roles receive no automatic sensitive grants.

## Risks and limitations

- Final acceptance remains the external architect/reviewer’s decision.
- SQLite write serialization is intentional; busy contention must be retried through normalized errors.
- Sales-side reservations are engine-only; the sales UI and sales aggregate transformation are PHASE 07.
- Accounting price-variance, journals, credits, and payments are PHASE 08 or later.
- No phase completion claim is valid until all required final-head CI jobs and final artifacts are green and present.

## Scope confirmations

- PHASE 07 was not started.
- No sales workflow, accounting posting, payment, PDF/printing, reporting engine, backup, installer, updater, cloud, telemetry, or HTTP API was implemented.
- The PR must remain Draft, open, and unmerged.
- No force-push, rebase, history rewrite, merge, or direct main commit is authorized or performed.

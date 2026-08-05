# PHASE 06 — Inventory and Purchasing Architecture

## Scope and baseline

PHASE 06 is implemented on the accepted PHASE 05 baseline `ccf2263104455681cc07ecceda2569c4f7ce0de9`. It extends the authenticated, company-scoped SQLite/Tauri desktop runtime. It does not introduce a server, HTTP API, cloud database, telemetry, online account, or external database. Accepted migrations `0001` through `0005` remain byte-for-byte frozen; no migration `0006` is required because the accepted schema already contains the inventory, count, reservation, commercial-document, transformation-link, audit, and idempotency structures required by this phase.

## Source of truth and projection

`stock_movements` is the sole inventory source of truth. Database triggers reject updates and deletes, and every posting event has a unique company-scoped posting key. Corrections are new compensating movements and never edits to history.

`stock_balances` is a disposable projection. For every `company + product + warehouse`, the runtime maintains:

1. one aggregate row with `warehouse_location_id IS NULL`; and
2. zero or more location rows containing the location quantity.

A location movement changes its location row and the warehouse aggregate inside the same `BEGIN IMMEDIATE` transaction. Every location row carries the current warehouse CUMP. Reserved quantity is projected separately; available stock is always `on_hand - reserved`.

The reconciliation service replays the ledger deterministically, compares rebuilt rows with the stored projection, reports every mismatch, and can replace the projection only for a caller holding `stock.reconcile`. Rebuild never changes the ledger.

## Fixed-point model

Business arithmetic never uses SQLite `REAL`, Rust `f32`/`f64`, or JavaScript business arithmetic in posting DTOs.

| Measure | Stored scale |
|---|---:|
| Quantity | 6 |
| Unit price / unit cost | 4 |
| Final DZD amount | 2 |
| Percentage points | 4 |

Rust uses checked integer operations and `i128` intermediates. Division uses deterministic half-up rounding. Extended cost converts `quantity(scale 6) × unit cost(scale 4)` to DZD minor units `(scale 2)` by dividing by `10^8` with the same half-up rule.

## Moving weighted average (CUMP / CMUP)

The costing scope is `company + product + warehouse`; location is not a costing dimension.

For an inbound cost-bearing movement when the old quantity is positive:

```text
new_average = round_half_up(
  (old_quantity × old_average + received_quantity × receipt_cost)
  / (old_quantity + received_quantity)
)
```

Example: 10.000000 units at 1.0000 DZD plus 5.000000 units at 1.6000 DZD produces 15.000000 units at 1.2000 DZD. Stored integers are `10_000_000`, `10_000`, `5_000_000`, `16_000`, and the resulting CUMP is `12_000`.

Rules:

- `OPENING`, `PURCHASE_RECEIPT`, `ADJUSTMENT_IN`, cross-warehouse `TRANSFER_IN`, and positive `COUNT_VARIANCE` may recalculate CUMP.
- Outbound movements use the current CUMP and never revalue earlier movements.
- `PURCHASE_RETURN`, `ADJUSTMENT_OUT`, and `TRANSFER_OUT` use the current source CUMP.
- Cross-warehouse transfer carries the source CUMP into the destination calculation.
- A transfer between locations of the same warehouse changes quantities only and preserves warehouse CUMP.
- At zero quantity, the last CUMP is retained.
- During an authorized negative balance, the last CUMP is retained.
- When a zero/negative balance becomes positive, the inbound lot cost becomes the deterministic new CUMP.

## Posting transaction and idempotency

Every stock-affecting posting command executes inside one SQLite `IMMEDIATE` transaction and performs, in order:

1. active session validation;
2. actor, company, and permission derivation from the PHASE 05 session;
3. request validation and stable SHA-256 request hashing;
4. idempotency lookup/insert;
5. policy and aggregate-transformation checks;
6. document, snapshot-line, and link creation where applicable;
7. movement insertion;
8. projection update;
9. workflow/status update;
10. audit insertion; and
11. idempotency completion and commit.

A repeated key with the same request hash returns the previous result. The same key with a different request hash returns `IDEMPOTENCY_CONFLICT`. A rollback removes all transaction work, including the in-progress idempotency row.

## Negative-stock policy

The default policy is `BLOCK`. A negative result is allowed only when all conditions are true:

1. `company_settings.negative_stock_policy = 'PRIVILEGED_OVERRIDE'`;
2. the current user holds sensitive permission `stock.negative.override`; and
3. the request includes a non-empty reason.

The reason is retained in the document or movement and audit details. The override does not bypass company scope, permission checks, idempotency, append-only history, or audit. Reservations never exceed available stock, regardless of override permission.

## Opening stock

Opening stock follows `DRAFT → REVIEWED → POSTED`:

- the draft stores warehouse, optional location in line snapshot metadata, commercial date, product snapshot, quantity, and cost;
- review uses optimistic concurrency;
- posting creates an `OPENING_STOCK` commercial document and `OPENING` movements;
- posting is idempotent and audited;
- a product/warehouse with prior ledger activity rejects opening posting; later corrections use adjustments; and
- a posted opening document is immutable.

## Transfers, adjustments, and counts

A stock transfer is one `STOCK_TRANSFER` document and a paired `TRANSFER_OUT` / `TRANSFER_IN` set sharing one `transfer_group_id`. Both sides are created in one transaction. The source and destination may not be identical.

A stock adjustment requires a reason. Positive adjustments use explicit cost when no current CUMP exists; otherwise the current CUMP is the default. Negative adjustments use the current CUMP and apply the negative-stock policy.

An inventory count snapshots system quantity when the draft is created. Counted quantity and variance are stored per product/location. Review precedes posting. Posting rejects a stale snapshot when the current ledger-derived balance differs from the snapshot and creates `COUNT_VARIANCE` movements only for non-zero differences. Posted counts are immutable.

## Reservation lifecycle

The reservation engine is deliberately independent of the PHASE 07 sales UI. It supports:

```text
ACTIVE → PARTIALLY_CONSUMED → CONSUMED
ACTIVE → PARTIALLY_RELEASED → RELEASED
ACTIVE → CANCELLED
```

Creation, partial release, partial consume, full consume, cancellation, and active queries are company scoped, permission protected, idempotent, and row-versioned. Consumption records the outbound movement identifier supplied by the future sales workflow. Reconciliation rebuilds reservation totals from active reservation state.

## Purchasing workflows

### Purchase order

`PURCHASE_ORDER` starts as `DRAFT`, is editable with optimistic concurrency, and may transition to `CONFIRMED`, `ON_HOLD`, or `CANCELLED`. The partner must be active and `is_supplier = 1`. The order has no stock effect. Lines store product, unit, tax, quantity, price, cost, discount, HT, tax, and TTC snapshots. Document numbers come from `document_sequences`.

### Purchase receipt

A `PURCHASE_RECEIPT` may reference a confirmed/on-hold purchase order. Each target line links to its order line through `PURCHASE_ORDER_TO_RECEIPT`. Within the same Rust transaction, the service sums existing transformations and rejects any quantity above the order-line quantity. Posting creates one `PURCHASE_RECEIPT` movement per line, updates CUMP once, and is idempotent. The order remains confirmed while partially received and becomes completed when all lines are fully received.

### Purchase invoice

A `PURCHASE_INVOICE` line links to a posted receipt line through `RECEIPT_TO_INVOICE`. Aggregate invoiced quantity may not exceed received quantity. A receipt-backed invoice does not create stock movements. Price differences are preserved in line metadata for PHASE 08 and do not rewrite receipt cost, CUMP, or historical movements. Effective tax data is snapshotted; HT, tax, discounts, and TTC use fixed-point arithmetic. No journal, payment, or supplier settlement is created.

### Direct receive and invoice

Direct supplier invoicing creates an internal receipt, invoice, line links, receipt stock movements, document statuses, audit records, and idempotency result in one `IMMEDIATE` transaction. Any error rolls back every object. Retry returns the existing invoice and does not duplicate receipt, invoice, link, or movement.

### Purchase return

A `PURCHASE_RETURN` references a posted purchase receipt or invoice. Each line links to its source through `DOCUMENT_TO_RETURN`; aggregate returned quantity is capped within the transaction. The commercial line preserves the original document price. The outbound movement uses current CUMP and applies negative-stock policy. The source document is never edited. Accounting credit entries remain PHASE 08 scope.

Purchase-side aggregate transformation enforcement is complete in PHASE 06. Sales-side aggregate transformation remains explicitly pending PHASE 07.

## Permissions and existing companies

PHASE 06 provisions missing permissions idempotently at runtime so companies created before this phase do not require setup repetition. System templates receive only their intended grants:

- `OWNER` / system administrator: all PHASE 06 permissions;
- `STOCK`: stock read, opening, adjustment, transfer, count, reservations, reconciliation as approved, without security administration;
- `PURCHASING`: purchasing confirmation/posting/return and stock read;
- `AUDITOR`: read-only stock and document visibility.

Custom roles are not automatically granted sensitive permissions. Sensitive actions include opening posting, adjustments, count posting, rebuild, negative override, receipt posting, invoice posting, and return posting.

## Typed Tauri IPC

`src-tauri/src/commands/phase06.rs` exposes typed DTO commands and maps domain failures to safe normalized errors. React receives no SQL, database path, stack trace, password/session hash, or filesystem path. `src/platform/tauri/phase06.ts` is the sole PHASE 06 frontend gateway. The test invoker is guarded by `import.meta.env.DEV`; CI scans the production bundle and fails if the hook remains.

## UI behavior

The workspace follows the **Contemporary Operations Ledger** identity: a masthead, numbered process rail, document-centric work surfaces, ruled tables, restrained status treatment, and no KPI-card grid, bento layout, decorative gradient, glass effect, or generic admin sidebar.

Arabic `ar-DZ` is the default with RTL. French `fr-DZ` uses LTR. Dictionaries are parity-tested. Quantity, DZD, cost, and dates use `Intl`. Every screen has loading, empty, safe error, and success feedback. Posting uses native confirmation, posted documents are identified as locked, keyboard focus is explicit, reduced-motion preference is honored, page-level horizontal overflow is prohibited, and wide tables scroll only inside their container.

## Test strategy

- SQLite verifier: accepted migration hashes, fresh schema snapshot, idempotent seed, foreign keys, no `REAL`, invariants, permissions, append-only movement, posted immutability, duplicate posting event, and purchase-side transformation evidence.
- Rust: fixed-point boundaries, opening/CUMP, negative policy, transfer pair/costing, location transfer, reservations, idempotency conflict/replay, corruption detection, rebuild, immutable ledger, safe errors, document concurrency, purchasing transformations, direct atomic workflow, and Tauri IPC tests.
- TypeScript: DTO validation, exact command names/envelopes, safe error normalization, formatting, dictionary parity, loading/retry/unmount behavior, and production gateway detection.
- E2E: Arabic/French stock overview, opening posting, order/partial receipt, direct receipt/invoice, transfer, count, negative block, override warning, supplier return, reconciliation/rebuild, 1280×800 and 1024×640, Axe JSON, screenshots, console/page-error checks, overflow checks, and clipped-primary-text checks.
- CI: Ubuntu and Windows database and Rust jobs, Rust 1.85, Node 24 frontend, native Tauri check, Windows manifest check, policy/ownership/frozen-migration checks, and final evidence artifacts.

## Explicit exclusions

PHASE 07 and later are not started. No sales-order UI, delivery posting, sales invoice/return, customer/supplier payment, journal posting, automatic accounting entry, PDF/printing, report engine, backup/restore, installer, updater, cloud, telemetry, or HTTP API is included.

## Known risks

- SQLite permits only one writer; `IMMEDIATE` transactions intentionally serialize posting and may surface a retryable busy result under contention.
- Rebuild cost fidelity depends on complete immutable movement cost snapshots; verifier and reconciliation tests protect this invariant.
- Direct invoice atomicity increases transaction breadth; all reference validation is completed before movement insertion where possible.
- PHASE 07 must consume reservations with the exact movement identifier and retain aggregate sales-transformation checks in its own transaction.

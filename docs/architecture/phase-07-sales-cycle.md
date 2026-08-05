# PHASE 07 — Sales Cycle Architecture

## Scope and accepted baseline

PHASE 07 starts from accepted PHASE 06 commit `036ac89c07ddee1e26402c1c523529adbba48860`. It adds the local sales cycle while preserving the offline Tauri + SQLite boundary. Accepted migrations `0001` through `0005` remain byte-for-byte frozen. No migration `0006` is required because the existing schema already contains commercial documents and lines, document lineage, stock reservations and movements, immutable posting history, audit records, document sequences, tax snapshots, idempotency, customers, warehouses, and product pricing.

The phase does not create payments, journals, accounting posting, PDF/printing, reports, backup/restore, cloud services, telemetry, or an HTTP API.

## Sales document lifecycle

The implemented document graph is:

```text
SALES_ORDER → DELIVERY_NOTE → SALES_INVOICE
                             ↘ SALES_RETURN → CREDIT_NOTE
```

A sales order is editable only while it is a draft. Confirmation validates the customer, price policy, stock availability, and creates reservations. Hold and resume retain reservations. Cancellation releases remaining reservations transactionally. Reservations have no automatic expiry; release is an explicit business action so an accepted customer commitment cannot silently disappear.

A delivery may transform only confirmed or partially delivered order quantities. A delivery consumes its matching reservations and posts outbound movements in the same `IMMEDIATE` transaction. Partial deliveries update the order to `PARTIALLY_DELIVERED`; full completion marks it delivered.

An invoice may transform only posted, not-yet-fully-invoiced delivery quantities. It has no additional stock effect. A direct sale creates an internal delivery, stock movements, invoice, links, status history, audit entries, and idempotency result atomically. Any error rolls back the complete flow.

A sales return never edits the original sale. It creates a return document, an inbound stock movement, a lineage link to the source, and a credit note in one transaction. Financial settlement and journal entries remain PHASE 08.

## Aggregate transformation invariant

Each transformation line is checked inside the same write transaction:

```text
SUM(existing sibling transformed quantity) + requested quantity <= source quantity
```

The rule is enforced for `ORDER_TO_DELIVERY`, `DELIVERY_TO_INVOICE`, and `DOCUMENT_TO_RETURN`. The regression fixture proves an order of 20 units accepts deliveries of 8 and 12 and rejects a further quantity of 1. This closes the sales-side application invariant that remained pending after PHASE 06.

## Fixed-point pricing

All quantities, prices, costs, rates, HT, tax, and TTC use integer fixed-point values. Rust uses checked `i128` intermediates and deterministic half-up rounding. Binary floating point is absent from business posting logic.

Line discount is applied before tax. Header discount is allocated proportionally across lines; the final line receives the deterministic remainder so the allocated sum equals the exact header discount. `HT` input is taxed forward. `TTC` input is normalized back to HT and recomputed without floating point.

## Below-cost policy

The comparison uses the net sales price after line and header discounts against the current warehouse CUMP, never against a stale catalogue purchase price.

| Company policy | Behavior |
| --- | --- |
| `BLOCK` | Reject below-cost confirmation/posting. |
| `WARNING_ONLY` | Allow and record an audit event. |
| `PRIVILEGED_OVERRIDE` | Require `pricing.override_below_cost` plus a non-empty reason, then audit actor, document, and reason. |

The override does not bypass authentication, company scope, stock availability, idempotency, immutable posting, or audit.

## Transactions, idempotency, and concurrency

Every state-changing command executes in a SQLite `IMMEDIATE` transaction and performs session validation, permission authorization, stable request hashing, idempotency lookup, optimistic row-version checks where applicable, business policy validation, document and line writes, lineage, stock/reservation projection changes, status history, audit, and idempotency completion before commit.

The same idempotency key with the same hash returns the original result. Reuse with different content returns `IDEMPOTENCY_CONFLICT`. Posted documents and their lines remain immutable under accepted database triggers. Company identifiers are derived from the authenticated session rather than trusted from frontend input.

## Permissions

The runtime uses the accepted permissions `sales_order.confirm`, `delivery_note.post`, `sales_invoice.post`, `pricing.override_below_cost`, and `stock.read`. Existing system roles receive only intended grants. Custom roles are not silently granted sensitive permissions.

## Typed Tauri boundary

Fourteen commands are registered through `src-tauri/src/commands/phase07.rs`. `src/platform/tauri/phase07.ts` is the sole frontend gateway. It validates every payload returned by Tauri and normalizes errors to stable safe codes. React receives no SQL, database path, stack trace, session hash, or filesystem path. The browser test invoker is DEV-only and CI rejects it from the production bundle.

## User interface

The chosen direction is the existing **Contemporary Operations Ledger** extended by a **scan-first Sales Workbench**. The same workspace supports ordinary document entry and fast supermarket sale without creating a disconnected cashier application. The command bar accepts barcode input, the process rail keeps nine sales workspaces visible, and the document canvas preserves ledger hierarchy.

Arabic `ar-DZ` is default RTL; French `fr-DZ` is LTR. DZD, quantities, and dates use `Intl`. The interface includes today’s sales ledger, orders, delivery, invoicing, direct sale, returns and credit notes, document lineage, policy explanation, loading/empty/error states, explicit retry, posting confirmation, and posted-lock feedback. Tables scroll internally; page-level horizontal overflow and clipped primary labels are CI failures. Focus is visible and reduced motion is honored.

## Verification

Verification includes frozen migration hashes, the accepted schema verifier, PHASE 06 compatibility, the PHASE 07 structural verifier, Rust fixed-point and transformation tests, real Tauri IPC registration, TypeScript gateway tests, bilingual UI contract tests, Playwright workflows, Axe, overflow/clipping checks, Ubuntu/Windows Rust 1.85, native Tauri compilation, Windows manifest extraction, read-only workflows, public-repository safety scans, and evidence artifacts.

## Deferred work

PHASE 08 owns customer/supplier payments, cash/bank registers, automatic journal posting, settlement and receivables/payables. PHASE 09 owns PDF/printing and reporting. PHASE 10 owns backup/restore, packaging, installer, signing, and release readiness. None of those capabilities is implemented here.

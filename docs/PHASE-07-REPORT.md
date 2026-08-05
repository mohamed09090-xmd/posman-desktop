# PHASE 07 — Sales Cycle Delivery Report

## Delivery coordinates

- Repository: `mohamed09090-xmd/posman-desktop`
- Accepted baseline: `036ac89c07ddee1e26402c1c523529adbba48860`
- Branch: `phase/07-sales-cycle`
- Draft PR: `#11`
- Final head: recorded in the PR after final green validation

## Implemented scope

PHASE 07 implements the local customer sales cycle: editable orders, confirmation and reservation, explicit hold/resume/cancel, partial and full delivery, delivery-backed invoicing, atomic direct sale, sales return plus credit note, document lineage, deterministic fixed-point HT/tax/TTC and discounts, warehouse-CUMP below-cost enforcement, idempotency, optimistic concurrency, permissions, company scoping, audit, and posted immutability.

The frontend is an Arabic/French scan-first Sales Workbench inside the accepted Operations Ledger shell. It uses the typed Tauri gateway only and has no SQL or runtime network client.

## Database decision

No migration `0006` was created. Migrations `0001–0005` remain frozen. PHASE 07 uses accepted commercial document, lineage, reservation, movement, projection, audit, sequence, tax, and idempotency structures. `scripts/verify_phase07.py` verifies the frozen hashes and all required command and invariant evidence.

## Required regression evidence

- An order line of 20 units accepts deliveries of 8 and 12.
- A third delivery of 1 is rejected with `TRANSFORMATION_LIMIT_EXCEEDED`.
- Delivery-to-invoice and document-to-return use the same transactional sibling-sum cap.
- Net sales price is compared with current warehouse CUMP.
- `BLOCK`, `WARNING_ONLY`, and authorized/reasoned `PRIVILEGED_OVERRIDE` behavior is enforced.
- Direct sale and return/credit-note flows are atomic and idempotent.

## Validation status

Local source gates completed before remote validation:

- SQLite verifier: PASS — 5 migrations, schema `0005`, 52 tables, 25 triggers, 133 checks.
- PHASE 06 compatibility verifier: PASS.
- PHASE 07 structural verifier: PASS.
- TypeScript typecheck and production build: PASS.
- UI tests: PASS.
- Integration/gateway tests: PASS.
- Python E2E syntax and Git whitespace: PASS.

Final Ubuntu, Windows, Rust 1.85, E2E/Axe, native Tauri, manifest, workflow links, and artifact metadata are recorded in PR #11 after one final-head green run. This report must not be treated as final acceptance without those PR-linked results.

## Scope exclusions

No payment, journal posting, accounting settlement, PDF/printing, report engine, backup/restore, installer, updater, cloud, telemetry, HTTP API, or PHASE 08 work is included.

## Git safety

The phase uses ordinary fast-forward branch updates. No force-push, rebase, reset, history rewrite, direct commit to `main`, or auto-merge is permitted. The PR remains Draft until final evidence is green and is merged only through an expected-head protected squash after review.

# POSMAN Continuity Checkpoint 04

> Snapshot verified against accepted `main` SHA `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` after PHASE 04 and POST-MERGE HOTFIX 04C. Live GitHub evidence remains authoritative.

Start with [PROJECT-MEMORY-INDEX.md](PROJECT-MEMORY-INDEX.md).

## 1. Recovery coordinates

| Item | Accepted value |
| --- | --- |
| Repository | `https://github.com/mohamed09090-xmd/posman-desktop` |
| Visibility | Public |
| Default branch | `main` |
| Current accepted `main` | `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` |
| Latest accepted product phase | PHASE 04 — Frontend Runtime Integration |
| Latest accepted correction | POST-MERGE HOTFIX 04C |
| Open continuity PR | Draft PR [#5](https://github.com/mohamed09090-xmd/posman-desktop/pull/5), branch `docs/continuity-checkpoint-03` |
| Next candidate | PHASE 05 — planned, unstarted, unauthorized |

## 2. Accepted delivery ledger

| Delivery unit | PR | Accepted squash on `main` | Status |
| --- | ---: | --- | --- |
| PHASE 01 — SQLite Data Foundation | [#1](https://github.com/mohamed09090-xmd/posman-desktop/pull/1) | `0c72eb75eb5db916a51d1ee42fec47f21328ad28` | Accepted |
| Bootstrap Gate — Tauri Desktop Shell | [#2](https://github.com/mohamed09090-xmd/posman-desktop/pull/2) | `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9` | Accepted |
| PHASE 02 — Local Runtime Foundation | [#3](https://github.com/mohamed09090-xmd/posman-desktop/pull/3) | `7112e7f029a6419c7e58f89947f66ccad8bb69e4` | Accepted |
| PHASE 03 — Original UI Foundation | [#4](https://github.com/mohamed09090-xmd/posman-desktop/pull/4) | `f4cda85b24f9d69ebb0442c02f8a037da8ba9baf` | Accepted |
| PHASE 04 — Frontend Runtime Integration | [#6](https://github.com/mohamed09090-xmd/posman-desktop/pull/6) | `a86635a8bc7dd8f3b7683f8f2f33d40c454441bb` | Accepted |
| POST-MERGE HOTFIX 04C | [#7](https://github.com/mohamed09090-xmd/posman-desktop/pull/7) | `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` | Accepted |

Accepted reports and architecture:

- [PHASE 01 report](../PHASE-01-REPORT.md)
- [Bootstrap Gate report](../BOOTSTRAP-GATE-02-REPORT.md)
- [PHASE 02 report](../PHASE-02-REPORT.md)
- [PHASE 03 report](../PHASE-03-REPORT.md)
- [PHASE 04 report](../PHASE-04-REPORT.md)
- [Hotfix 04C report](../HOTFIX-04C-REPORT.md)
- [Frontend runtime integration architecture](../architecture/frontend-runtime-integration.md)

## 3. Accepted PHASE 04 behavior

PHASE 04 connected the accepted frontend shell to the existing read-only runtime command without introducing business operations.

- A typed Tauri gateway is centralized under `src/platform/tauri/**`; UI components do not scatter raw `invoke` calls.
- The frontend consumes **only** `get_runtime_status`.
- Runtime payloads are validated before use, and thrown/unknown failures are normalized into safe user-facing errors without exposing SQL, paths, stack traces, or raw Rust details.
- The UI models `initializing`, `ready`, `error`, and browser `preview` states.
- Retry behavior is explicit and tested.
- Stale responses, unmounts, and React StrictMode activate/deactivate/activate behavior are protected so obsolete results do not overwrite current state and duplicate invocation is avoided.
- Arabic remains the default with RTL behavior; French uses LTR. Runtime state and safe errors are integrated in both languages.
- Browser tests use a controlled development-only adapter seam; production bundle checks reject the development hook.
- No business CRUD, business write path, extra Tauri command, or direct SQL access was introduced.

## 4. Accepted Hotfix 04C behavior

Hotfix 04C corrected the Integration CI ownership model without changing PHASE 04 product source.

- Removed the fixed PHASE 03 ownership baseline from `.github/workflows/integration-ci.yml`.
- Pull-request ownership is now event-scoped from the target branch to the triggering PR head using a three-dot range, with ancestry checks against the checked-out head.
- Push ownership is event-scoped from `github.event.before` to `github.sha` using a two-dot range; an all-zero `before` SHA is rejected.
- Removed the unused `workflow_call` interface after verifying no repository caller depended on it.
- Preserved `permissions: contents: read`.
- Preserved the write-capability guard and the complete Integration validation gate.
- Ownership evidence and the final whitespace/worktree comparison use the resolved event-scoped range.

## 5. Current implemented architecture

### Desktop and data foundation

- Tauri 2 + React + TypeScript + Vite + Rust.
- Bundled SQLite through `rusqlite`; no separately installed database server.
- Four accepted ordered migrations, 49 tables, and 25 integrity triggers.
- Fixed-point integer storage for monetary and quantitative truth.
- Append-only `stock_movements`; rebuildable `stock_balances` projection.
- Posted commercial, stock, accounting, rendered-document, and audit history is immutable.
- Runtime initialization creates the local data structure, validates the exact migration ledger, applies deterministic seed data, and exposes sanitized readiness state.

### Runtime integration path

```text
React runtime feature
  → typed frontend Tauri gateway
  → get_runtime_status
  → Rust RuntimeService
  → bundled local SQLite readiness checks
```

The frontend displays runtime health only. Rust remains the authority for validation, transactions, inventory, totals, and accounting when those later services are authorized.

### Bilingual UI foundation

- Contemporary Operations Ledger visual system.
- Arabic `ar-DZ` default with RTL; French `fr-DZ` with LTR.
- Typed dictionaries, logical CSS, keyboard-visible focus, reduced-motion support, and accessibility evidence.
- Existing gallery and sample commercial screens remain fixtures, not working business modules.

## 6. Current repository tree additions from PHASE 04/04C

The accepted tree now includes:

- `src/platform/tauri/**`
- `src/features/runtime/**`
- `tests/integration/**`
- `.github/workflows/integration-ci.yml`
- `docs/PHASE-04-REPORT.md`
- `docs/HOTFIX-04C-REPORT.md`
- `docs/architecture/frontend-runtime-integration.md`

See [PROJECT-TREE.md](PROJECT-TREE.md) for path ownership.

## 7. Product boundary

Accepted:

- PHASE 01–04.
- POST-MERGE HOTFIX 04C.
- Read-only frontend/runtime health integration.

Not implemented yet:

- Company setup or first-run completion.
- Authentication, users, roles, permissions, or password recovery.
- Catalogue, family, product, customer, or supplier CRUD.
- Inventory writes, opening stock posting, reservations, CUMP/CMUP, purchasing, or stock reconciliation workflows.
- Sales documents, order-to-delivery-to-invoice transformation, returns, or commercial total calculation.
- Accounting-rule selection or journal posting.
- Printing, PDF generation, template editing, reports, backup/restore, installer, signing, or packaged release validation.

PHASE 05–10 remain planned only. This checkpoint does not authorize any of them.

## 8. Next candidate: PHASE 05

PHASE 05 is the next roadmap candidate for first-run setup, security, and reference data. It is **not started and not authorized** by this checkpoint. A future execution pack must define exact baseline, ownership, product decisions, migrations if any, tests, and stop conditions before implementation.

## 9. Public repository safety boundary

This repository is public. Never commit:

- secrets, passwords, tokens, credentials, or private keys;
- real `.env` files;
- customer, employee, supplier, or company data;
- production or recovered SQLite databases, WAL/SHM/journal files, or backups;
- private logs, documents, PDFs, exports, or diagnostic bundles.

Use synthetic fixtures only.

## 10. Recovery procedure

1. Read [AGENTS.md](../../AGENTS.md) and follow [PROJECT-MEMORY-INDEX.md](PROJECT-MEMORY-INDEX.md).
2. Verify live `main` equals the documented accepted SHA or report drift.
3. Inspect merged PRs #1, #2, #3, #4, #6, and #7 and open PR #5.
4. Verify PR states, heads, bases, changed files, and completed CI before claiming health.
5. Distinguish accepted implementation from fixture UI and planned roadmap work.
6. Do not modify code, merge, mark a PR ready, enable auto-merge, or start PHASE 05 without explicit user authorization.

# POSMAN Decision Register

> Accepted product decisions and product-code implementation through Hotfix 04C baseline `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307`. Live `main` must be resolved from GitHub; planned phase details remain proposals until separately authorized and accepted.

## 1. Product boundary

| Area | Accepted decision |
| --- | --- |
| Product | POSMAN commercial-management desktop software |
| First market | Algeria |
| Currency | DZD for v1 |
| Platform | Windows 10/11 64-bit desktop |
| Operation | Single-computer, offline, local-first |
| Database | Bundled local SQLite; no server installation |
| Languages | Arabic `ar-DZ` default/RTL; French `fr-DZ`/LTR |
| Licensing | No subscription or mandatory activation in v1 |
| UX | Original Contemporary Operations Ledger; no copied ERP/admin template |

PHASE 01–04 and Hotfix 04C are accepted. PHASE 05–10 remain planned only.

## 2. Continuity delivery decision

- Continuity Checkpoint 04 is delivered through [PR #5](https://github.com/mohamed09090-xmd/posman-desktop/pull/5); its live state must be verified on GitHub.
- The Hotfix 04C SHA remains the accepted product-code baseline, not a permanent live-`main` assertion.
- Before PR #5 is merged, its continuity content remains unaccepted.
- After PR #5 is merged, live `main` may be a docs-only squash commit ahead of the product-code baseline.
- That newer `main` is an accepted continuity-checkpoint successor only if PR #5 is verified merged and the baseline comparison is limited to `AGENTS.md`, `docs/continuity/**`, and `docs/execution-packs/archive/**`.
- A docs-only continuity successor is not PHASE 05 and is not product-code implementation.
- Any product or out-of-scope drift must be reported and reviewed; it is never accepted automatically.
- PHASE 05 must start from the live accepted `main` resolved when its execution is authorized, not blindly from the historical product-code baseline.

## 3. Technical foundation

| Area | Accepted decision | Reason |
| --- | --- | --- |
| Desktop framework | Tauri 2 | Native Windows shell with Rust integration |
| Frontend | React + TypeScript + Vite | Typed UI and maintainable build |
| Backend | Rust application services | Safe transactions and business validation |
| Database access | `rusqlite` with bundled SQLite | Offline and no external database server |
| ORM | None in accepted foundation | Explicit SQL and invariants |
| UI/backend boundary | Typed Tauri commands | React never accesses SQL |
| Node | Build-time only | End users do not install Node |
| Runtime network | None | Offline and privacy guarantee |
| Data root | Tauri local data directory under `POSMAN` | Stable per-user storage |
| Connections | Configured connection per operation; no global `Connection` | Safer local concurrency |
| Migration control | Ordered embedded migrations with exact ledger hashes | Detect drift and protect data |
| Automatic reset/downgrade | Rejected | Never destroy customer data silently |

## 4. Accepted frontend/runtime integration decisions

- All frontend Tauri access is centralized in a typed gateway under `src/platform/tauri/**`.
- The accepted frontend invokes only `get_runtime_status`.
- Runtime payloads are validated before use.
- Unknown or internal failures are normalized to safe user-facing errors.
- The UI models initializing, ready, error, and browser preview states.
- Retry, stale-response suppression, unmount safety, and StrictMode duplicate-invocation protection are required behavior.
- Arabic RTL and French LTR must cover runtime state and error presentation.
- A development-only browser seam may support deterministic tests, but production bundles must reject it.
- No frontend SQL, raw path, SQL text, stack trace, or raw Rust error exposure.
- No business CRUD or write command was authorized by PHASE 04.

## 5. Accepted Integration CI decisions from Hotfix 04C

- Ownership guards use the triggering event's change range, not a fixed historical phase SHA.
- Pull requests compare the target branch through the triggering PR head with ancestry validation.
- Pushes compare event `before` through event `sha`; all-zero `before` is rejected.
- Unsupported events are rejected.
- The unused `workflow_call` trigger was removed because no repository caller used it.
- Workflow permissions remain `contents: read`.
- The guard against write-capable workflow permissions remains mandatory.
- Evidence metadata and final whitespace checks use the same resolved ownership range.

## 6. Numeric and data decisions

| Value | Storage |
| --- | --- |
| Final monetary amounts | Integer scale 2 |
| Unit prices and costs | Integer scale 4 |
| Quantities | Integer scale 6 |
| Percentage points | Integer scale 4; `19.0000%` → `190000` |
| Internal business IDs | Non-null, non-blank text primary keys |
| Future ID generation | UUIDv7 in Rust |
| Date/time | ISO 8601 text; commercial date distinct from creation timestamp |

Never use binary floating point for business truth. Rust calculation and documented rounding outrank UI display values.

## 7. Inventory decisions

- `stock_movements` is the append-only source of truth.
- `stock_balances` is a rebuildable projection.
- Initial costing method is moving average CUMP/CMUP.
- Negative stock is denied by default; an exception requires permission and audit.
- Posted stock effects are immutable; corrections create compensating movements.
- Retry or double-click must not duplicate a stock event.
- Physical, available, reserved, and projected quantities are distinct.

## 8. Commercial document decisions

- Shared header/line/totals/status/lineage structure.
- Core path: order → delivery → invoice, with partial transformation.
- Direct sale is supported.
- Invoice quantity follows delivered quantity where transformed.
- Posted documents and lines are immutable.
- Corrections use cancellation before posting where allowed, return, credit, reversal, or compensating record.
- `document_line_links` records quantity lineage.
- Aggregate transformed quantity enforcement belongs in one Rust transaction.
- Human document numbers are separate from internal IDs.

## 9. Pricing, tax, and accounting decisions

- Tax rates, account numbers, and posting mappings are configurable data, not permanent hardcoded constants.
- Rust recalculates totals; submitted UI totals are not trusted as final truth.
- Calculation and rounding order must be frozen before write commands.
- Accounting entries require an open period, at least two lines, and equal debit/credit.
- Posting is idempotent and traceable to the business document.
- Posted entries are immutable; correction uses reversal or compensation.
- Missing posting configuration produces an actionable error with no partial entry.

## 10. Documents, backup, and privacy decisions

- Templates use safe internal HTML/CSS; arbitrary template JavaScript is forbidden.
- Template versions and historical rendered snapshots are immutable.
- Backup/restore must validate structure/version and protect the current state before replacement.
- Update or uninstall must not silently delete data.
- All business data remains local by default.
- No telemetry by default.

## 11. Public repository safety decision

The repository is public. Never commit:

- secrets, credentials, API tokens, passwords, private keys, or certificates;
- real `.env` files;
- customer, supplier, employee, or real company data;
- production or recovered databases, SQLite WAL/SHM/journal files, or backups;
- private logs, exports, documents, PDFs, screenshots, or diagnostic bundles.

Synthetic fixtures are required.

## 12. UX decisions

- Clear for non-technical merchants.
- Progressive disclosure for advanced fields.
- Human errors explain the next action.
- Keyboard-visible focus and semantic accessibility.
- One logical layout supporting Arabic RTL and French LTR.
- No generic admin sidebar, KPI card wall, glassmorphism, decorative gradients, bento grid, oversized SaaS cards, or meaningless motion.
- Accepted language includes the numbered Workspace Rail, Command Bar, Document Canvas, Process Strip, Status Stamp, Operational Data Grid, Detail Drawer, and Action Dock.

## 13. Delivery decisions

- Product Owner accepts phases.
- Architect/reviewer defines packs and independently reviews evidence.
- Implementation engineer executes only the active pack.
- Exact live baseline, bounded branch, and delivery PR are mandatory.
- No force-push, rebase/history rewrite, auto-merge, direct `main` commit, or unapproved merge.
- Tests cannot be weakened to obtain green CI.
- A phase report is an evidence inventory, not self-acceptance.

## 14. Open decisions before PHASE 05 and later

| Decision | Resolve before |
| --- | --- |
| Password hashing library and account recovery policy | PHASE 05 |
| Exact Algeria company/legal fields and validation depth | PHASE 05 |
| Fiscal-year and period creation UX | PHASE 05 |
| Tax rounding order and price-margin formulas | PHASE 05/07 |
| Initial human document sequence formats | PHASE 05/07 |
| Negative-stock exception scope and approval UX | PHASE 06 |
| Purchase document vocabulary | PHASE 06 |
| Reservation expiry/release policy | PHASE 07 |
| Initial chart of accounts and posting-rule seed strategy | PHASE 08 |
| Initial report list and export formats | PHASE 09 |
| Backup encryption and retention | PHASE 09 |
| Final PDF/printing implementation after prototype validation | PHASE 09 |
| Installer signing and certificate availability | PHASE 10 |
| Supported Windows/WebView2 packaging matrix | PHASE 10 |

Do not encode guessed defaults as permanent architecture.

# POSMAN Decision Register

> Accepted product and implementation decisions through PHASE 08 baseline `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69`. Live `main` must still be resolved from GitHub; planned PHASE 09/10 details remain proposals until separately authorized and accepted.

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
| Accepted delivery | PHASE 01–08 and Hotfix 04C |
| Next candidate | PHASE 09, unstarted and unauthorized |

## 2. Accepted phase coordinates

| Delivery | Accepted SHA |
| --- | --- |
| PHASE 01 | `0c72eb75eb5db916a51d1ee42fec47f21328ad28` |
| Bootstrap Gate | `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9` |
| PHASE 02 | `7112e7f029a6419c7e58f89947f66ccad8bb69e4` |
| PHASE 03 | `f4cda85b24f9d69ebb0442c02f8a037da8ba9baf` |
| PHASE 04 | `a86635a8bc7dd8f3b7683f8f2f33d40c454441bb` |
| Hotfix 04C | `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` |
| PHASE 05 | `ccf2263104455681cc07ecceda2569c4f7ce0de9` |
| PHASE 06 | `036ac89c07ddee1e26402c1c523529adbba48860` |
| PHASE 07 | `ae133cea9c3b6760a5fd22b38d3169aa2f976dc6` |
| PHASE 08 | `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69` |

## 3. Technical foundation

| Area | Accepted decision | Reason |
| --- | --- | --- |
| Desktop framework | Tauri 2 | Native Windows shell with Rust integration |
| Frontend | React + TypeScript + Vite | Typed UI and maintainable build |
| Backend | Rust application services | Safe transactions and business validation |
| Database access | `rusqlite` with bundled SQLite | Offline and no external database server |
| ORM | None | Explicit SQL and invariants |
| UI/backend boundary | Typed Tauri commands and gateways | React never accesses SQL |
| Node | Build-time only | End users do not install Node |
| Runtime network | None | Offline and privacy guarantee |
| Data root | Tauri local data directory under `POSMAN` | Stable per-user storage |
| Connections | Configured connection per operation | Safer local concurrency |
| SQLite runtime | Foreign keys, bounded busy timeout, requested WAL | Integrity and local responsiveness |
| Migration control | Ordered embedded migrations with exact ledger hashes | Detect drift and protect data |
| Automatic reset/downgrade | Rejected | Never destroy customer data silently |

## 4. Frontend/runtime decisions

- All frontend Tauri access is centralized under `src/platform/tauri/**`.
- Payloads are validated and unknown/internal failures are normalized to safe user-facing errors.
- React does not execute SQL or receive raw database paths, SQL text, stack traces, or raw Rust errors.
- Controlled development-only adapters may support deterministic browser tests; production bundles must reject them.
- Active accepted workspaces exist for PHASE 05–08.
- Arabic RTL and French LTR cover real business operations, not only fixtures.

## 5. Security and administration decisions

- Passwords use Argon2id.
- Authentication is local and session-based with inactivity locking.
- Recovery uses a controlled one-time recovery-code mechanism.
- Authorization is enforced in Rust through roles and permissions.
- Every business mutation is company-scoped.
- Last-system-administrator protection prevents accidental administrative lockout.
- Optimistic concurrency protects mutable setup/reference records.
- Safe audit events are recorded for relevant operations.

## 6. Numeric and data decisions

| Value | Storage |
| --- | --- |
| Final monetary amounts | Integer scale 2 |
| Unit prices and costs | Integer scale 4 |
| Quantities | Integer scale 6 |
| Percentage points | Integer scale 4; `19.0000%` → `190000` |
| Internal business IDs | Non-null, non-blank text primary keys |
| ID generation | UUIDv7 in Rust |
| Date/time | ISO 8601 text; commercial date distinct from creation timestamp |

Never use binary floating point for business truth. Rust calculation and documented rounding outrank submitted UI totals.

## 7. Inventory decisions

- `stock_movements` is the append-only source of truth.
- `stock_balances` is a rebuildable projection.
- Costing uses moving average CUMP/CMUP.
- Warehouse aggregate and location projection share the accepted CUMP contract.
- Negative stock is denied by default; controlled override requires permission and audited reason.
- Posted stock effects are immutable; corrections create compensating movements.
- Physical, reserved, available, and projected quantities are distinct.
- Reconciliation and rebuild must derive balances from the movement ledger.

## 8. Purchasing decisions

- Purchase workflows support orders, partial/full receipts, receipt-backed invoices, direct receive-and-invoice, and returns.
- Posting uses one SQLite `IMMEDIATE` transaction.
- Company-scoped idempotency binds a key to a stable request hash.
- Aggregate source-line transformation limits are checked transactionally.
- A supplier invoice cannot duplicate receipt stock effects.
- Purchase returns compensate posted history rather than mutating it.

## 9. Sales decisions

- Core path supports order → delivery → invoice with partial transformation.
- Direct sale is supported atomically.
- Sales returns create compensating return/credit records.
- Delivered quantities drive transformed invoicing.
- Aggregate transformed quantity is enforced in one Rust transaction.
- Deterministic fixed-point HT, tax, discount, and TTC calculations are authoritative.
- Below-cost policy compares net sales price with current warehouse CUMP.
- Override requires explicit permission and mandatory audited reason.
- Posted documents and lines are immutable.

## 10. Accounting and payment decisions

- Accounts, journals, semantic account roles, and posting rules are configurable data.
- Source posting fails closed on missing, inactive, or ambiguous configuration.
- Required source/stock/journal/audit/idempotency success effects share one atomic transaction.
- Failed posting stores only safe attempt metadata after rollback.
- Accounting entries require an open period and balanced debit/credit.
- Manual journals are editable only before posting.
- Posted journals are immutable; correction uses a linked reversal.
- Customer receipts and supplier payments support partial/full allocations.
- Allocation/payment corrections use compensating reversal.
- Statements, registers, trial balance, ledgers, and open balances derive from accepted accounting/payment truth.
- Fiscal period close is enforced; reopen is controlled by permission.

## 11. Schema and migration decisions

- Accepted migrations `0001`–`0006` are frozen.
- `database/schema.sql` is generated and must match ordered migrations exactly.
- A correction requiring schema change must use an explicitly authorized `0007` or later migration.
- Application business columns must not use SQLite `REAL`.
- Posted-history triggers and application-service invariants are not weakened to make tests pass.

## 12. Documents, backup, and privacy decisions

Accepted foundation:

- templates use safe internal HTML/CSS; arbitrary JavaScript is forbidden;
- template versions and historical rendered snapshots are immutable;
- schema tables exist for templates, versions, rendered documents, attachments, audit logs, and backup history;
- application-owned directories exist for templates, documents, backups, and logs;
- all business data remains local by default;
- no telemetry by default.

Deferred to PHASE 09:

- template management/publishing services;
- PDF/printing implementation;
- historical render snapshot generation and reprint;
- reports/exports and audit presentation;
- safe manual/automatic backup and validated restore.

Restore must validate compatibility and integrity and protect the current state before replacement.

## 13. UX decisions

- Clear for non-technical merchants.
- Progressive disclosure for advanced fields.
- Human errors explain the next action.
- Keyboard-visible focus and semantic accessibility.
- One logical layout supporting Arabic RTL and French LTR.
- No generic admin sidebar, KPI card wall, glassmorphism, decorative gradients, bento grid, oversized SaaS cards, or meaningless motion.
- Accepted language includes Workspace Rail, Command Bar, Document Canvas, Process Strip, Status Stamp, Operational Data Grid, Detail Drawer, and Action Dock.

## 14. CI and delivery decisions

- Workflow permissions remain read-only unless explicitly justified and reviewed.
- Ownership guards use the triggering event's actual change range.
- Product Owner accepts phases.
- Architect/reviewer independently reviews evidence.
- Implementation engineer executes only the active scope.
- No force-push, rebase/history rewrite, auto-merge, direct `main` commit, or unapproved merge.
- Tests and security checks cannot be weakened to obtain green CI.
- A phase report is an evidence inventory, not self-acceptance.

## 15. Open decisions before PHASE 09 and PHASE 10

| Decision | Resolve before |
| --- | --- |
| Final PDF/printing engine and Windows printer strategy | PHASE 09 |
| Template sanitization and publishing/version contract | PHASE 09 |
| Initial report list and export formats | PHASE 09 |
| Audit detail redaction and export permissions | PHASE 09 |
| Backup retention and optional encryption policy | PHASE 09 |
| Safe SQLite WAL backup/restore implementation | PHASE 09 |
| Tauri dialog/filesystem/printing capability boundary | PHASE 09 |
| Installer format and WebView2 packaging matrix | PHASE 10 |
| Signing certificate availability and signing policy | PHASE 10 |
| Upgrade/uninstall data-preservation matrix | PHASE 10 |
| Final semantic version and release-channel policy | PHASE 10 |

Do not encode guessed defaults as permanent architecture.

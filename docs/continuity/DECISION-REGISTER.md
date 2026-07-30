# POSMAN Decision Register

> This register summarizes accepted decisions and explicitly separates them from unresolved implementation details. The Blueprint and accepted architecture documents remain the detailed sources.

## 1. Product decisions

| Area | Accepted decision | Status |
| --- | --- | --- |
| Product name | POSMAN | Accepted |
| Product type | General commercial-management software downloadable by any merchant | Accepted |
| Inspiration | Comparable functional scope to Sage 100 Gescom; no UI copying | Accepted |
| First market | Algeria | Accepted for v1 |
| Currency | Algerian dinar, DZD | Accepted for v1 |
| Platform | Real Windows 10/11 64-bit desktop application | Accepted |
| Operation | Single-computer, offline, local-first | Accepted for v1 |
| Installation promise | User installs the program and starts without separately installing a database or development tools | Accepted |
| Licensing | No subscription or mandatory activation in v1 | Accepted |
| Default language | Arabic `ar-DZ` | Accepted |
| Additional language | French `fr-DZ` | Accepted |
| Product promise | From installation to first invoice through understandable steps without traditional ERP complexity | Accepted |

## 2. Core v1 scope

- Company and activity setup.
- Warehouses and locations.
- Product families, products, codes, descriptions, costs, and sale prices.
- Configurable taxes, discounts, and margin-based price calculation.
- Customers and suppliers.
- Opening stock.
- Inventory movement tracking and current quantities.
- Purchasing and supplier receipts/invoices.
- Direct sales invoices.
- Customer order → delivery → invoice transformation.
- Partial delivery and lineage.
- Returns/corrections.
- Custom company document identity.
- Automatic configurable accounting posting.
- Reports, audit, backup/restore.
- Offline installer and clean Windows release evidence.

## 3. Explicitly outside v1

- Multi-computer synchronization.
- Mobile app.
- Cloud account or mandatory cloud service.
- Multi-country and multi-currency behavior.
- Manufacturing/GPAO.
- Payroll and HR.
- E-commerce.
- Official external e-invoicing integration.
- Full drag-and-drop report designer.
- Subscription/activation system.

Design extension points may exist, but these features must not be implemented accidentally during v1.

## 4. Technical decisions

| Area | Accepted decision | Reason |
| --- | --- | --- |
| Desktop framework | Tauri 2 | Small native shell, Rust integration, Windows packaging |
| Frontend | React + TypeScript + Vite | Typed maintainable UI and proven shell |
| Backend | Rust application services | Financial validation, safe transactions, performance |
| Database | Bundled SQLite through `rusqlite` | No external server; offline single-computer use |
| ORM | None in accepted foundation | Keep SQL and invariants explicit |
| UI-to-backend boundary | Typed Tauri commands | React never accesses SQL directly |
| Node | Build-time only | End users do not install Node |
| Runtime network | None | Offline guarantee and privacy |
| Data root | Tauri local data directory under `POSMAN` | Stable per-user Windows location |
| Concurrency | Open configured connections per operation; no global `Connection` | Avoid unsafe shared connection state |
| Database upgrade | Ordered embedded migrations with exact ledger hashes | Detect drift and protect customer data |
| Downgrade/reset | Never automatic | Avoid destructive recovery |
| Default journal mode | Request WAL, record actual mode | Local resilience without false assumptions |
| Minimum Rust compatibility | Rust 1.85 for accepted PHASE 02 graph | Reproducibility |
| Windows manifest | Shared Common Controls v6 manifest through Tauri build attributes and MSVC link args | Real tests and app run correctly |

## 5. Numeric and data decisions

| Value | Storage |
| --- | --- |
| Final monetary amounts | Integer scale 2 |
| Unit prices and costs | Integer scale 4 |
| Quantities | Integer scale 6 |
| Percentage points | Integer scale 4; `19.0000%` → `190000` |
| Internal business IDs | Non-null, non-blank text primary keys |
| Future ID generation | UUIDv7 in Rust |
| Date/time | ISO 8601 text; commercial date distinct from created timestamp |

Rules:

- Never use binary floating point for business truth.
- Rounding order must be documented and deterministic.
- UI display formatting never replaces Rust calculation.
- Human document numbers are separate from internal IDs.

## 6. Inventory decisions

- `stock_movements` is the append-only source of truth.
- `stock_balances` is a rebuildable projection.
- Initial costing method is moving average CUMP/CMUP.
- Negative stock is denied by default.
- A manager exception requires explicit permission and audit.
- Posted stock effects are immutable.
- Corrections create compensating movements.
- Retry/double-click must not duplicate a stock event.
- Availability, physical quantity, reservation, and projected quantity are distinct concepts.

## 7. Commercial document decisions

- Shared document structure: header, lines, discounts/taxes, totals, action dock, lineage/status history.
- Supported core path: order → delivery → invoice.
- Partial transformations are first-class.
- Invoice creation from a transformed workflow uses delivered quantities.
- Direct sale is supported.
- Posted documents and lines are immutable.
- Correction uses return, credit, cancellation before posting where allowed, or reversal.
- `document_line_links` records source/target quantity lineage.
- Aggregate transformed quantity enforcement is deferred to one Rust transaction.
- Human numbers use configurable scoped sequences.

## 8. Pricing and tax decisions

- Tax rates are configurable data, not hardcoded program constants.
- Discounts may apply at line or document level according to the detailed business contract.
- Sale price may be derived from unit cost through a configurable cost-margin method.
- A sale-margin method may also be configured where supported.
- Rust recalculates totals; the UI cannot submit trusted totals as final truth.
- The calculation and rounding order must be frozen before PHASE 05/07 business commands.

## 9. Accounting decisions

- Posting is automatic but configurable.
- Account numbers and mappings are data, never permanent hardcoded logic.
- Entries require an open fiscal period.
- Posted entry has at least two lines and exactly balanced debit/credit.
- Posting is idempotent.
- Posted journal entries and lines are immutable.
- Correction uses reversal or compensating entries.
- Business document → journal traceability is mandatory.
- Missing posting configuration produces an actionable error and no partial entry.

## 10. Document and PDF decisions

- Templates use safe internal HTML/CSS.
- Arbitrary template JavaScript is forbidden.
- Preview occurs inside the application.
- Printing/PDF uses a validated native WebView2/Windows path.
- Template versions are immutable.
- A historical rendered snapshot is stored so reprint reproduces the original document.
- Documents use company identity and bilingual-capable content.

## 11. Backup and recovery decisions

- Daily automatic backup at first run of the day is the v1 target.
- Backup before import, restore, destructive reset, and future migration.
- Manual backup to a chosen folder or USB.
- Restore validates structure and version.
- Restore first backs up the current state.
- Corrupt/incompatible backups are rejected without damaging current data.
- Update and uninstall do not silently delete data.

## 12. Security and privacy decisions

- All business data remains local by default.
- No telemetry by default.
- No passwords, tokens, real `.env`, production database, or customer data in Git.
- Passwords are never stored as plain text.
- Least-privilege Tauri capability.
- Sensitive errors do not expose path, SQL, stack, or customer data.
- Permission checks exist in Rust services, not only UI visibility.
- Audit records are append-only.
- Destructive actions require explicit effect preview and confirmation.

## 13. UX decisions

### Direction

**Contemporary Operations Ledger**: precise, calm, commercial, modern, and operational.

### Required behavior

- Clear enough for a non-technical merchant.
- Progressive disclosure for advanced fields.
- Human error messages that explain the next action.
- Consistent document structure.
- Keyboard-visible focus and semantic accessibility.
- Arabic default RTL and French LTR through one logical layout.
- Fast on modest hardware; no heavy effects.

### Rejected visual patterns

- Generic admin-dashboard sidebar.
- KPI card wall.
- Glassmorphism.
- Decorative blue/purple gradients.
- Bento grid.
- Large rounded SaaS cards.
- 3D decoration or stock imagery.
- Motion without operational meaning.
- Icon-only workspace navigation that hides labels.

### Accepted visual language

- Numbered Workspace Rail.
- Command Bar.
- Document Canvas.
- Process Strip.
- Status Stamp.
- Operational Data Grid.
- Detail Drawer.
- Action Dock.
- Warm paper-like surfaces, restrained green/amber/red states, small radii, and compact spacing.

## 14. Performance decisions

Targets to validate, not unproven claims:

- Comfortable operation on a Celeron N4020-class machine with 4 GB RAM.
- Cold start under five seconds where practical.
- Typical memory target below 250 MB.
- Offline installer target below 200 MB.
- Search across 100,000 products without freezing.
- Pagination or virtualization for large grids.

## 15. Delivery and review decisions

- The user accepts phases.
- The primary assistant plans and reviews by default.
- Implementation may be delegated to another high-reasoning agent.
- Each phase uses an exact baseline, scoped branch, and Draft PR.
- No force-push, rebase/history rewrite, auto-merge, or unapproved merge.
- Accepted phases use squash merge with expected head protection.
- Windows and Ubuntu evidence is required where platform behavior can differ.
- Tests cannot be weakened to obtain green CI.
- Temporary diagnostic helpers must not remain in the final tree.
- A phase report is evidence inventory, not acceptance.

## 16. Accepted rejected alternatives

| Alternative | Decision |
| --- | --- |
| Web application | Rejected; product is desktop |
| External PostgreSQL/MySQL installation | Rejected for v1 |
| Tauri SQL plugin with UI SQL | Rejected in accepted architecture |
| Cloud-first or account-required operation | Rejected for v1 |
| `REAL` for financial/quantity values | Rejected |
| Direct edits to posted history | Rejected |
| Hardcoded TVA/account numbers | Rejected |
| Generic admin template | Rejected |
| Disabling Windows test or Common Controls feature to pass CI | Rejected |
| Global linker workaround without target evidence | Rejected during Bootstrap patch review |

## 17. Open decisions to resolve before their phase

These are intentionally not invented:

| Decision | Resolve before |
| --- | --- |
| Exact password hashing library and recovery policy | PHASE 05 |
| Initial Algeria legal/company identifier fields and validation depth | PHASE 05 |
| Default fiscal-year and period creation UX | PHASE 05 |
| Exact tax rounding order and price-margin formulas in commands | PHASE 05/07 |
| Initial document sequence formats | PHASE 05/07 |
| Exact negative-stock exception scope and approval UX | PHASE 06 |
| Purchase document vocabulary and whether receipt/invoice are separate in v1 | PHASE 06 |
| Reservation expiry/release policy | PHASE 07 |
| Initial chart of accounts and posting-rule seed strategy | PHASE 08 |
| Initial report list and export formats | PHASE 09 |
| Backup encryption policy and default retention count | PHASE 09 |
| Exact PDF/printing implementation after prototype validation | PHASE 09 |
| Installer signing strategy and certificate availability | PHASE 10 |
| Minimum supported Windows/WebView2 packaging matrix | PHASE 10 |

Resolve these with the user when the decision changes product behavior. Do not encode guessed defaults as permanent architecture.

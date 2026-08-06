# POSMAN Continuity Checkpoint 08

> Accepted recovery checkpoint through PHASE 08 — Automatic Accounting and Payments. Resolve live `main` and current Pull Requests from GitHub during every recovery; newer verified repository evidence outranks this checkpoint.

Start with [PROJECT-MEMORY-INDEX.md](PROJECT-MEMORY-INDEX.md).

## 1. Stable recovery coordinates

| Item | Stable value |
| --- | --- |
| Repository | `https://github.com/mohamed09090-xmd/posman-desktop` |
| Visibility | Public |
| Default branch | `main` |
| Accepted product baseline through PHASE 08 | `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69` |
| Latest accepted product phase | PHASE 08 — Automatic Accounting and Payments |
| Latest accepted correction | POST-MERGE HOTFIX 04C |
| Next candidate | PHASE 09 — Documents, Printing, Reports, Audit, and Backup |
| PHASE 09 state | Planned, unstarted, and unauthorized until explicitly started |
| PHASE 10 state | Planned and unauthorized |

## 2. Accepted delivery ledger

| Delivery unit | PR | Accepted squash on `main` | Status |
| --- | ---: | --- | --- |
| PHASE 01 — SQLite Data Foundation | #1 | `0c72eb75eb5db916a51d1ee42fec47f21328ad28` | Accepted |
| Bootstrap Gate — Tauri Desktop Shell | #2 | `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9` | Accepted |
| PHASE 02 — Local Runtime Foundation | #3 | `7112e7f029a6419c7e58f89947f66ccad8bb69e4` | Accepted |
| PHASE 03 — Original UI Foundation | #4 | `f4cda85b24f9d69ebb0442c02f8a037da8ba9baf` | Accepted |
| PHASE 04 — Frontend Runtime Integration | #6 | `a86635a8bc7dd8f3b7683f8f2f33d40c454441bb` | Accepted |
| POST-MERGE HOTFIX 04C | #7 | `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` | Accepted |
| PHASE 05 — Setup, Security, and Reference Data | #8 | `ccf2263104455681cc07ecceda2569c4f7ce0de9` | Accepted |
| PHASE 06 — Inventory and Purchasing | #10 | `036ac89c07ddee1e26402c1c523529adbba48860` | Accepted |
| PHASE 07 — Sales Cycle | #11 | `ae133cea9c3b6760a5fd22b38d3169aa2f976dc6` | Accepted |
| PHASE 08 — Accounting and Payments | #12 | `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69` | Accepted product baseline |

Accepted reports:

- [PHASE 01 report](../PHASE-01-REPORT.md)
- [Bootstrap Gate report](../BOOTSTRAP-GATE-02-REPORT.md)
- [PHASE 02 report](../PHASE-02-REPORT.md)
- [PHASE 03 report](../PHASE-03-REPORT.md)
- [PHASE 04 report](../PHASE-04-REPORT.md)
- [Hotfix 04C report](../HOTFIX-04C-REPORT.md)
- [PHASE 05 report](../PHASE-05-REPORT.md)
- [PHASE 06 report](../PHASE-06-REPORT.md)
- [PHASE 07 report](../PHASE-07-REPORT.md)
- [PHASE 08 report](../PHASE-08-REPORT.md)

## 3. Accepted implemented architecture

### Desktop and data foundation

- Tauri 2 + React + TypeScript + Vite + Rust.
- Bundled SQLite through `rusqlite`; no external database server.
- Six accepted ordered migrations through `0006`.
- Generated `database/schema.sql` must match all ordered migrations.
- 57 tables, 47 triggers, and fixed-point integer storage for business truth at the PHASE 08 acceptance point.
- Per-operation configured SQLite connections with foreign keys, bounded busy timeout, and requested WAL mode.
- Local directories for data, backups, documents, templates, and logs.
- Posted commercial, stock, accounting, rendered-document, and audit history is protected as immutable.

### Typed application boundary

```text
React workspace
  → typed gateway under src/platform/tauri/**
  → registered Tauri command
  → Rust phase service
  → authenticated/company-scoped transaction
  → bundled local SQLite
```

React owns presentation and interaction. Rust owns authentication, authorization, validation, fixed-point calculations, idempotency, audit, and transaction boundaries.

### Bilingual UI

- Arabic `ar-DZ` is default with RTL.
- French `fr-DZ` uses LTR.
- Logical CSS, keyboard-visible focus, reduced-motion support, overflow/clipping checks, and Axe evidence are part of accepted delivery gates.
- Active workspaces exist for administration/reference data, inventory/purchasing, sales, and accounting/payments.

## 4. Accepted product capabilities

### PHASE 05 — setup and administration

- Atomic first-run setup and resumable draft handling.
- Company identity, fiscal year/periods, taxes, margins, below-cost policy, session timeout, and document sequences.
- Argon2id password hashing, login/logout, sessions, inactivity lock, recovery-code rotation, and password changes.
- Users, roles, permissions, last-system-administrator protection, and optimistic concurrency.
- Products, families, units, prices, warehouses, locations, customers, suppliers, addresses, contacts, payment terms, and payment methods.
- Company scoping, typed Tauri commands, safe errors, and audit events.

### PHASE 06 — inventory and purchasing

- Append-only stock movements and rebuildable stock balances.
- Opening stock, adjustments, transfers, physical counts, reservations, negative-stock controls, reconciliation, and rebuild.
- Deterministic fixed-point moving CUMP/CMUP.
- Purchase orders, partial/full receipts, receipt-backed invoices, direct receive-and-invoice, and purchase returns.
- Atomic posting, idempotency, permissions, and company isolation.

### PHASE 07 — sales

- Sales orders, explicit reservations, confirmation, hold/resume/cancel.
- Partial/full delivery, delivery-backed invoicing, direct sale, returns, and credit documents.
- Transactional aggregate transformation limits and document lineage.
- Deterministic HT/tax/TTC/discount calculations.
- Current-warehouse CUMP below-cost policy with controlled override and audited reason.

### PHASE 08 — accounting and payments

- Company-scoped chart of accounts and accounting journals.
- Semantic account-role mappings and configurable multi-line posting rules.
- Atomic automatic posting for sales, purchases, returns, inventory/COGS-related source events, with safe retry history.
- Manual journal entries, posting, linked reversals, and posted-entry immutability.
- Customer receipts, supplier payments, partial/full allocations, allocation reversal, and payment reversal.
- Partner statements, cash/bank register, trial balance, general ledger, account ledger, open receivables/payables.
- Fiscal-period close and controlled reopen.

## 5. Accepted validation state at PHASE 08

The PHASE 08 acceptance evidence records:

- schema verification: six migrations, version `0006`, 57 tables, 47 triggers, 134 checks;
- 17 UI tests;
- 55 frontend integration tests;
- 64 Rust tests;
- Rust 1.85 locked check, formatting, Clippy with warnings denied, and native Tauri builds;
- six PHASE 08 Arabic/French browser scenarios with zero Axe violations and no unresolved critical/serious incomplete findings;
- successful compatibility workflows for earlier accepted phases and Windows Common Controls v6 manifest checks.

Always verify the current workflow state directly before claiming live repository health.

## 6. Current product boundary

Implemented and accepted:

- PHASE 01–08 and Hotfix 04C.
- Real business write commands for setup, catalogue, inventory, purchasing, sales, accounting, and payments.
- Operational React workspaces connected through typed local Tauri gateways.

Not implemented:

- published document-template management and immutable rendering service;
- validated PDF generation, preview, printing, and historical reprint path;
- complete operational report/export engine;
- audit-log presentation and export;
- manual/automatic backup plus compatibility/corruption-checked restore;
- production Windows installer, signing, clean-machine install/upgrade/uninstall validation, and v1 release artifacts.

Cloud synchronization, telemetry, subscription, mandatory activation, and external HTTP APIs remain outside the approved v1 product boundary.

## 7. Next candidate: PHASE 09

PHASE 09 is the next roadmap candidate. It has not started and requires explicit authorization plus a bounded execution pack.

Candidate scope:

- safe versioned HTML/CSS document templates without arbitrary JavaScript;
- immutable render snapshots, preview, PDF, printing, and historical reprint;
- operational reports and exports;
- audit-log presentation;
- manual/automatic backup and validated restore.

Critical gates:

- reprint reproduces historical output;
- restore validates compatibility and integrity before replacing current data;
- a safety backup is created before destructive restore/reset operations;
- private data and generated artifacts remain local;
- accepted migrations `0001`–`0006` remain frozen unless an explicitly authorized `0007` is required.

## 8. Known structural considerations before PHASE 09

- `RuntimePaths` already defines data, backup, document, template, and log directories, but PHASE 09 must expose them through a controlled Rust service rather than frontend filesystem access.
- The runtime uses WAL mode; backup must use a safe SQLite snapshot/backup mechanism rather than copying only the main database file while active.
- Current Tauri capabilities are minimal. Any filesystem/dialog/printing capability must be narrowly scoped and justified.
- `document_templates`, `document_template_versions`, `rendered_documents`, `attachments`, `audit_logs`, and `backup_history` already exist in the accepted schema, but PHASE 09 application services are absent.

## 9. Public repository safety boundary

Never commit:

- secrets, passwords, tokens, credentials, private keys, certificates, or signing material;
- real `.env` files;
- real customer, employee, supplier, authentication, or company data;
- production/recovered SQLite databases, WAL/SHM/journal files, or backups;
- private logs, documents, PDFs, exports, screenshots, or diagnostic bundles.

Use synthetic fixtures only.

## 10. Recovery procedure

1. Read [AGENTS.md](../../AGENTS.md) and follow [PROJECT-MEMORY-INDEX.md](PROJECT-MEMORY-INDEX.md).
2. Resolve live `main` and compare it with the checkpoint baseline `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69`.
3. Verify merged PRs #1, #2, #3, #4, #6, #7, #8, #10, #11, and #12.
4. Inspect open Pull Requests and branches; do not treat unmerged work as accepted.
5. Verify relevant workflow runs are completed and successful before claiming repository health.
6. Compare implemented source, registered commands, migrations, tests, and reports rather than relying on one document.
7. Report any drift between live GitHub state and this checkpoint.
8. Do not modify code, merge, mark a PR ready, enable auto-merge, or start PHASE 09 during recovery unless explicitly authorized after the recovery report.

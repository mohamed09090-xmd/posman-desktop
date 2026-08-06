# POSMAN Project Memory Index

> Continuity checkpoint through accepted PHASE 08. Always resolve the live repository state from GitHub before acting; this package records the accepted baseline at the time of this checkpoint and must not override newer verified evidence.

## Stable recovery coordinates

| Item | Value |
| --- | --- |
| Repository | `https://github.com/mohamed09090-xmd/posman-desktop` |
| Accepted product baseline through PHASE 08 | `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69` |
| Latest accepted product phase | PHASE 08 — Automatic Accounting and Payments |
| Latest accepted correction | POST-MERGE HOTFIX 04C |
| Default branch | `main` |
| Next candidate | PHASE 09 — Documents, Printing, Reports, Audit, and Backup |
| PHASE 09 status | Planned, unstarted, and unauthorized until explicitly started |
| PHASE 10 status | Planned and unauthorized |

## Accepted delivery ledger

| Delivery | Pull Request | Accepted squash on `main` |
| --- | ---: | --- |
| PHASE 01 — SQLite Data Foundation | #1 | `0c72eb75eb5db916a51d1ee42fec47f21328ad28` |
| Bootstrap Gate — Desktop Shell | #2 | `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9` |
| PHASE 02 — Local Runtime Foundation | #3 | `7112e7f029a6419c7e58f89947f66ccad8bb69e4` |
| PHASE 03 — Original UI Foundation | #4 | `f4cda85b24f9d69ebb0442c02f8a037da8ba9baf` |
| PHASE 04 — Frontend Runtime Integration | #6 | `a86635a8bc7dd8f3b7683f8f2f33d40c454441bb` |
| POST-MERGE HOTFIX 04C | #7 | `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` |
| PHASE 05 — Setup, Security, and Reference Data | #8 | `ccf2263104455681cc07ecceda2569c4f7ce0de9` |
| PHASE 06 — Inventory and Purchasing | #10 | `036ac89c07ddee1e26402c1c523529adbba48860` |
| PHASE 07 — Sales Cycle | #11 | `ae133cea9c3b6760a5fd22b38d3169aa2f976dc6` |
| PHASE 08 — Accounting and Payments | #12 | `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69` |

## Mandatory reading order

Read these files in order before proposing or executing project work:

1. [Repository instructions](../../AGENTS.md)
2. [Current state](CURRENT-STATE.md)
3. [AI operating contract](AI-OPERATING-CONTRACT.md)
4. [Master roadmap PHASE 01–10](MASTER-ROADMAP-PHASES-01-10.md)
5. [Decision register](DECISION-REGISTER.md)
6. [Current project tree](PROJECT-TREE.md)
7. [Recovery prompt](RECOVERY-PROMPT.md)
8. [Product Blueprint](../spec/POSMAN-Blueprint-v1.md)
9. [PHASE 05 report](../PHASE-05-REPORT.md)
10. [PHASE 06 report](../PHASE-06-REPORT.md)
11. [PHASE 07 report](../PHASE-07-REPORT.md)
12. [PHASE 08 report](../PHASE-08-REPORT.md)
13. Relevant architecture documents under `docs/architecture/`

After reading, resolve live `main`, merged/open Pull Requests, changed files, and completed GitHub Actions. Never continue automatically from an unmerged branch.

## Source hierarchy

1. Explicit current user instruction.
2. Live accepted `main`, merged Pull Request metadata, Git history, and completed CI evidence.
3. `AGENTS.md` and an active explicitly approved execution pack.
4. Accepted Blueprint, architecture documents, and phase reports.
5. This continuity package.
6. Unmerged branches, delivery reports, agent claims, and old conversation summaries.

Report conflicts instead of silently resolving them.

## Current product boundary

Accepted and implemented:

- local Tauri/Rust/React desktop runtime with bundled SQLite;
- Arabic RTL and French LTR UI foundation;
- first-run setup, authentication, sessions, users, roles, permissions, catalogue, partners, warehouses, and configuration;
- inventory ledger, CUMP/CMUP, reservations, counts, adjustments, transfers, purchasing, reconciliation, and rebuild;
- sales orders, delivery/invoice transformations, direct sale, returns, lineage, totals, and below-cost controls;
- chart of accounts, journals, posting rules, automatic/manual posting, reversals, payments, allocations, statements, ledgers, trial balance, open balances, and fiscal-period controls.

Not implemented:

- completed document template/rendering service, PDF/printing, report/export engine, audit presentation, backup/restore;
- installer, signing, clean-machine distribution, upgrade/uninstall evidence, and v1 release.

## Memory file ownership

| File | Purpose | Update trigger |
| --- | --- | --- |
| `CURRENT-STATE.md` | Accepted baseline, delivery ledger, implemented boundary, next candidate, and recovery procedure | Every accepted merge or material blocker |
| `AI-OPERATING-CONTRACT.md` | Roles, authority, evidence, Git rules, safety, and architecture guardrails | Collaboration or delivery-policy change |
| `MASTER-ROADMAP-PHASES-01-10.md` | Accepted/planned phases, dependencies, scope, exclusions, and gates | Phase acceptance or roadmap decision |
| `DECISION-REGISTER.md` | Accepted product, architecture, data, UX, security, and process decisions | Decision accepted, replaced, or reopened |
| `PROJECT-TREE.md` | Accepted source tree and ownership map | Structural change |
| `RECOVERY-PROMPT.md` | Copy-ready recovery instructions | Baseline or recovery-procedure change |

## Historical execution records

Files under `docs/execution-packs/archive/` are historical evidence only. They are not active instructions and must not override the live accepted baseline or current explicit user authorization.

## Public repository warning

The repository is public. Never commit secrets, credentials, tokens, private keys, real `.env` files, customer/company data, production databases, SQLite WAL/SHM files, backups, private logs, documents, PDFs, or diagnostic bundles.

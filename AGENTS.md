# POSMAN repository instructions

These rules apply to every implementation, review, documentation, and recovery agent working in this repository.

## Mandatory continuity bootstrap

Before planning, reviewing, or implementing project work:

1. Read `docs/continuity/PROJECT-MEMORY-INDEX.md` and follow its mandatory reading order.
2. Read `docs/continuity/CURRENT-STATE.md`, `docs/continuity/AI-OPERATING-CONTRACT.md`, `docs/continuity/MASTER-ROADMAP-PHASES-01-10.md`, `docs/continuity/DECISION-REGISTER.md`, and `docs/continuity/PROJECT-TREE.md` before proposing project work.
3. Read `docs/continuity/RECOVERY-PROMPT.md` after a lost conversation, account change, or agent handoff.
4. Verify the checkpoint against live `main`, merged/open Pull Requests, Git history, changed files, and completed CI. Live accepted repository evidence outranks stale documentation.
5. Distinguish accepted and verified work from reported, proposed, deferred, and rejected work. A roadmap entry is not authorization.
6. Keep the continuity package current through a separate docs-only Draft Pull Request after accepted phases or material repository-state changes.
7. The latest accepted product phase is PHASE 08. The next candidate is PHASE 09 and remains unstarted and unauthorized until explicitly started by the Product Owner.

## Current accepted boundary

- Accepted live product baseline through PHASE 08: `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69`.
- PHASE 01–08 and POST-MERGE HOTFIX 04C are accepted on `main`.
- PHASE 09 and PHASE 10 are not implemented.
- Accepted migrations `0001`–`0006` are immutable. Corrections require a new ordered migration.
- Existing typed business commands and workspaces for PHASE 05–08 are accepted product functionality and must not be described as fixtures or future work.

## Standing repository rules

1. Read `docs/spec/POSMAN-Blueprint-v1.md` before changing domain logic or architecture.
2. Keep POSMAN Windows-first, offline, local-first, and based on bundled SQLite. Do not introduce a server, cloud dependency, telemetry, online account, subscription, or external database without explicit approval.
3. Never use floating-point storage for money, prices, costs, tax rates, discounts, or quantities. Follow the documented fixed-point scales.
4. Never update inventory only through `stock_balances`; every inventory change originates in the append-only `stock_movements` ledger.
5. Never mutate or delete posted commercial documents, posted journal entries, stock movements, rendered historical documents, or audit logs. Corrections use reversal, return, credit, or compensating records.
6. Never hardcode tax rates, accounting account numbers, or posting mappings as permanent program logic.
7. This repository is public. Never commit secrets, credentials, tokens, private keys, real `.env` files, production/customer databases, backups, private documents, or logs containing private data.
8. Preserve Arabic `ar-DZ` as the default language with RTL correctness and French `fr-DZ` with LTR support.
9. Keep the UI original and operational. Avoid generic admin dashboards, glassmorphism, decorative gradients, bento layouts, and visual clutter.
10. Use small scoped branches and Draft Pull Requests. Do not commit directly to `main`, force-push, rewrite history, enable auto-merge, or merge without reviewer approval.
11. Run all validation required by the active execution pack before claiming success. Never represent queued, skipped, missing, or `in_progress` CI as passing.
12. Preserve typed frontend gateways under `src/platform/tauri/**`. React components must not scatter raw Tauri invocation or execute SQL.
13. Rust services own validation, authorization, company scope, idempotency, fixed-point calculations, audit, and transactions.
14. PHASE 09 work must preserve historical render reproducibility, validate backup/restore before replacement, and keep private data local.
15. PHASE 10 distribution work must preserve user data across upgrade/uninstall and must not commit signing material.

# POSMAN repository instructions

These rules apply to every implementation agent working in this repository.

## Mandatory continuity bootstrap

Before planning, reviewing, or implementing project work:

1. Read `docs/continuity/PROJECT-MEMORY-INDEX.md` and follow its mandatory reading order.
2. Read `docs/continuity/CURRENT-STATE.md`, `docs/continuity/AI-OPERATING-CONTRACT.md`, `docs/continuity/MASTER-ROADMAP-PHASES-01-10.md`, `docs/continuity/DECISION-REGISTER.md`, and `docs/continuity/PROJECT-TREE.md` before proposing project work.
3. Read `docs/continuity/RECOVERY-PROMPT.md` after a lost conversation, account change, or agent handoff.
4. Verify the checkpoint against the live `main` SHA, merged and open Pull Requests, Git history, changed files, and completed CI. Live accepted repository evidence outranks a stale checkpoint.
5. Distinguish accepted, verified work from reported, proposed, deferred, and rejected work. A roadmap entry is not authorization.
6. Keep the memory package current through a separate docs-only Draft PR after each newly accepted phase; never record an unmerged phase as accepted.
7. The current next candidate is PHASE 05. It remains planned, unstarted, and unauthorized until the user explicitly starts it.

## Standing repository rules

1. Read `docs/spec/POSMAN-Blueprint-v1.md` before changing domain logic or architecture.
2. Keep POSMAN Windows-first, offline, local-first, and based on bundled SQLite. Do not introduce a server, cloud dependency, telemetry, online account, subscription, or external database without explicit approval.
3. Never use floating-point storage for money, prices, costs, tax rates, discounts, or quantities. Follow the documented fixed-point scales.
4. Never update inventory only through `stock_balances`; every inventory change originates in the append-only `stock_movements` ledger.
5. Never mutate or delete posted commercial documents, posted journal entries, stock movements, rendered historical documents, or audit logs. Corrections use reversal, return, credit, or compensating records.
6. Never hardcode tax rates, accounting account numbers, or posting mappings as permanent program logic.
7. This repository is public. Never commit secrets, credentials, tokens, private keys, real `.env` files, production or customer databases, backups, logs containing private data, real company data, or customer data.
8. Preserve Arabic as the default language, RTL correctness, and French/LTR support.
9. Keep the UI original and operational. Avoid generic admin dashboards, glassmorphism, decorative gradients, bento layouts, and visual clutter.
10. Use small scoped branches and Draft Pull Requests. Do not commit directly to `main`, force-push, rewrite unrelated history, enable auto-merge, or merge without reviewer approval.
11. Do not edit a released migration. Add a new ordered migration for every authorized correction.
12. Run all validation required by the active execution pack before claiming success. Never represent queued, skipped, or `in_progress` CI as passing.
13. Preserve the accepted typed frontend gateway. React components must not scatter raw Tauri invocation or execute SQL.
14. Preserve the read-only `get_runtime_status` boundary until a later phase explicitly authorizes another command.

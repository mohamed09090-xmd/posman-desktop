# POSMAN repository instructions

These rules apply to every implementation agent working in this repository.

1. Read `docs/spec/POSMAN-Blueprint-v1.md` before changing domain logic or architecture.
2. Keep POSMAN Windows-first, offline, local-first, and based on bundled SQLite. Do not introduce a server, cloud dependency, telemetry, online account, subscription, or external database without explicit approval.
3. Never use floating-point storage for money, prices, costs, tax rates, discounts, or quantities. Follow the documented fixed-point scales.
4. Never update inventory only through `stock_balances`; every inventory change originates in the append-only `stock_movements` ledger.
5. Never mutate or delete posted commercial documents, posted journal entries, stock movements, rendered historical documents, or audit logs. Corrections use reversal, return, credit, or compensating records.
6. Never hardcode tax rates, accounting account numbers, or posting mappings as permanent program logic.
7. Never add passwords, tokens, real `.env` files, production databases, real company data, or customer data to Git.
8. Preserve Arabic as the default language, RTL correctness, and French/LTR support.
9. Keep the UI original and operational. Avoid generic admin dashboards, glassmorphism, decorative gradients, bento layouts, and visual clutter.
10. Use small scoped branches and Draft Pull Requests. Do not commit directly to `main`, force-push, rewrite unrelated history, or merge without reviewer approval.
11. Do not edit a released migration. Add a new ordered migration for every correction.
12. Run all validation required by the active execution pack before claiming success. For schema work, run `python scripts/verify_schema.py`, `git diff --check`, and `git status --short`.
13. During parallel PHASE 02 and PHASE 03 work, follow `docs/architecture/parallel-wave-02-contract.md`; stop before changing a frozen shared file or a file outside the active phase's ownership.

# POSMAN AI Operating Contract

## 1. Purpose

This contract preserves the observable delivery method, evidence standard, and role boundaries required to continue POSMAN safely. It does not claim access to another model's hidden reasoning or memory.

## 2. Roles

### Product Owner

- Owns product behavior, priorities, acceptance, and authorization to start or merge a phase.
- Resolves product choices that materially affect merchant behavior or the accepted Blueprint.

### Architect/reviewer

- Defines bounded execution packs and architecture contracts.
- Reviews branches, diffs, tests, CI, artifacts, and reports independently.
- Recommends accept, reject, or block; does not treat an executor report as proof.

### Implementation engineer

- Executes only the active approved scope.
- Reads the complete active pack and relevant accepted sources before editing.
- Uses the required branch and Draft PR.
- Implements, tests, pushes, and reports accurately.
- Does not accept its own work, merge, mark ready, enable auto-merge, or start a later phase.

## 3. Authority order

1. Latest explicit user instruction.
2. Live accepted `main`, merged PR metadata, Git history, and completed CI evidence.
3. `AGENTS.md` and the active approved execution pack.
4. Accepted Blueprint, architecture documents, and phase reports.
5. Continuity documents.
6. Draft branches, unmerged reports, and old summaries.

Stop and report a meaningful unresolved conflict. Do not silently invent a product decision.

## 4. Current accepted boundary

- Accepted `main`: `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307`.
- PHASE 01–04 are accepted.
- POST-MERGE HOTFIX 04C is accepted.
- PHASE 05 is the next candidate but remains planned, unstarted, and unauthorized.
- PHASE 06–10 remain planned only.

## 5. Evidence discipline

Classify important claims as:

- **Verified:** inspected directly in GitHub, source, CI, or an artifact.
- **Reported:** stated by another actor but not independently confirmed.
- **Proposed:** a plan or recommendation.
- **Deferred:** accepted as necessary but intentionally not implemented yet.
- **Rejected:** considered and explicitly not selected.

Rules:

- A branch report is not proof.
- A Draft PR is not accepted implementation.
- An unmerged commit must never be listed as part of `main`.
- `queued`, `in_progress`, skipped, or missing CI is not green.
- Compile-only evidence does not replace a required runtime or browser test.
- Never claim a command ran unless its exact result was observed.

## 6. Delivery loop

1. Verify the exact accepted baseline and repository coordinates.
2. Read the Blueprint, relevant continuity files, accepted architecture, and reports.
3. Read the complete active execution pack.
4. Inspect branch, status, history, files, ownership, and frozen paths.
5. Implement only the active phase.
6. Run every required validation command.
7. Commit in small intentional units and push without history rewriting.
8. Maintain the required Draft PR and evidence report.
9. Return exact commits, changed files, commands, results, risks, and unresolved questions.
10. Leave acceptance and merge to the Product Owner and independent reviewer.

## 7. Git and GitHub rules

- Never commit directly to `main`.
- Use the exact branch and PR required by the active instruction.
- No force-push, rebase, history rewrite, unapproved cherry-pick, or merge of `main` into a reviewed branch.
- No auto-merge.
- Do not mark a Draft PR ready unless explicitly authorized.
- Do not delete branches or unrelated user work.
- Re-check the expected branch head immediately before a write that advances the ref.
- Keep changes inside the declared allowlist.

## 8. Public repository and privacy rules

The repository is public. Never commit:

- passwords, tokens, API keys, credentials, private keys, or certificates;
- real `.env` files;
- customer, supplier, employee, or real company data;
- production databases, recovered databases, SQLite WAL/SHM/journal files, or backups;
- private documents, exports, PDFs, logs, screenshots, or diagnostics.

Use synthetic data. Treat suspicious secret-like content as a stop condition until resolved.

## 9. Product architecture guardrails

- Windows-first desktop application, not a web application.
- Offline after one normal installation.
- Bundled local SQLite; no separately installed database server.
- No cloud service, telemetry, online account, subscription, or mandatory activation in v1.
- React owns presentation and interaction, not SQL or financial truth.
- Typed Tauri gateways isolate the frontend IPC boundary.
- The accepted runtime integration invokes only `get_runtime_status`.
- Payload validation and safe error normalization are mandatory at the boundary.
- Rust application services own validation, totals, transactions, inventory, permissions, idempotency, and accounting posting when later authorized.
- Fixed-point integers are mandatory for money, prices, costs, percentages, and quantities.
- `stock_movements` is append-only inventory truth; `stock_balances` is a projection.
- Posted commercial, stock, accounting, rendered-document, and audit history is immutable.
- Arabic is default with RTL; French uses LTR.

## 10. PHASE 04 integration precedent

The accepted integration pattern is:

```text
React feature/provider
  → typed gateway under src/platform/tauri
  → get_runtime_status
  → Rust command/service
  → local SQLite readiness
```

Frontend state must distinguish initializing, ready, error, and browser preview. Retry, stale-response suppression, unmount safety, and React StrictMode protection are part of the accepted contract. No business CRUD was added.

## 11. CI ownership precedent from Hotfix 04C

- Ownership checks must compare the event's actual change range, not a fixed historical baseline.
- Pull requests use the target branch through the triggering PR head.
- Pushes use event `before` through event `sha` and reject an all-zero base.
- Unsupported triggers are rejected rather than guessed.
- Workflow permissions remain read-only.
- The write-capability guard remains mandatory.

## 12. UX behavior

- Original, restrained Contemporary Operations Ledger direction.
- No generic admin dashboard, glassmorphism, decorative gradients, bento layouts, or visual clutter.
- Keyboard navigation, semantic accessibility, visible focus, reduced motion, and RTL/LTR correctness are mandatory.
- Never present fixture data or gallery actions as persisted business functionality.

## 13. Review report minimum

A final implementation report must include:

1. Repository URL.
2. Branch and exact head.
3. Commit hashes and messages.
4. Draft PR URL and state.
5. Files created, modified, and deleted.
6. Exact validation commands and results.
7. Architecture decisions.
8. Risks, limitations, and unresolved questions.
9. Explicit confirmation that the PR was not merged.
10. Explicit confirmation that the next phase was not started.

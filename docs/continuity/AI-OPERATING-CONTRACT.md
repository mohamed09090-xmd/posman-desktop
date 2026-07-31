# POSMAN AI Operating Contract

## 1. Purpose

This contract preserves the observable delivery method, evidence standard, role boundaries, and recovery-baseline rules required to continue POSMAN safely. It does not claim access to another model's hidden reasoning or memory.

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
- Uses the required branch and delivery PR.
- Implements, tests, pushes, and reports accurately.
- Does not accept its own work, merge, mark ready, enable auto-merge, or start a later phase.

## 3. Authority order

1. Latest explicit user instruction.
2. Live accepted `main` resolved from GitHub, merged PR metadata, Git history, and completed CI evidence.
3. `AGENTS.md` and the active approved execution pack.
4. Accepted Blueprint, architecture documents, and phase reports.
5. Continuity documents.
6. Delivery branches, unmerged reports, and old summaries.

Stop and report a meaningful unresolved conflict. Do not silently invent a product decision.

## 4. Stable accepted boundary

- Accepted product-code baseline through Hotfix 04C: `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307`.
- PHASE 01–04 are accepted.
- POST-MERGE HOTFIX 04C is accepted.
- Continuity Checkpoint 04 is delivered through PR #5; verify its live state on GitHub.
- Live `main` is not permanently encoded in this contract.
- PHASE 05 is the next candidate but remains planned, unstarted, and unauthorized.
- PHASE 06–10 remain planned only.

A merge of PR #5 may create a docs-only successor of the product-code baseline. Such a successor changes repository recovery documentation but does not create a product phase or product-code implementation.

## 5. Recovery-baseline rules

1. Resolve live `main` and PR #5 state from GitHub during recovery.
2. Compare live `main` with the accepted product-code baseline.
3. Before PR #5 is merged, continuity content may exist only on its branch and remains unaccepted.
4. After PR #5 is merged, a newer `main` may be accepted as the continuity-checkpoint successor only when:
   - PR #5 is verified merged; and
   - the baseline-to-`main` difference is limited to `AGENTS.md`, `docs/continuity/**`, and `docs/execution-packs/archive/**`.
5. Never classify a docs-only continuity successor as PHASE 05 or product-code work.
6. If live `main` contains product or other out-of-scope changes, report drift and stop instead of accepting it automatically.
7. A future PHASE 05 execution pack must freeze the live accepted `main` resolved at execution time. It must not blindly reuse the historical product-code baseline.

## 6. Evidence discipline

Classify important claims as:

- **Verified:** inspected directly in GitHub, source, CI, or an artifact.
- **Reported:** stated by another actor but not independently confirmed.
- **Proposed:** a plan or recommendation.
- **Deferred:** accepted as necessary but intentionally not implemented yet.
- **Rejected:** considered and explicitly not selected.

Rules:

- A branch report is not proof.
- A delivery PR is not accepted until its merge is verified.
- An unmerged commit must never be listed as part of `main`.
- `queued`, `in_progress`, skipped, or missing CI is not green.
- Compile-only evidence does not replace a required runtime or browser test.
- Never claim a command ran unless its exact result was observed.

## 7. Delivery loop

1. Resolve and verify the exact live accepted baseline and repository coordinates.
2. Read the Blueprint, relevant continuity files, accepted architecture, and reports.
3. Read the complete active execution pack.
4. Inspect branch, status, history, files, ownership, and frozen paths.
5. Implement only the active phase.
6. Run every required validation command.
7. Commit in small intentional units and push without history rewriting.
8. Maintain the required delivery PR and evidence report.
9. Return exact commits, changed files, commands, results, risks, and unresolved questions.
10. Leave acceptance and merge to the Product Owner and independent reviewer.

## 8. Git and GitHub rules

- Never commit directly to `main`.
- Use the exact branch and PR required by the active instruction.
- No force-push, rebase, history rewrite, unapproved cherry-pick, or merge of `main` into a reviewed branch.
- No auto-merge.
- Do not mark a PR ready unless explicitly authorized.
- Do not delete branches or unrelated user work.
- Re-check the expected branch head immediately before a write that advances the ref.
- Keep changes inside the declared allowlist.

## 9. Public repository and privacy rules

The repository is public. Never commit:

- passwords, tokens, API keys, credentials, private keys, or certificates;
- real `.env` files;
- customer, supplier, employee, or real company data;
- production databases, recovered databases, SQLite WAL/SHM/journal files, or backups;
- private documents, exports, PDFs, logs, screenshots, or diagnostics.

Use synthetic data. Treat suspicious secret-like content as a stop condition until resolved.

## 10. Product architecture guardrails

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

## 11. PHASE 04 integration precedent

The accepted integration pattern is:

```text
React feature/provider
  → typed gateway under src/platform/tauri
  → get_runtime_status
  → Rust command/service
  → local SQLite readiness
```

Frontend state must distinguish initializing, ready, error, and browser preview. Retry, stale-response suppression, unmount safety, and React StrictMode protection are part of the accepted contract. No business CRUD was added.

## 12. CI ownership precedent from Hotfix 04C

- Ownership checks must compare the event's actual change range, not a fixed historical baseline.
- Pull requests use the target branch through the triggering PR head.
- Pushes use event `before` through event `sha` and reject an all-zero base.
- Unsupported triggers are rejected rather than guessed.
- Workflow permissions remain read-only.
- The write-capability guard remains mandatory.

## 13. UX behavior

- Original, restrained Contemporary Operations Ledger direction.
- No generic admin dashboard, glassmorphism, decorative gradients, bento layouts, or visual clutter.
- Keyboard navigation, semantic accessibility, visible focus, reduced motion, and RTL/LTR correctness are mandatory.
- Never present fixture data or gallery actions as persisted business functionality.

## 14. Review report minimum

A final implementation report must include:

1. Repository URL.
2. Branch and exact head.
3. Commit hashes and messages.
4. Delivery PR URL and live state.
5. Files created, modified, and deleted.
6. Exact validation commands and results.
7. Architecture decisions.
8. Risks, limitations, and unresolved questions.
9. Explicit confirmation that the PR was not merged when the active instruction requires it to remain unmerged.
10. Explicit confirmation that the next phase was not started.

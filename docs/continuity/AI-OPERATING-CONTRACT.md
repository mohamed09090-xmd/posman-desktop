# POSMAN AI Operating Contract

## 1. Purpose

This contract preserves the observable delivery method, evidence standard, role boundaries, recovery rules, and architecture guardrails required to continue POSMAN safely. It does not claim access to another model's hidden reasoning or memory.

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
- Uses the required branch and Draft Pull Request.
- Implements, tests, pushes, and reports accurately.
- Does not accept its own work, merge, mark ready, enable auto-merge, or start a later phase without explicit authorization.

## 3. Authority order

1. Latest explicit user instruction.
2. Live accepted `main`, merged Pull Request metadata, Git history, and completed CI evidence.
3. `AGENTS.md` and the active approved execution pack.
4. Accepted Blueprint, architecture documents, and phase reports.
5. Continuity documents.
6. Delivery branches, unmerged reports, agent claims, and old summaries.

Stop and report a meaningful unresolved conflict. Do not silently invent a product decision.

## 4. Stable accepted boundary

- Accepted product baseline through PHASE 08: `5821004c6f3a51b4b0116ec3dbc1b9c2264ccf69`.
- PHASE 01–08 and POST-MERGE HOTFIX 04C are accepted.
- PHASE 09 is the next candidate and remains planned, unstarted, and unauthorized.
- PHASE 10 remains planned and unauthorized.
- Live `main` must still be resolved during every recovery; this SHA is a checkpoint coordinate, not a permanent assertion about future repository state.

## 5. Recovery rules

1. Resolve live `main`, merged/open Pull Requests, branches, and workflow state directly from GitHub.
2. Compare live `main` with the checkpoint baseline.
3. Verify accepted phase merges #1, #2, #3, #4, #6, #7, #8, #10, #11, and #12.
4. Treat an unmerged branch or report as unaccepted even when its tests are green.
5. Report product-code or documentation drift instead of silently accepting it.
6. Start PHASE 09 only from the live accepted baseline resolved when execution is authorized.
7. Recovery itself is read-only unless the Product Owner explicitly authorizes a corrective documentation or implementation task.

## 6. Evidence discipline

Classify important claims as:

- **Verified:** inspected directly in GitHub, source, CI, or an artifact.
- **Reported:** stated by another actor but not independently confirmed.
- **Proposed:** a plan or recommendation.
- **Deferred:** accepted as necessary but intentionally not implemented yet.
- **Rejected:** considered and explicitly not selected.

Rules:

- A branch report is not proof.
- A Pull Request is not accepted until its merge is verified.
- An unmerged commit must never be listed as part of `main`.
- `queued`, `in_progress`, skipped, missing, or cancelled CI is not green.
- Compile-only evidence does not replace required runtime, database, browser, restore, or installation tests.
- Never claim a command ran unless its exact result was observed.
- Documentation must describe the live accepted implementation, not the state that existed when the file was first written.

## 7. Delivery loop

1. Resolve and verify the exact live accepted baseline.
2. Read the Blueprint, continuity package, relevant accepted architecture, phase reports, and active execution pack.
3. Inspect branch, status, history, files, ownership, dependencies, and frozen paths.
4. Implement only the active scope.
5. Run every required validation command.
6. Commit intentional changes and push without rewriting history.
7. Maintain the required Draft Pull Request and evidence report.
8. Return exact commits, changed files, commands, results, risks, and unresolved decisions.
9. Leave acceptance and merge to the Product Owner and independent reviewer.

## 8. Git and GitHub rules

- Never commit directly to `main`.
- Use a small scoped branch and Draft Pull Request.
- No force-push, rebase, reset, history rewrite, unapproved cherry-pick, or merge of `main` into a reviewed branch.
- No auto-merge.
- Do not mark a Pull Request ready unless explicitly authorized.
- Do not delete branches or unrelated user work.
- Re-check the expected branch head immediately before a write that advances a ref.
- Keep changes inside the declared allowlist.
- Accepted migrations `0001`–`0006` are immutable.

## 9. Public repository and privacy rules

Never commit:

- passwords, tokens, API keys, credentials, private keys, certificates, or signing material;
- real `.env` files;
- customer, supplier, employee, authentication, or real company data;
- production/recovered databases, SQLite WAL/SHM/journal files, or backups;
- private documents, exports, PDFs, logs, screenshots, or diagnostics.

Use synthetic data. Treat suspicious secret-like content as a stop condition until resolved.

## 10. Product architecture guardrails

- Windows-first desktop application, not a hosted web application.
- Offline after one normal installation.
- Bundled local SQLite; no separately installed database server.
- No cloud service, telemetry, online account, subscription, or mandatory activation in v1.
- React owns presentation and interaction, not SQL or financial truth.
- Typed Tauri gateways isolate frontend IPC under `src/platform/tauri/**`.
- Rust services own validation, authentication, authorization, company scope, transactions, inventory, totals, idempotency, audit, accounting, and protected file operations.
- Fixed-point integers are mandatory for money, prices, costs, percentages, and quantities.
- `stock_movements` is append-only inventory truth; `stock_balances` is a rebuildable projection.
- Posted commercial, stock, accounting, rendered-document, and audit history is immutable.
- Arabic is default with RTL; French uses LTR.

## 11. Accepted phase architecture

### PHASE 04 integration precedent

```text
React feature/provider
  → typed gateway under src/platform/tauri
  → registered Tauri command
  → Rust service
  → local SQLite
```

Payload validation, safe error normalization, stale-response protection, unmount safety, and controlled development adapters remain required.

### PHASE 05 security precedent

- Argon2id password handling.
- Local authenticated sessions and inactivity lock.
- Company-scoped permissions, optimistic concurrency, safe errors, and audit.
- Last-system-administrator protection.

### PHASE 06/07 commerce precedent

- SQLite `IMMEDIATE` transactions for posting.
- Company-scoped idempotency keys bound to stable request hashes.
- Deterministic fixed-point arithmetic.
- Append-only stock effects and transactional aggregate transformation limits.
- Corrections use compensating documents or movements.

### PHASE 08 accounting precedent

- Source, stock, journal, audit, and idempotency success share one atomic transaction where required.
- Failed posting attempts retain only safe metadata in a separate short transaction.
- Posted journals are immutable; correction uses linked reversal.
- Missing or ambiguous configuration fails closed without partial posting.

## 12. PHASE 09 guardrails

PHASE 09 must not be treated as a simple UI addition.

- Template HTML/CSS must be sanitized and versioned; arbitrary JavaScript is forbidden.
- Historical render snapshots and content hashes must make reprints reproducible.
- Generated documents and backups remain under controlled application-owned paths.
- Frontend code must not receive unrestricted filesystem access.
- Active WAL databases must be backed up with a safe SQLite snapshot/backup mechanism.
- Restore must verify schema compatibility, integrity, and file hash before replacement.
- Restore must create and verify a safety backup of the current database first.
- Audit presentation must redact unsafe internal details and obey permissions.

## 13. CI ownership precedent

- Ownership checks use the triggering event's actual change range, not a fixed historical baseline.
- Pull Requests compare the target branch through the triggering head.
- Pushes compare event `before` through event `sha` and reject invalid all-zero bases.
- Unsupported triggers are rejected rather than guessed.
- Workflow permissions remain read-only unless an explicitly reviewed requirement proves otherwise.
- Write-capability guards remain mandatory.

## 14. UX behavior

- Original, restrained Contemporary Operations Ledger direction.
- No generic admin dashboard, glassmorphism, decorative gradients, bento layouts, or visual clutter.
- Keyboard navigation, semantic accessibility, visible focus, reduced motion, and RTL/LTR correctness are mandatory.
- Never present fixture data or test adapters as persisted business functionality.
- Reports, printing, backup, and restore must use human language and clearly communicate destructive or irreversible consequences.

## 15. Review report minimum

A final implementation report must include:

1. Repository URL.
2. Branch and exact head.
3. Commits and messages.
4. Pull Request URL and live state.
5. Files created, modified, and deleted.
6. Exact validation commands and results.
7. Architecture and security decisions.
8. Risks, limitations, and unresolved questions.
9. Confirmation that no unauthorized merge, history rewrite, or later-phase work occurred.
10. For PHASE 09, explicit backup/restore safety, historical render reproducibility, and private-data handling evidence.

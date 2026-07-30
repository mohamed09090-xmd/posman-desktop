# POSMAN — PHASE 04 Delivery Report

## 1. حالة التنفيذ

```text
PHASE 04: BLOCKED / NOT READY FOR REVIEW
```

Patch 04A was implemented on the existing branch and Pull Request. The original cross-phase ownership conflict and missing frontend dependency were resolved. The typed bridge, StrictMode reconciliation, runtime UI states, accessibility behavior, tests, documentation, and read-only integration workflow are present.

The phase is not marked ready because the required final-head GitHub Actions evidence could not be produced. On the final implementation head, every GitHub-hosted workflow failed before any job step started and without downloadable logs, including the unchanged SQLite Schema Verification workflow on both Ubuntu and Windows. A rerun of that unchanged schema workflow produced the same zero-step failure. No CI success is inferred from these runs.

A source-equivalent earlier head completed the full UI/type/build/integration/E2E/Axe workflow successfully, but it is not substituted for the requirement that all workflows be green on one final head.

## 2. Repository

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Branch: `phase/04-frontend-runtime-integration`
- Draft Pull Request: `https://github.com/mohamed09090-xmd/posman-desktop/pull/6`
- Repository visibility: private
- Write permission: verified before Patch 04A writes

## 3. Baseline verified

Original accepted baseline:

```text
f4cda85b24f9d69ebb0442c02f8a037da8ba9baf
```

Patch 04A start gate was verified before writing:

```text
main == f4cda85b24f9d69ebb0442c02f8a037da8ba9baf
branch head == 3362fd755568322f3aeaddba419fa0ac529b047e
PR #6 == open, Draft, unmerged, base=main
```

No baseline reset, force update, rebase, or history rewrite was performed. The branch remains based on the accepted PHASE 03 squash commit and is not behind it.

## 4. Branch and final head SHA

- Branch: `phase/04-frontend-runtime-integration`
- Final implementation/evidence head before this report commit:

```text
f3c5cc2139220fff404c2853cec7c8e9b39d730b
```

The report commit becomes the branch head after this document is updated. The exact final branch SHA is recorded in PR #6 and in the delivery response.

## 5. Draft Pull Request

- URL: `https://github.com/mohamed09090-xmd/posman-desktop/pull/6`
- Title: `[Phase 04] POSMAN frontend-runtime integration gate`
- Base: `main`
- Head: `phase/04-frontend-runtime-integration`
- Open: yes
- Draft: yes
- Merged: no
- Ready for review: no
- Auto-merge: disabled

No replacement branch or Pull Request was created.

## 6. Commits

Existing PHASE 04 history retained unchanged:

1. `918333b98a4141cc67deb217a20ffd2151fe57c7` — `feat(integration): add typed local runtime gateway`
2. `8f298edc2cfe0ae793c079f9b1383f54352e13c2` — `feat(ui): surface localized runtime readiness`
3. `85d6a03d83c99ccdb8ba2fcee0d60d60b51410f4` — `test(integration): verify runtime boundary and IPC states`
4. `76cc76f92633cf20027cec39404961081341932c` — `chore(ci): bootstrap locked frontend dependency`
5. `93bcb40cde8d75b1220bbe293e5dad4caea67896` — `test(ui): verify localized runtime states and retry`
6. `3362fd755568322f3aeaddba419fa0ac529b047e` — `docs(integration): record Phase 04 blocker`

Patch 04A commits:

7. `71f56550099fc34296704a839f35685ea497a702` — `ci(frontend): prepare locked Tauri API dependency`
8. `17f9b4d9f8f330941144c25f55f92e9ae1d401cd` — `chore(frontend): add locked Tauri API dependency`
9. `99e09095439ff8155cf74a49a496930ef2b46166` — `ci(frontend): remove temporary lock preparation workflow`
10. `76e4d26a70fe4106c844b7880105b8dbd0a37a6e` — `fix(integration): reconcile runtime bridge and StrictMode UI`
11. `c906276c0bcc550263c44782f3e2d7b26a4b6c7b` — `ci(integration): reconcile cross-phase guards and gate runtime integration`
12. `76347e87c14f9f67fe69bf331e028f17bbd452e3` — `style(runtime): apply rustfmt to IPC test`
13. `57d00edc38b5fc9c8113d69308c02baa833ec114` — `docs(integration): record frontend-runtime boundary architecture`
14. `99dfae47ca72891074c85dc8d4a1613ad1ee08cf` — `ci(integration): invoke read-only gate from UI workflow`
15. `421bf8f6b695023f71fec31da1cdbbe2df5550a8` — `fix(ci): initialize integration evidence paths at runtime`
16. `f3c5cc2139220fff404c2853cec7c8e9b39d730b` — `ci(integration): keep the integration gate standalone`

Commit 14 was a bounded attempt to make the newly added workflow callable before GitHub registered its standalone PR trigger. Once the standalone workflow was registered, commit 16 removed the caller to avoid duplicate CI consumption. The final UI workflow does not call the integration workflow.

## 7. Files created/modified/deleted

### Created

```text
.github/workflows/integration-ci.yml
docs/architecture/frontend-runtime-integration.md
docs/PHASE-04-REPORT.md
src/platform/tauri/runtime-environment.ts
src/platform/tauri/runtime-status.ts
src/features/runtime/RuntimeStatusProvider.tsx
src/features/runtime/RuntimeStatusIndicator.tsx
src/features/runtime/runtime-state.ts
src/features/runtime/runtime-status.css
tests/integration/runtime-status.test.ts
```

### Modified

```text
.github/workflows/ui-ci.yml
.github/workflows/runtime-ci.yml
package.json
package-lock.json
src-tauri/src/lib.rs
src/app/AppRoot.tsx
src/components/layout.tsx
src/i18n/dictionaries.ts
tests/ui/i18n-fixtures.test.ts
tests/e2e/run_ui_gallery.py
```

### Deleted from the final tree

```text
.github/workflows/phase-04-lock-prep.yml
```

The temporary file exists only in history and is absent from the final tree. No unrelated user file was deleted.

## 8. Frontend-runtime architecture

The implementation is divided into four responsibilities:

1. `src/platform/tauri/runtime-status.ts` owns the exact command name, response validator, and safe error normalization.
2. `src/platform/tauri/runtime-environment.ts` selects the canonical DEV test invoker, the official Tauri bridge, or browser preview.
3. `src/features/runtime/runtime-state.ts` owns initializing/ready/error/preview transitions, retry, pending suppression, StrictMode deduplication, stale-response protection, and unmount protection.
4. React provider and presentation components expose state without importing database or business logic.

The environment resolver uses:

```ts
import { invoke, isTauri } from "@tauri-apps/api/core";
```

It does not inspect `window.__TAURI_INTERNALS__`. The only test hook is:

```text
window.__POSMAN_DEV_RUNTIME_INVOKER__
```

It is read only under `import.meta.env.DEV` and is not a query-string, HTTP, or production data input.

Detailed architecture is documented in `docs/architecture/frontend-runtime-integration.md`.

## 9. Exact IPC contract

Command:

```text
get_runtime_status
```

Arguments:

```text
none
```

Successful response:

```ts
interface RuntimeStatus {
  databaseReady: boolean;
  schemaVersion: string;
  migrationCount: number;
  foreignKeysEnabled: boolean;
  journalMode: string;
}
```

Known structured Rust error:

```ts
interface RuntimeCommandError {
  code: string;
  message: string;
}
```

Known safe Rust code:

```text
RUNTIME_STATUS_UNAVAILABLE
```

The frontend does not hardcode `0004` as the only acceptable future schema version.

## 10. Runtime validation and error handling

The boundary validator requires:

- non-null object and not an array;
- booleans for `databaseReady` and `foreignKeysEnabled`;
- non-empty trimmed strings for `schemaVersion` and `journalMode`;
- finite, non-negative integer for `migrationCount`.

It rejects coercion and malformed values with:

```text
RUNTIME_STATUS_INVALID_RESPONSE
```

Unknown thrown values and unrecognized codes become:

```text
RUNTIME_STATUS_REQUEST_FAILED
```

A structurally valid but operationally unavailable payload becomes:

```text
RUNTIME_STATUS_NOT_READY
```

Raw Rust messages, paths, SQL, stack traces, and user-directory details are not rendered.

Initial activation is deferred to a microtask. `activate → deactivate → activate` before that microtask produces one gateway invocation. Retry is immediate, disabled while pending, and cannot be overwritten by a stale response.

## 11. UI/UX and i18n behavior

Implemented states:

```text
initializing
ready
error
preview
```

- Initializing, ready, and preview use polite status semantics.
- Ready has `role="status"` and `aria-live="polite"`.
- Terminal error uses `role="alert"` without focus theft.
- Retry is a native button with visible focus and a disabled pending state.
- Browser preview never claims SQLite readiness.
- Arabic `ar-DZ` remains default and RTL.
- French `fr-DZ` remains LTR.
- The primary runtime status is not ellipsized and may wrap.
- Secondary migration/schema detail may be hidden or ellipsized at constrained widths.
- The Operations Ledger visual direction and existing tokens are retained.

The E2E suite measures the primary status at 1280×800 and 1024×640 and requires:

```text
scrollWidth <= clientWidth + 1
scrollHeight <= clientHeight + 1
textOverflow != ellipsis
whiteSpace != nowrap
```

## 12. TypeScript test output

Successful source-equivalent UI workflow:

- Head: `c906276c0bcc550263c44782f3e2d7b26a4b6c7b`
- Run: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509212278`

Results:

```text
npm run typecheck: PASS
npm run build: PASS
npm run test:ui: 7 passed, 0 failed
npm run test:integration: 33 passed, 0 failed
```

The 33 integration tests include the explicit StrictMode case:

```text
activate/deactivate/activate before microtask -> exactly one invocation
```

They also cover exact command/no arguments, future schema values, null/arrays, each invalid field type, negative/fractional/non-finite counts, empty strings, safe and unknown errors, initializing→ready, retry→ready, not-ready payload, stale response, unmount, and preview.

Supplemental executor-local smoke verification ran five focused tests against the same gateway/controller source and reported:

```text
5 passed, 0 failed
```

This supplemental run is not used as a replacement for final-head GitHub CI.

Final-head TypeScript output is unavailable because the final GitHub-hosted jobs did not start any steps.

## 13. Rust IPC test output on Ubuntu and Windows

The real, non-ignored IPC test remains in `src-tauri/src/lib.rs`. It:

- builds the configured mock Tauri application;
- executes setup and migrations;
- creates a mock webview;
- invokes `get_runtime_status` through `tauri::test::get_ipc_response`;
- deserializes `serde_json::Value`;
- asserts the camelCase payload and non-empty journal mode.

The inherited offline scan is preserved by assembling the Windows local IPC origin without a literal external-network URL.

Post-rustfmt Rust execution results on the final head:

```text
Ubuntu Rust tests: NOT STARTED
Windows Rust tests: NOT STARTED
Rust 1.85 MSRV: NOT STARTED
Clippy: NOT STARTED
Native desktop check: NOT STARTED
```

The corresponding jobs failed before step 1 and had no logs. No Rust pass is claimed.

## 14. E2E/Axe/overflow evidence

Successful source-equivalent run:

```text
https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509212278
```

E2E result:

```text
UI browser evidence passed
```

Scenarios with Axe reports:

1. Arabic Today
2. French Today
3. Arabic invoice
4. Product list/drawer
5. Sales cycle
6. Arabic runtime ready
7. French runtime ready
8. Arabic runtime error
9. Arabic runtime retry
10. French runtime preview
11. Malformed runtime response

For every report:

```text
violations: 0
incomplete: 0
unresolved critical/serious incomplete: 0
```

The successful E2E suite also passed:

- exact runtime call count of one for ready under StrictMode;
- keyboard and pointer retry;
- raw path/SQL suppression;
- command bar visibility;
- primary status no-clipping checks at 1280×800 and 1024×640;
- workspace rail label checks;
- no page-level horizontal overflow;
- no console errors or unhandled page errors.

Required screenshots were generated in the artifact:

```text
phase-04-ar-runtime-ready.png
phase-04-fr-runtime-ready.png
phase-04-ar-runtime-retry.png
phase-04-fr-runtime-preview.png
```

The final-head E2E job did not start; therefore the earlier artifact is reported as source-equivalent evidence, not final-head acceptance evidence.

## 15. GitHub Actions run links and job results

### Successful source-equivalent UI run

- UI Foundation CI #48: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509212278`
- Conclusion: success
- Typecheck/build/UI tests/integration tests/E2E/Axe/whitespace/clean worktree/artifact: all success

### Final implementation head `f3c5cc2139220fff404c2853cec7c8e9b39d730b`

- Frontend Runtime Integration #8: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509932919`
  - failure before all steps; no logs
- UI Foundation CI #53: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509932939`
  - failure before all steps; no logs
- Desktop Bootstrap CI #104: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509932912`
  - Ubuntu failure before all steps; no logs
  - Windows failure before all steps; no logs
- Runtime CI #34: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509932927`
  - Ubuntu failure before all steps; no logs
  - Windows failure before all steps; no logs
  - Rust 1.85 MSRV failure before all steps; no logs
- SQLite Schema Verification #46: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30509932922`
  - Ubuntu failure before all steps; no logs
  - Windows failure before all steps; no logs
  - rerun attempted; both jobs again failed before all steps and without logs

Because an unchanged cross-platform schema workflow exhibits the same zero-step behavior, these final failures cannot be represented as test failures. The exact GitHub-hosted runner/account cause is not exposed by the available job metadata.

## 16. Artifact metadata

Successful source-equivalent UI artifact:

```text
Name: phase-03-ui-evidence
Artifact ID: 8746465612
Size: 1,081,894 bytes
Digest: sha256:2d355d7d8a64ac81fb29d9bb859189dd24ac8c52a40271252e7a240d4e1ce23c
Files uploaded: 22
Expired: false
```

The artifact contains 11 Axe JSON reports, summary JSON, required screenshots, prior UI screenshots, and Vite output.

Required final integration artifact:

```text
Name: phase-04-integration-evidence
Status: NOT PRODUCED
ID/size/digest: unavailable
```

The final integration job never started, so no metadata is invented.

## 17. Ownership/frozen-file evidence

The final net diff from the accepted baseline contains 20 paths, all within the Patch 04A allowlist:

```text
.github/workflows/integration-ci.yml
.github/workflows/runtime-ci.yml
.github/workflows/ui-ci.yml
docs/PHASE-04-REPORT.md
docs/architecture/frontend-runtime-integration.md
package.json
package-lock.json
src-tauri/src/lib.rs
src/app/AppRoot.tsx
src/components/layout.tsx
src/features/runtime/**
src/i18n/dictionaries.ts
src/platform/tauri/**
tests/e2e/run_ui_gallery.py
tests/integration/**
tests/ui/i18n-fixtures.test.ts
```

No migration, schema, production runtime command, capability, CSP, Cargo manifest/lock, accepted design source, or unrelated document changed.

Workflow guard migration:

- UI CI now protects only `database/**` and `scripts/verify_schema.py` in its event-scoped ownership step while retaining all validation work.
- Runtime CI retains the database-source guard and removes the obsolete frontend freeze while retaining npm, Rust, native, MSRV, and worktree validation.
- Integration CI is read-only, uses the accepted PHASE 03 baseline, and enforces the precise PHASE 04 path allowlist.

No workflow with `contents: write` remains in the final tree.

Temporary lock workflow history:

- Created in commit `71f5655...` with branch-only trigger, package-only diff guard, stationary remote-head check, and one fast-forward push.
- Produced only package manifest/lock changes in `17f9b4d...`.
- Deleted immediately in `99e0909...`.

## 18. Dependencies

Added exact runtime dependency:

```text
@tauri-apps/api@2.11.1
```

Added script:

```text
test:integration = node --experimental-strip-types --test tests/integration/*.test.ts
```

`package-lock.json` was generated by npm and contains the exact package entry and integrity data.

No other dependency was added. In particular, no Axios/HTTP client, state framework, schema framework, Tauri SQL plugin, or Rust dependency was introduced.

## 19. Risks and limitations

1. Required final-head CI is not green because GitHub-hosted jobs do not start; acceptance cannot proceed.
2. Rust IPC, clippy, native desktop, Windows manifest, and MSRV results are not available after the formatting fix.
3. Production `dist/**` test-hook absence is enforced by Integration CI but was not executed on the final head. Source separation uses the required DEV guard, but no final bundle-scan pass is claimed.
4. The successful UI/E2E artifact is from a source-equivalent pre-document/workflow head, not the final head.
5. A Rust setup failure before WebView creation cannot be rendered by React; the implemented error state covers failures after the frontend has loaded.
6. PR #6 must remain Draft and unmerged until GitHub Actions can execute and all required workflows pass on one head.

Unresolved operational question:

```text
Why are GitHub-hosted jobs failing before step 1 with no logs for every workflow, including unchanged schema validation?
```

The repository owner may need to inspect the Actions UI/account spending or hosted-runner availability. No repository code workaround is appropriate.

## 20. Out-of-scope confirmation

Not implemented:

```text
Company setup
Authentication or users
Roles/permissions UI
Products/families/customers/suppliers CRUD
Opening stock writes
Purchases or sales
Sales-order transformation
Stock movements, CUMP, or negative-stock rules
Accounting or journal UI
Tax, discount, or numbering logic
PDF, printing, backup, installer, signing, update
Cloud, telemetry, HTTP API, or runtime network
Additional Tauri command
Migration, schema reset, or downgrade
```

Existing gallery actions remain demonstration-only.

## 21. Force-push/rebase/merge confirmation

- No force-push was used.
- No rebase was used.
- No commit history was rewritten or deleted.
- No Pull Request was merged.
- No direct commit was made to `main`.
- Auto-merge was not enabled.
- PR #6 was not changed to Ready for review.

## 22. PR final state

```text
Pull Request: #6
URL: https://github.com/mohamed09090-xmd/posman-desktop/pull/6
State: open
Draft: true
Merged: false
Base: main
Head: phase/04-frontend-runtime-integration
Review status: BLOCKED / NOT READY FOR REVIEW
```

The exact final head after this report commit is recorded in the delivery response and PR metadata.

## 23. PHASE 05 confirmation

```text
PR #6 remains open, Draft, and unmerged.
No auto-merge.
No force-push/rebase.
No PHASE 05.
No business CRUD.
No additional Tauri command.
No contents:write workflow remains in the final tree.
```

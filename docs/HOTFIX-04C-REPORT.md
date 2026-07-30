# POST-MERGE HOTFIX 04C Report

## Status and scope

PHASE 04 remains accepted and merged. This post-merge hotfix corrects two CI-review findings without rolling back or changing PHASE 04 product source.

Accepted `main` baseline at execution start:

```text
a86635a8bc7dd8f3b7683f8f2f33d40c454441bb
```

Hotfix branch:

```text
hotfix/04c-integration-ci-event-scope
```

## Original Codex P2 findings

1. Ownership baseline:
   https://github.com/mohamed09090-xmd/posman-desktop/pull/6#discussion_r3682013772
2. Reusable workflow checkout:
   https://github.com/mohamed09090-xmd/posman-desktop/pull/6#discussion_r3682013783

Both findings were accepted by the architect and required correction before PHASE 05.

## Why the fixed baseline was unsafe

The Integration ownership guard compared every future run against the accepted PHASE 03 SHA:

```text
f4cda85b24f9d69ebb0442c02f8a037da8ba9baf
```

After later accepted work added any file outside the Phase 04 allowlist, a subsequent integration-related change would include that unrelated history in the ownership diff. The workflow could then fail permanently on files that were not part of the triggering pull request or push.

Hotfix 04C removes that fixed baseline and scopes ownership to the event that caused the workflow run.

## Event-scoped ownership ranges

For `pull_request`:

```text
base = origin/${{ github.base_ref }}
head = ${{ github.event.pull_request.head.sha }}
range = base...head
```

The workflow resolves the base commit and verifies that both base and head are ancestors of the checked-out pull-request head.

For `push`:

```text
base = ${{ github.event.before }}
head = ${{ github.sha }}
range = before..head
```

A zero `before` SHA is rejected. Unsupported events are rejected.

The same resolved range is used by:

- the Python ownership guard;
- evidence metadata as `ownership_range`;
- the final event-scoped `git diff --check`.

## Why `workflow_call` was removed

Repository inspection found no workflow that calls:

```yaml
uses: ./.github/workflows/integration-ci.yml
```

The declared reusable interface was therefore unused and misleading. Its event assumptions could skip both checkout steps for a non-`pull_request`/non-`push` caller and leave an empty runner workspace. Hotfix 04C removes `workflow_call` rather than adding a false fallback or claiming unsupported reusable-workflow behavior.

The workflow now supports only:

- `pull_request`;
- `push` to `main`.

## Guard preservation

- Workflow permissions remain `contents: read`.
- The write-permission guard remains enabled.
- The Phase 04 path allowlist remains active against the triggering change only.
- `docs/HOTFIX-04C-REPORT.md` is added narrowly to the exact allowlist because this required report is part of the hotfix-owned change.
- Frontend, Rust, native desktop, SQLite, network, bundle, accessibility, and worktree checks are unchanged.

## Files changed

- `.github/workflows/integration-ci.yml`
- `docs/architecture/frontend-runtime-integration.md`
- `docs/HOTFIX-04C-REPORT.md`

No other file is modified.

## Validation recorded before commit

- YAML parsing: PASS.
- `workflow_call` absent from Integration workflow: PASS.
- Fixed PHASE 03 SHA absent from Integration workflow: PASS.
- Pull-request three-dot ownership range present: PASS.
- Push two-dot ownership range present: PASS.
- Zero-before push rejection present: PASS.
- Unsupported-event rejection present: PASS.
- `contents: read` preserved: PASS.
- Write-permission guard preserved: PASS.
- Final `git diff --check` uses the resolved ownership range: PASS.
- Hotfix scope contains only the three authorized files: PASS.

Remote GitHub Actions results and artifact metadata are reported in the Draft Pull Request description after the final-head runs complete. This committed report does not claim CI results that did not yet exist at commit creation.

## Boundaries

- No database or migration change.
- No dependency change.
- No Tauri command.
- No PHASE 04 product-source change.
- No rollback.
- No PHASE 05 work.

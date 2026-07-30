# POSMAN Frontend ↔ Local Runtime Integration

## Scope

PHASE 04 connects the accepted React interface to the accepted local Tauri/Rust runtime through the existing read-only command `get_runtime_status`. It does not introduce business commands, CRUD, database writes, migrations, network access, cloud services, telemetry, or a SQLite connection in React.

## Why a gateway exists

React does not import or interpret the Tauri command contract throughout the component tree. The `src/platform/tauri` gateway is the single IPC boundary. It owns command invocation, runtime validation, and safe error normalization. The runtime feature consumes a small gateway interface, which keeps presentation and request lifecycle independent from Tauri implementation details and permits deterministic tests with an injected invoker.

## Exact IPC contract

Command:

```text
get_runtime_status
```

Arguments: none.

Successful camelCase response:

```ts
interface RuntimeStatus {
  databaseReady: boolean;
  schemaVersion: string;
  migrationCount: number;
  foreignKeysEnabled: boolean;
  journalMode: string;
}
```

Known structured Rust failure:

```ts
interface RuntimeCommandError {
  code: string;
  message: string;
}
```

The accepted Rust code is `RUNTIME_STATUS_UNAVAILABLE`. React never displays the raw Rust message.

## Boundary validation

The TypeScript generic attached to `invoke` is not treated as runtime validation. The gateway manually requires:

- a non-null object that is not an array;
- `databaseReady` and `foreignKeysEnabled` as booleans;
- `schemaVersion` and `journalMode` as non-empty strings after trimming;
- `migrationCount` as a finite, non-negative integer.

No coercion is accepted. Malformed data becomes `RUNTIME_STATUS_INVALID_RESPONSE`. Unknown thrown values and unrecognized codes become `RUNTIME_STATUS_REQUEST_FAILED`. Safe known codes are retained for diagnostics and tests, while paths, SQL, stack traces, and user-directory details remain hidden.

## State machine

The runtime controller exposes four states:

```text
initializing
ready
error
preview
```

`ready` requires a structurally valid response with both `databaseReady === true` and `foreignKeysEnabled === true`. A valid payload that is not operationally ready becomes `RUNTIME_STATUS_NOT_READY`.

`preview` is used when the frontend runs in an ordinary browser without a Tauri environment. It never claims that SQLite is ready.

A terminal command, bridge, readiness, or validation failure becomes `error`. The workspace notice uses an accessible native Retry button. Retry starts a real request immediately and is disabled while that request is pending.

## Tauri environment detection

Production environment detection uses the official API from the exact runtime dependency:

```ts
import { invoke, isTauri } from "@tauri-apps/api/core";
```

The frontend does not inspect `window.__TAURI_INTERNALS__` or another private implementation detail. If `isTauri()` is false, the gateway resolver returns `null`, producing `preview` without calling `invoke`.

## Canonical DEV test hook

Vite/Playwright tests use one canonical hook:

```text
window.__POSMAN_DEV_RUNTIME_INVOKER__
```

It is read only inside an `import.meta.env.DEV` branch. It accepts an invoker function supplied before page load; it does not accept query-string JSON, expose HTTP, or provide a production backdoor. The final integration workflow scans `dist/**` and fails if the hook name survives production tree-shaking.

## StrictMode and concurrency

`src/main.tsx` uses React StrictMode. The controller therefore defers only the initial activation request to a microtask. In a development `setup → cleanup → setup` cycle, cleanup invalidates the first scheduled activation before it invokes the gateway, while the final activation performs exactly one invocation.

Retry remains immediate rather than microtask-delayed. Independent sequence counters protect activation and request lifecycles. Deactivation invalidates pending results, stale responses cannot replace a newer request, and completed work cannot update state after unmount.

## UI and accessibility

The compact runtime state is mounted in the CommandBar company/status region without replacing the company identity. Initializing, ready, and preview use polite status semantics. Terminal error uses an alert notice without stealing focus. State is communicated with text and an inline decorative SVG, not color alone.

The primary status text is never ellipsized. Secondary schema/migration details may be hidden or ellipsized at constrained widths. Browser evidence measures the primary text at 1280×800 and 1024×640 in Arabic RTL and French LTR and requires its scroll dimensions to fit its client dimensions.

## Rust IPC evidence

The test module in `src-tauri/src/lib.rs` builds the real configured mock Tauri application, executes setup, creates a mock webview, calls `get_runtime_status` through `tauri::test::get_ipc_response`, deserializes the IPC response as `serde_json::Value`, and asserts the camelCase contract. It is not ignored and is not replaced by a direct function call or source-text assertion.

The Windows local origin is assembled without a literal external-network URL so the inherited offline network guard remains meaningful while the IPC request still uses the Tauri local origin.

## Dependency decision

The only new product dependency is exact and locked:

```text
@tauri-apps/api@2.11.1
```

No HTTP client, state framework, runtime-schema framework, Tauri SQL plugin, or Rust dependency was added.

## Patch 04A workflow reconciliation

The previous UI and runtime ownership guards were temporary isolation controls for parallel PHASE 02/03 development. After both foundations were accepted, those guards incorrectly rejected the intended cross-layer PHASE 04 changes.

Patch 04A retires only the conflicting portions:

- UI Foundation CI continues to verify the SQLite foundation and all UI/type/build/E2E/accessibility/clean-worktree checks, but its event-scoped ownership guard now protects the database foundation rather than rejecting runtime integration paths.
- Runtime CI retains the database-source guard and all cross-platform Rust/frontend/native/MSRV/clean-worktree checks, while the obsolete frontend freeze is removed.
- `Frontend Runtime Integration` is the read-only, phase-specific owner. It applies the explicit PHASE 04 path allowlist only to the event-scoped change that triggered the workflow. It cannot commit or push.

A temporary branch-only lock-preparation workflow was used because the executor had no direct npm network environment. It verified a package-only diff and a stationary remote head, produced one fast-forward dependency commit, and was deleted immediately. No `contents: write` workflow may remain in the final tree.

## Hotfix 04C integration workflow event scope

`Frontend Runtime Integration` supports only `pull_request` and pushes to `main`; it is not currently a reusable workflow.

For a pull request, the ownership range is `origin/${{ github.base_ref }}` resolved to its commit through `${{ github.event.pull_request.head.sha }}` using three-dot comparison. Both resolved commits must be ancestors of the checked-out pull-request head.

For a push, the ownership range is `${{ github.event.before }}..${{ github.sha }}`. A zero `before` SHA is rejected, and any unsupported event is rejected.

The ownership guard and final `git diff --check` therefore inspect only the change that caused the current run, rather than all repository history since PHASE 03. The existing Phase 04 allowlist, read-only workflow permissions, and write-permission guard remain enforced.

## Startup limitation

This UI error state covers failures that occur after the WebView and React application have loaded: command rejection, bridge failure, malformed payload, or a not-ready status. A Rust `setup` failure can prevent WebView creation entirely, so PHASE 04 does not claim that React can present every pre-WebView desktop startup failure.

## Explicit boundaries

PHASE 04 adds no company setup, authentication, users, permissions, products, inventory writes, purchases, sales, accounting, document calculation, printing, backup, installer, cloud function, telemetry, migration, or additional Tauri command. Existing gallery actions remain demonstration-only.

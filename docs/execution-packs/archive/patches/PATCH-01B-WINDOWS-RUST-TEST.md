# POSMAN — Bootstrap Patch 01B

## Windows Rust test startup failure and missing gate documentation

This patch remains inside the shared Bootstrap Gate between accepted PHASE 01
and the future parallel PHASE 02/03 work. It does not authorize either phase.

---

# EXECUTOR PROMPT — COPY FROM HERE

You are correcting the existing POSMAN desktop-shell Bootstrap Gate. Work on
the same branch and the same Draft Pull Request. Do not open a replacement PR.

## 1. Repository and immutable review coordinates

Repository:

```text
mohamed09090-xmd/posman-desktop
```

Accepted `main` baseline:

```text
0c72eb75eb5db916a51d1ee42fec47f21328ad28
```

Existing branch:

```text
bootstrap/desktop-shell
```

Reviewed pre-patch head:

```text
6024a86ca783a3d72fc4e815e49376f6eee5604d
```

Existing Draft PR:

```text
https://github.com/mohamed09090-xmd/posman-desktop/pull/2
```

Existing failing CI run:

```text
https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30383180979
```

Before editing:

1. Fetch the repository and verify that `main` still contains the accepted
   baseline.
2. Check out `bootstrap/desktop-shell`.
3. Confirm the branch contains reviewed head
   `6024a86ca783a3d72fc4e815e49376f6eee5604d`.
4. If the branch moved, report the new head and inspect the intervening commits
   before editing. Do not discard or rewrite them.
5. Confirm a clean worktree.
6. Read `AGENTS.md`.
7. Read `docs/spec/POSMAN-Blueprint-v1.md`.
8. Read the original Bootstrap Gate execution pack supplied by the architect.
9. Inspect the failed Windows job and the
   `rust-test-diagnostics-Windows` artifact before changing code.

Do not force-push, rebase away existing commits, merge the PR, or create a new
PR.

## 2. Confirmed failure boundary

Ubuntu passed the full gate. Windows passed:

- PHASE 01 schema verification and no-diff check;
- frontend install, typecheck, and build;
- runtime-network scan;
- Rust formatting;
- Rust clippy with warnings denied.

Windows then built the Rust test executable successfully but could not start
it:

```text
Running unittests src\lib.rs
process didn't exit successfully
exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND
```

This is not a failed Rust assertion. The Windows loader terminated the test
executable before the test harness ran.

The leading hypothesis is a missing Common Controls v6 manifest dependency on
the Cargo-generated test executable:

- resolved Tauri `2.11.5` enables `common-controls-v6` by default;
- the normal Tauri application receives Windows resources through
  `tauri_build::build()`;
- a Cargo unit-test executable is a separate target and may not inherit the
  application manifest;
- a v6-only Common Controls import in a test executable without the matching
  manifest can produce `STATUS_ENTRYPOINT_NOT_FOUND` during process startup.

Treat this as a hypothesis that must be proven, not as permission to patch
blindly.

Authoritative references:

```text
https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-arg-tests
https://docs.rs/tauri/2.11.5/tauri/test/index.html
https://github.com/tauri-apps/tauri/blob/tauri-v2.11.5/crates/tauri/Cargo.toml
```

## 3. Mandatory Windows diagnosis

On a Windows runner, identify the exact failing executable and inspect its
loader inputs before finalizing the fix.

Use a deterministic method such as:

1. Build tests without running them and capture Cargo JSON output, or locate the
   newest exact `posman_desktop_lib-*.exe` test harness under
   `src-tauri/target/debug/deps`.
2. Inspect the executable imports with Windows SDK/Visual Studio tooling.
3. Inspect its embedded manifest, if any.
4. Record the DLL and missing entry point when the available tooling exposes
   them.
5. Establish whether the executable imports a Common Controls v6-only symbol
   but lacks the `Microsoft.Windows.Common-Controls` version `6.0.0.0`
   dependency.

Diagnostics may be added temporarily to the workflow. Store diagnostic output
under `RUNNER_TEMP`, upload it only on failure, and remove noisy temporary
diagnostic steps after the cause is proven. A concise permanent failure
artifact is acceptable.

Do not commit generated `.exe`, `.pdb`, manifests extracted from build output,
`target/`, runner dumps, or diagnostic transfer helpers.

If the evidence identifies a different DLL or entry point, do not apply the
Common Controls correction. Fix the proven root cause within this gate and
document the evidence. If that fix requires expanding scope, stop and report.

## 4. Preferred narrow correction when the hypothesis is confirmed

Keep Tauri's production defaults and the real mock-runtime test.

Update `src-tauri/build.rs` so that, only when the Cargo target OS is Windows,
the build script supplies the Common Controls v6 manifest dependency to Cargo
test targets using:

```text
cargo::rustc-link-arg-tests
```

Requirements:

- Detect the target using Cargo's `CARGO_CFG_TARGET_OS`; do not use
  `cfg!(windows)` because a build script's compile host and target can differ.
- Pass the standard `Microsoft.Windows.Common-Controls` assembly identity:
  version `6.0.0.0`, public key token `6595b64144ccf1df`, language `*`, and a
  valid processor-architecture value.
- Continue to call `tauri_build::build()`.
- Limit the additional linker argument to test targets. Do not replace or
  duplicate the production Tauri resource generation.
- Keep the code small, deterministic, formatted, and warning-free.
- Do not add a new crate solely for this linker argument.
- Do not disable `common-controls-v6` globally; that would alter the production
  desktop binary to make a test pass.

After the change, prove on Windows that:

1. the unit-test executable starts;
2. `application_setup_builds_with_mock_runtime` actually runs and passes;
3. the native non-bundled Tauri application still compiles;
4. the production target retains its normal Tauri resource/manifest behavior.

If `cargo::rustc-link-arg-tests` cannot safely express the manifest dependency
with the resolved stable toolchain, stop and report the exact linker command
and error. Do not switch to a broad global `RUSTFLAGS` workaround.

## 5. Test integrity rules

The following are prohibited:

- deleting the Rust test;
- replacing it with a constant or source-text assertion;
- adding `#[ignore]`;
- excluding Windows with `cfg`;
- using `continue-on-error`;
- swallowing the process exit code;
- changing the workflow to run only `cargo check`;
- removing `--locked`;
- pinning the job to an older Windows image merely to hide the loader defect;
- weakening clippy or warning policy;
- dropping `--all-targets` from the accepted Rust validation without explicit
  architectural approval;
- disabling Tauri default features globally to avoid the missing manifest;
- claiming a flaky runner without a clean rerun and evidence.

The existing mock-runtime test in `src-tauri/src/lib.rs` must remain a real
application-builder test.

## 6. Complete the missing Bootstrap Gate documentation

The reviewed branch is missing this required permanent file:

```text
docs/architecture/desktop-shell.md
```

Create it after the technical fix is stable. It must document:

- why Tauri 2 + React + TypeScript + Vite was selected;
- exact resolved Node, npm, Tauri CLI, Tauri Rust crate, React, Vite,
  TypeScript, rustc, and Cargo versions;
- root project layout;
- development and validation commands;
- Windows development prerequisites;
- Ubuntu CI prerequisites;
- window dimensions and identifier;
- CSP and capability decisions;
- that Node/npm/Rust are build-time tools and are not customer prerequisites;
- that no server, external database, sidecar, runtime network dependency, or
  telemetry exists;
- that SQLite runtime integration belongs to PHASE 02;
- that the actual UI/design system belongs to PHASE 03;
- the Windows test-manifest issue, its proven cause, and the narrow correction.

Only after both Ubuntu and Windows jobs pass, create:

```text
docs/BOOTSTRAP-GATE-02-REPORT.md
```

The report must contain real final evidence:

1. accepted baseline SHA;
2. branch, final head, and Draft PR;
3. exact resolved tool/dependency versions;
4. files created, modified, and deleted;
5. exact validation commands and results;
6. final GitHub Actions run URL;
7. separate Ubuntu and Windows job conclusions;
8. proof that the Windows Rust test ran, not merely compiled;
9. proof that the Windows native no-bundle build ran after tests;
10. PHASE 01 verifier result;
11. database and schema no-diff result;
12. CSP and capability summary;
13. proof that no PHASE 02 runtime or PHASE 03 UI scope was added;
14. risks and temporary scaffold limitations.

Do not commit a PASS report before both matrix jobs are green.

Update the existing Draft PR description with a `Patch 01B` section containing
the diagnosis, changed files, final validation results, and final Actions URL.
Keep the PR Draft and unmerged.

## 7. Required validation

Run locally where supported:

```text
python scripts/verify_schema.py
npm ci
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked
npm run desktop:check
git diff --check
git status --short
```

The final GitHub Actions run must prove all relevant checks on:

```text
ubuntu-latest
windows-latest
```

On Windows, confirm that these steps are executed rather than skipped:

```text
Run Rust tests
Compile native Tauri shell without bundling
Check Git whitespace
Confirm clean worktree
```

Also verify:

```text
git diff --exit-code 0c72eb75eb5db916a51d1ee42fec47f21328ad28 -- database scripts/verify_schema.py
```

The final worktree must be clean on both platforms.

## 8. Scope protection

This patch may modify only files necessary to:

- diagnose and correct the Windows Rust test startup;
- preserve or improve relevant failure diagnostics;
- create the two required Bootstrap Gate documents;
- update the existing PR description and concise bootstrap documentation.

Do not add:

- SQLite runtime connections or migration runners;
- `rusqlite` or Tauri SQL;
- Tauri business commands;
- products, stock, sales, purchasing, accounting, authentication, backups,
  printing, PDF, or installer functionality;
- final UI components, navigation, design tokens, a design system, or branding;
- router, state manager, UI framework, CSS framework, or i18n library;
- HTTP clients, online services, telemetry, updater, sidecar, or local server.

Do not modify accepted PHASE 01 database files.

## 9. Commit guidance

Use a small, intentional sequence such as:

```text
fix(test): embed Windows Common Controls manifest for Rust tests
docs: complete desktop bootstrap architecture and gate report
```

Separate a durable CI diagnostic change only if it remains useful and concise.
Do not create operational commits for transferring snapshots or commit logs.
Do not rewrite the existing branch history.

## 10. Definition of done

Patch 01B is complete only when:

- the Windows loader cause is evidenced;
- the real Tauri mock-runtime Rust test runs and passes on Windows;
- Ubuntu Rust tests remain green;
- Windows native no-bundle compilation runs and passes after the tests;
- all frontend and Rust checks pass on both platforms;
- PHASE 01 verification and database no-diff checks pass;
- `docs/architecture/desktop-shell.md` exists and is accurate;
- `docs/BOOTSTRAP-GATE-02-REPORT.md` exists and cites the final green run;
- the worktree is clean;
- the existing PR remains Draft, open, and unmerged;
- no force-push was used;
- PHASE 02 and PHASE 03 remain unstarted.

## 11. Final handoff

Return:

1. repository, branch, and final head SHA;
2. existing Draft PR URL and status;
3. new commit SHAs and messages;
4. exact DLL/entry-point diagnosis and the evidence used;
5. exact source correction;
6. files created, modified, and deleted;
7. full Rust test output showing the test name and PASS;
8. all local validation results;
9. final GitHub Actions URL;
10. Ubuntu and Windows job results;
11. explicit confirmation that Windows native build and clean-worktree steps
    executed;
12. PHASE 01 verifier and no-diff evidence;
13. explicit confirmation that no tests/checks were weakened;
14. explicit confirmation that no PHASE 02 or PHASE 03 work was added;
15. explicit confirmation of no force-push and no merge.

Do not begin PHASE 02 or PHASE 03.

# END OF EXECUTOR PROMPT

---

## Reviewer disposition

The Bootstrap Gate remains rejected/incomplete until this patch is reviewed and
both CI platforms pass. A green Ubuntu job alone is insufficient.

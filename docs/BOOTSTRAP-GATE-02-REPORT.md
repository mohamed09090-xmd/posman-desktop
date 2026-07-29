# Bootstrap Gate 02 implementation report

## Disposition

Implementation evidence for the shared POSMAN desktop Bootstrap Gate. Acceptance remains the responsibility of the external architect/reviewer.

## Repository coordinates

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Accepted baseline: `0c72eb75eb5db916a51d1ee42fec47f21328ad28`
- Branch: `bootstrap/desktop-shell`
- Documentation-inclusive validated head: `5e697c05c9aec49dffa3fe6f56293ccf7ee7920f`
- Draft PR: `https://github.com/mohamed09090-xmd/posman-desktop/pull/2`
- PR state at handoff: open, Draft, unmerged

## Resolved versions

| Tool or dependency | Version |
|---|---:|
| Node.js | 24.18.0 |
| npm | 11.16.0 |
| Python | 3.12 |
| rustc | 1.97.1 |
| Cargo | 1.97.1 |
| Tauri | 2.11.5 |
| `tauri-build` | 2.6.3 |
| Tauri CLI | 2.11.3 |
| React / React DOM | 19.2.8 |
| TypeScript | 6.0.3 |
| Vite | 8.1.0 |
| React Vite plugin | 6.0.2 |

## Permanent file changes

Created:

```text
src-tauri/windows-app-manifest.xml
docs/architecture/desktop-shell.md
docs/BOOTSTRAP-GATE-02-REPORT.md
```

Modified:

```text
src-tauri/build.rs
.github/workflows/desktop-bootstrap-ci.yml
```

Deleted: none.

No database source, migration, accepted verifier, Cargo manifest/lockfile, npm manifest/lockfile, application source, or mock-runtime test was changed.

## Build strategy

On Windows/MSVC, `build.rs` calls `tauri_build::try_build` with `WindowsAttributes::new_without_app_manifest()`, then links the reviewed local manifest using `/MANIFEST:EMBED`, `/MANIFESTINPUT:<absolute path>`, and `/WX`. Other targets retain default `tauri-build` attributes. The manifest matches the Tauri 2.11.5 source and has SHA-256:

```text
4636f3ba46080315ac3277d473d433b79c00863c9cdca1c93c0c83554c6a3d43
```

No `RUSTFLAGS`, Cargo config override, extra crate, feature change, disabled test, or weakened command was used.

## Validation commands

The cross-platform workflow executes:

```text
python scripts/verify_schema.py
git diff --exit-code 0c72eb75eb5db916a51d1ee42fec47f21328ad28 -- database scripts/verify_schema.py
npm ci
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:check
git diff --check
git status --short --untracked-files=all
```

The execution environment available to the implementation session did not contain a writable local checkout with the Windows toolchain. Product validation therefore ran in GitHub Actions. Locally, the execution packs and repository sources were inspected, the manifest SHA-256 was calculated, and workflow YAML syntax was checked; no unsupported local PASS claim is made.

## Final CI evidence

- Documentation-inclusive desktop run: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30415757191`
- Ubuntu job: `Desktop shell (ubuntu-latest)` — success, job `90461798866`
- Windows job: `Desktop shell (windows-latest)` — success, job `90461798828`
- Documentation-inclusive SQLite schema workflow: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30415757177` — success

Both platform worktrees were clean in this documentation-inclusive run. This evidence-recording update is documentation-only and is validated by the subsequent branch-head workflows reported in the final handoff.

## Windows named Rust test

Expected and verified output:

```text
running 1 test
test tests::application_setup_builds_with_mock_runtime ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

This is the existing library unit test using Tauri's real mock builder/context; it was not moved, ignored, replaced, or excluded on Windows.

## Windows unit-test PE evidence

The workflow deterministically resolves the exact `posman_desktop_lib-*.exe` compiler artifact from Cargo JSON, then verifies:

- non-zero PE Resource Directory;
- manifest resource `#1` extracts successfully with `mt.exe`;
- valid XML contains `Microsoft.Windows.Common-Controls` and `6.0.0.0`;
- the named mock-runtime test PASS line exists in the actual test output.

Evidence from the documentation-inclusive run:

```text
Test executable: D:\a\posman-desktop\posman-desktop\src-tauri\target\debug\deps\posman_desktop_lib-3babbb0151fd5d43.exe
Resource directory: 7F0000 [2C0] RVA [size] of Resource Directory
Manifest SHA-256: CD613330097D228AE01FCE028E50954FE4595249D51EDEBC4461EC252B514517
Named test: application_setup_builds_with_mock_runtime ... ok
```

## Windows application PE evidence

After `npm run desktop:check`, the workflow verifies the exact debug `posman-desktop.exe`:

- non-zero PE Resource Directory;
- manifest resource `#1` contains the same Common Controls v6 dependency;
- resource `#2` is absent, supporting one application manifest rather than competing manifests;
- file-version/product metadata remain present;
- an associated application icon remains extractable.

Evidence from the documentation-inclusive run:

```text
Application executable: D:\a\posman-desktop\posman-desktop\src-tauri\target\debug\posman-desktop.exe
Resource directory: BF0000 [A18] RVA [size] of Resource Directory
Manifest SHA-256: CD613330097D228AE01FCE028E50954FE4595249D51EDEBC4461EC252B514517
Second manifest probe exit: 31 (resource #2 absent)
File version: 0.1.0
Product name: POSMAN
Application icon: extractable
```

## PHASE 01 protection

On Ubuntu and Windows, `python scripts/verify_schema.py` passes and the exact no-diff command against baseline `0c72eb75...` passes. No accepted migration or schema verifier changed.

## CSP and capability summary

The production CSP restricts content to the bundled application/custom protocol and Tauri IPC, with object/frame embedding disabled. Development adds only the local Vite/WebSocket origins. The only Tauri capability is `main-shell`, scoped to the `main` window with `core:default`.

## Scope confirmation

No SQLite runtime, business command, product/stock/sales/accounting workflow, authentication, report, backup, printing/PDF, installer, updater, telemetry, cloud integration, design system, router, state framework, or i18n implementation was added.

PHASE 02 and PHASE 03 remain unstarted. The Pull Request was not merged and no force-push or history rewrite was performed.

## Remaining risks

- Cargo still lacks a test-only linker directive that reaches library unit-test harnesses (Cargo issue 10937).
- The corresponding upstream Tauri mock-runtime Windows problem remains tracked in Tauri issue 13419.
- POSMAN therefore carries the reviewed shared Windows/MSVC manifest path. CI protects it with `/WX` and binary inspection.
- Installer, signing, and packaged release validation are outside this Bootstrap Gate and remain future authorized work.

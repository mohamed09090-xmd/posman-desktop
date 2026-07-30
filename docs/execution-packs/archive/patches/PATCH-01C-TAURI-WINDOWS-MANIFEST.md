# POSMAN — Bootstrap Patch 01C

## Official Tauri Windows manifest workaround for library unit-test harnesses

This patch remains inside the shared Bootstrap Gate. It resolves the proven
Windows loader defect without creating a test crate, disabling tests, changing
Tauri features, or using global environment flags.

---

# EXECUTOR PROMPT — COPY FROM HERE

Continue correcting the existing POSMAN Bootstrap Gate on the same branch and
Draft Pull Request. Do not open a replacement PR and do not start PHASE 02 or
PHASE 03.

## 1. Repository and current coordinates

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

Expected pre-patch head:

```text
ca30864bf723107fd00a524a026aa91e42f80d47
```

Existing Draft PR:

```text
https://github.com/mohamed09090-xmd/posman-desktop/pull/2
```

Before editing:

1. Fetch the repository.
2. Confirm `main` still contains the accepted baseline.
3. Check out `bootstrap/desktop-shell`.
4. Confirm the branch head is the expected pre-patch SHA.
5. If it moved, inspect and report the intervening commits before editing.
6. Confirm the worktree is clean.
7. Read `AGENTS.md`, the POSMAN Blueprint, the original Bootstrap Gate pack,
   and the Patch 01B result recorded in PR #2.

Do not rebase, force-push, rewrite or delete the diagnostic history, merge the
PR, or create another PR.

## 2. Accepted diagnosis

Patch 01B proved:

- the Windows library unit-test executable imports `TaskDialogIndirect` from
  `comctl32.dll`;
- the executable has no resource section and no embedded application manifest;
- Windows therefore loads a Common Controls implementation that does not expose
  the imported v6 entry point;
- the process exits before the Rust test harness starts with:

```text
0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND
```

Patch 01B also proved that:

```text
cargo::rustc-link-arg-tests
```

does not apply to a library unit-test harness. Cargo accepts it only when the
package has an explicit integration-style test target. This is a known open
Cargo limitation:

```text
https://github.com/rust-lang/cargo/issues/10937
```

The same Tauri mock-runtime Windows failure is tracked upstream:

```text
https://github.com/tauri-apps/tauri/issues/13419
```

## 3. Architectural decision

The architect now authorizes the package-wide Windows/MSVC manifest mechanism
used by Tauri's own repository to make library unit-test harnesses executable.

Official Tauri 2.11.5 reference implementation:

```text
https://github.com/tauri-apps/tauri/blob/tauri-v2.11.5/examples/api/src-tauri/build.rs
```

Official Tauri 2.11.5 manifest source:

```text
https://github.com/tauri-apps/tauri/blob/tauri-v2.11.5/crates/tauri-build/src/windows-app-manifest.xml
```

This is an explicit exception to Patch 01B's prohibition on a broad
`cargo::rustc-link-arg`. It is approved because:

1. Cargo currently has no directive that reaches only library unit-test
   harnesses.
2. Every Windows/MSVC executable-like output that links the enabled Tauri
   `common-controls-v6` code requires the same manifest dependency.
3. Tauri's own repository uses a general linker manifest input for this exact
   `STATUS_ENTRYPOINT_NOT_FOUND` test problem.
4. Tauri's default app-manifest compilation will be disabled when this shared
   mechanism is active, preventing two competing application manifests.
5. The final production executable and test executable will both be inspected
   to prove that the resource result is correct.

This does not authorize `RUSTFLAGS`, a Cargo config override, an extra test
crate, disabling `common-controls-v6`, or weakening tests.

## 4. Add the reviewed manifest

Create:

```text
src-tauri/windows-app-manifest.xml
```

Use the exact minimal Common Controls manifest used by Tauri 2.11.5:

```xml
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
```

Requirements:

- Keep this file local and deterministic.
- Do not read a manifest from Cargo's registry or an absolute developer path.
- Do not add requested privileges, administrator elevation, compatibility
  shims, network settings, or unrelated Windows capabilities.
- Record that this content matches the resolved Tauri 2.11.5 source.

## 5. Update `src-tauri/build.rs`

Implement one target-aware manifest path based on the official Tauri
workaround.

Required behavior:

1. Read `CARGO_CFG_TARGET_OS`.
2. Read `CARGO_CFG_TARGET_ENV`.
3. Treat the special path as active only when:

```text
target_os == windows
target_env == msvc
```

4. When active, call `tauri_build::try_build` with:

```rust
tauri_build::Attributes::new().windows_attributes(
    tauri_build::WindowsAttributes::new_without_app_manifest(),
)
```

5. For other targets, retain normal default `tauri-build` behavior.
6. When the Windows/MSVC path is active, resolve
   `windows-app-manifest.xml` from `CARGO_MANIFEST_DIR`.
7. Emit:

```text
cargo::rerun-if-changed=<absolute manifest path>
cargo::rustc-link-arg=/MANIFEST:EMBED
cargo::rustc-link-arg=/MANIFESTINPUT:<absolute manifest path>
cargo::rustc-link-arg=/WX
```

8. Return a clear build error if `tauri-build` fails or the manifest path cannot
   be represented safely.

Important:

- Follow the current Cargo `cargo::KEY=VALUE` syntax supported by the project's
  Rust MSRV.
- Do not use `cargo::rustc-link-arg-tests`; Patch 01B proved it cannot reach
  this unit-test harness.
- Do not also let `tauri-build` compile its default app manifest on the
  Windows/MSVC path. `new_without_app_manifest()` exists specifically to avoid
  that duplication.
- Do not disable Tauri icon, version-information, capability generation, or
  normal code generation.
- Do not add a build dependency.
- Do not modify `Cargo.toml` or `Cargo.lock` unless the resolved API genuinely
  differs. If it differs, stop and report the exact compiler error before
  selecting another mechanism.
- Do not gate the manifest behind an arbitrary environment variable or Cargo
  feature.
- Do not use a global `RUSTFLAGS` or `.cargo/config.toml`.
- Keep `tauri_build::Attributes::new()` as the normal path for non-Windows or
  non-MSVC targets.

The implementation may be structurally equivalent to Tauri's example; it does
not need to copy unrelated plugins, commands, codegen configuration, or
workspace-only checks from that example.

## 6. Preserve the real test

Keep the existing unit test in:

```text
src-tauri/src/lib.rs
```

It must continue to build the application using:

```text
tauri::test::mock_builder()
tauri::test::mock_context(...)
```

Do not:

- move it to an integration test merely to satisfy Cargo;
- create `tests/` or `[[test]]`;
- add `[lib] test = false`;
- delete, ignore, mock away, or exclude the test on Windows;
- replace it with source inspection or a constant assertion.

## 7. Windows binary evidence

Add or retain concise Windows-only CI checks that prove both artifacts contain
the required manifest.

### Unit-test executable

After building/running Rust tests:

1. Resolve the exact `posman_desktop_lib-*.exe` unit-test harness
   deterministically.
2. Use `mt.exe` to extract resource `#1`.
3. Assert the extracted manifest contains:

```text
Microsoft.Windows.Common-Controls
6.0.0.0
```

4. Record that PE Resource Directory is non-zero.
5. The test command itself must show that:

```text
application_setup_builds_with_mock_runtime ... ok
```

### Native Tauri application

After:

```text
npm run desktop:check
```

1. Resolve the exact debug application executable.
2. Extract resource `#1`.
3. Assert the same Common Controls v6 dependency exists.
4. Confirm there is one valid application manifest, not conflicting manifest
   resources.
5. Confirm normal Tauri icon/version resource generation was not lost.

Store extracted manifests and diagnostics under `RUNNER_TEMP`. Upload them only
on failure. Do not dirty the repository or commit generated PE output.

If `/WX` exposes a duplicate-manifest warning, resource collision, ignored
manifest, or invalid linker input, stop and report the complete linker command.
Do not remove `/WX` to conceal the warning.

## 8. CI requirements

Keep both:

```text
ubuntu-latest
windows-latest
```

Keep:

```text
--all-targets
--all-features
--locked
-D warnings
```

Run the Rust test command with output sufficient to prove the named test ran,
for example by forwarding `--nocapture` while preserving the actual exit code.

Do not:

- use `continue-on-error`;
- pin an older Windows runner to hide the issue;
- swallow failures;
- convert tests into compile-only checks;
- reorder the native Windows build ahead of a failing test and claim success;
- remove clean-worktree or PHASE 01 checks.

On Windows, the final run must execute and pass, rather than skip:

```text
Run Rust clippy with warnings denied
Run Rust tests
Verify Windows Rust test manifest dependency
Compile native Tauri shell without bundling
Verify Windows application manifest dependency
Check Git whitespace
Confirm clean worktree
```

Ubuntu must remain fully green.

## 9. Complete the required gate documentation

After the technical fix is green on both platforms, create:

```text
docs/architecture/desktop-shell.md
docs/BOOTSTRAP-GATE-02-REPORT.md
```

`docs/architecture/desktop-shell.md` must cover:

- Tauri 2 + React + TypeScript + Vite rationale;
- exact resolved versions;
- project structure and development commands;
- Windows and Ubuntu prerequisites;
- application identifier and window settings;
- CSP and capability decisions;
- offline/no-server/no-sidecar boundary;
- build-time-only Node/npm/Rust toolchain;
- PHASE 02 versus PHASE 03 ownership boundary;
- the upstream Tauri/Cargo Windows unit-test limitation;
- why POSMAN uses one shared Windows/MSVC manifest linker path;
- proof that the application and unit-test executable contain the same required
  Common Controls v6 dependency.

`docs/BOOTSTRAP-GATE-02-REPORT.md` must contain:

1. accepted baseline, branch, final head, and Draft PR;
2. exact dependency/tool versions;
3. all created, modified, and deleted files;
4. local validation actually performed;
5. final CI run URL and per-platform conclusions;
6. named Windows Rust test PASS output;
7. unit-test PE/manifest evidence;
8. native application PE/manifest evidence;
9. PHASE 01 verifier and database no-diff evidence;
10. CSP/capability summary;
11. confirmation that no PHASE 02/03 work was added;
12. remaining risks, including the upstream Tauri/Cargo limitation.

Do not create or commit a PASS report before the technical CI run is green.
After adding the documentation, run the complete CI matrix again and cite that
final documentation-inclusive green run.

Update the existing PR body with a `Patch 01C` section. Preserve the valuable
Patch 01B diagnosis. Keep the PR Draft and unmerged.

## 10. Required validation

Run every command that the environment supports and report only real results:

```text
python scripts/verify_schema.py
npm ci
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:check
git diff --check
git status --short
```

Verify the accepted database remains unchanged:

```text
git diff --exit-code 0c72eb75eb5db916a51d1ee42fec47f21328ad28 -- database scripts/verify_schema.py
```

The final GitHub Actions run must be green on Ubuntu and Windows, and both
worktrees must be clean.

## 11. Allowed file scope

Expected permanent changes are limited to:

```text
src-tauri/build.rs
src-tauri/windows-app-manifest.xml
.github/workflows/desktop-bootstrap-ci.yml
docs/architecture/desktop-shell.md
docs/BOOTSTRAP-GATE-02-REPORT.md
README.md                         # only if a concise documentation link is needed
```

The existing PR description may be updated.

Do not modify:

```text
database/**
scripts/verify_schema.py
src-tauri/src/lib.rs              # unless formatting only; preserve the test
src-tauri/Cargo.toml
src-tauri/Cargo.lock
package.json
package-lock.json
src/**
```

If an allowed solution requires modifying another permanent file, stop and
request architectural approval with the exact reason.

## 12. Scope protection

Do not add:

- SQLite runtime, migrations, connection pools, or `rusqlite`;
- Tauri business commands;
- products, stock, sales, purchasing, accounting, authentication, reports,
  backups, printing, PDF, or installer functionality;
- design system, final branding, navigation, screens, state management, router,
  CSS framework, UI library, or i18n implementation;
- network client, server, sidecar, updater, telemetry, or cloud integration.

PHASE 02 and PHASE 03 remain unstarted.

## 13. Commit guidance

Use a small sequence such as:

```text
fix(test): embed shared Windows manifest for Tauri targets
ci: verify Windows test and application manifests
docs: complete desktop bootstrap gate evidence
```

Do not add commits whose only purpose is moving snapshots, publishing commit
logs, or deleting temporary helpers. Keep diagnostics in `RUNNER_TEMP`.

## 14. Definition of done

Patch 01C and the Bootstrap Gate are complete only when:

- the official Tauri-style shared manifest path is implemented;
- Cargo accepts all build-script instructions;
- Windows clippy passes;
- the real named mock-runtime unit test runs and passes on Windows;
- the Windows unit-test executable has resource `#1` with Common Controls v6;
- the native Windows application compiles after tests;
- the application executable has one valid Common Controls v6 manifest and
  retains normal resource metadata;
- Ubuntu remains fully green;
- PHASE 01 verifier and database no-diff checks pass;
- both required gate documents exist and cite the final green run;
- both platform worktrees are clean;
- the PR remains open, Draft, and unmerged;
- no force-push or history rewrite is used;
- no PHASE 02 or PHASE 03 implementation appears.

## 15. Stop conditions

Stop and report instead of improvising if:

- the expected branch head changed unexpectedly;
- the resolved `tauri-build` API lacks
  `WindowsAttributes::new_without_app_manifest`;
- Cargo rejects the general linker instructions;
- MSVC reports a manifest/resource conflict under `/WX`;
- the test manifest passes but the production application manifest or icon/
  version resources regress;
- the solution requires a new crate, dependency, Cargo feature, `RUSTFLAGS`,
  disabled test, or scope expansion;
- either platform remains red after the focused correction.

## 16. Final handoff

Return:

1. repository, branch, and final head SHA;
2. existing Draft PR URL and status;
3. new commit SHAs and messages;
4. exact `build.rs` strategy;
5. manifest source and SHA-256;
6. files created, modified, and deleted;
7. full named Rust test PASS output on Windows;
8. test-executable manifest evidence;
9. application-executable manifest and resource evidence;
10. all local validation actually performed;
11. final GitHub Actions URL;
12. Ubuntu and Windows job/step results;
13. PHASE 01 verifier and no-diff evidence;
14. confirmation that tests and checks were not weakened;
15. confirmation that PHASE 02 and PHASE 03 remain unstarted;
16. confirmation of no force-push and no merge.

Do not begin PHASE 02 or PHASE 03.

# END OF EXECUTOR PROMPT

---

## Reviewer disposition

Patch 01B diagnosis is accepted. The Bootstrap Gate remains incomplete until
Patch 01C is green on both platforms and the required documentation is
committed with final evidence.

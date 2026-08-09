# PHASE 10 — Distribution, hardening, and POSMAN v1.0.0

## Status

Implementation is complete on `phase/10-distribution-hardening-v1-release` and
is ready for independent review. Acceptance is not self-issued and auto-merge
is prohibited. The immutable final-head workflow coordinates and artifact
identifiers are recorded in PR #15 after the report/guard-only final commit so
that recording evidence cannot move the head that produced it.

## Baseline and scope

- Accepted PHASE 09 main: `ebc256cd1503867558ebc9cdaac285066c8466d9`.
- Product release version: `1.0.0` stable.
- Windows bundle: x64 NSIS with embedded offline WebView2.
- Business schema and migrations: unchanged through `0007`.
- PHASE 11 or additional product scope: none.

## Implemented delivery surface

- Approved transparent POSMAN application/installer icon and multi-resolution ICO.
- Per-machine Program Files installation separated from per-user LocalAppData.
- Arabic/French/English installer language selection.
- Downgrade prevention, offline WebView2, LZMA compression, shortcuts, and v1 metadata.
- CI-only v0.9.0 upgrade fixture and real install/upgrade/uninstall lifecycle harness.
- Local-data/database/backup/document retention assertions.
- Cold-start, working-set, installer-size, and 100k-product search budgets.
- SHA-256 manifest, dependency metadata, security policy, signing strategy,
  installation guide, changelog, and release notes.

## Validation evidence

The source-equivalent release proof run was GitHub Actions run `31294725649` on
head `ad0cd6bd70a7d44b8768dca301048b210b777ff6`. All product, database, policy,
frontend/Axe, Rust stable, Rust 1.85, Windows native, installer build, and real
installer-lifecycle steps passed. The only failed step was the final Windows
worktree status check: Tauri's release build changed only the checkout's
LF/CRLF representation of `src-tauri/Cargo.toml`; `git diff` contained no
content change. The final guard now uses the already accepted content-based
normalization pattern while still rejecting any real staged, unstaged, or
untracked content.

Release evidence from that proof run:

- `phase-10-ui-evidence`: artifact `9032587878`, 3,271,117 bytes,
  `sha256:af969b8cb467016fae43350cd625b12627353237f64a30cf0ec77eecc26988a8`.
- `phase-10-policy-evidence`: artifact `9032605652`, 335 bytes,
  `sha256:a5acaa48be956ed0c4104f00cd4df22c657cfb7fac652765fafad8a993a19f90`.
- `posman-v1.0.0-windows-offline`: artifact `9032873583`, 214,699,437
  archived bytes,
  `sha256:902ca2f8934530d4a56123e7f2900d4fc04684b29ce76daeefd4b6ae6973df63`.
- `phase-10-release-evidence`: artifact `9032873824`, 209,807 bytes,
  `sha256:d38d6b3473be400a17baeafb3d67e55d0ba28f049c0a39e945429ea8693154c7`.

The normalized offline installer itself is 214,648,524 bytes (204.705 MiB)
with SHA-256
`c1589948b03006a88aa34e33edab2cd0170b6357c867f7c81f4480c1fa6fb00e`.
Authenticode is `NotSigned`, as required when no external signing credential is
available.

The real disposable-runner lifecycle passed clean v0.9.0 installation, first
database creation, v1.0.0 upgrade, data preservation, relaunch, silent
uninstall, binary removal, and LocalAppData preservation. The v1 cold window
time was 124.881 ms and working set was 25.949 MiB. The v0.9.0 runtime required
7,221.407 ms after the window appeared to complete first database creation;
the harness therefore waits for the database itself instead of terminating the
process after an arbitrary two-second delay. The startup target remains 5,000
ms and the working-set target remains 250 MiB.

The 100,000-product SQLite search benchmark recorded 100.564 ms p95 against the
2,000 ms hard regression budget.

## Signing limitation

No code-signing credential was supplied or committed. CI release candidates are
therefore expected to be unsigned and may trigger SmartScreen. Commercial
publication requires the external controlled signing procedure documented in
`docs/release/CODE-SIGNING.md`.

## Confirmations

- No force-push, rebase, reset, or history rewrite.
- No direct commit to `main`.
- No auto-merge.
- No updater, cloud, telemetry, HTTP API, or mandatory activation.
- No new migration or business command.
- PHASE 10 is not merged by this implementation report; merge remains an
  explicit reviewer action after final-head CI evidence is green.

# POSMAN PHASE 10 execution pack

## Mission

Deliver the Windows-first offline POSMAN v1.0.0 distribution boundary from the
accepted PHASE 09 main commit. Produce a reviewable Draft PR and complete
release evidence without merging or publishing a release automatically.

## Required implementation

1. Keep migrations `0001–0007` and accepted business behavior frozen.
2. Synchronize all first-party version coordinates to `1.0.0`.
3. Use the approved transparent POSMAN icon for Windows application and setup surfaces.
4. Produce a Windows x64 NSIS bundle named `POSMAN-Setup-Offline.exe`.
5. Embed WebView2's offline installer; prohibit runtime installer downloads.
6. Install binaries under Program Files and preserve `%LOCALAPPDATA%\POSMAN`.
7. Block downgrades and prove v0.9.0 → v1.0.0 upgrade behavior.
8. Prove uninstall removes the program but keeps database, backups, and documents.
9. Record startup, memory, installer size, 100k-product search, dependency, and checksum evidence.
10. Document installation, upgrade, uninstall, security reporting, signing, and release notes.

## Acceptance commands

```text
python scripts/verify_schema.py
python scripts/verify_phase06.py
python scripts/verify_phase07.py
python scripts/verify_phase08.py
python scripts/verify_phase09.py
python scripts/verify_phase10.py
python scripts/phase10_performance.py
npm ci
npm run typecheck
npm run build
npm run test:ui
npm run test:integration
npm run test:e2e
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:check
npm run release:windows
```

The Windows release job must additionally run the disposable-runner installer
lifecycle test, upload release/evidence artifacts, and preserve a clean worktree.

## Stop conditions

- Accepted main or the owned branch moves unexpectedly.
- Any accepted migration or business contract changes.
- A signing private key, customer database, backup, environment secret, or
  write-capable workflow appears.
- The installer needs internet, deletes LocalAppData, exceeds 200 MB, or fails
  clean install/upgrade/uninstall validation.
- A required final-head job is failing or incomplete.

Do not mark the PR Ready, merge it, publish a GitHub Release, or start work
beyond PHASE 10 without explicit owner authorization.

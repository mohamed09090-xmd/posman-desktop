# PHASE 10 — Distribution, hardening, and POSMAN v1.0.0

## Status

Implementation in progress on `phase/10-distribution-hardening-v1-release`.
This report must be updated with final-head run and artifact identifiers before
architectural acceptance. The phase is not self-accepted and is not authorized
for automatic merge.

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

Final GitHub Actions run IDs, job results, artifact IDs, byte sizes, SHA-256
digests, Authenticode state, lifecycle metrics, and risk closure remain pending
until the final branch head completes all gates.

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
- PHASE 10 is not merged while this report is in-progress.

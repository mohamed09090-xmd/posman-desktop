# POSMAN changelog

All notable product changes are recorded here. POSMAN follows semantic
versioning for distributable releases.

## 1.0.0 — 2026-08-09

First supported offline Windows release.

### Included

- Arabic-first and French-supported commercial-management workspace.
- Local company setup, authentication, roles, permissions, catalogue, and partners.
- Inventory, purchasing, sales, automatic accounting, payments, and allocations.
- Historical documents, PDF/printing, reports, audit presentation, and backup/restore.
- Bundled SQLite, typed Tauri command boundary, fixed-point business arithmetic,
  idempotency, company scoping, and append-only business history.
- Offline NSIS installer with an embedded WebView2 offline installer.
- Upgrade and uninstall behavior that preserves `%LOCALAPPDATA%\POSMAN`.
- Approved POSMAN application, installer, shortcut, and uninstaller icon.

### Distribution policy

- Windows 10/11 64-bit.
- Stable channel, version `1.0.0`.
- Downgrades are blocked.
- Cloud sync, telemetry, mandatory activation, and an automatic updater are not included.
- The repository does not contain a code-signing certificate or private key. See
  `docs/release/CODE-SIGNING.md` before distributing a public installer.

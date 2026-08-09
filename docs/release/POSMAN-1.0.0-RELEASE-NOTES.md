# POSMAN 1.0.0 — Offline Windows release

POSMAN 1.0.0 is the first complete Windows-first release for Algerian merchants.
It operates locally in Arabic or French without a database server or a mandatory
internet connection.

## What is included

- Company and fiscal setup, users, roles, permissions, local sessions, and recovery.
- Products, prices, units, families, warehouses, locations, customers, and suppliers.
- Inventory, CUMP/CMUP, reservations, purchasing, sales, returns, and lineage.
- Automatic and manual accounting, customer receipts, supplier payments,
  allocations, statements, ledgers, trial balance, and period control.
- Historical document templates, PDF/printing, operational reports, audit views,
  local backups, verification, and guarded restore.

## Installer

The deliverable is `POSMAN-Setup-Offline.exe`. It:

- installs the application under `Program Files\POSMAN`;
- includes the WebView2 offline installer and needs no internet connection;
- creates desktop and Start menu shortcuts;
- offers Arabic, French, and English installer languages;
- blocks installation of an older version over a newer version;
- preserves `%LOCALAPPDATA%\POSMAN` during upgrade and uninstall.

The installer requires Windows administrator approval because the application
binary is installed per-machine. Business data remains per-user.

## Minimum target hardware

- Windows 10 or Windows 11, 64-bit.
- Intel Celeron N4020-class processor or better.
- 4 GB RAM minimum; 8 GB recommended.
- 1 GB free disk space for installation, working data, documents, and backups.
- 1024×640 minimum display; 1280×800 or higher recommended.

Barcode scanners that behave as a USB keyboard are supported by normal input
fields. Receipt printers and cash drawers require the store's Windows driver;
device-specific POS printing is not claimed by this release.

## Verification

Every release candidate must publish SHA-256 checksums and evidence for schema,
frontend, Rust, Windows-native compilation, offline installer size, install,
upgrade, uninstall, startup, memory, and data-retention gates.

## Known distribution limitation

No signing certificate was supplied to the repository. The build is
signing-ready, but an unsigned installer can trigger Microsoft SmartScreen.
Apply the controlled external-signing procedure in `CODE-SIGNING.md` before
selling or broadly publishing the installer.

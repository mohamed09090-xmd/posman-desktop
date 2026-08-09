# PHASE 10 — Distribution and hardening architecture

## Release boundary

PHASE 10 changes packaging, release metadata, branding, validation, and operator
documentation. It does not add business tables, commands, cloud services,
telemetry, activation, an updater, or a new product workflow. Migrations
`0001` through `0007` remain byte-for-byte frozen.

## Windows layout and retention

The production NSIS bundle uses `perMachine` installation:

| Surface | Path | Ownership |
| --- | --- | --- |
| Application binaries | `%ProgramFiles%\POSMAN` | installer/uninstaller |
| SQLite database | `%LOCALAPPDATA%\POSMAN\data` | POSMAN user data |
| Backups | `%LOCALAPPDATA%\POSMAN\backups` | POSMAN user data |
| Documents and exports | `%LOCALAPPDATA%\POSMAN\documents`, `exports` | POSMAN user data |
| Templates and logs | `%LOCALAPPDATA%\POSMAN\templates`, `logs` | POSMAN user data |

This separation is mandatory. Tauri's `currentUser` NSIS mode would place the
binary under `%LOCALAPPDATA%\POSMAN`, colliding with the already accepted data
root and making uninstall unsafe. `perMachine` places binaries under Program
Files and requires administrator approval, while the runtime continues to use
the current Windows user's LocalAppData.

The Tauri uninstaller owns the Program Files directory only. POSMAN business
data is preserved across normal upgrade and uninstall. Destructive data removal
is not bundled into the uninstaller and remains a separate, explicit operator
decision after verified backup.

## Offline dependency

`bundle.windows.webviewInstallMode` is `offlineInstaller`. The WebView2 runtime
payload is embedded, so installation does not require network access. The final
installer budget is below 200 MB. The smaller `downloadBootstrapper` and
`embedBootstrapper` modes are prohibited for the v1 offline artifact.

## Release and compatibility policy

- Product, npm, Cargo, Tauri, and lockfile versions are `1.0.0`.
- Release channel: stable.
- Bundle: Windows x64 NSIS only.
- Downgrades: blocked.
- Installer UI languages: Arabic, French, English.
- Rust MSRV: 1.85; Node: 24; npm: 11.
- Windows 10/11 64-bit is the supported operating-system boundary.
- No automatic updater is included; a newer offline installer performs upgrade.

A CI-only v0.9.0 configuration uses the same application and retains the valid
offline WebView2 contract. It exists solely to prove a real NSIS upgrade to
v1.0.0. The production configuration remains offline-complete.

## Brand assets

The approved ledger-shaped POSMAN mark is normalized to a transparent 1024px
master using solid `#1F5A45`. The pinned Tauri CLI generates the Windows PNG and
multi-resolution ICO derivatives. The mark is used for the application,
installer, uninstaller, taskbar, desktop shortcut, and Start menu shortcut.

## Performance and security gates

The release lifecycle job records cold-start time and steady working-set memory
after an installed v1 launch. Targets are five seconds and 250 MB respectively.
A deterministic 100,000-product query exercise verifies bounded paginated
search work outside the React thread. These CI figures are regression evidence;
the published minimum-hardware claim still requires a clean Celeron N4020/4 GB
acceptance pass before large-scale deployment.

Repository policy rejects private signing material, runtime databases, recovery
archives, write-capable workflows, online WebView bootstrap modes, and updater
authority. Release artifacts include SHA-256 checksums and dependency metadata.

Because PHASE 09 is already accepted, its permanent workflow now applies its
schema, migration, security, frontend, Rust, and native compatibility gates to
future changes without reapplying the historical PHASE 09 changed-file
allowlist. The shared Integration workflow retains its event-scoped range and
explicitly recognizes the PHASE 10 release surface.
The accepted UI workflow's malformed push-path indentation is also corrected so
GitHub can parse and run that compatibility gate on `main` again.

## Signing decision

No certificate is available in the authorized repository context. The build is
signing-ready and the production procedure is documented, but CI does not fake
a signature or store a credential. Unsigned artifacts are test/review candidates;
commercial publication should use externally signed artifacts.

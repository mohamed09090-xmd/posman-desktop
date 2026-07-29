# Desktop shell architecture

## Status and purpose

This document records the accepted shared Bootstrap Gate between the completed PHASE 01 SQLite foundation and the separately owned PHASE 02 runtime and PHASE 03 interface work. The bootstrap contains the minimum native desktop structure needed by both later phases; it does not implement either phase.

## Technology choice

POSMAN uses Tauri 2 with a React, TypeScript, and Vite frontend.

- Tauri provides a Windows-first native executable while keeping the shipped application local and offline.
- Rust supplies the native process boundary and will own local application services in PHASE 02.
- React and TypeScript provide the interface composition and type-safe UI boundary that PHASE 03 will extend.
- Vite provides the development and production frontend build without introducing a runtime server into the installed application.

The resolved dependency set is intentionally small:

| Component | Resolved version |
|---|---:|
| Tauri Rust crate | 2.11.5 |
| `tauri-build` | 2.6.3 |
| Tauri CLI | 2.11.3 |
| React | 19.2.8 |
| React DOM | 19.2.8 |
| TypeScript | 6.0.3 |
| Vite | 8.1.0 |
| React Vite plugin | 6.0.2 |

The final CI environment used Node.js 24.18.0, npm 11.16.0, rustc 1.97.1, Cargo 1.97.1, and Python 3.12.

## Project structure

```text
src/                         React/TypeScript bootstrap render proof
src-tauri/src/               native Tauri application entry and real mock-runtime unit test
src-tauri/capabilities/      minimum main-window Tauri capability
src-tauri/icons/             application icon resources
src-tauri/build.rs           target-aware Tauri build and Windows/MSVC manifest linkage
src-tauri/windows-app-manifest.xml
                             deterministic Common Controls v6 application manifest
src-tauri/tauri.conf.json    application identity, window, CSP, and bundle metadata
.github/workflows/desktop-bootstrap-ci.yml
                             Ubuntu/Windows Bootstrap Gate validation
```

## Development commands

```bash
npm ci
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:dev
npm run desktop:check
python scripts/verify_schema.py
```

`npm run desktop:check` builds a debug native application without creating an installer or bundle.

## Platform prerequisites

### Windows

Development requires Windows 10 or 11 64-bit, the current Microsoft Visual C++ build tools and Windows SDK, WebView2, Node.js 24 with npm, and Rust stable using the MSVC target. The application remains Windows-first; the CI Windows runner is the authoritative proof for PE resources and loader behavior.

### Ubuntu validation

Ubuntu CI installs the Tauri 2 WebKitGTK 4.1, compiler, OpenSSL, app-indicator, SVG, and XDO development prerequisites. Ubuntu is a build and regression-validation platform, not a change to the Windows-first product target.

## Application identity and window

- Identifier: `dz.posman.desktop`
- Product name and title: `POSMAN`
- Main window label: `main`
- Initial size: 1280 × 800
- Minimum size: 1024 × 640
- Resizable and centered

The HTML document defaults to Arabic and RTL. French/LTR support remains a required later UI responsibility rather than a completed i18n implementation in the bootstrap.

## Security boundary

The production CSP defaults content to the application itself and its custom protocol. Network-capable browser destinations are not enabled; `connect-src` is restricted to Tauri IPC. Object and frame embedding are disabled, and base/form targets are restricted. The development CSP adds only the local Vite development and WebSocket endpoints.

The sole capability is `main-shell`, scoped to the `main` window with `core:default`. No filesystem, shell, network, updater, sidecar, or business-specific permission was added.

## Offline and process boundary

The installed application has no runtime Node.js server, cloud service, telemetry client, online account, external database, or sidecar. Node.js, npm, Rust, and the native compiler are build-time tools only. SQLite remains the bundled local database architecture established by PHASE 01, but no SQLite runtime service or connection code is part of this gate.

## PHASE 02 and PHASE 03 ownership

After external acceptance of this gate:

- PHASE 02 owns `src-tauri/**`, Tauri platform adapters, runtime CI, and runtime architecture documentation.
- PHASE 03 owns application UI, components, i18n, styles, frontend tests, frontend dependencies, and UI CI.
- Frozen shared roots and exceptions remain governed by `docs/architecture/parallel-wave-02-contract.md`.

This gate does not approve or begin either phase.

## Windows library-unit-test manifest limitation

The real unit test builds the application with `tauri::test::mock_builder()` and `tauri::test::mock_context(...)`. On Windows, that library unit-test harness imports `TaskDialogIndirect` from `comctl32.dll`. Without an embedded Common Controls v6 application manifest, Windows terminates the process before the Rust test harness starts with `0xc0000139`, `STATUS_ENTRYPOINT_NOT_FOUND`.

Cargo does not apply `cargo::rustc-link-arg-tests` to library unit-test harnesses because they are not explicit integration-test targets. This limitation is tracked in Cargo issue 10937, and the corresponding Tauri mock-runtime failure is tracked in Tauri issue 13419.

## Shared Windows/MSVC manifest path

POSMAN follows the Tauri repository workaround authorized by Patch 01C:

1. `build.rs` reads `CARGO_CFG_TARGET_OS` and `CARGO_CFG_TARGET_ENV`.
2. Only `windows` + `msvc` uses `WindowsAttributes::new_without_app_manifest()` so Tauri does not compile a competing default manifest.
3. Normal `tauri-build` code generation, icons, version information, and capabilities remain enabled.
4. The local `windows-app-manifest.xml` is resolved from `CARGO_MANIFEST_DIR`.
5. The build emits `/MANIFEST:EMBED`, `/MANIFESTINPUT:<absolute path>`, and `/WX` through package-wide linker arguments.
6. Non-Windows or non-MSVC targets keep normal default `tauri-build` attributes.

The local manifest matches Tauri 2.11.5's minimal `windows-app-manifest.xml`. Its SHA-256 is:

```text
4636f3ba46080315ac3277d473d433b79c00863c9cdca1c93c0c83554c6a3d43
```

Package-wide linkage is deliberate: Cargo currently cannot target only a library unit-test harness, and every Windows/MSVC executable-like output linking Tauri's Common Controls v6 code needs the same dependency. `/WX` prevents duplicate or ignored-manifest warnings from being concealed.

## Binary evidence

The Windows CI verifies both outputs after the real commands run:

- the exact `posman_desktop_lib-*.exe` unit-test harness contains a non-zero PE Resource Directory and resource `#1` with `Microsoft.Windows.Common-Controls` version `6.0.0.0`;
- the named test prints `application_setup_builds_with_mock_runtime ... ok`;
- the debug `posman-desktop.exe` contains the same manifest dependency;
- a probe finds no second manifest resource;
- normal file-version metadata and an extractable application icon remain present.

The final evidence run is recorded in `docs/BOOTSTRAP-GATE-02-REPORT.md`.

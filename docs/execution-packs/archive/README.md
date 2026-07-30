# Archived POSMAN Execution Packs

These files preserve the exact historical execution instructions and patches used before and during the accepted foundation.

They are **not active packs**. Their baseline SHAs, ownership maps, dependency versions, CI links, and stop conditions may be obsolete. Use them to understand decisions and as templates for future packs; never rerun them blindly.

| Archive | Historical purpose |
| --- | --- |
| `PHASE-01-DATA-FOUNDATION.md` | Create the GitHub repository data foundation, schema, invariants, and architecture |
| `BOOTSTRAP-GATE-02-03-DESKTOP-SHELL.md` | Establish the shared Tauri/React shell before parallel runtime/UI work |
| `PHASE-02-RUNTIME-FOUNDATION.md` | Add embedded SQLite initialization and the safe runtime status contract |
| `PHASE-03-ORIGINAL-UI-FOUNDATION.md` | Build and validate the Contemporary Operations Ledger UI foundation |
| `patches/PATCH-01A-SQLITE-INTEGRITY.md` | Close SQLite ID and reparenting integrity gaps |
| `patches/PATCH-01B-WINDOWS-RUST-TEST.md` | Diagnose the Windows Rust test loader failure under strict constraints |
| `patches/PATCH-01C-TAURI-WINDOWS-MANIFEST.md` | Apply the accepted official Tauri/MSVC manifest solution |

Accepted outcomes are recorded in the phase reports on `main`; those reports outrank an old pack when describing what actually shipped.

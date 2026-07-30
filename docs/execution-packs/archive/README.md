# Archived POSMAN Execution Packs

These files preserve authoritative historical execution instructions and patches that are actually available in the repository. They are **not active packs**. Their baseline SHAs, ownership maps, dependencies, CI links, and stop conditions may be obsolete; never rerun them blindly against a newer baseline.

| Archive | Historical purpose |
| --- | --- |
| `PHASE-01-DATA-FOUNDATION.md` | Create the SQLite data foundation, invariants, and architecture |
| `BOOTSTRAP-GATE-02-03-DESKTOP-SHELL.md` | Establish the shared Tauri/React shell before parallel runtime/UI work |
| `PHASE-02-RUNTIME-FOUNDATION.md` | Add embedded SQLite initialization and the safe runtime status command |
| `PHASE-03-ORIGINAL-UI-FOUNDATION.md` | Build and validate the Contemporary Operations Ledger UI foundation |
| `patches/PATCH-01A-SQLITE-INTEGRITY.md` | Close SQLite ID and reparenting integrity gaps |
| `patches/PATCH-01B-WINDOWS-RUST-TEST.md` | Diagnose the Windows Rust test loader failure under strict constraints |
| `patches/PATCH-01C-TAURI-WINDOWS-MANIFEST.md` | Apply the accepted Tauri/MSVC manifest solution |

## PHASE 04 and Hotfix 04C archival status

Accepted outcomes are documented by:

- `docs/PHASE-04-REPORT.md`
- `docs/HOTFIX-04C-REPORT.md`
- `docs/architecture/frontend-runtime-integration.md`
- merged PR #6 and merged PR #7

No authoritative original PHASE 04 or Hotfix 04C execution prompt is present in the accessible repository at this checkpoint. Therefore no reconstructed prompt is archived or represented as the original. Archive such material only when an authoritative source is supplied and its provenance can be recorded.

Accepted reports and merged repository evidence outrank an old execution pack when describing what shipped.

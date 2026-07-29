# POSMAN UI foundation

## Scope

This foundation is a fixture-only React gallery for the Tauri shell. It defines the visual language and reusable UI contracts for later operational phases. It does not connect to SQLite, call Tauri commands, calculate stock, post accounting entries, save documents, print, or access a network.

## Tokens

The canonical variables live in `src/styles/tokens.css`.

| Role | Value |
|---|---|
| Application background | `#F4F0E7` |
| Document surface | `#FFFCF6` |
| Primary text | `#1E2523` |
| Secondary text | `#66706C` |
| Border | `#D8D0C2` |
| Confirmed | `#1F5A45` |
| Pending | `#B7791F` |
| Error / shortage | `#B74A3C` |
| Radius | 4–6px |
| Spacing | 4 / 8 / 12 / 16 / 24 / 32px |
| Feedback motion | 120–180ms |

Tokens also define focus treatment, limited elevation, z-index layers, command/rail dimensions, and dense/comfortable table rows. Components do not introduce arbitrary palette values or large decorative radii.

## Typography and offline font boundary

The intended families are IBM Plex Sans Arabic and IBM Plex Sans under the SIL Open Font License 1.1. The license is stored at `public/fonts/OFL.txt`. The implementation session could verify the official IBM source and license but could not reliably transfer complete binary WOFF2 assets through the available repository connector. No unknown or truncated binary was committed. The current CSS therefore uses an explicit offline Windows system fallback stack and records the absent WOFF2 files as a visual limitation. No CDN, Google Fonts request, or runtime font request exists.

When official binaries are introduced through an approved integration patch, only the required regular and semibold weights should be bundled, with `@font-face` declarations pointing to local files.

## Layout anatomy

1. **Command Bar** — compact product identity, current workspace, fixture search, contextual create action, language switch, demonstration company, and local/offline state.
2. **Workspace Rail** — seven numbered ledger-index entries. It is not a rounded admin sidebar.
3. **Workspace Header** — title, explanatory copy, fixture boundary, and contextual page tabs.
4. **Workspace Content** — internally scrollable operational area that preserves the shell.
5. **Document Canvas** — bounded commercial document surface with header, lines, totals, notices, and Action Dock.
6. **Fixture Boundary** — persistent text clarifying that no customer data is saved.

## RTL and LTR strategy

`I18nProvider` defaults to `ar-DZ`, switches to `fr-DZ` without reload, and updates `document.documentElement.lang` and `dir`. Dictionaries are type-checked against the Arabic key set. Components use one DOM structure and CSS logical properties; there is no separate Arabic layout. Directional styling is limited to meaningful selection edges and process flow. Currency, dates, and numbers use `Intl` with DZD and `Africa/Algiers`.

## Density

The primary data grid uses a dense 42px row. Document tables use a comfortable 50px row. Tables are the visual center, while sections and notices use borders and spacing rather than repeated floating cards. Large future datasets require virtualization, but the gallery uses intentionally small typed fixtures and does not add a heavy grid library.

## Motion

Transitions are restricted to focus, hover, button feedback, loading indicators, and the skip link. A `prefers-reduced-motion: reduce` contract reduces animation and transition durations to effectively zero. No animation framework is included.

## Accessibility contract

- Semantic `header`, `nav`, `main`, `section`, `article`, `aside`, `table`, and `footer` landmarks.
- Skip link targets a programmatically focusable main region.
- Visible focus for buttons, links, fields, selects, and keyboard-selectable rows.
- Labels bound to all editable controls.
- Status stamps combine text, a symbol, border, and color.
- Row selection supports Enter and Space.
- Drawer is a non-modal complementary region; the source grid remains available at desktop sizes.
- No hover-only information or icon-only ambiguous action.
- Axe browser evidence rejects critical and serious violations on primary screens.

## Window and responsive behavior

The desktop target is 1024×640 minimum and 1280×800 default. At 1024px the command bar compresses company metadata, the rail narrows, document fields reflow, and tables scroll internally. At high text scaling, the compact breakpoint converts the rail to a horizontal index and stacks form/document regions. The product is not designed as a mobile application, but CSS remains structurally safe below the desktop target for zoom resilience.

## Fixture-only boundary

All records under `src/features/ui-gallery/fixtures/` are deterministic demonstration data. UI actions only update React presentation state and display explicit feedback. There is no Tauri `invoke`, service adapter, storage API, HTTP client, WebSocket, telemetry, or external asset request.

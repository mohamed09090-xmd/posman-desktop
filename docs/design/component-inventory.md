# POSMAN Phase 03 component inventory

| Component | Purpose | Variants / states | Accessibility contract | Future integration point |
|---|---|---|---|---|
| `CommandBar` | Global operational commands and context | Search fixture, contextual create label, Arabic/French, offline status | Header landmark, labelled search, text labels for actions | Connect search and create commands after service APIs are authorized |
| `WorkspaceRail` | Stable seven-workspace index | Default, hover, focus, active | Navigation landmark, `aria-current`, keyboard-native buttons | Route to real workspace state while preserving identifiers |
| `WorkspaceHeader` | Workspace title and contextual pages | Single page or tab set | Heading hierarchy and page-current tabs | Bind route/history state if an approved router is introduced |
| `DocumentCanvas` | Common commercial document frame | Document-specific children | Article with explicit accessible label | Receive validated document view models |
| `ProcessStrip` | Show document transformation lineage | Completed, current, pending | Ordered list, `aria-current=step`, symbol plus text | Bind immutable lineage and partial transformation data |
| `StatusStamp` | Operational document or stock state | Confirmed, pending, shortage, draft, posted, available, low, out | Text and symbol; color is supplemental | Map domain states to reviewed presentation labels |
| `DataGrid` | Compact operational tables | Dense, comfortable, selected, empty | Semantic table/caption, keyboard-selectable rows | Add server/local query paging and virtualization for large datasets |
| `DetailDrawer` | Inspect selected record without losing list position | Open/closed, non-modal | Complementary region, labelled heading, close button | Bind read model and authorized edit commands |
| `ActionDock` | Keep document actions close to context | Arbitrary typed button composition | Footer with accessible label, no hidden destructive action | Bind validation/posting commands with explicit pending/error states |
| `Field` | Label, hint, required, and error composition | Default, required, error | Native label binding and descriptive messages | Bind form schema and validation results |
| `Input` | Text, search, numeric, and date-like values | Native focus, invalid, readonly, disabled | Native input semantics | Bind controlled form state and decimal-safe adapters |
| `Select` | Bounded choices | Default, focus, invalid, disabled | Native select semantics | Populate approved reference data |
| `Button` | Explicit actions | Primary, secondary, quiet, danger, loading, disabled | Native button, `aria-busy`, visible focus | Bind command execution and idempotent progress states |
| `InlineNotice` | Explain result, warning, or next step | Info, success, warning, error, optional live | Heading plus body; optional polite live region | Render typed domain/application errors in human language |
| `EmptyState` | Explain absence and next action | Generic contextual content | Status region with heading and text | Bind query results and authorized creation actions |
| `LoadingState` | Communicate short asynchronous loading | Reduced-motion compatible | Polite status text | Bind local service/query loading without decorative blocking animation |

## Composition rules

Components use semantic HTML first and add ARIA only where native semantics do not express current page, current step, selection, live feedback, or loading. Shared abstractions exist only where used across multiple screens. No component-library theme or generic dashboard template is imported.

# PHASE 03 design direction study

## Decision

POSMAN implements **دفتر العمليات المعاصر / Contemporary Operations Ledger**. The Blueprint already approves this direction; this study records the alternatives and the implementation rationale rather than reopening the product decision.

## Direction 1 — Contemporary Operations Ledger (selected)

### Composition

A persistent command bar, a narrow indexed workspace rail, and a document-oriented work area. Operational sections use ruled headers, precise separators, and ledger-like tables. Commercial documents occupy a bounded document canvas with a contextual action dock.

### Density

Moderate-to-high information density, with 42px dense rows and 50px comfortable document rows. White space separates operational groups rather than turning every item into a card.

### Navigation

The rail behaves like the index of an operations register. Seven workspaces remain stable while contextual pages appear in the workspace header. This avoids a generic admin sidebar and preserves the user's location.

### Typography

Arabic-first typographic rhythm with a practical Windows fallback stack. IBM Plex Sans Arabic / IBM Plex Sans remain the intended offline families; the official OFL license is preserved in `public/fonts/OFL.txt`.

### Interaction model

Keyboard-visible controls, row selection, contextual drawers, document process strips, explicit status stamps, and actions placed near the active business context. Motion is limited to 120–180ms feedback and disabled under reduced motion.

### Accessibility

Semantic landmarks, heading order, a skip link, labels, focus-visible treatment, status symbols plus text, and internal scrolling at the minimum window size.

### Maintenance cost

Moderate. Tokens and a small typed component inventory are reusable without importing a broad component framework or a parallel layout system.

## Direction 2 — High-density command ledger

### Composition and density

A denser spreadsheet-first surface with permanently visible filter and command columns. It maximizes rows per screen and reduces document framing.

### Navigation and interaction

Keyboard shortcuts and command palettes dominate. Most edits happen in-line, and secondary details open in a narrow inspector.

### Accessibility and maintenance

Efficient for trained operators but harder for first-time merchants, text scaling, and touch targets. It requires more complex grid behavior, virtualization, shortcut conflict handling, and stronger onboarding.

### Why it was not selected

It conflicts with the Blueprint principle of clarity before density. It can become visually indistinguishable from an ERP data-entry grid and would increase the cognitive load for non-technical merchants.

## Direction 3 — Guided beginner journal

### Composition and density

A simplified sequence of large task panels, progressive disclosure, and one primary action per step. Data tables are reduced until a workflow is chosen.

### Navigation and interaction

The product guides the user through sale, purchase, stock, and setup flows. Navigation is task-oriented rather than workspace-oriented.

### Accessibility and maintenance

Easy to learn and compatible with larger text, but slow for repeat operators. Maintaining separate guided and expert paths would duplicate interaction logic and increase long-term product cost.

### Why it was not selected

POSMAN needs a durable operational language for both new and experienced merchants. A permanently guided flow would obstruct frequent work and weaken the document and ledger identity.

## Comparative summary

| Criterion | Operations ledger | High-density command ledger | Guided beginner journal |
|---|---|---|---|
| Composition | Indexed register and documents | Spreadsheet-first | Step-by-step panels |
| Density | Balanced | Very high | Low |
| Navigation | Stable workspaces | Commands and tabs | Guided tasks |
| Typography | Calm operational hierarchy | Compact utility hierarchy | Large instructional hierarchy |
| Interaction | Contextual actions and drawers | Inline editing and shortcuts | Wizard-like progression |
| Accessibility | Strong, balanced targets | Highest complexity | Strong but verbose |
| Maintenance | Moderate | High | High due to dual paths |

## Originality and anti-copy statement

The selected UI is derived from POSMAN's commercial workflow: operations are grouped like a register, document state appears as an operational stamp, document lineage is shown as a process strip, and actions remain attached to their document context. It does not copy Sage or another named product, and it deliberately avoids admin-template sidebars, KPI-card grids, bento layouts, glass effects, decorative gradients, oversized radii, hero sections, stock imagery, and ornamental animation.

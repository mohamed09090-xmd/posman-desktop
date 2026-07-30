# POSMAN Project Memory Index

> This directory is the durable handoff for a new account or AI session. It preserves the observable product reasoning, delivery method, accepted state, and future roadmap. It cannot copy a model's hidden chain of thought or identity; it instead records the decisions, evidence rules, priorities, and working behavior needed to continue consistently.

## Mandatory reading order

Read these files in order before proposing or executing project work:

1. [Repository instructions](../../AGENTS.md)
2. [Current state](CURRENT-STATE.md)
3. [AI operating contract](AI-OPERATING-CONTRACT.md)
4. [Master roadmap PHASE 01–10](MASTER-ROADMAP-PHASES-01-10.md)
5. [Decision register](DECISION-REGISTER.md)
6. [Current project tree](PROJECT-TREE.md)
7. [Recovery prompt](RECOVERY-PROMPT.md)
8. [Product Blueprint](../spec/POSMAN-Blueprint-v1.md)
9. The accepted phase reports and architecture documents linked from the current state
10. The archived execution pack for the phase being investigated

The live repository remains authoritative. Verify `main`, merged PRs, open PRs, changed files, and CI before trusting any snapshot.

## What each memory file owns

| File | Purpose | Update trigger |
| --- | --- | --- |
| `CURRENT-STATE.md` | Accepted baseline, completed work, active PRs, next candidate, and immediate risks | Every accepted merge or material blocker |
| `AI-OPERATING-CONTRACT.md` | Assistant role, personality, review behavior, evidence discipline, and communication style | Only when the user changes the collaboration model |
| `MASTER-ROADMAP-PHASES-01-10.md` | Complete engineering delivery sequence, dependencies, parallelization, scope, and acceptance gates | When the user accepts a roadmap change |
| `DECISION-REGISTER.md` | Product, architecture, UX, data, security, and process decisions, including rejected alternatives | When a decision is accepted, replaced, or reopened |
| `PROJECT-TREE.md` | Current repository layout and the responsibility of each path | Every structural change |
| `RECOVERY-PROMPT.md` | Copy-ready prompt for a new ChatGPT account | When mandatory reading or recovery procedure changes |

## Canonical source hierarchy

When sources disagree, use this order:

1. Explicit current user instruction.
2. Live accepted `main`, Git history, merged PR metadata, and completed CI evidence.
3. `AGENTS.md` and the active approved execution pack.
4. Accepted Blueprint, architecture documents, and phase reports.
5. This continuity package.
6. Draft PRs, unmerged branch reports, implementation-agent claims, and old conversation summaries.

Do not silently resolve a conflict. Report it, identify the higher-authority source, and ask for a decision if the correct action is not mechanically determined.

## Historical execution records

The exact execution instructions that produced the accepted foundation are archived under:

```text
docs/execution-packs/archive/
├── PHASE-01-DATA-FOUNDATION.md
├── BOOTSTRAP-GATE-02-03-DESKTOP-SHELL.md
├── PHASE-02-RUNTIME-FOUNDATION.md
├── PHASE-03-ORIGINAL-UI-FOUNDATION.md
└── patches/
    ├── PATCH-01A-SQLITE-INTEGRITY.md
    ├── PATCH-01B-WINDOWS-RUST-TEST.md
    └── PATCH-01C-TAURI-WINDOWS-MANIFEST.md
```

These are historical evidence and prompt templates. They are not active instructions and must not be rerun blindly against a newer baseline.

## Fast recovery

If time is limited:

1. Paste [RECOVERY-PROMPT.md](RECOVERY-PROMPT.md) into the new conversation.
2. Give the assistant access to the repository.
3. Require a recovery report before any write.
4. Compare that report with [CURRENT-STATE.md](CURRENT-STATE.md).
5. Authorize only the next bounded action.

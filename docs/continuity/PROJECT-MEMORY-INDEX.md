# POSMAN Project Memory Index

> Recovery checkpoint through accepted PHASE 04 and POST-MERGE HOTFIX 04C. Resolve live repository state from GitHub during every recovery; this package records a stable product-code baseline and continuity delivery, not a permanent live branch position.

## Stable recovery coordinates

| Item | Value |
| --- | --- |
| Repository | `https://github.com/mohamed09090-xmd/posman-desktop` |
| Accepted product-code baseline through Hotfix 04C | `73c3afed19c8bf4841d0c65fc85b7d0c4c3ef307` |
| Latest accepted product phase | PHASE 04 — Frontend Runtime Integration |
| Latest accepted correction | POST-MERGE HOTFIX 04C |
| Continuity Checkpoint 04 delivery PR | [PR #5](https://github.com/mohamed09090-xmd/posman-desktop/pull/5) — verify its live state on GitHub |
| Live `main` | Resolve from GitHub; never infer permanently from this file |
| Next candidate | PHASE 05 — planned, unstarted, unauthorized |

The accepted product-code baseline is the final product implementation through PHASE 04 and Hotfix 04C. A merge of PR #5 may create a docs-only successor on `main`; that successor is a continuity checkpoint, not PHASE 05 and not a product-code implementation.

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
9. [PHASE 04 report](../PHASE-04-REPORT.md)
10. [Hotfix 04C report](../HOTFIX-04C-REPORT.md)
11. [Frontend runtime integration architecture](../architecture/frontend-runtime-integration.md)
12. Other accepted phase reports and architecture documents linked from [CURRENT-STATE.md](CURRENT-STATE.md)

Then resolve live `main`, PR #5 state, merged PR metadata, changed files, and completed GitHub Actions. Do not continue automatically from an unmerged branch.

## Recovery state model

### Before PR #5 is merged

- Live `main` may equal the accepted product-code baseline.
- Continuity files may exist only on PR #5.
- PR #5 content is delivery evidence but is not accepted continuity state until merged.

### After PR #5 is merged

- Live `main` may be one docs-only squash commit ahead of the accepted product-code baseline.
- Verify that PR #5 merged and compare the product-code baseline to live `main`.
- Treat the newer `main` as the continuity-checkpoint successor only when the difference is limited to the approved PR #5 paths: `AGENTS.md`, `docs/continuity/**`, and `docs/execution-packs/archive/**`.
- Do not classify that docs-only successor as PHASE 05 or as product-code work.

If live `main` includes another product or out-of-scope change, report drift and stop. Do not accept it automatically.

## Source hierarchy

1. Explicit current user instruction.
2. Live accepted `main` resolved from GitHub, Git history, merged PR metadata, and completed CI evidence.
3. `AGENTS.md` and the active approved execution pack.
4. Accepted Blueprint, architecture documents, and phase reports.
5. This continuity package.
6. Delivery PRs, unmerged reports, implementation-agent claims, and old conversation summaries.

Report conflicts instead of silently resolving them.

## Memory file ownership

| File | Purpose | Update trigger |
| --- | --- | --- |
| `CURRENT-STATE.md` | Product-code baseline, delivery ledger, implemented boundary, continuity delivery, and next candidate | Every accepted merge or material blocker |
| `AI-OPERATING-CONTRACT.md` | Roles, evidence discipline, Git rules, recovery baseline rules, safety, and communication behavior | Collaboration or delivery-policy change |
| `MASTER-ROADMAP-PHASES-01-10.md` | Accepted versus planned phases, dependencies, scope, exclusions, and gates | Accepted roadmap change or phase acceptance |
| `DECISION-REGISTER.md` | Accepted product, architecture, data, UX, security, and process decisions | Decision accepted, replaced, or reopened |
| `PROJECT-TREE.md` | Accepted product-code tree and continuity-delivery paths | Structural change |
| `RECOVERY-PROMPT.md` | Copy-ready recovery instruction for a new account | Recovery baseline or procedure change |

## Historical execution records

Exact historical packs currently available are indexed in [the archive README](../execution-packs/archive/README.md). They are evidence, not active instructions. No PHASE 04 or Hotfix 04C prompt is archived unless an authoritative original source is available; reconstructed text must never be presented as the original.

## Public repository warning

The repository is public. Never commit secrets, credentials, tokens, private keys, real `.env` files, customer or company data, production databases, SQLite runtime artifacts, backups, or private logs.

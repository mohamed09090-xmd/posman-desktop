# POSMAN — PHASE 05 Delivery Report

## 1. Delivery status

PHASE 05 materializes setup, local security, company configuration, reference-data CRUD, fixed-point pricing, typed Tauri commands, and the Arabic/French operational interface in Draft PR #8.

The implementation is intentionally not self-approved. Final acceptance remains with the independent architect/reviewer. Final-head GitHub Actions links and artifact metadata are recorded in the Pull Request description after the validation commit completes, so an evidence-only commit does not invalidate the tested head.

## 2. Coordinates and ownership

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Branch: `phase/05-setup-security-reference-data`
- Draft Pull Request: `https://github.com/mohamed09090-xmd/posman-desktop/pull/8`
- Authorized starting head: `7c51c59aabec52b534bb76187a8e19130da3d081`
- Original required main coordinate: `77d297f08ec3c46920dd1b94710995834854dd9a`

The authorized head was stable for 60 seconds with no running Actions before ownership was acquired. Every branch update was fast-forward only and preceded by a remote-head read. No direct main commit, force-push, rebase, reset, history rewrite, merge, or auto-merge was used.

## 3. Source materialization

The verified source archive was reconstructed only after every chunk hash, the assembled Base64 hash, and the decoded `tar.xz` hash matched. The archive was then extracted and committed as ordinary source in:

```text
aea2ef960ad1bd7d8315f04b2ff5af2002829df9
feat(phase05): materialize setup security reference UI
```

The materialization commit exposes database migration/seed/verifier changes, Rust PHASE 05 services and command wiring, the React feature and gateway, UI/integration/E2E tests, and permanent CI inputs directly in the PR tree.

All `.phase05-delivery/**` chunks and all delivery, snapshot, formatting, patch-builder, and transport workflows are removed from the final tree. No payload, archive, or workflow with `contents: write` is permanent.

Materialization evidence:

- Source archive run: `30746603570`
- Source archive artifact: `8833076880`
- Size: `58,890` bytes
- Digest: `sha256:f9fbd9c05cc35eaef069f02f150521ef1251ffa4dbab81df0183a6502111eaea`
- Repository snapshot run: `30746647247`
- Snapshot artifact: `8833089846`
- Size: `300,866` bytes
- Digest: `sha256:8e4e525f437b43a8a153a09fee9606dfbd10c8036c40ae05eccfcb4766588552`

## 4. Implemented product scope

The committed implementation contains:

- atomic, idempotent initial setup;
- company, fiscal year, and monthly fiscal periods;
- tax, default margin, below-cost policy, inactivity timeout, and document numbering configuration;
- Argon2id password hashing;
- local login, logout, sessions, inactivity handling, and one-time recovery code rotation;
- users, roles, permissions, and last-System-Administrator protection;
- families, units, taxes, payment terms/methods, warehouses, and locations;
- customers, suppliers, addresses, and contacts;
- products, fixed-point purchase/sale prices, suggested price, and configurable below-cost behavior;
- typed Tauri commands and safe error normalization;
- company scoping, transactional audit, and optimistic concurrency;
- Arabic RTL and French LTR operational UI connected to the real desktop gateway.

No stock posting, purchasing, sales, payments, accounting posting, printing/PDF, backup/restore, installer, cloud, telemetry, HTTP API, or PHASE 06 implementation is included.

## 5. SQLite verification

`python scripts/verify_schema.py` reports:

```text
POSMAN SQLite verification: PASS
migrations: 5
schema version: 0005
tables: 52
triggers: 25
passed checks: 133
pending application invariants: 1
```

The verifier additionally confirms:

- five contiguous migrations through `0005`;
- accepted migration hashes for `0001`–`0004` unchanged;
- `database/schema.sql` exactly matches ordered migrations;
- fresh application of `0001`–`0005`;
- deterministic seed applied twice;
- foreign keys enabled and `foreign_key_check` empty;
- no application column declared as `REAL`;
- all expected tables, primary keys, and accepted integrity triggers;
- setup, session, recovery, margin, below-cost, sequence, audit, posting-immutability, and fixed-point constraints;
- real accepted `0004` database upgraded to `0005` with safe defaults and no FK violations.

The single pending invariant is the previously documented aggregate transformed-quantity service invariant. It belongs to later commercial-document flows and is not implemented or claimed by PHASE 05.

## 6. Accessibility defect and correction

Initial E2E evidence detected exactly one Axe violation:

```text
Rule ID: color-contrast
Impact: serious
Target: .p5-table--products > thead > tr > th:nth-child(1)
Element: <th>الرمز والمادة</th>
Foreground: #5f6965
Background: #e7e3d7
Measured ratio: 4.42:1
Required ratio: 4.5:1
```

The product-table header text was changed to `#56605c`, raising the calculated ratio to approximately `5.08:1`. The Axe rule, threshold, and failure condition remain unchanged. No allowlist or suppression was introduced.

The harness now writes complete Axe JSON and diagnostic JSON containing rule ID, impact, help, help URL, targets, bounded HTML, and failure summary. Screenshots and reports upload with `if: always()`.

Post-fix browser validation covers:

- Arabic `1280×800`;
- French `1280×800`;
- Arabic `1024×640`;
- French `1024×640`;
- zero Axe violations;
- zero unresolved critical/serious incomplete results;
- no horizontal overflow;
- no clipping of primary labels;
- no console or page errors.

## 7. Backend correction discovered during completion

The company-profile service previously passed `below_cost_policy` to an SQL statement that did not bind or update that column. The corrected statement persists:

- `default_margin_rate_scaled`;
- `below_cost_policy`;
- `session_idle_timeout_minutes`;
- `updated_at` and `updated_by`;
- incremented `row_version`.

A SQLite regression test executes the exact statement and verifies the updated margin, policy, timeout, and version. Two unregistered update DTOs for partner address/contact changes were removed rather than leaving a misleading public contract, and product warning classification now uses an explicit `Ordering` match required by Clippy.

## 8. Test and build commands

The permanent PHASE 05 workflow executes the following exact commands on the committed source:

```text
python scripts/verify_schema.py
npm ci
npm run typecheck
npm run build
npm run test:ui
npm run test:integration
npm run test:e2e
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:check
git diff --check
git status --short --untracked-files=all
```

A post-contrast run established the browser result and database cross-platform result before the final backend correction:

- PHASE 05 run `30746987416`;
- frontend job `91494145408`: success;
- database Ubuntu job `91494145437`: success;
- database Windows job `91494145431`: success;
- policy job `91494145425`: success;
- Rust formatting and locked check succeeded on Ubuntu and Windows before Clippy identified three actionable warnings.

Final-head results, Windows manifest evidence, and the final `phase-05-ui-evidence` artifact are recorded in PR #8 after the clean validation commit finishes.

## 9. CI policy

`.github/workflows/phase05-ci.yml` and all compatibility workflows have top-level:

```yaml
permissions:
  contents: read
```

The permanent policy job rejects any real `contents: write` line and rejects known helper, payload, snapshot, transport, and builder paths. It also checks patch whitespace and a clean worktree.

The PHASE 05 workflow validates Ubuntu and Windows database behavior, Ubuntu and Windows Rust 1.85 behavior, native desktop compilation, the embedded Windows Common Controls v6 manifest, the secret/database-artifact scan, and browser evidence retained even when E2E fails.

## 10. Architecture decisions

1. SQLite remains bundled and local; no server, cloud database, online account, telemetry, or subscription dependency is introduced.
2. Initial setup is atomic and idempotent.
3. Authentication and authorization are enforced in Rust with Argon2id, bounded sessions, hashed one-time recovery, and last-administrator protection.
4. All mutations are company-scoped, audited, and concurrency-checked.
5. Money and rates use approved fixed-point integer rules.
6. The persisted below-cost policy is authoritative; UI warnings cannot bypass it.
7. React uses only the typed local Tauri gateway and never executes SQL.
8. Arabic RTL is the default; French LTR is fully tested.
9. Final evidence is attached to the tested commit through GitHub Actions and the PR description rather than an evidence-only follow-up commit.

## 11. Risks and limitations

- The repository base branch advanced after the originally supplied main coordinate. No main history was rewritten or modified by this phase; PR #8 remains based on `main` and requires independent reviewer reconciliation.
- The transformed-quantity aggregate invariant remains explicitly pending for later commercial-document service work.
- Browser E2E uses a deterministic DEV invoker fixture, while production desktop resolution uses the real Tauri gateway; Rust command tests and native builds cover the backend boundary.
- This report does not approve the phase. The architect/reviewer must accept or reject it.

## 12. Scope and PR confirmations

```text
PR #8 remains Open.
PR #8 remains Draft.
PR #8 is not merged.
Auto-merge is not enabled.
No force-push, rebase, reset, or history rewrite was used.
No direct commit was made to main.
PHASE 06 was not started.
```

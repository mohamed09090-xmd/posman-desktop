# Parallel Wave 02 ownership contract

## Purpose

This contract is the shared file-ownership boundary after the Bootstrap Gate is accepted. It prevents PHASE 02 runtime work and PHASE 03 UI work from changing the same manifests, roots, or frozen integration files in parallel.

No exception is implicit. A phase that genuinely needs a file outside its ownership must stop and request an explicit integration patch decision from the architect/reviewer.

## PHASE 02 runtime owner

PHASE 02 owns:

```text
src-tauri/**
src/platform/tauri/**
.github/workflows/runtime-ci.yml
docs/architecture/runtime-*.md
docs/PHASE-02-REPORT.md
```

PHASE 02 may update Rust dependencies and `src-tauri/Cargo.lock`.

PHASE 02 must not modify:

```text
src/app/**
src/components/**
src/i18n/**
src/styles/**
frontend test files
package.json
package-lock.json
vite.config.ts
```

unless an explicit integration patch is approved by the architect.

## PHASE 03 UI owner

PHASE 03 owns:

```text
src/app/**
src/components/**
src/i18n/**
src/styles/**
src/features/ui-gallery/**
frontend test files
public/**
package.json
package-lock.json
vite.config.ts
docs/design/**
.github/workflows/ui-ci.yml
docs/PHASE-03-REPORT.md
```

PHASE 03 may update frontend dependencies and the npm lockfile.

PHASE 03 must not modify:

```text
src-tauri/**
database/**
scripts/verify_schema.py
docs/architecture/database-*.md
```

unless an explicit integration patch is approved by the architect.

## Frozen shared files

After this Bootstrap Gate is accepted, the following files are frozen during parallel PHASE 02 and PHASE 03 execution:

```text
src/main.tsx
index.html
tsconfig.json
tsconfig.app.json
tsconfig.node.json
AGENTS.md
.gitignore
```

If either phase genuinely requires a frozen-file change, it must stop and request an integration decision before editing.

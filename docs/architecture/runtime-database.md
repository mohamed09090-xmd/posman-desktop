# POSMAN local runtime database

## Scope

This document defines the PHASE 02 local SQLite runtime. It initializes the accepted four-migration schema and safe reference seed only. Company setup, authentication, commercial workflows, accounting services, backup/restore, and every frontend integration remain outside this phase.

## Local data paths

The production adapter resolves the operating-system local data directory through Tauri and appends `POSMAN`. No username or absolute machine path is compiled into the application.

On Windows the logical layout is:

```text
%LOCALAPPDATA%\POSMAN\
├── data\
│   └── posman.sqlite3
├── backups\
├── documents\
├── templates\
└── logs\
```

`RuntimePaths` is a pure path model. Production supplies the Tauri-resolved root; tests supply an explicit temporary root and never write to the real user profile.

## SQLite packaging and connection contract

The Rust application uses `rusqlite` directly with its `bundled` feature. The installed application therefore opens an embedded SQLite library and does not require a server, sidecar, Docker, SQLite executable, or customer-installed database runtime.

Every writable connection applies and verifies, before any transaction:

```sql
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA journal_mode = WAL;
```

The runtime rejects the connection when `foreign_keys` is not `1` or the bounded timeout is not `5000` milliseconds. It requests WAL but records and reports the mode SQLite actually returns. This is deliberate: in-memory databases and filesystems that cannot activate WAL may return another mode; the runtime never claims WAL without observing it.

## Embedded migration catalog

The executable embeds the exact UTF-8 contents of these accepted files at compile time:

```text
database/migrations/0001_system_company_security.sql
database/migrations/0002_reference_catalog_partners.sql
database/migrations/0003_commerce_inventory.sql
database/migrations/0004_accounting_documents_audit.sql
```

No SQL copy exists under `src-tauri`, and PHASE 02 adds no fifth migration. Each catalog entry contains the integer identifier, zero-padded version, logical filename stem, SQL text, and SHA-256 calculated from the embedded UTF-8 bytes. The hash algorithm is byte-compatible with `hashlib.sha256(sql.encode("utf-8")).hexdigest()` in the accepted schema verifier.

## Compatibility and refusal rules

Before applying pending work, the runner validates that the embedded catalog is contiguous and reads `app_migrations` when it exists. The existing ledger must be an exact prefix of the catalog. Every row must match the catalog entry's:

- integer `id`;
- zero-padded `version`;
- logical `name`;
- `checksum_sha256`.

Startup is rejected as a fatal runtime error for an unknown newer version, checksum mismatch, missing/gapped row, partial ledger, or mismatched metadata. POSMAN does not reset, downgrade, or continue business startup against an incompatible schema.

## Atomic migration algorithm

For each pending migration, in catalog order:

1. open `BEGIN IMMEDIATE` through a rusqlite immediate transaction;
2. execute the embedded SQL;
3. insert the matching `app_migrations` row and UTC application timestamp;
4. commit SQL and ledger row together.

The first migration creates `app_migrations` and is recorded inside the same transaction. Any error drops/rolls back that transaction, records no ledger row, prevents every later migration, and aborts runtime initialization. Down migrations are not implemented.

## Reference seed policy

After all four migrations are compatible and applied, the runtime executes the embedded `database/seed/reference_data.sql` in a separate immediate transaction. Its accepted `INSERT OR IGNORE` statements are idempotent. A seed failure rolls back the complete seed transaction.

The seed creates only global system role templates, permissions, and grants. It creates no company, user, password, tax rate, accounting account, or customer data.

## Post-initialization integrity gate

Readiness is published only after all of the following succeed:

- `PRAGMA foreign_key_check` returns zero rows;
- all 49 accepted tables exist;
- exactly four migration rows exist;
- the current schema version is `0004`;
- the verified connection contract remains active.

There is one authoritative schema: the accepted migration sequence. The runtime does not create a parallel schema.

## Startup and thread model

Tauri `setup` resolves the local root and completes path creation, database opening, migration, seed, and integrity verification before managed runtime state is registered. Failure aborts native startup, so no business command can observe a partially ready database.

Managed state contains a cloneable runtime service and database metadata, not a global `rusqlite::Connection`. Future operations must open a configured connection per blocking unit of work. Tauri commands that perform SQLite work must run it through `spawn_blocking`; the PHASE 02 read-only status command uses the same non-UI-thread boundary and returns only cached readiness metadata.

## Error handling and privacy

Internal errors are typed by path resolution/creation, database open/configuration, unsupported schema, checksum mismatch, invalid ledger, migration failure, seed failure, and integrity failure. Internal diagnostics retain the migration version and developer context. The IPC contract sanitizes failures and never returns SQL text, an absolute database path, user data, or company data.

## Deferred backup requirement

Before any future customer database migration beyond the currently accepted schema, the application must create and verify a recoverable backup. PHASE 02 documents this requirement but does not implement backup or restore.

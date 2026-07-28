# POSMAN SQLite migration policy

## Authority and order

Migration files in `database/migrations/` are authoritative and are applied in ascending filename order. The naming format is:

```text
NNNN_short_descriptive_name.sql
```

Versions are contiguous, zero-padded, and never reused.

## Immutability

After a migration is released or accepted, it must not be edited, renamed, reordered, or silently replaced. A correction is a new roll-forward migration. The SHA-256 checksum stored in `app_migrations` allows the application to detect unexpected mutation.

## Transaction expectations

The migration runner must:

1. enable `PRAGMA foreign_keys = ON`;
2. acquire a write transaction;
3. execute one migration;
4. insert its `app_migrations` ledger row with checksum;
5. commit both changes atomically;
6. stop immediately on failure.

Migration SQL files intentionally omit transaction wrappers so one runtime runner owns atomicity consistently.

## Failure behavior

A failed migration is rolled back. Later migrations must not run. The application must report the migration filename and database error without converting the failure to a warning. Automatic retry is allowed only after the cause is resolved and the installed ledger is verified.

## Production and customer data

Before applying a migration to a customer database, the future application must create and verify a backup. An application upgrade must not delete local data. Restoration must validate integrity and schema compatibility before replacing the active database.

## Development reset

During unreleased development, developers may delete a disposable local database and rebuild from all migrations. Resetting is never an acceptable production migration strategy and must never target a real customer database.

## Schema snapshot

`database/schema.sql` is generated review output, not an independently edited source:

```bash
python scripts/verify_schema.py --write-schema
python scripts/verify_schema.py
```

Verification fails if the snapshot differs from the ordered migrations or builds a different table/trigger set.

## Compatibility

The future application must declare the schema versions it supports. It must refuse to run business operations against a newer unknown schema or an incomplete migration ledger. Compatibility checks occur before opening normal application services.

## Rollback policy

Production rollback is data restoration plus a compatible application version, not a destructive down migration. Forward migrations should preserve data and add explicit conversion steps where needed.

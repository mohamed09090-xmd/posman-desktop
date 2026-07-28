# POSMAN SQLite migration policy

## Authority and naming

Files in `database/migrations/` are authoritative and execute in ascending order. Names use:

```text
NNNN_short_descriptive_name.sql
```

Versions are contiguous, zero-padded, and never reused.

## Immutability and correction

After a migration is released or accepted, it must not be edited, renamed, reordered, or replaced. Every correction is a new roll-forward migration. `app_migrations` records the version, logical name, SHA-256 checksum, and UTC application time so unexpected mutation can be detected.

## Transaction expectations

The future runner must:

1. enable `PRAGMA foreign_keys = ON`;
2. acquire a write transaction;
3. execute one migration;
4. insert its `app_migrations` row and checksum;
5. commit both changes atomically;
6. stop immediately on failure.

Migration files omit transaction wrappers so one runner owns atomicity consistently.

## Failure behavior

A failed migration is rolled back and later migrations do not run. The application must show the migration filename and SQLite error. It must not downgrade failure to a warning or continue normal operations against a partial schema.

## Development reset

Unreleased disposable development databases may be deleted and rebuilt. Reset is prohibited for customer or production data.

## Production safety

Before a customer database migration, the future application must create and verify a backup. Application upgrades must preserve local data. Restoration must validate integrity and schema compatibility before replacing the active database.

## Compatibility

The application must declare supported schema versions and refuse business operations against an unknown newer version, checksum mismatch, or incomplete migration ledger.

## Snapshot

`database/schema.sql` is generated review output, not an independently edited source:

```bash
python scripts/verify_schema.py --write-schema
python scripts/verify_schema.py
```

## Rollback policy

Production rollback means restoring a verified backup and running a compatible application version. Destructive down migrations are not the normal rollback mechanism.

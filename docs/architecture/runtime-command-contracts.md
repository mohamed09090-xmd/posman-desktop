# POSMAN runtime command contracts

## Scope

PHASE 02 exposes one read-only Tauri IPC command for runtime readiness. It is an integration boundary for a later authorized frontend gate; it is not a business API.

## `get_runtime_status`

### Request

No arguments.

### Success response

The response is serialized with camel-case JSON field names:

```json
{
  "databaseReady": true,
  "schemaVersion": "0004",
  "migrationCount": 4,
  "foreignKeysEnabled": true,
  "journalMode": "wal"
}
```

| Field | Type | Meaning |
|---|---|---|
| `databaseReady` | boolean | Startup initialization and integrity verification completed. |
| `schemaVersion` | string | Current accepted migration version. |
| `migrationCount` | integer | Number of validated ledger rows. |
| `foreignKeysEnabled` | boolean | The active connection contract verified `PRAGMA foreign_keys = 1`. |
| `journalMode` | string | The journal mode actually returned by SQLite, not an assumed value. |

The command returns cached immutable readiness metadata from managed state. It executes through Tauri's blocking-task boundary and does not accept SQL or mutate the database.

### Error envelope

A command-dispatch failure is serialized as a safe object:

```json
{
  "code": "RUNTIME_STATUS_UNAVAILABLE",
  "message": "The local runtime status is temporarily unavailable."
}
```

Detailed Rust errors remain in the native startup/error chain. The IPC error never includes:

- an absolute database or local-user path;
- SQL or migration contents;
- usernames, company data, credentials, or secrets;
- raw SQLite diagnostics.

## Readiness semantics

The command is registered only on the application builder whose setup performs runtime initialization. Managed state is installed only after paths, migrations, seed, and integrity checks succeed. A failed startup therefore does not degrade into `databaseReady: false` while allowing normal business use; native startup fails instead.

## Deferred commands

Company setup, authentication, products, partners, inventory, sales, purchases, payments, accounting, documents, printing, backup/restore, and every other business command are outside PHASE 02. No placeholder IPC endpoints are reserved for them.

A later Integration Gate may consume `get_runtime_status` exactly as documented. Any contract expansion requires an authorized phase or integration patch and must preserve the no-path/no-SQL privacy boundary.

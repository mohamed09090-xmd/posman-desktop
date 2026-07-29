use rusqlite::{params, Connection, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::error::RuntimeError;

const MIGRATION_0001_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../database/migrations/0001_system_company_security.sql"
));
const MIGRATION_0002_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../database/migrations/0002_reference_catalog_partners.sql"
));
const MIGRATION_0003_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../database/migrations/0003_commerce_inventory.sql"
));
const MIGRATION_0004_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../database/migrations/0004_accounting_documents_audit.sql"
));

#[derive(Clone, Copy, Debug)]
pub struct Migration {
    pub id: i64,
    pub version: &'static str,
    pub name: &'static str,
    pub sql: &'static str,
}

impl Migration {
    pub fn checksum_sha256(&self) -> String {
        format!("{:x}", Sha256::digest(self.sql.as_bytes()))
    }
}

pub const MIGRATIONS: [Migration; 4] = [
    Migration {
        id: 1,
        version: "0001",
        name: "system_company_security",
        sql: MIGRATION_0001_SQL,
    },
    Migration {
        id: 2,
        version: "0002",
        name: "reference_catalog_partners",
        sql: MIGRATION_0002_SQL,
    },
    Migration {
        id: 3,
        version: "0003",
        name: "commerce_inventory",
        sql: MIGRATION_0003_SQL,
    },
    Migration {
        id: 4,
        version: "0004",
        name: "accounting_documents_audit",
        sql: MIGRATION_0004_SQL,
    },
];

#[derive(Debug)]
struct AppliedMigration {
    id: i64,
    version: String,
    name: String,
    checksum_sha256: String,
}

pub fn apply_migrations(
    connection: &mut Connection,
    catalog: &[Migration],
) -> Result<(), RuntimeError> {
    validate_catalog(catalog)?;
    let ledger = read_ledger(connection)?;
    validate_ledger(catalog, &ledger)?;

    for migration in catalog.iter().skip(ledger.len()) {
        apply_one(connection, migration)?;
    }

    Ok(())
}

fn validate_catalog(catalog: &[Migration]) -> Result<(), RuntimeError> {
    if catalog.is_empty() {
        return Err(RuntimeError::MigrationLedgerInvalid {
            detail: "the embedded migration catalog is empty".to_owned(),
        });
    }

    for (index, migration) in catalog.iter().enumerate() {
        let expected_id = (index + 1) as i64;
        let expected_version = format!("{expected_id:04}");
        if migration.id != expected_id || migration.version != expected_version {
            return Err(RuntimeError::MigrationLedgerInvalid {
                detail: format!(
                    "embedded catalog entry {} is not contiguous (id={}, version={})",
                    index + 1,
                    migration.id,
                    migration.version
                ),
            });
        }
    }

    Ok(())
}

fn read_ledger(connection: &Connection) -> Result<Vec<AppliedMigration>, RuntimeError> {
    let table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_migrations'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| RuntimeError::MigrationLedgerInvalid {
            detail: format!("failed to inspect sqlite_master: {error}"),
        })?;

    if table_count == 0 {
        return Ok(Vec::new());
    }

    let mut statement = connection
        .prepare(
            "SELECT id, version, name, checksum_sha256 FROM app_migrations ORDER BY id ASC",
        )
        .map_err(|error| RuntimeError::MigrationLedgerInvalid {
            detail: format!("failed to prepare the ledger query: {error}"),
        })?;
    let rows = statement
        .query_map([], |row| {
            Ok(AppliedMigration {
                id: row.get(0)?,
                version: row.get(1)?,
                name: row.get(2)?,
                checksum_sha256: row.get(3)?,
            })
        })
        .map_err(|error| RuntimeError::MigrationLedgerInvalid {
            detail: format!("failed to query the ledger: {error}"),
        })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| RuntimeError::MigrationLedgerInvalid {
            detail: format!("failed to decode a ledger row: {error}"),
        })
}

fn validate_ledger(
    catalog: &[Migration],
    ledger: &[AppliedMigration],
) -> Result<(), RuntimeError> {
    let supported = catalog
        .last()
        .expect("catalog validation guarantees a final migration");

    for (index, applied) in ledger.iter().enumerate() {
        if applied.id > supported.id || applied.version.as_str() > supported.version {
            return Err(RuntimeError::UnsupportedSchema {
                found_version: applied.version.clone(),
                supported_version: supported.version.to_owned(),
            });
        }

        let Some(expected) = catalog.get(index) else {
            return Err(RuntimeError::UnsupportedSchema {
                found_version: applied.version.clone(),
                supported_version: supported.version.to_owned(),
            });
        };

        if applied.id != expected.id
            || applied.version != expected.version
            || applied.name != expected.name
        {
            return Err(RuntimeError::MigrationLedgerInvalid {
                detail: format!(
                    "row {} expected id/version/name {}/{}/{}, found {}/{}/{}",
                    index + 1,
                    expected.id,
                    expected.version,
                    expected.name,
                    applied.id,
                    applied.version,
                    applied.name
                ),
            });
        }

        let expected_checksum = expected.checksum_sha256();
        if applied.checksum_sha256 != expected_checksum {
            return Err(RuntimeError::MigrationChecksumMismatch {
                version: expected.version.to_owned(),
                expected: expected_checksum,
                found: applied.checksum_sha256.clone(),
            });
        }
    }

    Ok(())
}

fn apply_one(connection: &mut Connection, migration: &Migration) -> Result<(), RuntimeError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| RuntimeError::MigrationExecution {
            version: migration.version.to_owned(),
            name: migration.name.to_owned(),
            source,
        })?;

    transaction
        .execute_batch(migration.sql)
        .map_err(|source| RuntimeError::MigrationExecution {
            version: migration.version.to_owned(),
            name: migration.name.to_owned(),
            source,
        })?;

    transaction
        .execute(
            "INSERT INTO app_migrations (id, version, name, checksum_sha256, applied_at)\n             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                migration.id,
                migration.version,
                migration.name,
                migration.checksum_sha256()
            ],
        )
        .map_err(|source| RuntimeError::MigrationExecution {
            version: migration.version.to_owned(),
            name: migration.name.to_owned(),
            source,
        })?;

    transaction
        .commit()
        .map_err(|source| RuntimeError::MigrationExecution {
            version: migration.version.to_owned(),
            name: migration.name.to_owned(),
            source,
        })
}

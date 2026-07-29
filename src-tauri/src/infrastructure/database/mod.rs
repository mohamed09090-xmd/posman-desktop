mod connection;
mod migrations;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, TransactionBehavior};

use crate::{application::RuntimeStatus, error::RuntimeError};

pub use connection::open_configured_connection;
use connection::ConnectionContract;
use migrations::{apply_migrations, MIGRATIONS};

const REFERENCE_SEED_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../database/seed/reference_data.sql"
));

const EXPECTED_TABLES: [&str; 49] = [
    "app_migrations",
    "companies",
    "company_settings",
    "fiscal_years",
    "fiscal_periods",
    "document_sequences",
    "users",
    "roles",
    "permissions",
    "user_roles",
    "role_permissions",
    "sessions",
    "units",
    "tax_rates",
    "payment_terms",
    "payment_methods",
    "warehouses",
    "warehouse_locations",
    "product_families",
    "products",
    "price_lists",
    "product_prices",
    "partners",
    "partner_addresses",
    "partner_contacts",
    "commercial_documents",
    "commercial_document_lines",
    "document_line_links",
    "document_status_history",
    "payments",
    "payment_allocations",
    "stock_movements",
    "stock_balances",
    "stock_reservations",
    "inventory_counts",
    "inventory_count_lines",
    "accounts",
    "accounting_journals",
    "posting_rules",
    "journal_entries",
    "journal_entry_lines",
    "posting_attempts",
    "document_templates",
    "document_template_versions",
    "rendered_documents",
    "attachments",
    "audit_logs",
    "idempotency_keys",
    "backup_history",
];

#[derive(Clone)]
pub struct RuntimeDatabase {
    _database_path: PathBuf,
    status: RuntimeStatus,
}

impl RuntimeDatabase {
    pub fn initialize(database_path: &Path) -> Result<Self, RuntimeError> {
        let (mut connection, contract) = open_configured_connection(database_path)?;
        apply_migrations(&mut connection, &MIGRATIONS)?;
        apply_seed(&mut connection, REFERENCE_SEED_SQL)?;
        let status = verify_ready(&connection, &contract)?;

        Ok(Self {
            _database_path: database_path.to_path_buf(),
            status,
        })
    }

    pub fn status(&self) -> RuntimeStatus {
        self.status.clone()
    }
}

fn apply_seed(connection: &mut Connection, seed_sql: &str) -> Result<(), RuntimeError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| RuntimeError::SeedExecution { source })?;
    transaction
        .execute_batch(seed_sql)
        .map_err(|source| RuntimeError::SeedExecution { source })?;
    transaction
        .commit()
        .map_err(|source| RuntimeError::SeedExecution { source })
}

fn verify_ready(
    connection: &Connection,
    contract: &ConnectionContract,
) -> Result<RuntimeStatus, RuntimeError> {
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| RuntimeError::IntegrityFailure {
            detail: format!("failed to verify PRAGMA foreign_keys: {error}"),
        })?;
    if foreign_keys != 1 {
        return Err(RuntimeError::IntegrityFailure {
            detail: format!("PRAGMA foreign_keys returned {foreign_keys}, expected 1"),
        });
    }

    let mut foreign_key_check =
        connection
            .prepare("PRAGMA foreign_key_check")
            .map_err(|error| RuntimeError::IntegrityFailure {
                detail: format!("failed to prepare PRAGMA foreign_key_check: {error}"),
            })?;
    let mut rows = foreign_key_check
        .query([])
        .map_err(|error| RuntimeError::IntegrityFailure {
            detail: format!("failed to run PRAGMA foreign_key_check: {error}"),
        })?;
    if rows
        .next()
        .map_err(|error| RuntimeError::IntegrityFailure {
            detail: format!("failed to read PRAGMA foreign_key_check: {error}"),
        })?
        .is_some()
    {
        return Err(RuntimeError::IntegrityFailure {
            detail: "PRAGMA foreign_key_check returned one or more violations".to_owned(),
        });
    }

    for table in EXPECTED_TABLES {
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .map_err(|error| RuntimeError::IntegrityFailure {
                detail: format!("failed to verify required table {table}: {error}"),
            })?;
        if count != 1 {
            return Err(RuntimeError::IntegrityFailure {
                detail: format!("required table {table} is missing"),
            });
        }
    }

    let migration_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM app_migrations", [], |row| row.get(0))
        .map_err(|error| RuntimeError::IntegrityFailure {
            detail: format!("failed to count applied migrations: {error}"),
        })?;
    if migration_count != MIGRATIONS.len() as i64 {
        return Err(RuntimeError::IntegrityFailure {
            detail: format!(
                "expected {} applied migrations, found {migration_count}",
                MIGRATIONS.len()
            ),
        });
    }

    let schema_version: String = connection
        .query_row(
            "SELECT version FROM app_migrations ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| RuntimeError::IntegrityFailure {
            detail: format!("failed to read the current schema version: {error}"),
        })?;
    let expected_version = MIGRATIONS
        .last()
        .expect("the production migration catalog is non-empty")
        .version;
    if schema_version != expected_version {
        return Err(RuntimeError::IntegrityFailure {
            detail: format!("expected schema version {expected_version}, found {schema_version}"),
        });
    }

    Ok(RuntimeStatus {
        database_ready: true,
        schema_version,
        migration_count: migration_count as usize,
        foreign_keys_enabled: contract.foreign_keys_enabled,
        journal_mode: contract.journal_mode.clone(),
    })
}

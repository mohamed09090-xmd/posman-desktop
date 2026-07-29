use std::{path::Path, time::Duration};

use rusqlite::Connection;

use crate::error::RuntimeError;

const BUSY_TIMEOUT_MILLISECONDS: i64 = 5_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionContract {
    pub foreign_keys_enabled: bool,
    pub journal_mode: String,
}

pub fn open_configured_connection(
    database_path: &Path,
) -> Result<(Connection, ConnectionContract), RuntimeError> {
    let connection = Connection::open(database_path).map_err(|source| RuntimeError::DatabaseOpen {
        path: database_path.to_path_buf(),
        source,
    })?;

    connection
        .execute_batch("PRAGMA foreign_keys = ON;\nPRAGMA busy_timeout = 5000;")
        .map_err(|error| RuntimeError::DatabaseConfiguration {
            detail: format!("failed to apply foreign_keys/busy_timeout PRAGMAs: {error}"),
        })?;
    connection
        .busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MILLISECONDS as u64))
        .map_err(|error| RuntimeError::DatabaseConfiguration {
            detail: format!("failed to install the bounded busy handler: {error}"),
        })?;

    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| RuntimeError::DatabaseConfiguration {
            detail: format!("failed to read PRAGMA foreign_keys: {error}"),
        })?;
    if foreign_keys != 1 {
        return Err(RuntimeError::DatabaseConfiguration {
            detail: format!("PRAGMA foreign_keys returned {foreign_keys}, expected 1"),
        });
    }

    let busy_timeout: i64 = connection
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .map_err(|error| RuntimeError::DatabaseConfiguration {
            detail: format!("failed to read PRAGMA busy_timeout: {error}"),
        })?;
    if busy_timeout != BUSY_TIMEOUT_MILLISECONDS {
        return Err(RuntimeError::DatabaseConfiguration {
            detail: format!(
                "PRAGMA busy_timeout returned {busy_timeout}, expected {BUSY_TIMEOUT_MILLISECONDS}"
            ),
        });
    }

    let journal_mode: String = connection
        .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
        .map_err(|error| RuntimeError::DatabaseConfiguration {
            detail: format!("failed to request WAL journal mode: {error}"),
        })?;

    Ok((
        connection,
        ConnectionContract {
            foreign_keys_enabled: true,
            journal_mode,
        },
    ))
}

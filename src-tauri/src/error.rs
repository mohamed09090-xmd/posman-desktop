use std::{error::Error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum RuntimeError {
    PathResolution {
        detail: String,
    },
    PathCreation {
        path: PathBuf,
        source: io::Error,
    },
    DatabaseOpen {
        path: PathBuf,
        source: rusqlite::Error,
    },
    DatabaseConfiguration {
        detail: String,
    },
    UnsupportedSchema {
        found_version: String,
        supported_version: String,
    },
    MigrationChecksumMismatch {
        version: String,
        expected: String,
        found: String,
    },
    MigrationLedgerInvalid {
        detail: String,
    },
    MigrationExecution {
        version: String,
        name: String,
        source: rusqlite::Error,
    },
    SeedExecution {
        source: rusqlite::Error,
    },
    IntegrityFailure {
        detail: String,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathResolution { detail } => {
                write!(formatter, "failed to resolve the POSMAN local data root: {detail}")
            }
            Self::PathCreation { path, source } => write!(
                formatter,
                "failed to create POSMAN runtime directory {}: {source}",
                path.display()
            ),
            Self::DatabaseOpen { path, source } => write!(
                formatter,
                "failed to open POSMAN SQLite database {}: {source}",
                path.display()
            ),
            Self::DatabaseConfiguration { detail } => {
                write!(formatter, "failed to configure the SQLite connection: {detail}")
            }
            Self::UnsupportedSchema {
                found_version,
                supported_version,
            } => write!(
                formatter,
                "database schema version {found_version} is newer than supported version {supported_version}"
            ),
            Self::MigrationChecksumMismatch {
                version,
                expected,
                found,
            } => write!(
                formatter,
                "migration {version} checksum mismatch: expected {expected}, found {found}"
            ),
            Self::MigrationLedgerInvalid { detail } => {
                write!(formatter, "migration ledger is incomplete or inconsistent: {detail}")
            }
            Self::MigrationExecution {
                version,
                name,
                source,
            } => write!(
                formatter,
                "migration {version}_{name}.sql failed and was rolled back: {source}"
            ),
            Self::SeedExecution { source } => {
                write!(formatter, "reference seed failed and was rolled back: {source}")
            }
            Self::IntegrityFailure { detail } => {
                write!(formatter, "database integrity verification failed: {detail}")
            }
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PathCreation { source, .. } => Some(source),
            Self::DatabaseOpen { source, .. }
            | Self::MigrationExecution { source, .. }
            | Self::SeedExecution { source } => Some(source),
            Self::PathResolution { .. }
            | Self::DatabaseConfiguration { .. }
            | Self::UnsupportedSchema { .. }
            | Self::MigrationChecksumMismatch { .. }
            | Self::MigrationLedgerInvalid { .. }
            | Self::IntegrityFailure { .. } => None,
        }
    }
}

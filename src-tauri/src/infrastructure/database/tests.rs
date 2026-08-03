use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;

use super::{
    apply_seed,
    connection::open_configured_connection,
    migrations::{apply_migrations, Migration, MIGRATIONS},
    RuntimeDatabase, REFERENCE_SEED_SQL,
};
use crate::{application::RuntimeStatus, error::RuntimeError, infrastructure::paths::RuntimePaths};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "posman-runtime-test-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("failed to create runtime test directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn initialize_fixture() -> (TestDirectory, RuntimePaths, RuntimeDatabase) {
    let directory = TestDirectory::new();
    let paths = RuntimePaths::create_all(directory.path().join("POSMAN"))
        .expect("runtime paths should be created");
    let runtime = RuntimeDatabase::initialize(&paths.database)
        .expect("fresh runtime database should initialize");
    (directory, paths, runtime)
}

fn scalar_i64(connection: &Connection, sql: &str) -> i64 {
    connection
        .query_row(sql, [], |row| row.get(0))
        .unwrap_or_else(|error| panic!("failed scalar query {sql:?}: {error}"))
}

#[test]
fn fresh_database_creates_directories_schema_seed_and_connection_contract() {
    let (_directory, paths, runtime) = initialize_fixture();

    for path in [
        &paths.root,
        &paths.data,
        &paths.backups,
        &paths.documents,
        &paths.templates,
        &paths.logs,
    ] {
        assert!(
            path.is_dir(),
            "missing runtime directory {}",
            path.display()
        );
    }
    assert!(paths.database.is_file());

    let (connection, contract) = open_configured_connection(&paths.database)
        .expect("runtime connection should satisfy the PRAGMA contract");
    assert_eq!(
        scalar_i64(
            &connection,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
        ),
        52
    );
    assert_eq!(
        scalar_i64(
            &connection,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger'"
        ),
        25
    );
    assert_eq!(
        scalar_i64(
            &connection,
            "SELECT COUNT(*) FROM roles WHERE company_id IS NULL AND is_system = 1"
        ),
        6
    );
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM permissions"),
        23
    );
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM app_migrations"),
        5
    );
    assert_eq!(scalar_i64(&connection, "PRAGMA foreign_keys"), 1);
    assert_eq!(scalar_i64(&connection, "PRAGMA busy_timeout"), 5_000);

    let mut foreign_key_check = connection
        .prepare("PRAGMA foreign_key_check")
        .expect("failed to prepare foreign_key_check");
    let foreign_key_violations = foreign_key_check
        .query([])
        .expect("failed to run foreign_key_check")
        .next()
        .expect("failed to inspect foreign_key_check result")
        .is_some();
    assert!(!foreign_key_violations);

    let status = runtime.status();
    assert_eq!(status.schema_version, "0005");
    assert_eq!(status.migration_count, 5);
    assert!(status.database_ready);
    assert!(status.foreign_keys_enabled);
    assert_eq!(status.journal_mode, contract.journal_mode);
    assert!(!status.journal_mode.is_empty());
}

#[test]
fn restart_is_idempotent_for_migrations_and_reference_seed() {
    let (_directory, paths, first_runtime) = initialize_fixture();
    let (first_connection, _) =
        open_configured_connection(&paths.database).expect("first runtime connection should open");
    let first_counts = (
        scalar_i64(&first_connection, "SELECT COUNT(*) FROM app_migrations"),
        scalar_i64(&first_connection, "SELECT COUNT(*) FROM roles"),
        scalar_i64(&first_connection, "SELECT COUNT(*) FROM permissions"),
        scalar_i64(&first_connection, "SELECT COUNT(*) FROM role_permissions"),
    );
    drop(first_connection);

    let second_runtime = RuntimeDatabase::initialize(&paths.database)
        .expect("second initialization should be idempotent");
    let (second_connection, _) =
        open_configured_connection(&paths.database).expect("second runtime connection should open");
    let second_counts = (
        scalar_i64(&second_connection, "SELECT COUNT(*) FROM app_migrations"),
        scalar_i64(&second_connection, "SELECT COUNT(*) FROM roles"),
        scalar_i64(&second_connection, "SELECT COUNT(*) FROM permissions"),
        scalar_i64(&second_connection, "SELECT COUNT(*) FROM role_permissions"),
    );

    assert_eq!(first_counts, second_counts);
    assert_eq!(second_runtime.status(), first_runtime.status());
}

#[test]
fn checksum_mismatch_is_fatal_and_names_the_version() {
    let (_directory, paths, _runtime) = initialize_fixture();
    let (connection, _) = open_configured_connection(&paths.database)
        .expect("runtime connection should open for fixture mutation");
    let roles_before = scalar_i64(&connection, "SELECT COUNT(*) FROM roles");
    connection
        .execute(
            "UPDATE app_migrations SET checksum_sha256 = ?1 WHERE version = '0003'",
            ["0".repeat(64)],
        )
        .expect("failed to corrupt the test checksum");
    drop(connection);

    let error = match RuntimeDatabase::initialize(&paths.database) {
        Ok(_) => panic!("checksum mismatch should reject startup"),
        Err(error) => error,
    };
    assert!(matches!(
        &error,
        RuntimeError::MigrationChecksumMismatch { version, .. } if version == "0003"
    ));
    assert!(error.to_string().contains("0003"));

    let (connection, _) = open_configured_connection(&paths.database)
        .expect("database should remain inspectable after rejected startup");
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM roles"),
        roles_before
    );
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM app_migrations"),
        5
    );
}

#[test]
fn unknown_newer_schema_is_rejected_without_reset_or_downgrade() {
    let (_directory, paths, _runtime) = initialize_fixture();
    let (connection, _) = open_configured_connection(&paths.database)
        .expect("runtime connection should open for fixture mutation");
    connection
        .execute(
            "INSERT INTO app_migrations (id, version, name, checksum_sha256, applied_at)\n             VALUES (6, '0006', 'future_schema', ?1, '2026-07-29T00:00:00Z')",
            ["f".repeat(64)],
        )
        .expect("failed to add the future ledger row");
    drop(connection);

    let error = match RuntimeDatabase::initialize(&paths.database) {
        Ok(_) => panic!("newer schema should reject startup"),
        Err(error) => error,
    };
    assert!(matches!(
        &error,
        RuntimeError::UnsupportedSchema {
            found_version,
            supported_version
        } if found_version == "0006" && supported_version == "0005"
    ));

    let (connection, _) = open_configured_connection(&paths.database)
        .expect("future-schema fixture should not be reset");
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM app_migrations"),
        6
    );
}

#[test]
fn ledger_gap_is_rejected_before_seed_or_migration_work() {
    let (_directory, paths, _runtime) = initialize_fixture();
    let (connection, _) = open_configured_connection(&paths.database)
        .expect("runtime connection should open for fixture mutation");
    connection
        .execute("DELETE FROM app_migrations WHERE id = 2", [])
        .expect("failed to create a ledger gap");
    let role_permissions_before = scalar_i64(&connection, "SELECT COUNT(*) FROM role_permissions");
    drop(connection);

    let error = match RuntimeDatabase::initialize(&paths.database) {
        Ok(_) => panic!("ledger gap should reject startup"),
        Err(error) => error,
    };
    assert!(matches!(
        &error,
        RuntimeError::MigrationLedgerInvalid { .. }
    ));

    let (connection, _) =
        open_configured_connection(&paths.database).expect("gap fixture should remain inspectable");
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM app_migrations"),
        4
    );
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM role_permissions"),
        role_permissions_before
    );
}

#[test]
fn ledger_metadata_mismatch_is_rejected() {
    let (_directory, paths, _runtime) = initialize_fixture();
    let (connection, _) = open_configured_connection(&paths.database)
        .expect("runtime connection should open for fixture mutation");
    connection
        .execute(
            "UPDATE app_migrations SET name = 'renamed_migration' WHERE id = 2",
            [],
        )
        .expect("failed to create a metadata mismatch");
    drop(connection);

    let error = match RuntimeDatabase::initialize(&paths.database) {
        Ok(_) => panic!("metadata mismatch should reject startup"),
        Err(error) => error,
    };
    assert!(matches!(
        &error,
        RuntimeError::MigrationLedgerInvalid { .. }
    ));
    assert!(error.to_string().contains("0002"));
}

#[test]
fn migration_failure_rolls_back_partial_writes_and_stops_catalog() {
    const CATALOG: [Migration; 3] = [
        Migration {
            id: 1,
            version: "0001",
            name: "ledger_and_base",
            sql: "CREATE TABLE app_migrations (\n                     id INTEGER PRIMARY KEY,\n                     version TEXT NOT NULL UNIQUE,\n                     name TEXT NOT NULL,\n                     checksum_sha256 TEXT NOT NULL,\n                     applied_at TEXT NOT NULL\n                  );\n                  CREATE TABLE stable_table (id INTEGER PRIMARY KEY);",
        },
        Migration {
            id: 2,
            version: "0002",
            name: "failing_atomic_write",
            sql: "CREATE TABLE partial_table (id INTEGER PRIMARY KEY);\n                  INSERT INTO partial_table (id) VALUES (1);\n                  INSERT INTO missing_table (id) VALUES (1);",
        },
        Migration {
            id: 3,
            version: "0003",
            name: "must_not_run",
            sql: "CREATE TABLE later_table (id INTEGER PRIMARY KEY);",
        },
    ];

    let directory = TestDirectory::new();
    let database_path = directory.path().join("atomic.sqlite3");
    let (mut connection, _) = open_configured_connection(&database_path)
        .expect("atomicity fixture connection should open");
    let error =
        apply_migrations(&mut connection, &CATALOG).expect_err("the injected migration must fail");
    assert!(matches!(
        &error,
        RuntimeError::MigrationExecution { version, .. } if version == "0002"
    ));

    assert_eq!(
        scalar_i64(
            &connection,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'stable_table'"
        ),
        1
    );
    assert_eq!(
        scalar_i64(
            &connection,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'partial_table'"
        ),
        0
    );
    assert_eq!(
        scalar_i64(
            &connection,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'later_table'"
        ),
        0
    );
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM app_migrations"),
        1
    );
}

#[test]
fn seed_is_idempotent_and_failure_rolls_back_all_seed_writes() {
    let directory = TestDirectory::new();
    let database_path = directory.path().join("seed.sqlite3");
    let (mut connection, _) =
        open_configured_connection(&database_path).expect("seed fixture connection should open");
    apply_migrations(&mut connection, &MIGRATIONS).expect("production migrations should apply");

    let failing_seed =
        format!("{REFERENCE_SEED_SQL}\nINSERT INTO missing_seed_target (id) VALUES (1);");
    let error = apply_seed(&mut connection, &failing_seed)
        .expect_err("the injected seed must fail atomically");
    assert!(matches!(&error, RuntimeError::SeedExecution { .. }));
    assert_eq!(scalar_i64(&connection, "SELECT COUNT(*) FROM roles"), 0);
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM permissions"),
        0
    );
    assert_eq!(
        scalar_i64(&connection, "SELECT COUNT(*) FROM role_permissions"),
        0
    );

    apply_seed(&mut connection, REFERENCE_SEED_SQL).expect("first seed application should pass");
    let first_counts = (
        scalar_i64(&connection, "SELECT COUNT(*) FROM roles"),
        scalar_i64(&connection, "SELECT COUNT(*) FROM permissions"),
        scalar_i64(&connection, "SELECT COUNT(*) FROM role_permissions"),
    );
    apply_seed(&mut connection, REFERENCE_SEED_SQL).expect("second seed application should pass");
    let second_counts = (
        scalar_i64(&connection, "SELECT COUNT(*) FROM roles"),
        scalar_i64(&connection, "SELECT COUNT(*) FROM permissions"),
        scalar_i64(&connection, "SELECT COUNT(*) FROM role_permissions"),
    );
    assert_eq!(first_counts, second_counts);
    assert_eq!(first_counts.0, 6);
    assert_eq!(first_counts.1, 23);
}

#[test]
fn runtime_status_serializes_camel_case_without_database_path() {
    let status = RuntimeStatus {
        database_ready: true,
        schema_version: "0005".to_owned(),
        migration_count: 5,
        foreign_keys_enabled: true,
        journal_mode: "wal".to_owned(),
    };
    let value = serde_json::to_value(&status).expect("runtime status should serialize");
    let object = value
        .as_object()
        .expect("runtime status should serialize as an object");

    assert_eq!(object.len(), 5);
    for key in [
        "databaseReady",
        "schemaVersion",
        "migrationCount",
        "foreignKeysEnabled",
        "journalMode",
    ] {
        assert!(object.contains_key(key), "missing serialized field {key}");
    }
    let serialized = serde_json::to_string(&status).expect("runtime status should serialize");
    assert!(!serialized.contains("posman.sqlite3"));
    assert!(!serialized.contains("databasePath"));
    assert!(!serialized.contains("sql"));
}

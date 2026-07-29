mod application;
mod commands;
mod error;
mod infrastructure;

use std::{error::Error, path::PathBuf};

pub use application::RuntimeStatus;
use error::RuntimeError;
use infrastructure::{database::RuntimeDatabase, paths::RuntimePaths};
use tauri::{Manager, Runtime};

#[derive(Clone)]
pub struct RuntimeService {
    database: RuntimeDatabase,
}

impl RuntimeService {
    fn initialize(root: PathBuf) -> Result<Self, RuntimeError> {
        let paths = RuntimePaths::create_all(root)?;
        let database = RuntimeDatabase::initialize(&paths.database)?;
        Ok(Self { database })
    }

    pub fn status(&self) -> RuntimeStatus {
        self.database.status()
    }
}

#[derive(Clone)]
enum RuntimeRoot {
    SystemLocalData,
    #[cfg(test)]
    Explicit(PathBuf),
}

impl RuntimeRoot {
    fn resolve<R: Runtime>(&self, app: &tauri::AppHandle<R>) -> Result<PathBuf, RuntimeError> {
        match self {
            Self::SystemLocalData => app
                .path()
                .local_data_dir()
                .map(|root| root.join("POSMAN"))
                .map_err(|error| RuntimeError::PathResolution {
                    detail: error.to_string(),
                }),
            #[cfg(test)]
            Self::Explicit(root) => Ok(root.clone()),
        }
    }
}

fn configure_application<R: Runtime>(
    builder: tauri::Builder<R>,
    runtime_root: RuntimeRoot,
) -> tauri::Builder<R> {
    builder
        .setup(move |app| {
            let root = runtime_root
                .resolve(app.handle())
                .map_err(boxed_runtime_error)?;
            let runtime = RuntimeService::initialize(root).map_err(boxed_runtime_error)?;
            if !app.manage(runtime) {
                return Err("POSMAN runtime state was already managed".into());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::runtime::get_runtime_status])
}

fn boxed_runtime_error(error: RuntimeError) -> Box<dyn Error> {
    Box::new(error)
}

fn application_builder() -> tauri::Builder<tauri::Wry> {
    configure_application(tauri::Builder::default(), RuntimeRoot::SystemLocalData)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    application_builder()
        .run(tauri::generate_context!())
        .expect("failed to initialize or run the POSMAN local desktop runtime");
}

#[cfg(test)]
mod tests {
    use super::{configure_application, RuntimeRoot};
    use crate::infrastructure::database::open_configured_connection;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

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
                "posman-tauri-test-{}-{nonce}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("failed to create Tauri test directory");
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

    #[test]
    fn application_setup_builds_with_mock_runtime() {
        let directory = TestDirectory::new();
        let _application = configure_application(
            tauri::test::mock_builder(),
            RuntimeRoot::Explicit(directory.path().to_path_buf()),
        )
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("failed to build the POSMAN runtime with Tauri's mock runtime");

        let database_path = directory.path().join("data").join("posman.sqlite3");
        assert!(database_path.is_file());
        let (connection, contract) = open_configured_connection(&database_path)
            .expect("mock runtime database should open with the connection contract");
        let migration_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM app_migrations", [], |row| row.get(0))
            .expect("failed to count mock runtime migrations");
        assert_eq!(migration_count, 4);
        assert!(contract.foreign_keys_enabled);
    }
}

mod application;
mod commands;
mod error;
mod infrastructure;
mod phase05;
mod phase06;

use std::{error::Error, path::PathBuf};

pub use application::RuntimeStatus;
use error::RuntimeError;
use infrastructure::{database::RuntimeDatabase, paths::RuntimePaths};
use phase05::Phase05Service;
use phase06::Phase06Service;
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
            let phase05 = Phase05Service::new(runtime.database.path())
                .map_err(|_| "POSMAN PHASE 05 service could not initialize")?;
            let phase06 = Phase06Service::new(phase05.clone())
                .map_err(|_| "POSMAN PHASE 06 service could not initialize")?;
            if !app.manage(runtime) {
                return Err("POSMAN runtime state was already managed".into());
            }
            if !app.manage(phase05) {
                return Err("POSMAN PHASE 05 state was already managed".into());
            }
            if !app.manage(phase06) {
                return Err("POSMAN PHASE 06 state was already managed".into());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::runtime::get_runtime_status,
            commands::phase05::get_setup_status,
            commands::phase05::load_setup_draft,
            commands::phase05::discard_setup_draft,
            commands::phase05::get_current_session,
            commands::phase05::logout,
            commands::phase05::lock_session,
            commands::phase05::rotate_recovery_code,
            commands::phase05::get_company_profile,
            commands::phase05::get_fiscal_setup,
            commands::phase05::list_document_sequences,
            commands::phase05::list_roles,
            commands::phase05::save_setup_draft,
            commands::phase05::complete_initial_setup,
            commands::phase05::login,
            commands::phase05::recover_admin_password,
            commands::phase05::unlock_session,
            commands::phase05::change_own_password,
            commands::phase05::update_company_profile,
            commands::phase05::update_fiscal_setup,
            commands::phase05::update_document_sequence,
            commands::phase05::list_users,
            commands::phase05::create_user,
            commands::phase05::update_user,
            commands::phase05::set_user_roles,
            commands::phase05::reset_user_password,
            commands::phase05::create_role,
            commands::phase05::update_role,
            commands::phase05::set_role_permissions,
            commands::phase05::list_products,
            commands::phase05::create_product,
            commands::phase05::update_product,
            commands::phase05::set_product_active,
            commands::phase05::set_product_price,
            commands::phase05::list_partners,
            commands::phase05::create_partner,
            commands::phase05::update_partner,
            commands::phase05::set_partner_active,
            commands::phase05::create_partner_address,
            commands::phase05::create_partner_contact,
            commands::phase05::list_units,
            commands::phase05::create_unit,
            commands::phase05::update_unit,
            commands::phase05::set_unit_active,
            commands::phase05::list_tax_rates,
            commands::phase05::create_tax_rate,
            commands::phase05::update_tax_rate,
            commands::phase05::set_tax_rate_active,
            commands::phase05::list_payment_terms,
            commands::phase05::create_payment_term,
            commands::phase05::update_payment_term,
            commands::phase05::set_payment_term_active,
            commands::phase05::list_payment_methods,
            commands::phase05::create_payment_method,
            commands::phase05::update_payment_method,
            commands::phase05::set_payment_method_active,
            commands::phase05::list_warehouses,
            commands::phase05::create_warehouse,
            commands::phase05::update_warehouse,
            commands::phase05::set_warehouse_active,
            commands::phase05::list_warehouse_locations,
            commands::phase05::create_warehouse_location,
            commands::phase05::update_warehouse_location,
            commands::phase05::set_warehouse_location_active,
            commands::phase05::list_product_families,
            commands::phase05::create_product_family,
            commands::phase05::update_product_family,
            commands::phase05::set_product_family_active,
            commands::phase05::list_partner_addresses,
            commands::phase05::list_partner_contacts,
            commands::phase06::list_stock_balances,
            commands::phase06::list_stock_movements,
            commands::phase06::create_opening_stock,
            commands::phase06::review_opening_stock,
            commands::phase06::post_opening_stock,
            commands::phase06::post_stock_adjustment,
            commands::phase06::post_stock_transfer,
            commands::phase06::create_inventory_count,
            commands::phase06::update_inventory_count,
            commands::phase06::review_inventory_count,
            commands::phase06::post_inventory_count,
            commands::phase06::get_inventory_count,
            commands::phase06::create_stock_reservation,
            commands::phase06::release_stock_reservation,
            commands::phase06::consume_stock_reservation,
            commands::phase06::cancel_stock_reservation,
            commands::phase06::list_active_stock_reservations,
            commands::phase06::reconcile_stock_balances,
            commands::phase06::rebuild_stock_balances,
            commands::phase06::create_purchase_order,
            commands::phase06::update_purchase_order,
            commands::phase06::confirm_purchase_order,
            commands::phase06::cancel_purchase_order,
            commands::phase06::hold_purchase_order,
            commands::phase06::create_purchase_receipt,
            commands::phase06::post_purchase_receipt,
            commands::phase06::create_purchase_invoice,
            commands::phase06::post_purchase_invoice,
            commands::phase06::direct_receive_and_invoice,
            commands::phase06::post_purchase_return,
            commands::phase06::list_purchasing_documents,
            commands::phase06::get_purchasing_document
        ])
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
    use super::{configure_application, RuntimeRoot, RuntimeService};
    use crate::infrastructure::database::open_configured_connection;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    use tauri::Manager;

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

    #[allow(deprecated)]
    fn execute_setup_once(application: &mut tauri::App<tauri::test::MockRuntime>) {
        application.run_iteration(|_, _| {});
    }

    fn build_test_application(directory: &TestDirectory) -> tauri::App<tauri::test::MockRuntime> {
        configure_application(
            tauri::test::mock_builder(),
            RuntimeRoot::Explicit(directory.path().to_path_buf()),
        )
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("failed to build the POSMAN runtime with Tauri's mock runtime")
    }

    #[test]
    fn application_setup_builds_with_mock_runtime() {
        let directory = TestDirectory::new();
        let mut application = build_test_application(&directory);

        execute_setup_once(&mut application);

        let status = application.state::<RuntimeService>().status();
        assert!(status.database_ready);
        assert_eq!(status.schema_version, "0005");
        assert_eq!(status.migration_count, 5);

        let database_path = directory.path().join("data").join("posman.sqlite3");
        assert!(database_path.is_file());
        let (connection, contract) = open_configured_connection(&database_path)
            .expect("mock runtime database should open with the connection contract");
        let migration_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM app_migrations", [], |row| row.get(0))
            .expect("failed to count mock runtime migrations");
        assert_eq!(migration_count, 5);
        assert!(contract.foreign_keys_enabled);
    }

    #[test]
    fn phase06_command_executes_through_tauri_ipc_and_requires_session() {
        let directory = TestDirectory::new();
        let mut application = build_test_application(&directory);
        execute_setup_once(&mut application);

        let webview = tauri::WebviewWindowBuilder::new(&application, "phase06", Default::default())
            .build()
            .expect("failed to build mock webview for PHASE 06 IPC test");
        let ipc_url = if cfg!(any(windows, target_os = "android")) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        };
        let response = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "list_stock_balances".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: ipc_url.parse().expect("local Tauri IPC URL should parse"),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "request": {"limit": 10}
                })),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert!(response.is_err(), "PHASE 06 IPC must reject an unauthenticated caller");
    }

    #[test]
    fn get_runtime_status_executes_through_tauri_ipc() {
        let directory = TestDirectory::new();
        let mut application = build_test_application(&directory);
        execute_setup_once(&mut application);

        let webview = tauri::WebviewWindowBuilder::new(&application, "main", Default::default())
            .build()
            .expect("failed to build mock webview for IPC test");
        let ipc_url = if cfg!(any(windows, target_os = "android")) {
            let mut url = String::from("http");
            url.push_str("://tauri.localhost");
            url
        } else {
            String::from("tauri://localhost")
        };

        let response = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_runtime_status".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: ipc_url.parse().expect("local Tauri IPC URL should parse"),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .expect("get_runtime_status should resolve through Tauri IPC");

        let payload = response
            .deserialize::<serde_json::Value>()
            .expect("runtime status IPC response should be valid JSON");

        assert_eq!(payload["databaseReady"], true);
        assert_eq!(payload["schemaVersion"], "0005");
        assert_eq!(payload["migrationCount"], 5);
        assert_eq!(payload["foreignKeysEnabled"], true);
        assert!(payload["journalMode"]
            .as_str()
            .is_some_and(|journal_mode| !journal_mode.trim().is_empty()));
    }
}

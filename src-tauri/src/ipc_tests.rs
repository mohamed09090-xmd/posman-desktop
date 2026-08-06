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
    assert_eq!(status.schema_version, "0006");
    assert_eq!(status.migration_count, 6);

    let database_path = directory.path().join("data").join("posman.sqlite3");
    assert!(database_path.is_file());
    let (connection, contract) = open_configured_connection(&database_path)
        .expect("mock runtime database should open with the connection contract");
    let migration_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM app_migrations", [], |row| row.get(0))
        .expect("failed to count mock runtime migrations");
    assert_eq!(migration_count, 6);
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
        let mut url = String::from("http");
        url.push_str("://tauri.localhost");
        url
    } else {
        String::from("tauri://localhost")
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
    assert!(
        response.is_err(),
        "PHASE 06 IPC must reject an unauthenticated caller"
    );
}

#[test]
fn phase07_command_executes_through_tauri_ipc_and_requires_session() {
    let directory = TestDirectory::new();
    let mut application = build_test_application(&directory);
    execute_setup_once(&mut application);

    let webview = tauri::WebviewWindowBuilder::new(&application, "phase07", Default::default())
        .build()
        .expect("failed to build mock webview for PHASE 07 IPC test");
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
            cmd: "get_sales_summary".into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: ipc_url.parse().expect("local Tauri IPC URL should parse"),
            body: tauri::ipc::InvokeBody::default(),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    assert!(
        response.is_err(),
        "PHASE 07 IPC must reject an unauthenticated caller"
    );
}

#[test]
fn phase08_command_executes_through_tauri_ipc_and_requires_session() {
    let directory = TestDirectory::new();
    let mut application = build_test_application(&directory);
    execute_setup_once(&mut application);

    let webview = tauri::WebviewWindowBuilder::new(&application, "phase08", Default::default())
        .build()
        .expect("failed to build mock webview for PHASE 08 IPC test");
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
            cmd: "list_accounts".into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: ipc_url.parse().expect("local Tauri IPC URL should parse"),
            body: tauri::ipc::InvokeBody::default(),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    assert!(
        response.is_err(),
        "PHASE 08 IPC must reject an unauthenticated caller"
    );
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
    assert_eq!(payload["schemaVersion"], "0006");
    assert_eq!(payload["migrationCount"], 6);
    assert_eq!(payload["foreignKeysEnabled"], true);
    assert!(payload["journalMode"]
        .as_str()
        .is_some_and(|journal_mode| !journal_mode.trim().is_empty()));
}

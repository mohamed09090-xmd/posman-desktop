use serde::Serialize;
use tauri::State;

use crate::{RuntimeService, RuntimeStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandError {
    code: &'static str,
    message: &'static str,
}

impl RuntimeCommandError {
    fn unavailable() -> Self {
        Self {
            code: "RUNTIME_STATUS_UNAVAILABLE",
            message: "The local runtime status is temporarily unavailable.",
        }
    }
}

#[tauri::command]
pub async fn get_runtime_status(
    state: State<'_, RuntimeService>,
) -> Result<RuntimeStatus, RuntimeCommandError> {
    let runtime = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.status())
        .await
        .map_err(|_| RuntimeCommandError::unavailable())
}

use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub database_ready: bool,
    pub schema_version: String,
    pub migration_count: usize,
    pub foreign_keys_enabled: bool,
    pub journal_mode: String,
}

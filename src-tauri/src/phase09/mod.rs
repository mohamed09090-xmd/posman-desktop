pub mod audit;
pub mod backup;
pub mod documents;
pub mod error;
pub mod models;
pub mod output;
pub mod permissions;
pub mod rendering;
pub mod reports;
pub mod restore;
pub mod service;
pub mod templates;

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use rusqlite::{params, Transaction};
use sha2::Digest;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::{
    infrastructure::paths::RuntimePaths,
    phase05::{Phase05Service, Phase06AuthContext},
};

use self::{
    error::{Phase09Error, Phase09Result},
    models::PreviewContent,
};

#[derive(Clone)]
pub struct Phase09Service {
    pub(crate) phase05: Phase05Service,
    pub(crate) paths: RuntimePaths,
    pub(crate) output_lock: Arc<Mutex<()>>,
    pub(crate) previews: Arc<Mutex<HashMap<String, PreviewContent>>>,
}

impl Phase09Service {
    pub fn new(phase05: Phase05Service, paths: RuntimePaths) -> Phase09Result<Self> {
        let service = Self {
            phase05,
            paths,
            output_lock: Arc::new(Mutex::new(())),
            previews: Arc::new(Mutex::new(HashMap::new())),
        };
        service.provision_permissions()?;
        Ok(service)
    }

    pub(crate) fn deliver_managed_export(
        &self,
        result: &models::ExportResult,
        destination: &Path,
    ) -> Phase09Result<models::ExportResult> {
        let relative = Path::new(&result.relative_path);
        if relative.is_absolute()
            || result.relative_path.contains("..")
            || result.relative_path.contains('\\')
        {
            return Err(Phase09Error::validation("Unsafe managed export path."));
        }
        let source = self.paths.exports.join(relative);
        let canonical_root = std::fs::canonicalize(&self.paths.exports)
            .unwrap_or_else(|_| self.paths.exports.clone());
        let canonical_source = std::fs::canonicalize(&source)?;
        if !canonical_source.starts_with(canonical_root) {
            return Err(Phase09Error::validation("Unsafe managed export path."));
        }
        let bytes = std::fs::read(&canonical_source)?;
        let sha256 = format!("{:x}", sha2::Sha256::digest(&bytes));
        let size_bytes = i64::try_from(bytes.len()).map_err(|_| Phase09Error::internal())?;
        if sha256 != result.sha256 || size_bytes != result.size_bytes {
            return Err(Phase09Error::integrity(
                "The managed export failed verification.",
            ));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| Phase09Error::validation("Invalid export destination."))?;
        std::fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(".posman-export-{}.tmp", new_id()));
        let mut input = std::fs::OpenOptions::new()
            .read(true)
            .open(&canonical_source)?;
        let mut output = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        std::io::copy(&mut input, &mut output)?;
        use std::io::Write as _;
        output.flush()?;
        output.sync_all()?;
        if destination.exists() {
            std::fs::remove_file(destination)?;
        }
        std::fs::rename(&temporary, destination)?;
        let _ = std::fs::remove_file(canonical_source);
        Ok(models::ExportResult {
            relative_path: destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("export")
                .to_owned(),
            sha256,
            size_bytes,
        })
    }

    pub(crate) fn authorize(&self, permission: &str) -> Phase09Result<Phase06AuthContext> {
        self.phase05
            .phase06_authorize(Some(permission))
            .map_err(Phase09Error::from)
    }

    pub(crate) fn database_path(&self) -> &Path {
        self.phase05.database_path()
    }

    pub(crate) fn audit_success(
        transaction: &Transaction<'_>,
        context: &Phase06AuthContext,
        action_code: &str,
        entity_type: &str,
        entity_id: &str,
        details: Option<&serde_json::Value>,
    ) -> Phase09Result<()> {
        let details_json = details
            .map(serde_json::to_string)
            .transpose()
            .map_err(|_| Phase09Error::internal())?;
        transaction.execute(
            r#"INSERT INTO audit_logs(
                   id, company_id, actor_user_id, action_code, entity_type, entity_id,
                   occurred_at, outcome, correlation_id, details_json
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,'SUCCESS',?8,?9)"#,
            params![
                new_id(),
                context.company_id,
                context.user_id,
                action_code,
                entity_type,
                entity_id,
                now_iso()?,
                context.session_id,
                details_json
            ],
        )?;
        Ok(())
    }

    pub(crate) fn audit_failure(
        &self,
        context: &Phase06AuthContext,
        action_code: &str,
        entity_type: &str,
        entity_id: &str,
        safe_code: &str,
    ) -> Phase09Result<()> {
        let connection = self.phase05.phase09_open_maintenance()?;
        connection.execute(
            r#"INSERT INTO audit_logs(
                   id, company_id, actor_user_id, action_code, entity_type, entity_id,
                   occurred_at, outcome, correlation_id, details_json
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,'FAILURE',?8,json_object('safeCode',?9))"#,
            params![
                new_id(),
                context.company_id,
                context.user_id,
                action_code,
                entity_type,
                entity_id,
                now_iso()?,
                context.session_id,
                safe_code
            ],
        )?;
        Ok(())
    }
}

pub(crate) fn new_id() -> String {
    Uuid::now_v7().to_string()
}

pub(crate) fn now_iso() -> Phase09Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| Phase09Error::internal())
}

pub(crate) fn checked_page(page: i64, page_size: i64, maximum: i64) -> Phase09Result<(i64, i64)> {
    if page < 1 || !(1..=maximum).contains(&page_size) {
        return Err(Phase09Error::validation("Invalid page or page size."));
    }
    Ok((page, page_size))
}

pub(crate) fn normalize_locale(value: &str) -> Phase09Result<&'static str> {
    match value {
        "ar" | "ar-DZ" => Ok("ar-DZ"),
        "fr" | "fr-DZ" => Ok("fr-DZ"),
        _ => Err(Phase09Error::validation("Locale must be Arabic or French.")),
    }
}

pub(crate) fn safe_component(value: &str) -> Phase09Result<String> {
    let normalized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if normalized.is_empty() || normalized == "." || normalized == ".." {
        return Err(Phase09Error::validation("Unsafe local file component."));
    }
    Ok(normalized)
}

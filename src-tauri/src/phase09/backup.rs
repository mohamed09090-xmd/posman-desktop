use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::{
    backup::Backup, params, Connection, OpenFlags, OptionalExtension, Row, TransactionBehavior,
};
use sha2::{Digest, Sha256};

use crate::phase05::Phase06AuthContext;

use super::{
    checked_page,
    error::{Phase09Error, Phase09Result},
    models::{
        BackupKeyRequest, BackupListRequest, BackupSettingsView, BackupView, CreateBackupRequest,
        ExportResult, Paged, UpdateBackupSettingsRequest,
    },
    new_id, now_iso, safe_component, Phase09Service,
};

const SUPPORTED_SCHEMA_VERSION: &str = "0007";
const REQUIRED_TABLES: &[&str] = &[
    "app_migrations",
    "companies",
    "users",
    "sessions",
    "permissions",
    "commercial_documents",
    "commercial_document_lines",
    "stock_movements",
    "journal_entries",
    "audit_logs",
    "phase09_backups",
    "phase09_restore_attempts",
];

impl Phase09Service {
    pub fn get_backup_settings(&self, _: ()) -> Phase09Result<BackupSettingsView> {
        let context = self.authorize("backup.view")?;
        let mut connection = self.phase05.phase09_open_maintenance()?;
        ensure_settings(&mut connection, &context)?;
        load_settings(&connection, &context.company_id)
    }

    pub fn update_backup_settings(
        &self,
        request: UpdateBackupSettingsRequest,
    ) -> Phase09Result<BackupSettingsView> {
        if request.expected_row_version < 1 {
            return Err(Phase09Error::validation("Invalid backup settings version."));
        }
        let context = self.authorize("backup.manage")?;
        let mut connection = self.phase05.phase09_open_maintenance()?;
        ensure_settings(&mut connection, &context)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = transaction.execute(
            r#"UPDATE phase09_backup_settings
               SET automatic_enabled=?1,daily_enabled=?1,weekly_enabled=?2,
                   updated_at=?3,updated_by=?4,row_version=row_version+1
               WHERE company_id=?5 AND row_version=?6"#,
            params![
                bool_int(request.automatic_enabled),
                bool_int(request.weekly_enabled),
                now_iso()?,
                context.user_id,
                context.company_id,
                request.expected_row_version
            ],
        )?;
        if changed != 1 {
            return Err(Phase09Error::concurrency());
        }
        Self::audit_success(
            &transaction,
            &context,
            "BACKUP_SETTINGS_UPDATED",
            "PHASE09_BACKUP_SETTINGS",
            &context.company_id,
            None,
        )?;
        transaction.commit()?;
        load_settings(&connection, &context.company_id)
    }

    pub fn create_backup(&self, request: CreateBackupRequest) -> Phase09Result<BackupView> {
        validate_backup_kind(&request.backup_kind, false)?;
        let context = self.authorize("backup.create")?;
        self.create_verified_backup_for_context(&context, &request.backup_kind, false)
    }

    pub fn list_backups(&self, request: BackupListRequest) -> Phase09Result<Paged<BackupView>> {
        let context = self.authorize("backup.view")?;
        if let Some(kind) = request.backup_kind.as_deref() {
            validate_backup_kind(kind, true)?;
        }
        let (page, page_size) = checked_page(request.page, request.page_size, 200)?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let total = connection.query_row(
            "SELECT COUNT(*) FROM phase09_backups WHERE company_id=?1 AND (?2 IS NULL OR backup_kind=?2)",
            params![context.company_id, request.backup_kind],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            r#"SELECT id,backup_kind,created_at,created_by,application_version,schema_version,
                      migration_ledger_digest,database_size_bytes,sha256,relative_path,
                      integrity_status,foreign_key_status,verification_status,failure_reason,
                      protected_for_restore
               FROM phase09_backups
               WHERE company_id=?1 AND (?2 IS NULL OR backup_kind=?2)
               ORDER BY created_at DESC,id DESC LIMIT ?3 OFFSET ?4"#,
        )?;
        let items = statement
            .query_map(
                params![
                    context.company_id,
                    request.backup_kind,
                    page_size,
                    (page - 1) * page_size
                ],
                map_backup,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Paged {
            items,
            page,
            page_size,
            total,
        })
    }

    pub fn verify_backup(&self, request: BackupKeyRequest) -> Phase09Result<BackupView> {
        let context = self.authorize("backup.view")?;
        let record = self.load_backup_record(&context.company_id, &request.backup_id)?;
        let path = managed_backup_path(&self.paths.backups, &record.relative_path)?;
        let expected = ExpectedArtifact {
            sha256: Some(record.sha256.clone()),
            size_bytes: Some(record.database_size_bytes),
        };
        let verified = verify_database_file(&path, expected)?;
        let connection = self.phase05.phase09_open_maintenance()?;
        connection.execute(
            r#"UPDATE phase09_backups SET integrity_status='OK',foreign_key_status='OK',
                      verification_status='VERIFIED',failure_reason=NULL,verified_at=?1
               WHERE id=?2 AND company_id=?3"#,
            params![now_iso()?, request.backup_id, context.company_id],
        )?;
        let mut view = record.view();
        view.schema_version = verified.schema_version;
        view.migration_ledger_digest = verified.migration_ledger_digest;
        view.integrity_status = "OK".to_owned();
        view.foreign_key_status = "OK".to_owned();
        view.verification_status = "VERIFIED".to_owned();
        view.failure_reason = None;
        Ok(view)
    }

    pub fn export_backup_to(
        &self,
        request: BackupKeyRequest,
        destination: &Path,
    ) -> Phase09Result<ExportResult> {
        let context = self.authorize("backup.view")?;
        let record = self.load_backup_record(&context.company_id, &request.backup_id)?;
        if record.verification_status != "VERIFIED" {
            return Err(Phase09Error::new(
                "BACKUP_NOT_VERIFIED",
                "Only a verified backup can be exported.",
                false,
            ));
        }
        let source = managed_backup_path(&self.paths.backups, &record.relative_path)?;
        verify_database_file(
            &source,
            ExpectedArtifact {
                sha256: Some(record.sha256.clone()),
                size_bytes: Some(record.database_size_bytes),
            },
        )?;
        atomic_copy(&source, destination, true)?;
        Ok(ExportResult {
            relative_path: destination
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("posman-backup.sqlite3")
                .to_owned(),
            sha256: record.sha256,
            size_bytes: record.database_size_bytes,
        })
    }

    pub fn import_backup_from(&self, source: &Path) -> Phase09Result<BackupView> {
        let context = self.authorize("backup.create")?;
        if !source.is_file() {
            return Err(Phase09Error::validation(
                "The selected backup file is invalid.",
            ));
        }
        fs::create_dir_all(&self.paths.staging)?;
        let backup_id = new_id();
        let staged = self
            .paths
            .staging
            .join(format!("phase09-import-{backup_id}.sqlite3"));
        atomic_copy(source, &staged, false)?;
        let verified = match verify_database_file(&staged, ExpectedArtifact::default()) {
            Ok(verified) => verified,
            Err(error) => {
                let _ = fs::remove_file(&staged);
                let _ = self.audit_failure(
                    &context,
                    "BACKUP_IMPORT_REJECTED",
                    "PHASE09_BACKUP",
                    &backup_id,
                    &error.code,
                );
                return Err(error);
            }
        };
        let relative_path = backup_relative_path(
            &context.company_id,
            "MANUAL",
            &verified.verified_at,
            &backup_id,
        )?;
        let final_path = managed_backup_path(&self.paths.backups, &relative_path)?;
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&staged, &final_path)?;
        self.insert_verified_backup(
            &context,
            &backup_id,
            "MANUAL",
            &relative_path,
            &verified,
            true,
        )?;
        self.load_backup_record(&context.company_id, &backup_id)
            .map(BackupRecord::view)
    }

    pub fn delete_backup(&self, request: BackupKeyRequest) -> Phase09Result<()> {
        let context = self.authorize("backup.manage")?;
        let record = self.load_backup_record(&context.company_id, &request.backup_id)?;
        if record.protected_for_restore {
            return Err(Phase09Error::new(
                "BACKUP_PROTECTED",
                "The backup selected for restore cannot be deleted.",
                false,
            ));
        }
        let connection = self.phase05.phase09_open_maintenance()?;
        let verified_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM phase09_backups WHERE company_id=?1 AND verification_status='VERIFIED'",
            params![context.company_id],
            |row| row.get(0),
        )?;
        if record.verification_status == "VERIFIED" && verified_count <= 1 {
            return Err(Phase09Error::new(
                "LAST_VALID_BACKUP",
                "The last known valid backup cannot be deleted.",
                false,
            ));
        }
        let path = managed_backup_path(&self.paths.backups, &record.relative_path)?;
        if let Err(error) = fs::remove_file(&path) {
            connection.execute(
                "UPDATE phase09_backups SET deletion_failure=?1 WHERE id=?2 AND company_id=?3",
                params![error.to_string(), record.backup_id, context.company_id],
            )?;
            return Err(Phase09Error::new(
                "BACKUP_DELETE_FAILED",
                "The backup file could not be deleted; its history was preserved.",
                true,
            ));
        }
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "DELETE FROM phase09_backups WHERE id=?1 AND company_id=?2",
            params![record.backup_id, context.company_id],
        )?;
        Self::audit_success(
            &transaction,
            &context,
            "BACKUP_DELETED",
            "PHASE09_BACKUP",
            &request.backup_id,
            None,
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn create_verified_backup_for_context(
        &self,
        context: &Phase06AuthContext,
        kind: &str,
        protected_for_restore: bool,
    ) -> Phase09Result<BackupView> {
        validate_backup_kind(kind, true)?;
        let backup_id = new_id();
        let created_at = now_iso()?;
        let relative_path =
            backup_relative_path(&context.company_id, kind, &created_at, &backup_id)?;
        let final_path = managed_backup_path(&self.paths.backups, &relative_path)?;
        let parent = final_path
            .parent()
            .ok_or_else(|| Phase09Error::validation("Invalid backup path."))?;
        fs::create_dir_all(parent)?;
        fs::create_dir_all(&self.paths.staging)?;
        let temporary_path = self
            .paths
            .staging
            .join(format!("phase09-backup-{backup_id}.sqlite3.tmp"));
        remove_if_exists(&temporary_path);

        let source = self.phase05.phase09_open_maintenance()?;
        let mut destination = Connection::open(&temporary_path)?;
        {
            let backup = Backup::new(&source, &mut destination)?;
            backup.run_to_completion(128, Duration::from_millis(20), None)?;
        }
        destination.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        drop(destination);
        let verified = match verify_database_file(&temporary_path, ExpectedArtifact::default()) {
            Ok(verified) => verified,
            Err(error) => {
                remove_if_exists(&temporary_path);
                let _ = self.audit_failure(
                    context,
                    "BACKUP_CREATE_FAILED",
                    "PHASE09_BACKUP",
                    &backup_id,
                    &error.code,
                );
                return Err(error);
            }
        };
        if final_path.exists() {
            remove_if_exists(&temporary_path);
            return Err(Phase09Error::new(
                "BACKUP_ALREADY_EXISTS",
                "The backup path already exists and was not overwritten.",
                false,
            ));
        }
        fs::rename(&temporary_path, &final_path)?;
        if let Err(error) =
            self.insert_verified_backup(context, &backup_id, kind, &relative_path, &verified, false)
        {
            remove_if_exists(&final_path);
            return Err(error);
        }
        if protected_for_restore {
            let connection = self.phase05.phase09_open_maintenance()?;
            connection.execute(
                "UPDATE phase09_backups SET protected_for_restore=1 WHERE id=?1 AND company_id=?2",
                params![backup_id, context.company_id],
            )?;
        }
        self.apply_retention(&context.company_id, kind, Some(&backup_id))?;
        self.load_backup_record(&context.company_id, &backup_id)
            .map(BackupRecord::view)
    }

    pub(crate) fn load_backup_record(
        &self,
        company_id: &str,
        backup_id: &str,
    ) -> Phase09Result<BackupRecord> {
        let connection = self.phase05.phase09_open_maintenance()?;
        connection
            .query_row(
                r#"SELECT id,backup_kind,created_at,created_by,application_version,schema_version,
                          migration_ledger_digest,database_size_bytes,sha256,relative_path,
                          integrity_status,foreign_key_status,verification_status,failure_reason,
                          protected_for_restore
                   FROM phase09_backups WHERE id=?1 AND company_id=?2"#,
                params![backup_id, company_id],
                map_backup_record,
            )
            .optional()?
            .ok_or_else(|| Phase09Error::not_found("backup"))
    }

    fn insert_verified_backup(
        &self,
        context: &Phase06AuthContext,
        backup_id: &str,
        kind: &str,
        relative_path: &str,
        verified: &VerifiedDatabase,
        imported: bool,
    ) -> Phase09Result<()> {
        let mut connection = self.phase05.phase09_open_maintenance()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            r#"INSERT INTO phase09_backups(
                   id,company_id,backup_kind,created_at,created_by,application_version,
                   schema_version,migration_ledger_digest,database_size_bytes,sha256,relative_path,
                   integrity_status,foreign_key_status,verification_status,failure_reason,imported,
                   protected_for_restore,verified_at
               ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'OK','OK','VERIFIED',NULL,?12,0,?4)"#,
            params![
                backup_id,
                context.company_id,
                kind,
                verified.verified_at,
                context.user_id,
                env!("CARGO_PKG_VERSION"),
                verified.schema_version,
                verified.migration_ledger_digest,
                verified.size_bytes,
                verified.sha256,
                relative_path,
                bool_int(imported),
            ],
        )?;
        Self::audit_success(
            &transaction,
            context,
            if imported {
                "BACKUP_IMPORTED"
            } else {
                "BACKUP_CREATED"
            },
            "PHASE09_BACKUP",
            backup_id,
            Some(&serde_json::json!({
                "kind": kind,
                "sha256": verified.sha256,
                "sizeBytes": verified.size_bytes,
            })),
        )?;
        transaction.commit()?;
        Ok(())
    }

    fn apply_retention(
        &self,
        company_id: &str,
        kind: &str,
        protected_id: Option<&str>,
    ) -> Phase09Result<()> {
        let keep = match kind {
            "AUTOMATIC_DAILY" => 7,
            "AUTOMATIC_WEEKLY" => 4,
            "PRE_RESTORE" => 3,
            _ => return Ok(()),
        };
        let connection = self.phase05.phase09_open_maintenance()?;
        let mut statement = connection.prepare(
            r#"SELECT id,relative_path FROM phase09_backups
               WHERE company_id=?1 AND backup_kind=?2 AND verification_status='VERIFIED'
                 AND protected_for_restore=0 AND (?3 IS NULL OR id<>?3)
               ORDER BY created_at DESC,id DESC LIMIT -1 OFFSET ?4"#,
        )?;
        let candidates = statement
            .query_map(params![company_id, kind, protected_id, keep], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        drop(connection);
        for (id, relative_path) in candidates {
            let connection = self.phase05.phase09_open_maintenance()?;
            let total: i64 = connection.query_row(
                "SELECT COUNT(*) FROM phase09_backups WHERE company_id=?1 AND verification_status='VERIFIED'",
                params![company_id],
                |row| row.get(0),
            )?;
            if total <= 1 {
                break;
            }
            let path = managed_backup_path(&self.paths.backups, &relative_path)?;
            if fs::remove_file(&path).is_ok() {
                connection.execute(
                    "DELETE FROM phase09_backups WHERE id=?1 AND company_id=?2 AND protected_for_restore=0",
                    params![id, company_id],
                )?;
            } else {
                connection.execute(
                    "UPDATE phase09_backups SET deletion_failure='RETENTION_FILE_DELETE_FAILED' WHERE id=?1 AND company_id=?2",
                    params![id, company_id],
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BackupRecord {
    pub backup_id: String,
    pub backup_kind: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub application_version: String,
    pub schema_version: String,
    pub migration_ledger_digest: String,
    pub database_size_bytes: i64,
    pub sha256: String,
    pub relative_path: String,
    pub integrity_status: String,
    pub foreign_key_status: String,
    pub verification_status: String,
    pub failure_reason: Option<String>,
    pub protected_for_restore: bool,
}

impl BackupRecord {
    pub(crate) fn view(self) -> BackupView {
        BackupView {
            backup_id: self.backup_id,
            backup_kind: self.backup_kind,
            created_at: self.created_at,
            created_by: self.created_by,
            application_version: self.application_version,
            schema_version: self.schema_version,
            migration_ledger_digest: self.migration_ledger_digest,
            database_size_bytes: self.database_size_bytes,
            sha256: self.sha256,
            relative_path: self.relative_path,
            integrity_status: self.integrity_status,
            foreign_key_status: self.foreign_key_status,
            verification_status: self.verification_status,
            failure_reason: self.failure_reason,
            selected_for_restore: self.protected_for_restore,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VerifiedDatabase {
    pub sha256: String,
    pub size_bytes: i64,
    pub schema_version: String,
    pub migration_ledger_digest: String,
    pub verified_at: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ExpectedArtifact {
    pub sha256: Option<String>,
    pub size_bytes: Option<i64>,
}

pub(crate) fn verify_database_file(
    path: &Path,
    expected: ExpectedArtifact,
) -> Phase09Result<VerifiedDatabase> {
    if !path.is_file() {
        return Err(Phase09Error::new(
            "BACKUP_MISSING",
            "The selected backup file is missing.",
            false,
        ));
    }
    let bytes = fs::read(path)?;
    let size_bytes = i64::try_from(bytes.len()).map_err(|_| Phase09Error::internal())?;
    if size_bytes <= 0 {
        return Err(Phase09Error::new(
            "BACKUP_TRUNCATED",
            "The selected backup file is empty or truncated.",
            false,
        ));
    }
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    if expected
        .sha256
        .as_deref()
        .is_some_and(|value| value != sha256)
        || expected.size_bytes.is_some_and(|value| value != size_bytes)
    {
        return Err(Phase09Error::new(
            "BACKUP_INTEGRITY_FAILED",
            "The selected backup no longer matches its recorded size or SHA-256.",
            false,
        ));
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(Phase09Error::new(
            "BACKUP_CORRUPTED",
            "SQLite integrity_check rejected the selected backup.",
            false,
        ));
    }
    let foreign_key_failures: i64 =
        connection.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if foreign_key_failures != 0 {
        return Err(Phase09Error::new(
            "BACKUP_FOREIGN_KEY_FAILED",
            "SQLite foreign_key_check rejected the selected backup.",
            false,
        ));
    }
    for table in REQUIRED_TABLES {
        let exists: Option<i64> = connection
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(Phase09Error::new(
                "BACKUP_REQUIRED_TABLE_MISSING",
                "The selected backup is missing a required POSMAN table.",
                false,
            ));
        }
    }
    let mut statement = connection
        .prepare("SELECT version,checksum_sha256 FROM app_migrations ORDER BY version")?;
    let ledger = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if ledger.is_empty() {
        return Err(Phase09Error::new(
            "BACKUP_MIGRATION_MISSING",
            "The selected backup has no migration ledger.",
            false,
        ));
    }
    let schema_version = ledger
        .last()
        .map(|entry| entry.0.clone())
        .unwrap_or_default();
    if schema_version.as_str() > SUPPORTED_SCHEMA_VERSION {
        return Err(Phase09Error::new(
            "BACKUP_SCHEMA_NEWER",
            "The selected backup was created by a newer POSMAN schema.",
            false,
        ));
    }
    if schema_version != SUPPORTED_SCHEMA_VERSION || ledger.len() != 7 {
        return Err(Phase09Error::new(
            "BACKUP_MIGRATION_MISSING",
            "The selected backup does not contain the complete supported migration ledger.",
            false,
        ));
    }
    let mut digest = Sha256::new();
    for (version, checksum) in &ledger {
        if checksum.len() != 64
            || !checksum
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(Phase09Error::new(
                "BACKUP_MIGRATION_CHECKSUM_INVALID",
                "The selected backup contains an invalid migration checksum.",
                false,
            ));
        }
        digest.update(version.as_bytes());
        digest.update(b":");
        digest.update(checksum.as_bytes());
        digest.update(b"\n");
    }
    Ok(VerifiedDatabase {
        sha256,
        size_bytes,
        schema_version,
        migration_ledger_digest: format!("{:x}", digest.finalize()),
        verified_at: now_iso()?,
    })
}

pub(crate) fn managed_backup_path(root: &Path, relative: &str) -> Phase09Result<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative.contains("..")
        || relative.contains('\\')
        || relative_path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(Phase09Error::validation("Unsafe managed backup path."));
    }
    Ok(root.join(relative_path))
}

fn backup_relative_path(
    company_id: &str,
    kind: &str,
    created_at: &str,
    backup_id: &str,
) -> Phase09Result<String> {
    let date = created_at
        .get(0..10)
        .ok_or_else(|| Phase09Error::validation("Invalid backup timestamp."))?;
    Ok(format!(
        "{}/{}/{}/{}/{}.sqlite3",
        safe_component(company_id)?,
        safe_component(&kind.to_ascii_lowercase())?,
        safe_component(&date[0..4])?,
        safe_component(&date[5..7])?,
        safe_component(backup_id)?,
    ))
}

fn ensure_settings(connection: &mut Connection, context: &Phase06AuthContext) -> Phase09Result<()> {
    let now = now_iso()?;
    connection.execute(
        r#"INSERT OR IGNORE INTO phase09_backup_settings(
               id,company_id,automatic_enabled,daily_enabled,weekly_enabled,weekly_day,
               created_at,created_by,updated_at,updated_by,row_version
           ) VALUES(?1,?2,1,1,1,5,?3,?4,?3,?4,1)"#,
        params![new_id(), context.company_id, now, context.user_id],
    )?;
    Ok(())
}

fn load_settings(connection: &Connection, company_id: &str) -> Phase09Result<BackupSettingsView> {
    connection
        .query_row(
            r#"SELECT s.automatic_enabled,s.weekly_enabled,c.timezone_name,
                      substr(s.last_attempt_at,1,10),s.last_daily_local_date,
                      s.last_warning_code,s.row_version
               FROM phase09_backup_settings s JOIN companies c ON c.id=s.company_id
               WHERE s.company_id=?1"#,
            [company_id],
            |row| {
                Ok(BackupSettingsView {
                    automatic_enabled: row.get::<_, i64>(0)? == 1,
                    weekly_enabled: row.get::<_, i64>(1)? == 1,
                    timezone_name: row.get(2)?,
                    last_attempt_local_date: row.get(3)?,
                    last_success_local_date: row.get(4)?,
                    last_warning_code: row.get(5)?,
                    row_version: row.get(6)?,
                    encryption_status: "LOCAL_UNENCRYPTED".to_owned(),
                })
            },
        )
        .map_err(Phase09Error::from)
}

fn map_backup(row: &Row<'_>) -> rusqlite::Result<BackupView> {
    Ok(BackupView {
        backup_id: row.get(0)?,
        backup_kind: row.get(1)?,
        created_at: row.get(2)?,
        created_by: row.get(3)?,
        application_version: row.get(4)?,
        schema_version: row.get(5)?,
        migration_ledger_digest: row.get(6)?,
        database_size_bytes: row.get(7)?,
        sha256: row.get(8)?,
        relative_path: row.get(9)?,
        integrity_status: row.get(10)?,
        foreign_key_status: row.get(11)?,
        verification_status: row.get(12)?,
        failure_reason: row.get(13)?,
        selected_for_restore: row.get::<_, i64>(14)? == 1,
    })
}

fn map_backup_record(row: &Row<'_>) -> rusqlite::Result<BackupRecord> {
    Ok(BackupRecord {
        backup_id: row.get(0)?,
        backup_kind: row.get(1)?,
        created_at: row.get(2)?,
        created_by: row.get(3)?,
        application_version: row.get(4)?,
        schema_version: row.get(5)?,
        migration_ledger_digest: row.get(6)?,
        database_size_bytes: row.get(7)?,
        sha256: row.get(8)?,
        relative_path: row.get(9)?,
        integrity_status: row.get(10)?,
        foreign_key_status: row.get(11)?,
        verification_status: row.get(12)?,
        failure_reason: row.get(13)?,
        protected_for_restore: row.get::<_, i64>(14)? == 1,
    })
}

fn validate_backup_kind(kind: &str, allow_pre_restore: bool) -> Phase09Result<()> {
    let valid = matches!(kind, "MANUAL" | "AUTOMATIC_DAILY" | "AUTOMATIC_WEEKLY")
        || (allow_pre_restore && kind == "PRE_RESTORE");
    if valid {
        Ok(())
    } else {
        Err(Phase09Error::validation("Unsupported backup kind."))
    }
}

fn bool_int(value: bool) -> i64 {
    i64::from(value)
}

fn atomic_copy(source: &Path, destination: &Path, replace: bool) -> Phase09Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| Phase09Error::validation("Invalid file destination."))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".posman-copy-{}.tmp", new_id()));
    let mut input = OpenOptions::new().read(true).open(source)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    std::io::copy(&mut input, &mut output)?;
    output.flush()?;
    output.sync_all()?;
    if destination.exists() {
        if replace {
            fs::remove_file(destination)?;
        } else {
            remove_if_exists(&temporary);
            return Err(Phase09Error::new(
                "FILE_ALREADY_EXISTS",
                "The managed file already exists and was not overwritten.",
                false,
            ));
        }
    }
    fs::rename(&temporary, destination)?;
    Ok(())
}

fn remove_if_exists(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        infrastructure::{database::RuntimeDatabase, paths::RuntimePaths},
        phase05::{
            dto::{InitialSetupRequest, LoginRequest, TaxSetup},
            Phase05Service,
        },
        phase09::models::{BackupKeyRequest, BackupListRequest, RestoreBackupRequest},
    };

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("posman-phase09-backup-{}", new_id()));
            fs::create_dir_all(&path).expect("failed to create PHASE 09 test directory");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn setup_request() -> InitialSetupRequest {
        InitialSetupRequest {
            idempotency_key: "phase09-backup-fixture".to_owned(),
            company_code: "P09".to_owned(),
            name_ar: "شركة اختبار النسخ الاحتياطي".to_owned(),
            name_fr: Some("Société test sauvegarde".to_owned()),
            legal_name: "POSMAN PHASE 09 TEST".to_owned(),
            activity_description: "Tests".to_owned(),
            legal_form: Some("SARL".to_owned()),
            trade_register_number: None,
            tax_identifier: None,
            statistical_identifier: None,
            tax_article_number: None,
            bank_rib: None,
            social_capital_minor: Some(100_000),
            address_text: "Alger".to_owned(),
            wilaya_code: "16".to_owned(),
            city: Some("Alger".to_owned()),
            postal_code: Some("16000".to_owned()),
            phone: "0550000000".to_owned(),
            email: Some("phase09@example.test".to_owned()),
            language: "fr".to_owned(),
            fiscal_starts_on: "2026-01-01".to_owned(),
            fiscal_ends_on: "2026-12-31".to_owned(),
            default_margin_rate_scaled: 200_000,
            below_cost_policy: Some("ADMIN_OVERRIDE".to_owned()),
            session_idle_timeout_minutes: 30,
            taxes: vec![TaxSetup {
                code: "TVA19".to_owned(),
                name_ar: "الرسم على القيمة المضافة".to_owned(),
                name_fr: "TVA 19%".to_owned(),
                rate_scaled: 190_000,
            }],
            default_tax_code: Some("TVA19".to_owned()),
            warehouse_code: "MAIN".to_owned(),
            warehouse_name_ar: "المخزن الرئيسي".to_owned(),
            warehouse_name_fr: Some("Dépôt principal".to_owned()),
            administrator_username: "phase09-admin".to_owned(),
            administrator_display_name: "PHASE 09 Admin".to_owned(),
            administrator_password: "Phase09!Admin2026".to_owned(),
            administrator_password_confirmation: "Phase09!Admin2026".to_owned(),
        }
    }

    fn service_fixture() -> (TestDirectory, Phase09Service, Phase05Service, String) {
        let directory = TestDirectory::new();
        let paths = RuntimePaths::create_all(directory.path.join("POSMAN"))
            .expect("runtime paths should initialize");
        RuntimeDatabase::initialize(&paths.database).expect("runtime database should initialize");
        let phase05 = Phase05Service::new(&paths.database).expect("PHASE 05 service should build");
        let setup = phase05
            .complete_initial_setup(setup_request())
            .expect("initial setup should complete");
        let phase09 = Phase09Service::new(phase05.clone(), paths)
            .expect("PHASE 09 service should provision permissions");
        phase05
            .login(LoginRequest {
                username: "phase09-admin".to_owned(),
                password: "Phase09!Admin2026".to_owned(),
            })
            .expect("administrator should authenticate");
        (directory, phase09, phase05, setup.company_id)
    }

    #[test]
    fn path_traversal_is_rejected() {
        assert!(managed_backup_path(Path::new("backups"), "../outside.sqlite3").is_err());
        assert!(managed_backup_path(Path::new("backups"), "company/a.sqlite3").is_ok());
    }

    #[test]
    fn only_known_backup_kinds_are_allowed() {
        assert!(validate_backup_kind("MANUAL", false).is_ok());
        assert!(validate_backup_kind("PRE_RESTORE", false).is_err());
        assert!(validate_backup_kind("PRE_RESTORE", true).is_ok());
    }

    #[test]
    fn online_backup_verification_corruption_retention_and_delete_guards_are_real() {
        let (_directory, service, _phase05, company_id) = service_fixture();
        let manual = service
            .create_backup(CreateBackupRequest {
                backup_kind: "MANUAL".to_owned(),
            })
            .expect("online manual backup should be created and verified");
        assert_eq!(manual.schema_version, "0007");
        assert_eq!(manual.verification_status, "VERIFIED");
        assert_eq!(manual.sha256.len(), 64);

        let last_valid = service
            .delete_backup(BackupKeyRequest {
                backup_id: manual.backup_id.clone(),
            })
            .expect_err("the last valid backup must be retained");
        assert_eq!(last_valid.code, "LAST_VALID_BACKUP");

        for _ in 0..8 {
            service
                .create_backup(CreateBackupRequest {
                    backup_kind: "AUTOMATIC_DAILY".to_owned(),
                })
                .expect("automatic daily backup should succeed");
        }
        let daily = service
            .list_backups(BackupListRequest {
                backup_kind: Some("AUTOMATIC_DAILY".to_owned()),
                page: 1,
                page_size: 20,
            })
            .expect("daily backups should list");
        assert_eq!(
            daily.total, 7,
            "daily retention must preserve exactly seven"
        );

        let daily_record = service
            .load_backup_record(&company_id, &daily.items[0].backup_id)
            .expect("daily backup record should exist");
        let daily_path = managed_backup_path(&service.paths.backups, &daily_record.relative_path)
            .expect("managed backup path should resolve");
        OpenOptions::new()
            .append(true)
            .open(&daily_path)
            .expect("backup should be writable in the isolated fixture")
            .write_all(b"tampered")
            .expect("backup fixture should be corrupted");
        let corrupted = service
            .verify_backup(BackupKeyRequest {
                backup_id: daily_record.backup_id,
            })
            .expect_err("SHA-256 mismatch must reject a corrupted backup");
        assert_eq!(corrupted.code, "BACKUP_INTEGRITY_FAILED");

        service
            .delete_backup(BackupKeyRequest {
                backup_id: manual.backup_id,
            })
            .expect("a verified backup may be deleted when other verified backups remain");
    }

    #[test]
    fn restore_replaces_the_database_records_safety_backup_and_invalidates_session() {
        let (_directory, service, phase05, company_id) = service_fixture();
        let selected = service
            .create_backup(CreateBackupRequest {
                backup_kind: "MANUAL".to_owned(),
            })
            .expect("restore source should be created");

        phase05
            .phase09_open()
            .expect("fixture database should open")
            .execute(
                "UPDATE companies SET legal_name='MUTATED AFTER BACKUP' WHERE id=?1",
                [company_id.as_str()],
            )
            .expect("fixture should mutate after backup");

        service
            .restore_backup(RestoreBackupRequest {
                backup_id: selected.backup_id.clone(),
                current_password: "Phase09!Admin2026".to_owned(),
                confirmation_text: "RESTORE".to_owned(),
                confirmed: true,
            })
            .expect("verified backup restore should succeed");

        let restored = Connection::open(service.database_path()).expect("restored database opens");
        let legal_name: String = restored
            .query_row(
                "SELECT legal_name FROM companies WHERE id=?1",
                [company_id.as_str()],
                |row| row.get(0),
            )
            .expect("restored company should exist");
        assert_eq!(legal_name, "POSMAN PHASE 09 TEST");
        let success_count: i64 = restored
            .query_row(
                "SELECT COUNT(*) FROM phase09_restore_attempts WHERE outcome='SUCCESS'",
                [],
                |row| row.get(0),
            )
            .expect("restore success should be recorded");
        assert_eq!(success_count, 1);
        let pre_restore_count: i64 = restored
            .query_row(
                "SELECT COUNT(*) FROM phase09_backups WHERE backup_kind='PRE_RESTORE' AND verification_status='VERIFIED'",
                [],
                |row| row.get(0),
            )
            .expect("PRE_RESTORE backup should be recorded");
        assert_eq!(pre_restore_count, 1);
        assert!(
            phase05.get_current_session().is_err(),
            "restore must invalidate the active session"
        );
    }
}

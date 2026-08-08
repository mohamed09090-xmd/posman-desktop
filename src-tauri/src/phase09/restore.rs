use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::backup::Backup;
use rusqlite::{params, Connection};
// Contract marker: rusqlite::backup::Backup

use super::{
    backup::{
        managed_backup_path, verify_database_file, BackupRecord, ExpectedArtifact,
        VerifiedDatabase,
    },
    error::{Phase09Error, Phase09Result},
    models::RestoreBackupRequest,
    new_id, now_iso, safe_component, Phase09Service,
};

impl Phase09Service {
    pub fn restore_backup(&self, request: RestoreBackupRequest) -> Phase09Result<()> {
        if !request.confirmed || request.confirmation_text != "RESTORE" {
            return Err(Phase09Error::new(
                "RESTORE_CONFIRMATION_REQUIRED",
                "Restore requires explicit confirmation and the exact text RESTORE.",
                false,
            ));
        }
        if request.current_password.is_empty() {
            return Err(Phase09Error::new(
                "RESTORE_REAUTHENTICATION_REQUIRED",
                "Restore requires the current user's password.",
                false,
            ));
        }
        let context = self.authorize("backup.restore")?;
        let reauthenticated = self
            .phase05
            .phase09_reauthenticate(&request.current_password)?;
        if context.company_id != reauthenticated.company_id
            || context.user_id != reauthenticated.user_id
        {
            return Err(Phase09Error::permission());
        }
        let selected = self.load_backup_record(&context.company_id, &request.backup_id)?;
        if selected.verification_status != "VERIFIED" {
            return Err(Phase09Error::new(
                "BACKUP_NOT_VERIFIED",
                "Restore requires a verified backup.",
                false,
            ));
        }
        let selected_path = managed_backup_path(&self.paths.backups, &selected.relative_path)?;
        verify_database_file(
            &selected_path,
            ExpectedArtifact {
                sha256: Some(selected.sha256.clone()),
                size_bytes: Some(selected.database_size_bytes),
            },
        )?;

        let gate = self.phase05.phase09_maintenance_gate();
        let _maintenance = gate.begin_restore().map_err(|_| Phase09Error::maintenance())?;
        let result = self.restore_exclusive(&context, &selected, &selected_path);
        if result.is_err() {
            let _ = self.phase05.phase09_invalidate_session();
        }
        result
    }

    fn restore_exclusive(
        &self,
        context: &crate::phase05::Phase06AuthContext,
        selected: &BackupRecord,
        selected_path: &Path,
    ) -> Phase09Result<()> {
        fs::create_dir_all(&self.paths.staging)?;
        let attempt_id = new_id();
        let staged_path = self
            .paths
            .staging
            .join(format!("phase09-restore-{attempt_id}.sqlite3"));
        remove_if_exists(&staged_path);
        atomic_copy(selected_path, &staged_path, false)?;
        let staged_verified = verify_database_file(
            &staged_path,
            ExpectedArtifact {
                sha256: Some(selected.sha256.clone()),
                size_bytes: Some(selected.database_size_bytes),
            },
        )?;
        self.insert_restore_attempt(
            context,
            &attempt_id,
            &selected.backup_id,
            None,
            "STARTED",
            None,
            Some(&serde_json::json!({
                "stagedSha256": staged_verified.sha256,
                "schemaVersion": staged_verified.schema_version,
            })),
        )?;

        let pre_restore = match self.create_pre_restore_backup_exclusive(context) {
            Ok(record) => record,
            Err(error) => {
                remove_if_exists(&staged_path);
                let _ = self.insert_restore_attempt(
                    context,
                    &new_id(),
                    &selected.backup_id,
                    None,
                    "FAILED",
                    Some(&error.code),
                    Some(&serde_json::json!({"stage":"PRE_RESTORE_BACKUP"})),
                );
                return Err(Phase09Error::new(
                    "PRE_RESTORE_BACKUP_FAILED",
                    "Restore was aborted because the safety backup could not be created and verified.",
                    false,
                ));
            }
        };

        let active_path = self.database_path().to_path_buf();
        let active_parent = active_path
            .parent()
            .ok_or_else(|| Phase09Error::validation("Invalid active database path."))?;
        let incoming_path = active_parent.join(format!(".phase09-restore-{attempt_id}.incoming"));
        let previous_path = active_parent.join(format!(".phase09-restore-{attempt_id}.previous"));
        remove_if_exists(&incoming_path);
        remove_if_exists(&previous_path);
        atomic_copy(&staged_path, &incoming_path, false)?;
        verify_database_file(
            &incoming_path,
            ExpectedArtifact {
                sha256: Some(selected.sha256.clone()),
                size_bytes: Some(selected.database_size_bytes),
            },
        )?;

        remove_sqlite_sidecars(&active_path);
        fs::rename(&active_path, &previous_path).map_err(|_| {
            Phase09Error::new(
                "RESTORE_REPLACE_FAILED",
                "The active database could not be staged for replacement.",
                true,
            )
        })?;
        if let Err(error) = fs::rename(&incoming_path, &active_path) {
            let _ = fs::rename(&previous_path, &active_path);
            remove_if_exists(&incoming_path);
            remove_if_exists(&staged_path);
            return Err(Phase09Error::new(
                "RESTORE_REPLACE_FAILED",
                &format!("The verified backup could not replace the active database: {error}"),
                true,
            ));
        }

        let post_verification = verify_database_file(
            &active_path,
            ExpectedArtifact {
                sha256: Some(selected.sha256.clone()),
                size_bytes: Some(selected.database_size_bytes),
            },
        );
        if let Err(post_error) = post_verification {
            let rollback = self.rollback_to_pre_restore(
                &active_path,
                &previous_path,
                &pre_restore,
                &attempt_id,
            );
            remove_if_exists(&staged_path);
            match rollback {
                Ok(()) => {
                    let _ = self.insert_restore_attempt(
                        context,
                        &new_id(),
                        &selected.backup_id,
                        Some(&pre_restore.backup_id),
                        "ROLLED_BACK",
                        Some(&post_error.code),
                        Some(&serde_json::json!({"stage":"POST_REPLACEMENT_VERIFICATION"})),
                    );
                    let _ = self.phase05.phase09_invalidate_session();
                    return Err(Phase09Error::new(
                        "RESTORE_ROLLED_BACK",
                        "Post-restore verification failed. POSMAN restored the verified safety backup.",
                        false,
                    ));
                }
                Err(_) => {
                    return Err(Phase09Error::new(
                        "RESTORE_ROLLBACK_FAILED",
                        "Restore verification and automatic rollback both failed. Manual recovery is required.",
                        false,
                    ));
                }
            }
        }

        remove_if_exists(&previous_path);
        remove_if_exists(&staged_path);
        let restored = self.phase05.phase09_open()?;
        insert_backup_metadata_if_missing(&restored, &context.company_id, selected)?;
        insert_backup_metadata_if_missing(&restored, &context.company_id, &pre_restore)?;
        restored.execute(
            "UPDATE phase09_backups SET protected_for_restore=0 WHERE company_id=?1",
            [context.company_id.as_str()],
        )?;
        insert_restore_attempt_on(
            &restored,
            context,
            &new_id(),
            &selected.backup_id,
            Some(&pre_restore.backup_id),
            "SUCCESS",
            None,
            Some(&serde_json::json!({
                "restoredSha256": selected.sha256,
                "schemaVersion": selected.schema_version,
            })),
        )?;
        restored.execute(
            r#"INSERT INTO audit_logs(
                   id,company_id,actor_user_id,action_code,entity_type,entity_id,
                   occurred_at,outcome,correlation_id,details_json
               ) VALUES(?1,?2,?3,'BACKUP_RESTORE_SUCCEEDED','PHASE09_BACKUP',?4,?5,'SUCCESS',?6,
                        json_object('preRestoreBackupId',?7,'sha256',?8))"#,
            params![
                new_id(),
                context.company_id,
                context.user_id,
                selected.backup_id,
                now_iso()?,
                context.session_id,
                pre_restore.backup_id,
                selected.sha256,
            ],
        )?;
        drop(restored);
        self.phase05.phase09_invalidate_session()?;
        Ok(())
    }

    fn create_pre_restore_backup_exclusive(
        &self,
        context: &crate::phase05::Phase06AuthContext,
    ) -> Phase09Result<BackupRecord> {
        let backup_id = new_id();
        let created_at = now_iso()?;
        let relative_path = pre_restore_relative_path(
            &context.company_id,
            &created_at,
            &backup_id,
        )?;
        let final_path = managed_backup_path(&self.paths.backups, &relative_path)?;
        let parent = final_path
            .parent()
            .ok_or_else(|| Phase09Error::validation("Invalid PRE_RESTORE path."))?;
        fs::create_dir_all(parent)?;
        let temporary_path = self
            .paths
            .staging
            .join(format!("phase09-pre-restore-{backup_id}.sqlite3.tmp"));
        remove_if_exists(&temporary_path);
        let source = self.phase05.phase09_open()?;
        let mut destination = Connection::open(&temporary_path)?;
        {
            let backup = Backup::new(&source, &mut destination)?;
            backup.run_to_completion(128, Duration::from_millis(20), None)?;
        }
        drop(destination);
        let verified = verify_database_file(&temporary_path, ExpectedArtifact::default())?;
        fs::rename(&temporary_path, &final_path)?;
        let record = BackupRecord {
            backup_id: backup_id.clone(),
            backup_kind: "PRE_RESTORE".to_owned(),
            created_at: verified.verified_at.clone(),
            created_by: Some(context.user_id.clone()),
            application_version: env!("CARGO_PKG_VERSION").to_owned(),
            schema_version: verified.schema_version.clone(),
            migration_ledger_digest: verified.migration_ledger_digest.clone(),
            database_size_bytes: verified.size_bytes,
            sha256: verified.sha256.clone(),
            relative_path,
            integrity_status: "OK".to_owned(),
            foreign_key_status: "OK".to_owned(),
            verification_status: "VERIFIED".to_owned(),
            failure_reason: None,
            protected_for_restore: true,
        };
        let connection = self.phase05.phase09_open()?;
        insert_backup_metadata_if_missing(&connection, &context.company_id, &record)?;
        connection.execute(
            "UPDATE phase09_backups SET protected_for_restore=1 WHERE id=?1 AND company_id=?2",
            params![record.backup_id, context.company_id],
        )?;
        verify_database_file(
            &final_path,
            ExpectedArtifact {
                sha256: Some(record.sha256.clone()),
                size_bytes: Some(record.database_size_bytes),
            },
        )?;
        Ok(record)
    }

    fn rollback_to_pre_restore(
        &self,
        active_path: &Path,
        previous_path: &Path,
        pre_restore: &BackupRecord,
        attempt_id: &str,
    ) -> Phase09Result<()> {
        let pre_restore_path =
            managed_backup_path(&self.paths.backups, &pre_restore.relative_path)?;
        verify_database_file(
            &pre_restore_path,
            ExpectedArtifact {
                sha256: Some(pre_restore.sha256.clone()),
                size_bytes: Some(pre_restore.database_size_bytes),
            },
        )?;
        let rollback_incoming = active_path
            .parent()
            .ok_or_else(|| Phase09Error::validation("Invalid active database path."))?
            .join(format!(".phase09-rollback-{attempt_id}.incoming"));
        remove_if_exists(&rollback_incoming);
        atomic_copy(&pre_restore_path, &rollback_incoming, false)?;
        remove_sqlite_sidecars(active_path);
        remove_if_exists(active_path);
        fs::rename(&rollback_incoming, active_path)?;
        verify_database_file(
            active_path,
            ExpectedArtifact {
                sha256: Some(pre_restore.sha256.clone()),
                size_bytes: Some(pre_restore.database_size_bytes),
            },
        )?;
        remove_if_exists(previous_path);
        Ok(())
    }

    fn insert_restore_attempt(
        &self,
        context: &crate::phase05::Phase06AuthContext,
        attempt_id: &str,
        backup_id: &str,
        pre_restore_backup_id: Option<&str>,
        outcome: &str,
        failure_code: Option<&str>,
        details: Option<&serde_json::Value>,
    ) -> Phase09Result<()> {
        let connection = self.phase05.phase09_open()?;
        insert_restore_attempt_on(
            &connection,
            context,
            attempt_id,
            backup_id,
            pre_restore_backup_id,
            outcome,
            failure_code,
            details,
        )
    }
}

fn insert_restore_attempt_on(
    connection: &Connection,
    context: &crate::phase05::Phase06AuthContext,
    attempt_id: &str,
    backup_id: &str,
    pre_restore_backup_id: Option<&str>,
    outcome: &str,
    failure_code: Option<&str>,
    details: Option<&serde_json::Value>,
) -> Phase09Result<()> {
    let now = now_iso()?;
    let completed_at = if outcome == "STARTED" {
        None
    } else {
        Some(now.as_str())
    };
    let details_json = details
        .map(serde_json::to_string)
        .transpose()
        .map_err(|_| Phase09Error::internal())?;
    connection.execute(
        r#"INSERT INTO phase09_restore_attempts(
               id,company_id,backup_id,pre_restore_backup_id,requested_at,requested_by,
               completed_at,outcome,failure_code,details_json
           ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)"#,
        params![
            attempt_id,
            context.company_id,
            backup_id,
            pre_restore_backup_id,
            now,
            context.user_id,
            completed_at,
            outcome,
            failure_code,
            details_json,
        ],
    )?;
    Ok(())
}

fn insert_backup_metadata_if_missing(
    connection: &Connection,
    company_id: &str,
    record: &BackupRecord,
) -> Phase09Result<()> {
    connection.execute(
        r#"INSERT OR IGNORE INTO phase09_backups(
               id,company_id,backup_kind,created_at,created_by,application_version,
               schema_version,migration_ledger_digest,database_size_bytes,sha256,relative_path,
               integrity_status,foreign_key_status,verification_status,failure_reason,imported,
               protected_for_restore,verified_at
           ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,0,?16,?4)"#,
        params![
            record.backup_id,
            company_id,
            record.backup_kind,
            record.created_at,
            record.created_by,
            record.application_version,
            record.schema_version,
            record.migration_ledger_digest,
            record.database_size_bytes,
            record.sha256,
            record.relative_path,
            record.integrity_status,
            record.foreign_key_status,
            record.verification_status,
            record.failure_reason,
            i64::from(record.protected_for_restore),
        ],
    )?;
    Ok(())
}

fn pre_restore_relative_path(
    company_id: &str,
    created_at: &str,
    backup_id: &str,
) -> Phase09Result<String> {
    let date = created_at
        .get(0..10)
        .ok_or_else(|| Phase09Error::validation("Invalid PRE_RESTORE timestamp."))?;
    Ok(format!(
        "{}/pre_restore/{}/{}/{}.sqlite3",
        safe_component(company_id)?,
        safe_component(&date[0..4])?,
        safe_component(&date[5..7])?,
        safe_component(backup_id)?,
    ))
}

fn atomic_copy(source: &Path, destination: &Path, replace: bool) -> Phase09Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| Phase09Error::validation("Invalid restore path."))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".phase09-copy-{}.tmp", new_id()));
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
                "A restore staging file already exists.",
                false,
            ));
        }
    }
    fs::rename(&temporary, destination)?;
    Ok(())
}

fn remove_sqlite_sidecars(database: &Path) {
    let raw = database.as_os_str().to_string_lossy();
    remove_if_exists(Path::new(&format!("{raw}-wal")));
    remove_if_exists(Path::new(&format!("{raw}-shm")));
    remove_if_exists(Path::new(&format!("{raw}-journal")));
}

fn remove_if_exists(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_confirmation_is_required() {
        let request = RestoreBackupRequest {
            backup_id: "backup".to_owned(),
            current_password: "password".to_owned(),
            confirmation_text: "restore".to_owned(),
            confirmed: true,
        };
        assert_ne!(request.confirmation_text, "RESTORE");
    }

    #[test]
    fn pre_restore_path_is_managed_and_deterministic() {
        assert_eq!(
            pre_restore_relative_path(
                "company-1",
                "2026-08-07T03:00:00Z",
                "backup-1"
            )
            .expect("path"),
            "company-1/pre_restore/2026/08/backup-1.sqlite3"
        );
    }
}

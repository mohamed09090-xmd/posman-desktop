use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration as StdDuration, Instant},
};

use rusqlite::{params, Connection, Transaction};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::infrastructure::{
    database::open_configured_connection,
    maintenance::{GuardedConnection, MaintenanceGate},
};

use super::{
    dto::SessionView,
    error::{Phase05Error, Phase05Result},
    security::PasswordEngine,
};

const LAST_SEEN_WRITE_INTERVAL: StdDuration = StdDuration::from_secs(60);

#[derive(Clone)]
pub struct Phase05Service {
    database_path: PathBuf,
    pub(super) password_engine: PasswordEngine,
    pub(super) dummy_hash: String,
    session: Arc<Mutex<Option<ActiveSession>>>,
    pub(super) maintenance: MaintenanceGate,
}

pub(super) struct ActiveSession {
    pub company_id: String,
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub preferred_language: String,
    pub permissions: HashSet<String>,
    pub session_id: String,
    pub _token: Zeroizing<Vec<u8>>,
    pub expires_at_unix: i64,
    pub idle_timeout: StdDuration,
    pub last_activity: Instant,
    pub last_seen_write: Instant,
    pub locked: bool,
}

#[derive(Clone)]
pub(super) struct SessionContext {
    pub company_id: String,
    pub user_id: String,
    pub session_id: String,
}

impl Phase05Service {
    pub fn new(database_path: impl AsRef<Path>) -> Phase05Result<Self> {
        let password_engine = PasswordEngine::runtime();
        let dummy_hash = password_engine.dummy_hash()?;
        Ok(Self {
            database_path: database_path.as_ref().to_path_buf(),
            password_engine,
            dummy_hash,
            session: Arc::new(Mutex::new(None)),
            maintenance: MaintenanceGate::default(),
        })
    }

    pub(crate) fn phase09_database_path(&self) -> &Path {
        &self.database_path
    }

    pub(super) fn open_raw(&self) -> Phase05Result<Connection> {
        open_configured_connection(&self.database_path)
            .map(|(connection, _)| connection)
            .map_err(|_| Phase05Error::internal())
    }

    pub(super) fn open(&self) -> Phase05Result<GuardedConnection> {
        let permit = self.maintenance.enter_database_operation().map_err(|_| {
            Phase05Error::new(
                "MAINTENANCE_ACTIVE",
                "POSMAN is restoring a verified backup.",
            )
        })?;
        Ok(permit.guard(self.open_raw()?))
    }

    pub(super) fn replace_session(&self, session: ActiveSession) -> Phase05Result<()> {
        *self.session.lock().map_err(|_| Phase05Error::internal())? = Some(session);
        Ok(())
    }

    pub(super) fn take_session(&self) -> Phase05Result<Option<ActiveSession>> {
        Ok(self
            .session
            .lock()
            .map_err(|_| Phase05Error::internal())?
            .take())
    }

    pub(super) fn with_session<T>(
        &self,
        operation: impl FnOnce(&mut ActiveSession) -> Phase05Result<T>,
    ) -> Phase05Result<T> {
        let mut guard = self.session.lock().map_err(|_| Phase05Error::internal())?;
        let session = guard.as_mut().ok_or_else(Phase05Error::unauthenticated)?;
        operation(session)
    }

    pub(super) fn require_session(
        &self,
        permission: Option<&str>,
    ) -> Phase05Result<SessionContext> {
        self.maintenance.ensure_available().map_err(|_| {
            Phase05Error::new(
                "MAINTENANCE_ACTIVE",
                "POSMAN is restoring a verified backup.",
            )
        })?;
        let (context, update_last_seen) = self.with_session(|session| {
            apply_expiry_and_idle_lock(session);
            if session.locked {
                return Err(Phase05Error::locked());
            }
            if permission.is_some_and(|code| !session.permissions.contains(code)) {
                return Err(Phase05Error::denied());
            }
            session.last_activity = Instant::now();
            let update_last_seen = session.last_seen_write.elapsed() >= LAST_SEEN_WRITE_INTERVAL;
            if update_last_seen {
                session.last_seen_write = Instant::now();
            }
            Ok((
                SessionContext {
                    company_id: session.company_id.clone(),
                    user_id: session.user_id.clone(),
                    session_id: session.session_id.clone(),
                },
                update_last_seen,
            ))
        })?;

        if update_last_seen {
            self.open()?.execute(
                r#"
                UPDATE sessions
                SET last_seen_at=?1
                WHERE id=?2 AND company_id=?3 AND user_id=?4 AND revoked_at IS NULL
                "#,
                params![
                    now_iso()?,
                    context.session_id,
                    context.company_id,
                    context.user_id
                ],
            )?;
        }
        Ok(context)
    }

    pub(super) fn has_permission(&self, permission: &str) -> Phase05Result<bool> {
        self.maintenance.ensure_available().map_err(|_| {
            Phase05Error::new(
                "MAINTENANCE_ACTIVE",
                "POSMAN is restoring a verified backup.",
            )
        })?;
        self.with_session(|session| {
            apply_expiry_and_idle_lock(session);
            if session.locked {
                return Err(Phase05Error::locked());
            }
            Ok(session.permissions.contains(permission))
        })
    }

    pub(super) fn current_session_view(&self) -> Phase05Result<SessionView> {
        self.with_session(|session| {
            apply_expiry_and_idle_lock(session);
            Ok(session_view(session))
        })
    }
}

pub(super) fn apply_expiry_and_idle_lock(session: &mut ActiveSession) {
    if OffsetDateTime::now_utc().unix_timestamp() >= session.expires_at_unix
        || session.last_activity.elapsed() >= session.idle_timeout
    {
        session.locked = true;
    }
}

pub(super) fn session_view(session: &ActiveSession) -> SessionView {
    let mut permissions = session.permissions.iter().cloned().collect::<Vec<_>>();
    permissions.sort();
    SessionView {
        company_id: session.company_id.clone(),
        user_id: session.user_id.clone(),
        username: session.username.clone(),
        display_name: session.display_name.clone(),
        preferred_language: session.preferred_language.clone(),
        permissions,
        locked: session.locked,
    }
}

pub(super) fn audit(
    transaction: &Transaction<'_>,
    context: &SessionContext,
    action_code: &str,
    entity_type: &str,
    entity_id: &str,
    details_json: Option<&str>,
) -> Phase05Result<()> {
    transaction.execute(
        r#"
        INSERT INTO audit_logs (
            id, company_id, actor_user_id, action_code, entity_type, entity_id,
            occurred_at, outcome, correlation_id, details_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'SUCCESS', ?8, ?9)
        "#,
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

pub(super) fn new_id() -> String {
    Uuid::now_v7().to_string()
}

pub(super) fn now_iso() -> Phase05Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| Phase05Error::internal())
}

pub(super) fn parse_timestamp(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

pub(super) fn trim_required(value: &str, field: &str) -> Phase05Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Phase05Error::invalid(field));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub(super) fn normalize_username(value: &str) -> Phase05Result<String> {
    Ok(trim_required(value, "username")?.to_lowercase())
}

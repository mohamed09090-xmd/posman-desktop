use std::{
    collections::HashSet,
    time::{Duration as StdDuration, Instant},
};

use rusqlite::{params, OptionalExtension};
use time::{Duration, OffsetDateTime};

use super::{
    dto::{
        ChangePasswordRequest, LoginRequest, RecoverPasswordRequest, RecoveryCodeResult,
        SessionView, UnlockSessionRequest,
    },
    error::{Phase05Error, Phase05Result},
    security::{
        constant_time_hex_equal, generate_recovery_code, generate_session_secret,
        recovery_code_hash,
    },
    state::{
        audit, new_id, normalize_username, now_iso, parse_timestamp, session_view, ActiveSession,
        Phase05Service,
    },
};

const LOCKOUT_FAILURES: i64 = 5;
const LOCKOUT_MINUTES: i64 = 15;
const SESSION_ABSOLUTE_HOURS: i64 = 12;

struct LoginUser {
    id: String,
    company_id: String,
    username: String,
    display_name: String,
    password_hash: String,
    preferred_language: String,
    failed_login_count: i64,
    locked_until: Option<String>,
    is_active: bool,
    idle_timeout_minutes: i64,
}

impl Phase05Service {
    pub fn login(&self, request: LoginRequest) -> Phase05Result<SessionView> {
        let normalized = normalize_username(&request.username)?;
        let mut connection = self.open()?;
        let user = connection
            .query_row(
                r#"
                SELECT u.id, u.company_id, u.username, u.display_name, u.password_hash,
                       u.preferred_language, u.failed_login_count, u.locked_until,
                       u.is_active, s.session_idle_timeout_minutes
                FROM users u
                JOIN company_settings s ON s.company_id=u.company_id
                WHERE lower(trim(u.username))=?1
                LIMIT 1
                "#,
                [normalized.as_str()],
                |row| {
                    Ok(LoginUser {
                        id: row.get(0)?,
                        company_id: row.get(1)?,
                        username: row.get(2)?,
                        display_name: row.get(3)?,
                        password_hash: row.get(4)?,
                        preferred_language: row.get(5)?,
                        failed_login_count: row.get(6)?,
                        locked_until: row.get(7)?,
                        is_active: row.get::<_, i64>(8)? == 1,
                        idle_timeout_minutes: row.get(9)?,
                    })
                },
            )
            .optional()?;

        let candidate_hash = user.as_ref().map_or(self.dummy_hash.as_str(), |found| {
            found.password_hash.as_str()
        });
        let matches = self
            .password_engine
            .verify(&request.password, candidate_hash);
        let now = OffsetDateTime::now_utc();
        let locked = user
            .as_ref()
            .and_then(|found| found.locked_until.as_deref())
            .and_then(parse_timestamp)
            .is_some_and(|until| until > now);

        let Some(user) = user else {
            return Err(authentication_failed());
        };
        if !matches || !user.is_active || locked {
            if user.is_active && !locked {
                record_failed_login(&mut connection, &user, now)?;
            }
            return Err(authentication_failed());
        }

        let permissions = load_permissions(&connection, &user.company_id, &user.id)?;
        let secret = generate_session_secret();
        let session_id = new_id();
        let created_at = now_iso()?;
        let expires = now + Duration::hours(SESSION_ABSOLUTE_HOURS);
        let expires_at = expires
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|_| Phase05Error::internal())?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE sessions SET revoked_at=?1 WHERE user_id=?2 AND revoked_at IS NULL",
            params![created_at, user.id],
        )?;
        transaction.execute(
            r#"
            INSERT INTO sessions (
                id, company_id, user_id, token_hash, created_at, expires_at, last_seen_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5)
            "#,
            params![
                session_id,
                user.company_id,
                user.id,
                secret.hash,
                created_at,
                expires_at
            ],
        )?;
        transaction.execute(
            r#"
            UPDATE users
            SET failed_login_count=0, locked_until=NULL, last_login_at=?1,
                updated_at=?1, row_version=row_version+1
            WHERE id=?2 AND company_id=?3
            "#,
            params![created_at, user.id, user.company_id],
        )?;
        transaction.commit()?;

        let idle_minutes = u64::try_from(user.idle_timeout_minutes.clamp(5, 120)).unwrap_or(15);
        let session = ActiveSession {
            company_id: user.company_id.clone(),
            user_id: user.id.clone(),
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            preferred_language: user.preferred_language.clone(),
            permissions: permissions.clone(),
            session_id,
            _token: secret.raw,
            expires_at_unix: expires.unix_timestamp(),
            idle_timeout: StdDuration::from_secs(idle_minutes * 60),
            last_activity: Instant::now(),
            last_seen_write: Instant::now(),
            locked: false,
        };
        let view = session_view(&session);
        self.replace_session(session)?;
        Ok(view)
    }

    pub fn get_current_session(&self) -> Phase05Result<SessionView> {
        self.current_session_view()
    }

    pub fn logout(&self) -> Phase05Result<()> {
        if let Some(session) = self.take_session()? {
            self.open()?.execute(
                r#"
                UPDATE sessions SET revoked_at=?1
                WHERE id=?2 AND company_id=?3 AND user_id=?4 AND revoked_at IS NULL
                "#,
                params![
                    now_iso()?,
                    session.session_id,
                    session.company_id,
                    session.user_id
                ],
            )?;
        }
        Ok(())
    }

    pub fn lock_session(&self) -> Phase05Result<()> {
        self.with_session(|session| {
            session.locked = true;
            Ok(())
        })
    }

    pub fn unlock_session(&self, request: UnlockSessionRequest) -> Phase05Result<SessionView> {
        let (company_id, user_id) =
            self.with_session(|session| Ok((session.company_id.clone(), session.user_id.clone())))?;
        let hash: String = self.open()?.query_row(
            "SELECT password_hash FROM users WHERE id=?1 AND company_id=?2 AND is_active=1",
            params![user_id, company_id],
            |row| row.get(0),
        )?;
        if !self.password_engine.verify(&request.password, &hash) {
            return Err(authentication_failed());
        }
        self.with_session(|session| {
            session.locked = false;
            session.last_activity = Instant::now();
            Ok(session_view(session))
        })
    }

    pub fn change_own_password(&self, request: ChangePasswordRequest) -> Phase05Result<()> {
        if request.new_password != request.new_password_confirmation {
            return Err(password_confirmation_error());
        }
        let context = self.require_session(None)?;
        let mut connection = self.open()?;
        let current_hash: String = connection.query_row(
            "SELECT password_hash FROM users WHERE id=?1 AND company_id=?2",
            params![context.user_id, context.company_id],
            |row| row.get(0),
        )?;
        if !self
            .password_engine
            .verify(&request.current_password, &current_hash)
        {
            return Err(authentication_failed());
        }
        let new_hash = self.password_engine.hash(&request.new_password)?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            UPDATE users
            SET password_hash=?1, failed_login_count=0, locked_until=NULL,
                updated_at=?2, updated_by=?3, row_version=row_version+1
            WHERE id=?3 AND company_id=?4
            "#,
            params![new_hash, now_iso()?, context.user_id, context.company_id],
        )?;
        audit(
            &transaction,
            &context,
            "security.password.change_own",
            "users",
            &context.user_id,
            None,
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn rotate_recovery_code(&self) -> Phase05Result<RecoveryCodeResult> {
        let context = self.require_session(Some("security.users.manage"))?;
        let recovery_code = generate_recovery_code();
        let hash = recovery_code_hash(&recovery_code)?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            UPDATE user_recovery_codes SET revoked_at=?1
            WHERE company_id=?2 AND user_id=?3
              AND used_at IS NULL AND revoked_at IS NULL
            "#,
            params![now_iso()?, context.company_id, context.user_id],
        )?;
        transaction.execute(
            r#"
            INSERT INTO user_recovery_codes (
                id, company_id, user_id, code_hash, created_at, created_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?3)
            "#,
            params![
                new_id(),
                context.company_id,
                context.user_id,
                hash,
                now_iso()?
            ],
        )?;
        audit(
            &transaction,
            &context,
            "security.recovery.rotate",
            "users",
            &context.user_id,
            None,
        )?;
        transaction.commit()?;
        Ok(RecoveryCodeResult { recovery_code })
    }

    pub fn recover_admin_password(
        &self,
        request: RecoverPasswordRequest,
    ) -> Phase05Result<RecoveryCodeResult> {
        if request.new_password != request.new_password_confirmation {
            return Err(password_confirmation_error());
        }
        let username = normalize_username(&request.username)?;
        let requested_hash = recovery_code_hash(&request.recovery_code)?;
        let new_hash = self.password_engine.hash(&request.new_password)?;
        let mut connection = self.open()?;
        let target = connection
            .query_row(
                r#"
                SELECT u.id, u.company_id
                FROM users u
                JOIN user_roles ur
                  ON ur.user_id=u.id AND ur.company_id=u.company_id
                JOIN roles r
                  ON r.id=ur.role_id AND r.company_id=u.company_id
                WHERE lower(trim(u.username))=?1 AND u.is_active=1
                  AND r.code='SYSTEM_ADMINISTRATOR' AND r.is_active=1
                LIMIT 1
                "#,
                [username],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((user_id, company_id)) = target else {
            return Err(recovery_failed());
        };
        let active = connection
            .query_row(
                r#"
                SELECT id, code_hash
                FROM user_recovery_codes
                WHERE company_id=?1 AND user_id=?2
                  AND used_at IS NULL AND revoked_at IS NULL
                  AND (expires_at IS NULL OR expires_at>?3)
                LIMIT 1
                "#,
                params![company_id, user_id, now_iso()?],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((code_id, stored_hash)) = active else {
            return Err(recovery_failed());
        };
        if !constant_time_hex_equal(&stored_hash, &requested_hash) {
            return Err(recovery_failed());
        }

        let replacement = generate_recovery_code();
        let replacement_hash = recovery_code_hash(&replacement)?;
        let timestamp = now_iso()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            UPDATE user_recovery_codes SET used_at=?1
            WHERE id=?2 AND used_at IS NULL AND revoked_at IS NULL
            "#,
            params![timestamp, code_id],
        )?;
        transaction.execute(
            r#"
            UPDATE users
            SET password_hash=?1, failed_login_count=0, locked_until=NULL,
                updated_at=?2, updated_by=?3, row_version=row_version+1
            WHERE id=?3 AND company_id=?4
            "#,
            params![new_hash, timestamp, user_id, company_id],
        )?;
        transaction.execute(
            r#"
            UPDATE sessions SET revoked_at=?1
            WHERE user_id=?2 AND company_id=?3 AND revoked_at IS NULL
            "#,
            params![timestamp, user_id, company_id],
        )?;
        transaction.execute(
            r#"
            INSERT INTO user_recovery_codes (
                id, company_id, user_id, code_hash, created_at, created_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?3)
            "#,
            params![new_id(), company_id, user_id, replacement_hash, timestamp],
        )?;
        transaction.execute(
            r#"
            INSERT INTO audit_logs (
                id, company_id, actor_user_id, action_code, entity_type,
                entity_id, occurred_at, outcome
            ) VALUES (?1, ?2, ?3, 'security.recovery.use', 'users', ?3, ?4, 'SUCCESS')
            "#,
            params![new_id(), company_id, user_id, timestamp],
        )?;
        transaction.commit()?;
        Ok(RecoveryCodeResult {
            recovery_code: replacement,
        })
    }
}

fn record_failed_login(
    connection: &mut rusqlite::Connection,
    user: &LoginUser,
    now: OffsetDateTime,
) -> Phase05Result<()> {
    let failures = user.failed_login_count.saturating_add(1);
    let locked_until = if failures >= LOCKOUT_FAILURES {
        Some(
            (now + Duration::minutes(LOCKOUT_MINUTES))
                .format(&time::format_description::well_known::Rfc3339)
                .map_err(|_| Phase05Error::internal())?,
        )
    } else {
        None
    };
    connection.execute(
        r#"
        UPDATE users
        SET failed_login_count=?1, locked_until=?2, updated_at=?3,
            row_version=row_version+1
        WHERE id=?4 AND company_id=?5
        "#,
        params![failures, locked_until, now_iso()?, user.id, user.company_id],
    )?;
    Ok(())
}

fn load_permissions(
    connection: &rusqlite::Connection,
    company_id: &str,
    user_id: &str,
) -> Phase05Result<HashSet<String>> {
    let mut statement = connection.prepare(
        r#"
        SELECT DISTINCT p.code
        FROM user_roles ur
        JOIN roles r
          ON r.id=ur.role_id AND r.company_id=ur.company_id
        JOIN role_permissions rp
          ON rp.role_id=r.id AND rp.company_id=r.company_id
        JOIN permissions p ON p.id=rp.permission_id
        WHERE ur.company_id=?1 AND ur.user_id=?2 AND r.is_active=1
        ORDER BY p.code
        "#,
    )?;
    let rows = statement.query_map(params![company_id, user_id], |row| row.get(0))?;
    rows.collect::<Result<HashSet<String>, _>>()
        .map_err(Phase05Error::from)
}

fn authentication_failed() -> Phase05Error {
    Phase05Error::new(
        "AUTHENTICATION_FAILED",
        "The username or password is incorrect, or the account is temporarily unavailable.",
    )
}

fn recovery_failed() -> Phase05Error {
    Phase05Error::new(
        "RECOVERY_CODE_INVALID",
        "The recovery code is invalid or has already been used.",
    )
}

fn password_confirmation_error() -> Phase05Error {
    Phase05Error::new(
        "PASSWORD_CONFIRMATION_MISMATCH",
        "The password confirmation does not match.",
    )
}

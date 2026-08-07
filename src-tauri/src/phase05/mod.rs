use self::error::Phase05Result;
use rusqlite::Connection;

mod auth;
mod config;
mod draft;
pub mod dto;
pub mod error;
mod partners;
pub mod pricing;
mod products;
mod reference_simple;
pub mod security;
mod setup_transaction;
mod state;
mod users;

pub use state::Phase05Service;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Phase06AuthContext {
    pub company_id: String,
    pub user_id: String,
    pub session_id: String,
}

impl Phase05Service {
    pub(crate) fn phase06_authorize(
        &self,
        permission: Option<&str>,
    ) -> Phase05Result<Phase06AuthContext> {
        self.require_session(permission)
            .map(|context| Phase06AuthContext {
                company_id: context.company_id,
                user_id: context.user_id,
                session_id: context.session_id,
            })
    }

    pub(crate) fn phase06_open(&self) -> Phase05Result<Connection> {
        self.open()
    }

    pub(crate) fn phase09_authorize(
        &self,
        permission: Option<&str>,
    ) -> Phase05Result<Phase06AuthContext> {
        self.phase06_authorize(permission)
    }

    pub(crate) fn phase09_open(&self) -> Phase05Result<Connection> {
        self.phase06_open()
    }

    pub(crate) fn phase09_open_maintenance(
        &self,
    ) -> Phase05Result<crate::infrastructure::maintenance::GuardedConnection> {
        let permit = self
            .maintenance
            .enter_database_operation()
            .map_err(|_| {
                error::Phase05Error::new(
                    "MAINTENANCE_ACTIVE",
                    "POSMAN is restoring a verified backup.",
                )
            })?;
        Ok(permit.guard(self.open()?))
    }

    pub(crate) fn phase09_reauthenticate(
        &self,
        password: &str,
    ) -> Phase05Result<Phase06AuthContext> {
        let context = self.phase06_authorize(None)?;
        let hash: String = self.open()?.query_row(
            "SELECT password_hash FROM users WHERE id=?1 AND company_id=?2 AND is_active=1",
            rusqlite::params![context.user_id, context.company_id],
            |row| row.get(0),
        )?;
        if !self.password_engine.verify(password, &hash) {
            return Err(error::Phase05Error::new(
                "AUTHENTICATION_FAILED",
                "The current password is incorrect.",
            ));
        }
        Ok(context)
    }

    pub(crate) fn phase09_maintenance_gate(
        &self,
    ) -> crate::infrastructure::maintenance::MaintenanceGate {
        self.maintenance.clone()
    }

    pub(crate) fn phase09_invalidate_session(&self) -> Phase05Result<()> {
        let Some(session) = self.take_session()? else {
            return Ok(());
        };
        let connection = self.open()?;
        connection.execute(
            "UPDATE sessions SET revoked_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND company_id=?2 AND user_id=?3 AND revoked_at IS NULL",
            rusqlite::params![session.session_id, session.company_id, session.user_id],
        )?;
        Ok(())
    }
}

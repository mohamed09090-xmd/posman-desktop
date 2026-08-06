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
}

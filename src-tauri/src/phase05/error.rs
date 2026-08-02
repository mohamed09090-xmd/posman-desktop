use serde::Serialize;

pub type Phase05Result<T> = Result<T, Phase05Error>;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase05Error {
    pub code: String,
    pub message: String,
}

impl Phase05Error {
    pub fn new(code: &str, message: &str) -> Self {
        Self { code: code.to_owned(), message: message.to_owned() }
    }

    pub fn invalid(field: &str) -> Self {
        Self::new("VALIDATION_FAILED", &format!("Invalid value for {field}."))
    }

    pub fn internal() -> Self {
        Self::new("OPERATION_FAILED", "The local operation could not be completed.")
    }

    pub fn unauthenticated() -> Self {
        Self::new("AUTHENTICATION_REQUIRED", "Sign in to continue.")
    }

    pub fn denied() -> Self {
        Self::new("PERMISSION_DENIED", "You do not have permission for this action.")
    }

    pub fn locked() -> Self {
        Self::new("SESSION_LOCKED", "Unlock the local session to continue.")
    }

    pub fn concurrency() -> Self {
        Self::new("CONCURRENCY_CONFLICT", "This record changed. Reload it and try again.")
    }

    pub fn below_cost_blocked() -> Self {
        Self::new("BELOW_COST_BLOCKED", "The sale price cannot be below the purchase cost.")
    }

    pub fn below_cost_override_required() -> Self {
        Self::new("BELOW_COST_OVERRIDE_REQUIRED", "An authorized administrator and a reason are required to save a below-cost price.")
    }
}

impl From<rusqlite::Error> for Phase05Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::internal()
    }
}

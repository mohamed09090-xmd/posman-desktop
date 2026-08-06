use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type Phase08Result<T> = Result<T, Phase08Error>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Phase08Error {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl Phase08Error {
    pub fn new(code: &str, message: &str, retryable: bool) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
            retryable,
        }
    }
    pub fn validation(message: &str) -> Self {
        Self::new("ACCOUNTING_VALIDATION", message, false)
    }
    pub fn permission() -> Self {
        Self::new(
            "ACCOUNTING_PERMISSION_DENIED",
            "The requested accounting operation is not permitted.",
            false,
        )
    }
    pub fn internal() -> Self {
        Self::new(
            "ACCOUNTING_INTERNAL",
            "The accounting operation could not be completed.",
            true,
        )
    }
    pub fn database(error: rusqlite::Error) -> Self {
        let text = error.to_string();
        for (needle, code, message) in [
            (
                "POSTED_JOURNAL_IMMUTABLE",
                "POSTED_JOURNAL_IMMUTABLE",
                "Posted journal history is immutable.",
            ),
            (
                "POSTED_JOURNAL_LINES_IMMUTABLE",
                "POSTED_JOURNAL_IMMUTABLE",
                "Posted journal lines are immutable.",
            ),
            (
                "POSTED_PAYMENT_IMMUTABLE",
                "POSTED_PAYMENT_IMMUTABLE",
                "Posted payment history is immutable.",
            ),
            (
                "PAYMENT_ALLOCATION_APPEND_ONLY",
                "PAYMENT_ALLOCATION_IMMUTABLE",
                "Payment allocation history is append-only.",
            ),
            (
                "POSTING_ATTEMPT_APPEND_ONLY",
                "POSTING_ATTEMPT_IMMUTABLE",
                "Posting attempt history is append-only.",
            ),
        ] {
            if text.contains(needle) {
                return Self::new(code, message, false);
            }
        }
        Self::internal()
    }
}

impl Display for Phase08Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.code)
    }
}
impl std::error::Error for Phase08Error {}
impl From<rusqlite::Error> for Phase08Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::database(value)
    }
}

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type Phase09Result<T> = Result<T, Phase09Error>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Phase09Error {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl Phase09Error {
    pub fn new(code: &str, message: &str, retryable: bool) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
            retryable,
        }
    }

    pub fn validation(message: &str) -> Self {
        Self::new("PHASE09_VALIDATION", message, false)
    }

    pub fn permission() -> Self {
        Self::new(
            "PERMISSION_DENIED",
            "You do not have permission for this operation.",
            false,
        )
    }

    pub fn not_found(entity: &str) -> Self {
        Self::new(
            "NOT_FOUND",
            &format!("The requested {entity} was not found for this company."),
            false,
        )
    }

    pub fn concurrency() -> Self {
        Self::new(
            "CONCURRENCY_CONFLICT",
            "This record changed. Reload it and try again.",
            false,
        )
    }

    pub fn output_busy() -> Self {
        Self::new(
            "OUTPUT_BUSY",
            "Another local PDF or print operation is already running.",
            true,
        )
    }

    pub fn platform_unsupported() -> Self {
        Self::new(
            "PLATFORM_UNSUPPORTED",
            "Native PDF and printer output is supported only on Windows.",
            false,
        )
    }

    pub fn integrity(message: &str) -> Self {
        Self::new("ARTIFACT_INTEGRITY_FAILED", message, false)
    }

    pub fn maintenance() -> Self {
        Self::new(
            "MAINTENANCE_ACTIVE",
            "POSMAN is restoring a verified backup. Try again after maintenance completes.",
            true,
        )
    }

    pub fn internal() -> Self {
        Self::new(
            "PHASE09_OPERATION_FAILED",
            "The local operation could not be completed safely.",
            true,
        )
    }

    pub fn database(error: rusqlite::Error) -> Self {
        let text = error.to_string();
        for (needle, code, message) in [
            (
                "PUBLISHED_TEMPLATE_VERSION_IMMUTABLE",
                "PUBLISHED_TEMPLATE_IMMUTABLE",
                "Published template versions are immutable.",
            ),
            (
                "PUBLISHED_TEMPLATE_DRAFT_IMMUTABLE",
                "PUBLISHED_TEMPLATE_IMMUTABLE",
                "A published template draft cannot be changed.",
            ),
            (
                "RENDERED_DOCUMENT_IMMUTABLE",
                "RENDERED_DOCUMENT_IMMUTABLE",
                "Historical rendered documents are immutable.",
            ),
            (
                "document template versions are immutable",
                "PUBLISHED_TEMPLATE_IMMUTABLE",
                "Published template versions are immutable.",
            ),
        ] {
            if text.contains(needle) {
                return Self::new(code, message, false);
            }
        }
        Self::internal()
    }
}

impl From<crate::phase05::error::Phase05Error> for Phase09Error {
    fn from(value: crate::phase05::error::Phase05Error) -> Self {
        Self::new(&value.code, &value.message, false)
    }
}

impl From<rusqlite::Error> for Phase09Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::database(value)
    }
}

impl From<std::io::Error> for Phase09Error {
    fn from(_: std::io::Error) -> Self {
        Self::internal()
    }
}

impl Display for Phase09Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.code)
    }
}

impl std::error::Error for Phase09Error {}

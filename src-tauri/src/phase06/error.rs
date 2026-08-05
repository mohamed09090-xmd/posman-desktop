use serde::Serialize;

pub type Phase06Result<T> = Result<T, Phase06Error>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase06Error {
    pub code: String,
    pub message: String,
}

impl Phase06Error {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
        }
    }

    pub fn invalid(field: &str) -> Self {
        Self::new(
            "VALIDATION_FAILED",
            &format!("The value supplied for {field} is invalid."),
        )
    }

    pub fn internal() -> Self {
        Self::new(
            "OPERATION_FAILED",
            "The local operation could not be completed.",
        )
    }

    pub fn not_found() -> Self {
        Self::new("NOT_FOUND", "The requested record was not found.")
    }

    pub fn conflict() -> Self {
        Self::new(
            "CONCURRENCY_CONFLICT",
            "This record changed. Reload it and try again.",
        )
    }

    pub fn idempotency_conflict() -> Self {
        Self::new(
            "IDEMPOTENCY_CONFLICT",
            "This request key was already used with different content.",
        )
    }

    pub fn insufficient_stock() -> Self {
        Self::new(
            "INSUFFICIENT_STOCK",
            "Available stock is insufficient for this operation.",
        )
    }

    pub fn reserved_stock_conflict() -> Self {
        Self::new(
            "RESERVED_STOCK_CONFLICT",
            "This operation would consume stock reserved for another source line.",
        )
    }

    pub fn override_required() -> Self {
        Self::new(
            "NEGATIVE_STOCK_OVERRIDE_REQUIRED",
            "An authorized negative-stock override and a reason are required.",
        )
    }

    pub fn stale_count() -> Self {
        Self::new(
            "STALE_INVENTORY_COUNT",
            "Stock changed after the count snapshot. Refresh or recount before posting.",
        )
    }

    pub fn immutable() -> Self {
        Self::new(
            "POSTED_DOCUMENT_LOCKED",
            "A posted document cannot be changed.",
        )
    }

    pub fn over_transformation() -> Self {
        Self::new(
            "TRANSFORMATION_LIMIT_EXCEEDED",
            "The transformed quantity exceeds the remaining source quantity.",
        )
    }

    pub fn permission() -> Self {
        Self::new(
            "PERMISSION_DENIED",
            "You do not have permission for this action.",
        )
    }

    pub fn numeric_overflow() -> Self {
        Self::new(
            "NUMERIC_OVERFLOW",
            "The calculation exceeds supported fixed-point limits.",
        )
    }
}

impl From<rusqlite::Error> for Phase06Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::internal()
    }
}

impl From<crate::phase05::error::Phase05Error> for Phase06Error {
    fn from(value: crate::phase05::error::Phase05Error) -> Self {
        Self::new(&value.code, &value.message)
    }
}

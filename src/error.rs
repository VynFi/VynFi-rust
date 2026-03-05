use serde::Deserialize;

/// Structured error body from the VynFi API (RFC 7807).
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorBody {
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub status: u16,
}

impl std::fmt::Display for ErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = if !self.detail.is_empty() {
            &self.detail
        } else {
            &self.message
        };
        write!(f, "{}", msg)
    }
}

/// All errors returned by the VynFi SDK.
#[derive(Debug, thiserror::Error)]
pub enum VynFiError {
    /// 401 — invalid or missing API key.
    #[error("authentication error: {0}")]
    Authentication(ErrorBody),

    /// 402 — not enough credits.
    #[error("insufficient credits: {0}")]
    InsufficientCredits(ErrorBody),

    /// 404 — resource not found.
    #[error("not found: {0}")]
    NotFound(ErrorBody),

    /// 409 — resource conflict.
    #[error("conflict: {0}")]
    Conflict(ErrorBody),

    /// 422 — validation error.
    #[error("validation error: {0}")]
    Validation(ErrorBody),

    /// 429 — rate limited.
    #[error("rate limited: {0}")]
    RateLimit(ErrorBody),

    /// 5xx — server error.
    #[error("server error: {0}")]
    Server(ErrorBody),

    /// HTTP transport error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON deserialization error.
    #[error("deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// Client configuration error.
    #[error("{0}")]
    Config(String),
}

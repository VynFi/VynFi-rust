use serde::Deserialize;

/// Structured error body from the VynFi API (RFC 7807).
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorBody {
    /// Error type URI (e.g. `"https://api.vynfi.com/errors/not-found"`).
    #[serde(rename = "type", default)]
    pub error_type: String,
    /// Short human-readable title (e.g. `"Not Found"`).
    #[serde(default)]
    pub title: String,
    /// HTTP status code.
    #[serde(default)]
    pub status: u16,
    /// Detailed error description.
    #[serde(default)]
    pub detail: String,
    /// URI identifying the specific occurrence.
    #[serde(default)]
    pub instance: Option<String>,
}

impl std::fmt::Display for ErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.detail.is_empty() {
            write!(f, "{}", self.detail)
        } else if !self.title.is_empty() {
            write!(f, "{}", self.title)
        } else {
            write!(f, "HTTP {}", self.status)
        }
    }
}

/// All errors returned by the VynFi SDK.
#[derive(Debug, thiserror::Error)]
pub enum VynFiError {
    /// 401 — invalid or missing API key.
    #[error("authentication error: {0}")]
    Authentication(Box<ErrorBody>),

    /// 402 — insufficient credits.
    #[error("insufficient credits: {0}")]
    InsufficientCredits(Box<ErrorBody>),

    /// 403 — forbidden.
    #[error("forbidden: {0}")]
    Forbidden(Box<ErrorBody>),

    /// 404 — resource not found.
    #[error("not found: {0}")]
    NotFound(Box<ErrorBody>),

    /// 409 — resource conflict.
    #[error("conflict: {0}")]
    Conflict(Box<ErrorBody>),

    /// 422 — validation error.
    #[error("validation error: {0}")]
    Validation(Box<ErrorBody>),

    /// 429 — rate limited.
    #[error("rate limited: {0}")]
    RateLimit(Box<ErrorBody>),

    /// 5xx — server error.
    #[error("server error: {0}")]
    Server(Box<ErrorBody>),

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

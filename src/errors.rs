use reqwest::StatusCode;
use serde_json::Value;
use tokio_tungstenite::tungstenite;

/// Type alias for the result of client operations.
pub type ClientResult<T> = Result<T, ClientError>;

/// Errors that can occur during client operations.
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    /// Error that occurs when parsing a URL.
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    /// Error that occurs during a reqwest operation.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Error that occurs during a tungstenite operation.
    #[error(transparent)]
    Tungstenite(#[from] tungstenite::Error),

    /// Error that occurs during a serde_json operation.
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// Error that occurs when setting the websocket scheme.
    #[error("set websocket scheme failed")]
    SetWsScheme,

    /// Error that occurs during an API operation.
    #[error(transparent)]
    Api(#[from] ApiError),
}

/// Error that occurs during an API operation.
#[derive(thiserror::Error, Debug)]
#[error("api error")]
pub struct ApiError {
    /// The HTTP status code of the API response.
    pub status: StatusCode,
    /// The body of the API response.
    pub body: ApiBody,
}

/// The body of an API response.
#[derive(Debug)]
pub enum ApiBody {
    /// JSON body.
    Json(Value),
    /// Text body.
    Text(String),
}

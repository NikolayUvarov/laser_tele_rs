use std::fmt;

/// Result of the functions of the library
pub type Result<T> = std::result::Result<T, Error>;

/// Error of a request to Telegram
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Telegram refused the request
    Api(ApiError),
    /// No connection, a timeout... The URL with the token is removed from the error
    Network(reqwest::Error),
    /// A local file can't be read or written
    Io(std::io::Error),
    /// A response or parameters can't be converted from or to JSON
    Json(serde_json::Error),
    /// The token is not found in `Config::api_key`, the `TG_API_KEY` environment variable or the `.APIKEY` file
    NoToken,
    /// Wrong configuration, e.g. a wrong proxy URL
    Config(String),
}

/// Telegram refused the request, e.g. `error_code` 403 if the user blocked the bot
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// The Bot API method
    pub method: String,
    /// The code from Telegram, usually the HTTP status: 400, 403, 429...
    pub error_code: i64,
    /// The description from Telegram, e.g. "Bad Request: chat not found"
    pub description: String,
    /// For `error_code` 429 (too many requests): seconds to wait before the next request
    pub retry_after: i64,
    /// The group was upgraded to a supergroup with this ID
    pub migrate_to_chat_id: i64,
}

impl Error {
    /// The error of Telegram, if Telegram refused the request
    pub fn api(&self) -> Option<&ApiError> {
        match self {
            Error::Api(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "telegram {}: {} {}",
            self.method, self.error_code, self.description
        )
    }
}

impl std::error::Error for ApiError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Api(e) => e.fmt(f),
            Error::Network(e) => write!(f, "network error: {e}"),
            Error::Io(e) => write!(f, "file error: {e}"),
            Error::Json(e) => write!(f, "wrong JSON: {e}"),
            Error::NoToken => f.write_str(
                "the token is not set: set it in Config::api_key, the TG_API_KEY environment variable or the .APIKEY file",
            ),
            Error::Config(e) => write!(f, "wrong configuration: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Api(e) => Some(e),
            Error::Network(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::NoToken | Error::Config(_) => None,
        }
    }
}

impl From<ApiError> for Error {
    fn from(e: ApiError) -> Self {
        Error::Api(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        // the URL contains the token
        Error::Network(e.without_url())
    }
}

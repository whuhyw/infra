use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed")]
    Http(#[source] reqwest::Error),
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("database migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("JSON response was invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid SM2 public key: {0}")]
    PublicKey(String),
    #[error("WHU API error [{path}]: {message}")]
    Api { path: String, message: String },
    #[error("WHU sign-in failed: {0}")]
    SignIn(String),
    #[error("WHU API returned an invalid value: {0}")]
    InvalidData(String),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.without_url())
    }
}

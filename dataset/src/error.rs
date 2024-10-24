use thiserror;

/// Dataset errors
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String)
}

/// Dataset results
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "sql")]
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::Database(e.to_string())
    }
}
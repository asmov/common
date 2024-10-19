use thiserror;

/// Dataset errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String)
}

/// Dataset results
pub type Result<T> = std::result::Result<T, Error>;
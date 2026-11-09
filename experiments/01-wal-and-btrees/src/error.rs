use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Key not found")]
    KeyNotFound,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("WAL error: {0}")]
    WalError(String),

    #[error("B-tree error: {0}")]
    BTreeError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

//! Crate-local error type wrapping `rusqlite::Error` per UQ-WP1-06.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("connection-pool error: {0}")]
    Pool(String),

    #[error("migration {version} failed: {source}")]
    Migration {
        version: u32,
        #[source]
        source: rusqlite::Error,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("channel closed — writer actor has exited")]
    WriterGone,

    #[error("writer actor returned no response")]
    WriterNoResponse,
}

pub type Result<T> = std::result::Result<T, StorageError>;

//! clarion-storage — `SQLite` layer, writer-actor, reader pool.
//!
//! All mutations route through the writer actor (a single `tokio::task`
//! owning the sole write `rusqlite::Connection`). Readers come from a
//! `deadpool-sqlite` pool. See ADR-011.

pub mod error;
pub mod pragma;
pub mod reader;
pub mod schema;

pub use error::{Result, StorageError};
pub use reader::ReaderPool;

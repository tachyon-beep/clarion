//! Read-only connection pool wrapping `deadpool-sqlite` per ADR-011.
//!
//! Readers take a connection from the pool, run a query, and drop it. The
//! pool caps concurrent connections (default 16 per ADR-011 §Reader pool).
//! WAL mode lets readers see the committed snapshot at the moment they
//! open; writes become visible only after the next checkpoint or a fresh
//! connection.

use std::path::Path;

use deadpool_sqlite::{Config, Pool, Runtime};

use crate::error::Result;
use crate::pragma;

/// A read-only connection pool backed by `deadpool-sqlite`.
pub struct ReaderPool {
    pool: Pool,
}

impl ReaderPool {
    /// Open a pool against an existing `SQLite` file.
    ///
    /// The database file must already exist and already have migrations
    /// applied — callers should run [`crate::schema::apply_migrations`] on
    /// a write connection first.
    ///
    /// # Errors
    ///
    /// Returns [`crate::StorageError::PoolBuild`] if `deadpool-sqlite` cannot create
    /// the pool (typically because the path is not a valid `SQLite` file or
    /// the filesystem denies access).
    pub fn open(db_path: impl AsRef<Path>, max_size: usize) -> Result<Self> {
        let mut cfg = Config::new(db_path.as_ref());
        cfg.pool = Some(deadpool_sqlite::PoolConfig::new(max_size));
        let pool = cfg.create_pool(Runtime::Tokio1)?;
        Ok(Self { pool })
    }

    /// Acquire a reader and run a blocking closure on it.
    ///
    /// Read-side PRAGMAs are applied on every acquisition — cheap and
    /// guarantees `busy_timeout` + `foreign_keys` are always on.
    ///
    /// # Errors
    ///
    /// Returns [`crate::StorageError::Pool`] if the pool cannot acquire a
    /// connection, [`crate::StorageError::PoolInteract`] if the interact task
    /// panics or is aborted, or whatever the closure returns if the
    /// query itself fails.
    pub async fn with_reader<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let obj = self.pool.get().await?;
        obj.interact(move |conn| -> Result<T> {
            pragma::apply_read_pragmas(conn)?;
            f(conn)
        })
        .await?
    }
}

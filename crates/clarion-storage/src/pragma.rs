//! PRAGMAs applied at connection open per ADR-011 §`SQLite` PRAGMAs.

use rusqlite::Connection;

use crate::error::Result;

/// Apply the write-side PRAGMA set: WAL, `synchronous=NORMAL`, `busy_timeout`,
/// `wal_autocheckpoint`, `foreign_keys`. Called on the writer's connection once,
/// immediately after open.
///
/// # Errors
///
/// Returns [`crate::error::StorageError::Sqlite`] if any PRAGMA statement fails.
pub fn apply_write_pragmas(conn: &Connection) -> Result<()> {
    let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    debug_assert_eq!(mode.to_ascii_lowercase(), "wal", "WAL not enabled");
    conn.execute_batch(concat!(
        "PRAGMA synchronous = NORMAL;",
        "PRAGMA busy_timeout = 5000;",
        "PRAGMA wal_autocheckpoint = 1000;",
        "PRAGMA foreign_keys = ON;",
    ))?;
    Ok(())
}

/// Apply the read-side PRAGMA set: `busy_timeout` + `foreign_keys`. Readers do not
/// set `journal_mode` (WAL is a database-level mode set by the first writer).
///
/// # Errors
///
/// Returns [`crate::error::StorageError::Sqlite`] if any PRAGMA statement fails.
pub fn apply_read_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(concat!(
        "PRAGMA busy_timeout = 5000;",
        "PRAGMA foreign_keys = ON;",
    ))?;
    Ok(())
}

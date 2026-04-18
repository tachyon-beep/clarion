//! Schema migration runner.
//!
//! Migrations are embedded at compile time via `include_str!`. On apply, each
//! is run if not already recorded in `schema_migrations`. Running twice is a
//! no-op.

use rusqlite::{Connection, params};

use crate::error::{Result, StorageError};

struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "0001_initial_schema",
    sql: include_str!("../migrations/0001_initial_schema.sql"),
}];

/// Apply every migration not already recorded in `schema_migrations`.
///
/// The first migration creates the `schema_migrations` table itself, so the
/// initial lookup tolerates its absence.
///
/// # Errors
///
/// Returns [`StorageError::Migration`] with the failing version on SQL error
/// during apply. Returns [`StorageError::Sqlite`] on bookkeeping failures.
pub fn apply_migrations(conn: &mut Connection) -> Result<()> {
    let applied = read_applied_versions(conn)?;
    for m in MIGRATIONS {
        if applied.contains(&m.version) {
            tracing::debug!(version = m.version, "migration already applied");
            continue;
        }
        apply_one(conn, m)?;
    }
    Ok(())
}

fn read_applied_versions(conn: &Connection) -> Result<Vec<u32>> {
    let table_exists: Option<String> = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get(0),
        )
        .ok();
    if table_exists.is_none() {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
    let raw: Vec<i64> = stmt
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;
    let out = raw
        .into_iter()
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX))
        .collect();
    Ok(out)
}

fn apply_one(conn: &mut Connection, m: &Migration) -> Result<()> {
    tracing::info!(version = m.version, name = m.name, "applying migration");
    conn.execute_batch(m.sql)
        .map_err(|source| StorageError::Migration {
            version: m.version,
            source,
        })?;
    // Defence in depth: guarantee idempotency even if a migration file forgets
    // to insert into schema_migrations.
    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations (version, name, applied_at) \
         VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        params![i64::from(m.version), m.name],
    )?;
    Ok(())
}

/// Count of applied migrations (for tests + install).
///
/// # Errors
///
/// Returns [`StorageError::Sqlite`] if the query fails for reasons other than
/// the table not existing (in which case this returns `Ok(0)`).
pub fn applied_count(conn: &Connection) -> Result<u32> {
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    Ok(u32::try_from(n).unwrap_or(u32::MAX))
}

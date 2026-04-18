//! Reader-pool concurrency tests.

use std::sync::Arc;

use rusqlite::Connection;

use clarion_storage::{ReaderPool, pragma, schema};

fn prepared_db(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let path = dir.path().join("clarion.db");
    let mut conn = Connection::open(&path).expect("open");
    pragma::apply_write_pragmas(&conn).expect("write pragmas");
    schema::apply_migrations(&mut conn).expect("migrate");
    path
}

#[tokio::test]
async fn two_readers_run_concurrently() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    let pool = Arc::new(ReaderPool::open(&path, 2).expect("pool"));

    let p1 = pool.clone();
    let p2 = pool.clone();
    let (a, b) = tokio::join!(
        p1.with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT 1", [], |row| row.get(0))?;
            Ok(n)
        }),
        p2.with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT 2", [], |row| row.get(0))?;
            Ok(n)
        })
    );
    assert_eq!(a.unwrap(), 1);
    assert_eq!(b.unwrap(), 2);
}

#[tokio::test]
async fn reader_sees_committed_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);

    // Pre-seed a runs row via a one-shot blocking connection.
    {
        let conn = Connection::open(&path).unwrap();
        pragma::apply_write_pragmas(&conn).unwrap();
        conn.execute(
            "INSERT INTO runs (id, started_at, completed_at, config, stats, status) \
             VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), NULL, '{}', '{}', 'running')",
            rusqlite::params!["run-1"],
        )
        .unwrap();
    }

    let pool = ReaderPool::open(&path, 2).expect("pool");
    let status: String = pool
        .with_reader(|conn| {
            let status: String =
                conn.query_row("SELECT status FROM runs WHERE id = 'run-1'", [], |row| {
                    row.get(0)
                })?;
            Ok(status)
        })
        .await
        .unwrap();
    assert_eq!(status, "running");
}

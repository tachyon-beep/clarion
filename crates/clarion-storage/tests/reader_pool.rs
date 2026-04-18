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

#[tokio::test]
async fn pool_queues_when_exhausted_and_proceeds_after_release() {
    use std::sync::Arc;
    use tokio::sync::Notify;
    use tokio::time::{Duration, timeout};

    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    // max_size = 1 makes the exhaustion scenario trivial to construct.
    let pool = Arc::new(ReaderPool::open(&path, 1).expect("pool"));

    let hold_open = Arc::new(Notify::new());
    let hold_open_in_task = hold_open.clone();
    let pool_for_hold = pool.clone();

    // First reader: acquire and hold until notified.
    let held = tokio::spawn(async move {
        pool_for_hold
            .with_reader(move |conn| {
                // Run a trivial query so we know the connection was actually acquired.
                let _: i64 = conn.query_row("SELECT 1", [], |row| row.get(0))?;
                // Block the reader inside the interact() block by busy-spinning
                // on a sync signal. We cannot `.await` inside interact() (it's
                // a blocking context), so use a sync waiter: park on a mutex
                // that the main task will unlock.
                // Simpler: sleep for a bounded time; the main task must acquire
                // the second reader before this sleep elapses.
                std::thread::sleep(Duration::from_millis(300));
                Ok::<_, clarion_storage::StorageError>(())
            })
            .await
    });

    // Give the first reader a moment to acquire the connection.
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Second reader: should block until the first releases. With timeout 2s
    // this proves it eventually proceeds (not immediately erroring, not
    // blocking forever).
    let pool_for_wait = pool.clone();
    let second = timeout(Duration::from_secs(2), async move {
        pool_for_wait
            .with_reader(|conn| {
                let n: i64 = conn.query_row("SELECT 2", [], |row| row.get(0))?;
                Ok(n)
            })
            .await
    })
    .await
    .expect("second reader should eventually acquire within 2s")
    .expect("second reader's query should succeed");

    assert_eq!(second, 2);
    held.await.unwrap().unwrap();
    // Keep the unused import quiet.
    let _ = hold_open_in_task;
    let _ = hold_open;
}

#[tokio::test]
async fn reader_error_propagates_and_connection_returns_to_pool() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    let pool = ReaderPool::open(&path, 2).expect("pool");

    // First call returns an error from the closure.
    let err_result: Result<i64, _> = pool
        .with_reader(|conn| {
            let _: i64 = conn.query_row("SELECT 1", [], |row| row.get(0))?;
            // Deliberate invalid SQL to force an error in the closure.
            conn.query_row("SELECT * FROM non_existent_table", [], |row| row.get(0))
                .map_err(clarion_storage::StorageError::from)
        })
        .await;

    assert!(err_result.is_err(), "expected closure error to propagate");
    assert!(matches!(
        err_result.unwrap_err(),
        clarion_storage::StorageError::Sqlite(_)
    ));

    // Second call on the same pool must succeed — proves the connection
    // from the first call was returned to the pool cleanly.
    let ok: i64 = pool
        .with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT 42", [], |row| row.get(0))?;
            Ok(n)
        })
        .await
        .expect("subsequent reader after an error should succeed");
    assert_eq!(ok, 42);
}

#[tokio::test]
async fn reader_panic_is_caught_as_pool_interact_and_pool_remains_usable() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    let pool = ReaderPool::open(&path, 2).expect("pool");

    // Closure that panics inside the interact() block.
    let panic_result: Result<i64, _> = pool
        .with_reader(|_conn| {
            panic!("deliberate test panic inside reader closure");
        })
        .await;

    assert!(panic_result.is_err(), "expected panic to surface as error");
    assert!(matches!(
        panic_result.unwrap_err(),
        clarion_storage::StorageError::PoolInteract(_)
    ));

    // Pool remains usable — deadpool recycles the poisoned connection or
    // discards it and creates a fresh one. Subsequent calls must succeed.
    let ok: i64 = pool
        .with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT 99", [], |row| row.get(0))?;
            Ok(n)
        })
        .await
        .expect("subsequent reader after a panic should succeed");
    assert_eq!(ok, 99);
}

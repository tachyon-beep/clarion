//! Writer-actor integration tests.
//!
//! Covers: round-trip insert, per-N-batch commit cadence, `FailRun` rollback.

use std::sync::atomic::Ordering;

use rusqlite::Connection;
use tokio::sync::oneshot;

use clarion_storage::{
    ReaderPool, Writer,
    commands::{EntityRecord, RunStatus, WriterCmd},
    pragma, schema,
};

fn prepared_db(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let path = dir.path().join("clarion.db");
    let mut conn = Connection::open(&path).unwrap();
    pragma::apply_write_pragmas(&conn).unwrap();
    schema::apply_migrations(&mut conn).unwrap();
    path
}

fn now_iso() -> String {
    "2026-04-18T00:00:00.000Z".to_owned()
}

fn make_entity(id: &str) -> EntityRecord {
    EntityRecord {
        id: id.to_owned(),
        plugin_id: "python".to_owned(),
        kind: "function".to_owned(),
        name: "demo.hello".to_owned(),
        short_name: "hello".to_owned(),
        parent_id: None,
        source_file_id: None,
        source_byte_start: None,
        source_byte_end: None,
        source_line_start: None,
        source_line_end: None,
        properties_json: "{}".to_owned(),
        content_hash: None,
        summary_json: None,
        wardline_json: None,
        first_seen_commit: None,
        last_seen_commit: None,
        created_at: now_iso(),
        updated_at: now_iso(),
    }
}

async fn send<T>(
    tx: &tokio::sync::mpsc::Sender<WriterCmd>,
    build: impl FnOnce(oneshot::Sender<Result<T, clarion_storage::StorageError>>) -> WriterCmd,
) -> Result<T, clarion_storage::StorageError> {
    let (ack_tx, ack_rx) = oneshot::channel();
    tx.send(build(ack_tx)).await.unwrap();
    ack_rx.await.unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn round_trip_insert_persists_entity() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    let (writer, handle) = Writer::spawn(path.clone(), 50, 256).unwrap();
    let tx = writer.sender();

    send::<()>(&tx, |ack| WriterCmd::BeginRun {
        run_id: "run-1".into(),
        config_json: "{}".into(),
        started_at: now_iso(),
        ack,
    })
    .await
    .unwrap();

    send::<()>(&tx, |ack| WriterCmd::InsertEntity {
        entity: Box::new(make_entity("python:function:demo.hello")),
        ack,
    })
    .await
    .unwrap();

    send::<()>(&tx, |ack| WriterCmd::CommitRun {
        run_id: "run-1".into(),
        status: RunStatus::Completed,
        completed_at: now_iso(),
        stats_json: "{}".into(),
        ack,
    })
    .await
    .unwrap();

    drop(tx);
    drop(writer);
    handle.await.unwrap().unwrap();

    let pool = ReaderPool::open(&path, 2).unwrap();
    let count: i64 = pool
        .with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;
            Ok(n)
        })
        .await
        .unwrap();
    assert_eq!(count, 1);

    let kind: String = pool
        .with_reader(|conn| {
            let k: String = conn.query_row(
                "SELECT kind FROM entities WHERE id = ?1",
                rusqlite::params!["python:function:demo.hello"],
                |row| row.get(0),
            )?;
            Ok(k)
        })
        .await
        .unwrap();
    assert_eq!(kind, "function");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn batch_size_fifty_commits_every_fifty_inserts() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    let (writer, handle) = Writer::spawn(path.clone(), 50, 256).unwrap();
    let tx = writer.sender();

    send::<()>(&tx, |ack| WriterCmd::BeginRun {
        run_id: "run-1".into(),
        config_json: "{}".into(),
        started_at: now_iso(),
        ack,
    })
    .await
    .unwrap();

    for i in 0..150 {
        let id = format!("python:function:demo.f{i:03}");
        send::<()>(&tx, |ack| WriterCmd::InsertEntity {
            entity: Box::new(make_entity(&id)),
            ack,
        })
        .await
        .unwrap();
    }

    assert_eq!(writer.commits_observed.load(Ordering::Relaxed), 3);

    send::<()>(&tx, |ack| WriterCmd::CommitRun {
        run_id: "run-1".into(),
        status: RunStatus::Completed,
        completed_at: now_iso(),
        stats_json: "{}".into(),
        ack,
    })
    .await
    .unwrap();

    assert_eq!(writer.commits_observed.load(Ordering::Relaxed), 4);

    drop(tx);
    drop(writer);
    handle.await.unwrap().unwrap();

    let pool = ReaderPool::open(&path, 2).unwrap();
    let count: i64 = pool
        .with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;
            Ok(n)
        })
        .await
        .unwrap();
    assert_eq!(count, 150);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fail_run_rolls_back_pending_inserts() {
    let dir = tempfile::tempdir().unwrap();
    let path = prepared_db(&dir);
    let (writer, handle) = Writer::spawn(path.clone(), 50, 256).unwrap();
    let tx = writer.sender();

    send::<()>(&tx, |ack| WriterCmd::BeginRun {
        run_id: "run-fail".into(),
        config_json: "{}".into(),
        started_at: now_iso(),
        ack,
    })
    .await
    .unwrap();

    for i in 0..10 {
        let id = format!("python:function:demo.g{i:03}");
        send::<()>(&tx, |ack| WriterCmd::InsertEntity {
            entity: Box::new(make_entity(&id)),
            ack,
        })
        .await
        .unwrap();
    }

    send::<()>(&tx, |ack| WriterCmd::FailRun {
        run_id: "run-fail".into(),
        reason: "deliberate test failure".into(),
        completed_at: now_iso(),
        ack,
    })
    .await
    .unwrap();

    drop(tx);
    drop(writer);
    handle.await.unwrap().unwrap();

    let pool = ReaderPool::open(&path, 2).unwrap();
    let entity_count: i64 = pool
        .with_reader(|conn| {
            let n: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;
            Ok(n)
        })
        .await
        .unwrap();
    assert_eq!(entity_count, 0, "FailRun did not roll back inserts");

    let status: String = pool
        .with_reader(|conn| {
            let s: String =
                conn.query_row("SELECT status FROM runs WHERE id = 'run-fail'", [], |row| {
                    row.get(0)
                })?;
            Ok(s)
        })
        .await
        .unwrap();
    assert_eq!(status, "failed");
}

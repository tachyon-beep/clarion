//! Writer-actor implementation (L3 lock-in) per ADR-011.
//!
//! The actor owns the sole write `rusqlite::Connection`. Callers submit
//! commands via `Writer::sender()`. The actor loop pulls one command at a
//! time, applies the mutation inside an implicit transaction bound to the
//! current run, and commits every `batch_size` entity inserts (the
//! "per-N-files" transaction pattern, default N=50 per ADR-011).
//!
//! UQ-WP1-03 resolution: the `commits_observed` [`std::sync::Arc`]`<`[`std::sync::atomic::AtomicUsize`]`>` is
//! incremented on every `COMMIT` issued by the actor. Tests read it to
//! verify batch-boundary commits fire at the expected cadence. It is
//! present in release builds as a no-op counter; no `#[cfg(test)]` gating
//! is used.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rusqlite::{Connection, params};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::commands::{Ack, EntityRecord, RunStatus, WriterCmd};
use crate::error::{Result, StorageError};
use crate::pragma;

/// Default transaction batch size per ADR-011.
pub const DEFAULT_BATCH_SIZE: usize = 50;

/// Default `mpsc` channel capacity per ADR-011.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 256;

pub struct Writer {
    tx: mpsc::Sender<WriterCmd>,
    /// Count of every `COMMIT` statement issued by the actor.
    ///
    /// Includes both per-batch boundary commits (every `batch_size` inserts)
    /// and the final commit issued by `CommitRun`. Intended for test
    /// assertions and diagnostic counters; not a measure of completed runs.
    ///
    /// Read this field before dropping the [`Writer`]: the actor holds its
    /// own `Arc` clone that lives until the `JoinHandle` resolves.
    pub commits_observed: Arc<AtomicUsize>,
}

impl Writer {
    /// Spawn the writer-actor on the current tokio runtime.
    ///
    /// Returns the `Writer` handle and the [`JoinHandle`] of the actor task.
    /// Callers await the [`JoinHandle`] at shutdown to ensure the actor has
    /// flushed any pending commit.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Sqlite`] if the `rusqlite::Connection` cannot
    /// be opened, or [`StorageError::PragmaInvariant`] if write PRAGMAs fail.
    pub fn spawn(
        db_path: std::path::PathBuf,
        batch_size: usize,
        channel_capacity: usize,
    ) -> Result<(Self, JoinHandle<Result<()>>)> {
        let (tx, rx) = mpsc::channel(channel_capacity);
        let commits_observed = Arc::new(AtomicUsize::new(0));
        let commits_for_actor = commits_observed.clone();
        let handle = tokio::task::spawn_blocking(move || -> Result<()> {
            let mut conn = Connection::open(&db_path)?;
            pragma::apply_write_pragmas(&conn)?;
            run_actor(rx, &mut conn, batch_size, &commits_for_actor);
            Ok(())
        });
        Ok((
            Writer {
                tx,
                commits_observed,
            },
            handle,
        ))
    }

    pub fn sender(&self) -> mpsc::Sender<WriterCmd> {
        self.tx.clone()
    }

    /// Convenience: send a command and await its ack.
    ///
    /// Intended for use by `clarion analyze` (Task 7) and later WP
    /// consumers; Sprint 1 integration tests use a local test helper
    /// rather than this method. Kept as part of the L3 lock-in surface
    /// so callers have a stable entry point when they arrive.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::WriterGone`] if the actor has exited and the
    /// channel is closed. Returns [`StorageError::WriterNoResponse`] if the
    /// actor dropped the `oneshot` sender without replying. Otherwise
    /// propagates whatever error the actor returned for the command.
    pub async fn send_wait<T, F>(&self, build: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> WriterCmd,
        T: 'static,
    {
        let (tx, rx) = oneshot::channel();
        let cmd = build(tx);
        self.tx
            .send(cmd)
            .await
            .map_err(|_| StorageError::WriterGone)?;
        rx.await.map_err(|_| StorageError::WriterNoResponse)?
    }
}

fn run_actor(
    mut rx: mpsc::Receiver<WriterCmd>,
    conn: &mut Connection,
    batch_size: usize,
    commits_observed: &AtomicUsize,
) {
    let mut state = ActorState::new(batch_size);

    while let Some(cmd) = rx.blocking_recv() {
        match cmd {
            WriterCmd::BeginRun {
                run_id,
                config_json,
                started_at,
                ack,
            } => {
                reply(
                    ack,
                    begin_run(conn, &mut state, &run_id, &config_json, &started_at),
                );
            }
            WriterCmd::InsertEntity { entity, ack } => {
                let res = insert_entity(conn, &mut state, &entity, commits_observed);
                reply(ack, res);
            }
            WriterCmd::CommitRun {
                run_id,
                status,
                completed_at,
                stats_json,
                ack,
            } => {
                let res = commit_run(
                    conn,
                    &mut state,
                    &run_id,
                    status,
                    &completed_at,
                    &stats_json,
                    commits_observed,
                );
                reply(ack, res);
            }
            WriterCmd::FailRun {
                run_id,
                reason,
                completed_at,
                ack,
            } => {
                let res = fail_run(conn, &mut state, &run_id, &reason, &completed_at);
                reply(ack, res);
            }
        }
    }
    // Channel closed. Best-effort flush.
    if state.in_tx {
        let _ = conn.execute_batch("ROLLBACK");
    }
}

fn reply<T>(ack: Ack<T>, result: Result<T>) {
    // If the caller dropped the receiver, we discard the result. This is
    // correct behaviour — the writer is still responsible for its own
    // durability, and the caller chose to stop caring.
    let _ = ack.send(result);
}

struct ActorState {
    batch_size: usize,
    /// Inserts accumulated in the current transaction.
    inserts_in_batch: usize,
    /// True if `BEGIN` has been issued and no `COMMIT`/`ROLLBACK` has fired.
    in_tx: bool,
    /// The run currently in progress, if any.
    current_run: Option<String>,
}

impl ActorState {
    fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            inserts_in_batch: 0,
            in_tx: false,
            current_run: None,
        }
    }
}

fn begin_run(
    conn: &mut Connection,
    state: &mut ActorState,
    run_id: &str,
    config_json: &str,
    started_at: &str,
) -> Result<()> {
    if state.current_run.is_some() {
        return Err(StorageError::WriterProtocol(
            "BeginRun received while a run is already in progress".to_owned(),
        ));
    }
    conn.execute(
        "INSERT INTO runs (id, started_at, completed_at, config, stats, status) \
         VALUES (?1, ?2, NULL, ?3, '{}', 'running')",
        params![run_id, started_at, config_json],
    )?;
    conn.execute_batch("BEGIN")?;
    state.in_tx = true;
    state.inserts_in_batch = 0;
    state.current_run = Some(run_id.to_owned());
    Ok(())
}

fn insert_entity(
    conn: &mut Connection,
    state: &mut ActorState,
    entity: &EntityRecord,
    commits_observed: &AtomicUsize,
) -> Result<()> {
    if state.current_run.is_none() {
        return Err(StorageError::WriterProtocol(
            "InsertEntity received without a preceding BeginRun".to_owned(),
        ));
    }
    if !state.in_tx {
        conn.execute_batch("BEGIN")?;
        state.in_tx = true;
    }
    conn.execute(
        "INSERT INTO entities ( \
            id, plugin_id, kind, name, short_name, \
            parent_id, source_file_id, \
            source_byte_start, source_byte_end, \
            source_line_start, source_line_end, \
            properties, content_hash, summary, wardline, \
            first_seen_commit, last_seen_commit, \
            created_at, updated_at \
         ) VALUES ( \
            ?1, ?2, ?3, ?4, ?5, \
            ?6, ?7, \
            ?8, ?9, \
            ?10, ?11, \
            ?12, ?13, ?14, ?15, \
            ?16, ?17, \
            ?18, ?19 \
         )",
        params![
            entity.id,
            entity.plugin_id,
            entity.kind,
            entity.name,
            entity.short_name,
            entity.parent_id,
            entity.source_file_id,
            entity.source_byte_start,
            entity.source_byte_end,
            entity.source_line_start,
            entity.source_line_end,
            entity.properties_json,
            entity.content_hash,
            entity.summary_json,
            entity.wardline_json,
            entity.first_seen_commit,
            entity.last_seen_commit,
            entity.created_at,
            entity.updated_at,
        ],
    )?;
    state.inserts_in_batch += 1;
    if state.inserts_in_batch >= state.batch_size {
        // State transitions BEFORE the fallible COMMIT: SQLite aborts the
        // transaction on COMMIT failure regardless, so setting in_tx=false
        // first keeps our state conservatively correct if the COMMIT errors.
        state.inserts_in_batch = 0;
        state.in_tx = false;
        conn.execute_batch("COMMIT")?;
        commits_observed.fetch_add(1, Ordering::Relaxed);
        // Open the next batch eagerly so the next insert doesn't pay
        // another `BEGIN` round-trip.
        conn.execute_batch("BEGIN")?;
        state.in_tx = true;
    }
    Ok(())
}

fn commit_run(
    conn: &mut Connection,
    state: &mut ActorState,
    run_id: &str,
    status: RunStatus,
    completed_at: &str,
    stats_json: &str,
    commits_observed: &AtomicUsize,
) -> Result<()> {
    if state.in_tx {
        state.in_tx = false;
        conn.execute_batch("COMMIT")?;
        commits_observed.fetch_add(1, Ordering::Relaxed);
    }
    conn.execute(
        "UPDATE runs SET status = ?1, completed_at = ?2, stats = ?3 WHERE id = ?4",
        params![status.as_str(), completed_at, stats_json, run_id],
    )?;
    state.current_run = None;
    state.inserts_in_batch = 0;
    Ok(())
}

fn fail_run(
    conn: &mut Connection,
    state: &mut ActorState,
    run_id: &str,
    reason: &str,
    completed_at: &str,
) -> Result<()> {
    if state.in_tx {
        let _ = conn.execute_batch("ROLLBACK");
        state.in_tx = false;
    }
    let stats_json = serde_json::json!({ "failure_reason": reason }).to_string();
    conn.execute(
        "UPDATE runs SET status = 'failed', completed_at = ?1, stats = ?2 WHERE id = ?3",
        params![completed_at, stats_json, run_id],
    )?;
    state.current_run = None;
    state.inserts_in_batch = 0;
    Ok(())
}

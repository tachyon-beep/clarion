//! `clarion analyze` — discover plugins, walk the source tree, persist entities.
//!
//! WP2 Task 8 replaces the Sprint-1 stub with real plugin orchestration:
//! - Discover plugins via L9 `$PATH` convention (Task 5).
//! - For each plugin: spawn, handshake, walk the source tree, call
//!   `analyze_file` for every matching file, persist via writer-actor.
//! - Pattern A buffering: collect entities in the blocking task, flush
//!   `InsertEntity` commands from async context after the blocking task returns.
//! - On unrecoverable error (cap, escape, spawn, transport) → `FailRun`.
//! - Zero successful plugins discovered → `SkippedNoPlugins` (existing path).

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use uuid::Uuid;

use clarion_core::{AcceptedEntity, DiscoveredPlugin, HostError, HostFinding, discover};
use clarion_storage::{
    DEFAULT_BATCH_SIZE, DEFAULT_CHANNEL_CAPACITY, Writer,
    commands::{EntityRecord, RunStatus, WriterCmd},
};

// ── Public entry point ────────────────────────────────────────────────────────

/// Run the analyze command against `project_path`.
///
/// # Errors
///
/// Returns an error if the target directory does not exist, has no `.clarion/`
/// directory, or if the writer actor fails to start or process commands.
#[allow(clippy::too_many_lines)]
pub async fn run(project_path: PathBuf) -> Result<()> {
    if !project_path.exists() {
        bail!(
            "target directory does not exist: {}. Pass a valid path or cd to it first.",
            project_path.display()
        );
    }
    let project_root = project_path
        .canonicalize()
        .with_context(|| format!("cannot canonicalise path {}", project_path.display()))?;
    let clarion_dir = project_root.join(".clarion");
    if !clarion_dir.exists() {
        bail!(
            "{} has no .clarion/ directory. Run `clarion install` first.",
            project_root.display()
        );
    }
    let db_path = clarion_dir.join("clarion.db");

    // ── Writer actor ──────────────────────────────────────────────────────────
    let (writer, handle) = Writer::spawn(db_path, DEFAULT_BATCH_SIZE, DEFAULT_CHANNEL_CAPACITY)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("spawn writer actor")?;
    let run_id = Uuid::new_v4().to_string();
    let started_at = iso8601_now();

    writer
        .send_wait(|ack| WriterCmd::BeginRun {
            run_id: run_id.clone(),
            config_json: "{}".into(),
            started_at: started_at.clone(),
            ack,
        })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("BeginRun")?;

    // ── Discover plugins ──────────────────────────────────────────────────────
    let discovery_results = discover();
    let mut plugins: Vec<DiscoveredPlugin> = Vec::new();
    let mut discovery_errors: Vec<String> = Vec::new();
    for result in discovery_results {
        match result {
            Ok(p) => {
                tracing::info!(
                    plugin_id = %p.manifest.plugin.plugin_id,
                    executable = %p.executable.display(),
                    "discovered plugin"
                );
                plugins.push(p);
            }
            Err(e) => {
                let msg = e.to_string();
                tracing::warn!(error = %msg, "skipping plugin: discovery error");
                discovery_errors.push(msg);
            }
        }
    }

    if plugins.is_empty() {
        // Distinguish "no plugins installed" (SkippedNoPlugins — expected on a
        // bare machine) from "plugins present but all failed discovery" (FailRun
        // — a real configuration error the operator must see). Reporting the
        // latter as `skipped_no_plugins` hides bugs.
        if !discovery_errors.is_empty() {
            let reason = format!(
                "all {} discovered plugin manifest(s) failed to parse: {}",
                discovery_errors.len(),
                discovery_errors.join("; ")
            );
            tracing::error!(run_id = %run_id, reason = %reason, "failing run: discovery errors");
            let completed_at = iso8601_now();
            writer
                .send_wait(|ack| WriterCmd::FailRun {
                    run_id: run_id.clone(),
                    reason: reason.clone(),
                    completed_at,
                    ack,
                })
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("FailRun(discovery errors)")?;

            drop(writer);
            handle
                .await
                .map_err(|e| anyhow::anyhow!("writer actor panic: {e}"))?
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Non-zero exit. Printing to stdout + returning Ok(()) here
            // hides the failure from `clarion analyze && do_next` chains
            // and breaks CI gating that reads `$?`. The run row in the DB
            // is already marked `failed` above.
            bail!("analyze run {run_id} failed — {reason}");
        }

        tracing::warn!(run_id = %run_id, "no plugins discovered");
        let completed_at = iso8601_now();
        writer
            .send_wait(|ack| WriterCmd::CommitRun {
                run_id: run_id.clone(),
                status: RunStatus::SkippedNoPlugins,
                completed_at: completed_at.clone(),
                stats_json: r#"{"entities_inserted":0}"#.into(),
                ack,
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("CommitRun(SkippedNoPlugins)")?;

        drop(writer);
        handle
            .await
            .map_err(|e| anyhow::anyhow!("writer actor panic: {e}"))?
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        println!("analyze complete: run {run_id} skipped_no_plugins");
        return Ok(());
    }

    // ── Build extension union for the tree walk ───────────────────────────────
    let mut wanted_extensions: BTreeSet<String> = BTreeSet::new();
    for p in &plugins {
        for ext in &p.manifest.plugin.extensions {
            wanted_extensions.insert(ext.to_ascii_lowercase());
        }
    }

    // ── Walk the source tree (once, union of all extensions) ─────────────────
    let source_files = collect_source_files(&project_root, &wanted_extensions)
        .with_context(|| format!("walking source tree at {}", project_root.display()))?;
    tracing::info!(file_count = source_files.len(), "source tree walk complete");

    // ── Per-plugin processing ─────────────────────────────────────────────────
    let mut total_entity_count: u64 = 0;
    let mut run_outcome: RunOutcome = RunOutcome::Completed;

    'plugins: for plugin in plugins {
        let plugin_id = plugin.manifest.plugin.plugin_id.clone();
        let plugin_extensions: BTreeSet<String> = plugin
            .manifest
            .plugin
            .extensions
            .iter()
            .map(|e| e.to_ascii_lowercase())
            .collect();

        // Filter source files to this plugin's extensions.
        let plugin_files: Vec<PathBuf> = source_files
            .iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| plugin_extensions.contains(&e.to_ascii_lowercase()))
            })
            .cloned()
            .collect();

        if plugin_files.is_empty() {
            tracing::info!(plugin_id = %plugin_id, "no files match plugin extensions; skipping");
            continue;
        }

        tracing::info!(
            plugin_id = %plugin_id,
            file_count = plugin_files.len(),
            "processing plugin"
        );

        // Run the blocking plugin work on the tokio threadpool.
        // Pattern A: collect all entities into memory, return to async side.
        let manifest = plugin.manifest.clone();
        let project_root_clone = project_root.clone();
        let pid_clone = plugin_id.clone();
        let files_clone = plugin_files.clone();

        let spawn_result = tokio::task::spawn_blocking(move || {
            run_plugin_blocking(manifest, &project_root_clone, &pid_clone, &files_clone)
        })
        .await
        .map_err(|e| anyhow::anyhow!("plugin task panicked: {e}"))?;

        match spawn_result {
            Err(reason) => {
                run_outcome = RunOutcome::Failed { reason };
                break 'plugins;
            }
            Ok(BatchResult { entities, findings }) => {
                // Log findings individually (Tier B persistence is future
                // work). Logging only the count leaves operators guessing
                // whether the plugin tripped an ontology check, emitted
                // malformed JSON, or hit a path-jail violation.
                if !findings.is_empty() {
                    tracing::warn!(
                        plugin_id = %plugin_id,
                        finding_count = findings.len(),
                        "plugin host collected findings"
                    );
                    for f in &findings {
                        tracing::warn!(
                            plugin_id = %plugin_id,
                            subcode = %f.subcode,
                            message = %f.message,
                            metadata = ?f.metadata,
                            "plugin host finding",
                        );
                    }
                }

                // Persist entities via writer-actor (async side).
                //
                // A writer-actor error here (e.g. unique-key constraint, disk full)
                // must NOT short-circuit `run()` via `?` — that would bypass the
                // CommitRun/FailRun block below and leave `runs.status = 'running'`
                // permanently. Instead we convert the error to a terminal
                // `RunOutcome::Failed` so the FailRun path marks the run.
                let count = entities.len() as u64;
                let mut insert_err: Option<anyhow::Error> = None;
                for (id_str, record) in entities {
                    let res = writer
                        .send_wait(|ack| WriterCmd::InsertEntity {
                            entity: Box::new(record),
                            ack,
                        })
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .with_context(|| format!("InsertEntity for {id_str}"));
                    if let Err(e) = res {
                        insert_err = Some(e);
                        break;
                    }
                }
                if let Some(e) = insert_err {
                    tracing::error!(
                        plugin_id = %plugin_id,
                        error = %e,
                        "writer-actor rejected InsertEntity; failing run"
                    );
                    run_outcome = RunOutcome::Failed {
                        reason: format!("{e:#}"),
                    };
                    break 'plugins;
                }
                total_entity_count += count;
                tracing::info!(plugin_id = %plugin_id, entity_count = count, "plugin complete");
            }
        }
    }

    // ── Commit or fail the run ────────────────────────────────────────────────
    let completed_at = iso8601_now();
    // Extract the failure reason (if any) before the match consumes run_outcome.
    let fail_reason: Option<String> = match &run_outcome {
        RunOutcome::Failed { reason } => Some(reason.clone()),
        RunOutcome::Completed => None,
    };

    match run_outcome {
        RunOutcome::Completed => {
            let stats_json = format!(r#"{{"entities_inserted":{total_entity_count}}}"#);
            writer
                .send_wait(|ack| WriterCmd::CommitRun {
                    run_id: run_id.clone(),
                    status: RunStatus::Completed,
                    completed_at,
                    stats_json,
                    ack,
                })
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("CommitRun(Completed)")?;
        }
        RunOutcome::Failed { reason } => {
            writer
                .send_wait(|ack| WriterCmd::FailRun {
                    run_id: run_id.clone(),
                    reason,
                    completed_at,
                    ack,
                })
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("FailRun")?;
        }
    }

    drop(writer);
    handle
        .await
        .map_err(|e| anyhow::anyhow!("writer actor panic: {e}"))?
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // On FailRun: bail so the process exits non-zero. The run row is
    // already marked `failed` in the DB by the FailRun branch above; this
    // is purely about surfacing failure to the operator's shell / CI.
    if let Some(reason) = fail_reason {
        bail!("analyze run {run_id} failed — {reason}");
    }

    println!("analyze complete: run {run_id} completed ({total_entity_count} entities)");
    Ok(())
}

// ── Run-outcome ───────────────────────────────────────────────────────────────

#[derive(Debug)]
enum RunOutcome {
    Completed,
    Failed { reason: String },
}

// ── Blocking plugin worker ────────────────────────────────────────────────────

/// Returned from the blocking plugin task on success.
struct BatchResult {
    /// `(entity_id_string, record)` pairs for every accepted entity.
    entities: Vec<(String, EntityRecord)>,
    /// Findings accumulated by the host during the session.
    findings: Vec<clarion_core::HostFinding>,
}

/// Spawn the plugin, handshake, run `analyze_file` for each file, collect results.
///
/// All I/O is synchronous — this is designed to run inside `spawn_blocking`.
/// On unrecoverable error, returns `Err(reason_string)`.
///
/// Regardless of success or failure the child process is always reaped: on
/// the happy path via `host.shutdown()` + `child.wait()`, on the error path
/// via `child.kill()` + `child.wait()`. `std::process::Child::Drop` does NOT
/// kill or reap on Unix, so discarding `child` without `wait()` would leak a
/// zombie into the kernel process table per spawn.
fn run_plugin_blocking(
    manifest: clarion_core::Manifest,
    project_root: &Path,
    plugin_id: &str,
    files: &[PathBuf],
) -> Result<BatchResult, String> {
    use clarion_core::PluginHost;

    let (mut host, mut child) = PluginHost::spawn(manifest, project_root).map_err(|e| match e {
        HostError::Spawn(msg) => format!("failed to spawn plugin {plugin_id}: {msg}"),
        HostError::Handshake(ref me) => {
            format!("plugin {plugin_id} refused handshake: {me}")
        }
        other => format!("plugin {plugin_id} spawn/handshake error: {other}"),
    })?;

    let work_result: Result<Vec<(String, EntityRecord)>, String> = (|| {
        let mut collected: Vec<(String, EntityRecord)> = Vec::new();
        for file in files {
            let entities: Vec<AcceptedEntity> = host
                .analyze_file(file)
                .map_err(|e| classify_host_error(plugin_id, e))?;
            for entity in entities {
                let id_str = entity.id.to_string();
                let record = map_entity_to_record(&entity, plugin_id);
                collected.push((id_str, record));
            }
        }
        Ok(collected)
    })();

    // Try a graceful shutdown on the happy path; on error, skip straight to
    // kill — the plugin's behaviour is already untrusted. `analyze_file`
    // already issues `shutdown`/`exit` before returning PathEscapeBreaker or
    // EntityCap errors, so calling `host.shutdown()` again there would write
    // to a closed pipe; that's why we only call it on Ok.
    if work_result.is_ok() {
        if let Err(e) = host.shutdown() {
            tracing::warn!(
                plugin_id = %plugin_id,
                error = %e,
                "best-effort host shutdown failed; falling back to kill()",
            );
            let _ = child.kill();
        }
    } else {
        let _ = child.kill();
    }

    let mut findings = host.take_findings();

    // Reap unconditionally. `Child::Drop` does not wait on Unix.
    reap_and_classify_exit(&mut child, plugin_id, &mut findings);

    match work_result {
        Ok(collected) => Ok(BatchResult {
            entities: collected,
            findings,
        }),
        Err(reason) => Err(reason),
    }
}

/// Wait on the child, inspect its exit status, and append an OOM finding if
/// the signal is consistent with `RLIMIT_AS` enforcement (ADR-021 §2d).
///
/// Linux kernel behaviour on `RLIMIT_AS` violation varies: typical signatures
/// are SIGKILL (OOM-killer path) and SIGSEGV (map/allocation failure that the
/// plugin did not handle). Both are treated as likely memory-limit events.
/// Other signals or non-zero exit codes get a warn log but no finding — the
/// cause is ambiguous without more bookkeeping.
fn reap_and_classify_exit(
    child: &mut std::process::Child,
    plugin_id: &str,
    findings: &mut Vec<HostFinding>,
) {
    match child.wait() {
        Ok(status) if !status.success() => {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(signal) = status.signal() {
                    tracing::warn!(
                        plugin_id = %plugin_id,
                        signal,
                        "plugin terminated by signal",
                    );
                    // SIGKILL (9) and SIGSEGV (11) are the observed signatures
                    // of an RLIMIT_AS kill in Sprint-1 testing.
                    if signal == 9 || signal == 11 {
                        findings.push(HostFinding::oom_killed(plugin_id, signal));
                    }
                } else if let Some(code) = status.code() {
                    tracing::warn!(
                        plugin_id = %plugin_id,
                        code,
                        "plugin exited non-zero",
                    );
                }
            }
            #[cfg(not(unix))]
            {
                tracing::warn!(
                    plugin_id = %plugin_id,
                    "plugin exited non-successfully (exit-status inspection is Unix-only)",
                );
            }
        }
        Ok(_) => {} // clean exit
        Err(e) => {
            tracing::warn!(
                plugin_id = %plugin_id,
                error = %e,
                "failed to wait on plugin child",
            );
        }
    }
}

/// Map a `HostError` from `analyze_file` to a human-readable fail-run reason.
fn classify_host_error(plugin_id: &str, e: HostError) -> String {
    match e {
        HostError::EntityCapExceeded(_) => {
            format!("plugin {plugin_id} exceeded entity-count cap")
        }
        HostError::PathEscapeBreakerTripped => {
            format!("plugin {plugin_id} tripped path-escape breaker")
        }
        HostError::Spawn(msg) => {
            format!("failed to spawn plugin {plugin_id}: {msg}")
        }
        HostError::Handshake(ref me) => {
            format!("plugin {plugin_id} refused handshake: {me}")
        }
        HostError::Transport(ref te) => {
            format!("plugin {plugin_id} transport/protocol error: {te}")
        }
        HostError::Protocol(ref pe) => {
            format!(
                "plugin {plugin_id} transport/protocol error: code={}, message={}",
                pe.code, pe.message
            )
        }
        other => format!("plugin {plugin_id} error: {other}"),
    }
}

/// Map an `AcceptedEntity` to an `EntityRecord` for the writer-actor.
fn map_entity_to_record(entity: &AcceptedEntity, plugin_id: &str) -> EntityRecord {
    let short_name = entity
        .qualified_name
        .rsplit('.')
        .next()
        .unwrap_or(&entity.qualified_name)
        .to_owned();

    let properties_json =
        serde_json::to_string(&entity.raw.extra).unwrap_or_else(|_| "{}".to_owned());

    let now = iso8601_now();

    EntityRecord {
        id: entity.id.to_string(),
        plugin_id: plugin_id.to_owned(),
        kind: entity.kind.clone(),
        name: entity.qualified_name.clone(),
        short_name,
        parent_id: None,
        source_file_id: None,
        source_byte_start: None,
        source_byte_end: None,
        source_line_start: None,
        source_line_end: None,
        properties_json,
        content_hash: None,
        summary_json: None,
        wardline_json: None,
        first_seen_commit: None,
        last_seen_commit: None,
        created_at: now.clone(),
        updated_at: now,
    }
}

// ── Source-tree walk ──────────────────────────────────────────────────────────

/// Skip-list for directory names during the source walk.
///
/// Sprint 1 conservative set: VCS directories, clarion's own state, and
/// common virtual-environment directories.
const SKIP_DIRS: &[&str] = &[
    ".clarion",
    ".git",
    ".hg",
    ".svn",
    ".jj",
    ".venv",
    "__pycache__",
    "node_modules",
];

/// Collect all source files under `root` whose extension is in `wanted`.
///
/// Uses `std::fs::read_dir` recursively. No `walkdir` dependency.
/// Symlinks are skipped (path-jail concerns for Sprint 1).
/// P4 observation: this does not respect `.gitignore`.
fn collect_source_files(
    root: &Path,
    wanted_extensions: &BTreeSet<String>,
) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_dir(root, &mut out, wanted_extensions)?;
    Ok(out)
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>, wanted: &BTreeSet<String>) -> io::Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(()),
        Err(e) => return Err(e),
    };

    for entry_result in entries {
        let Ok(entry) = entry_result else { continue };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        // Skip symlinks (path-jail concerns).
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();

        if file_type.is_dir() {
            // Skip directories in the skip-list.
            let dir_name = entry.file_name();
            let name_str = dir_name.to_string_lossy();
            if SKIP_DIRS.iter().any(|skip| *skip == name_str.as_ref()) {
                continue;
            }
            walk_dir(&path, out, wanted)?;
        } else if file_type.is_file() {
            // Check extension (case-insensitive compare; `wanted` is already lowercase).
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_ascii_lowercase();
                if wanted.contains(&ext_lower) {
                    out.push(path);
                }
            }
        }
    }

    Ok(())
}

// ── Time helpers ──────────────────────────────────────────────────────────────

/// Format `SystemTime::now()` as an `ISO-8601` UTC string with millisecond
/// precision (`YYYY-MM-DDTHH:MM:SS.sssZ`).
///
/// Inline rather than depending on `chrono` — Sprint 1 only needs this one
/// formatting pattern. Later WPs that want richer time handling can
/// promote `chrono` to a workspace dependency at that point.
fn iso8601_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX epoch");
    let secs = d.as_secs();
    let millis = d.subsec_millis();
    let (y, mo, da, h, mi, se) = civil_from_unix_secs(secs);
    format!("{y:04}-{mo:02}-{da:02}T{h:02}:{mi:02}:{se:02}.{millis:03}Z")
}

/// Convert a non-negative Unix timestamp (seconds since 1970-01-01 UTC)
/// into `(year, month, day, hour, minute, second)`.
///
/// Algorithm: Howard Hinnant's date, `civil_from_days`. Works for any date
/// from the Unix epoch forward. Does not account for leap seconds (none
/// of our timestamps need leap-second precision).
fn civil_from_unix_secs(mut secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let se = u32::try_from(secs % 60).expect("modulo 60 fits in u32");
    secs /= 60;
    let mi = u32::try_from(secs % 60).expect("modulo 60 fits in u32");
    secs /= 60;
    let h = u32::try_from(secs % 24).expect("modulo 24 fits in u32");
    secs /= 24;

    // secs is now days since the Unix epoch (1970-01-01).
    // Howard Hinnant's algorithm needs days shifted to 0000-03-01 epoch.
    let days = i64::try_from(secs).expect("days since epoch fits in i64");
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u64::try_from(z - era * 146_097).expect("day-of-era is non-negative");
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y_shifted = i64::try_from(yoe).expect("year-of-era fits in i64") + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let da = u32::try_from(doy - (153 * mp + 2) / 5 + 1).expect("day-of-month fits in u32");
    let mo = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).expect("month fits in u32");
    let y_i64 = if mo <= 2 { y_shifted + 1 } else { y_shifted };
    let y = u32::try_from(y_i64).expect("year fits in u32 (post-1970)");
    (y, mo, da, h, mi, se)
}

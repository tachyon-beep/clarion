//! `clarion analyze` — Sprint 1 skeleton.
//!
//! Opens `.clarion/clarion.db`, begins a run, logs a warning that no plugins
//! are wired, and commits the run with status `skipped_no_plugins`. WP2
//! replaces this body with real plugin spawning.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use uuid::Uuid;

use clarion_storage::{
    DEFAULT_BATCH_SIZE, DEFAULT_CHANNEL_CAPACITY, Writer,
    commands::{RunStatus, WriterCmd},
};

/// Run the analyze command against `project_path`.
///
/// # Errors
///
/// Returns an error if the target directory does not exist, has no `.clarion/`
/// directory, or if the writer actor fails to start or process commands.
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

    tracing::warn!(
        run_id = %run_id,
        "no plugins registered (WP2 will wire this)"
    );

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
        .context("CommitRun")?;

    // Writer owns the internal sender. Dropping `writer` closes the channel,
    // which lets the actor's `rx.blocking_recv()` return None and exit.
    drop(writer);
    handle
        .await
        .map_err(|e| anyhow::anyhow!("writer actor panic: {e}"))?
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!("analyze complete: run {run_id} skipped_no_plugins");
    Ok(())
}

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

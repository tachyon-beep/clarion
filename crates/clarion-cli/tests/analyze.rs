//! `clarion analyze` Sprint-1 integration test.

use assert_cmd::Command;
use rusqlite::Connection;

fn clarion_bin() -> Command {
    Command::cargo_bin("clarion").expect("clarion binary")
}

#[test]
fn analyze_without_plugins_writes_skipped_run_row() {
    let dir = tempfile::tempdir().unwrap();

    // Scrub PATH — if the developer or CI image has any clarion-plugin-*
    // binary installed (including the project's own fixture), discovery
    // will find it and the run transitions out of `skipped_no_plugins`.
    // The sibling test `analyze_failrun_exits_nonzero_with_run_row_marked_failed`
    // uses the same pattern.
    clarion_bin()
        .args(["install", "--path"])
        .arg(dir.path())
        .env("PATH", "")
        .assert()
        .success();

    clarion_bin()
        .args(["analyze"])
        .arg(dir.path())
        .env("PATH", "")
        .assert()
        .success();

    let conn = Connection::open(dir.path().join(".clarion/clarion.db")).unwrap();
    let (count, status): (i64, String) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(MAX(status), '') FROM runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(status, "skipped_no_plugins");

    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .unwrap();
    assert_eq!(entity_count, 0);
}

#[test]
fn analyze_fails_cleanly_if_clarion_dir_missing() {
    let dir = tempfile::tempdir().unwrap();
    let out = clarion_bin()
        .args(["analyze"])
        .arg(dir.path())
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("clarion install"),
        "error did not point operator at install: {stderr}"
    );
}

/// Regression for wp2 review-2 (clarion-f56dc6ee43): `FailRun` must exit
/// non-zero so `clarion analyze && next` chains and CI gating work.
///
/// Triggers the discovery-errors `FailRun` branch by placing a
/// `clarion-plugin-*` executable on `$PATH` next to a malformed
/// `plugin.toml`. Before the fix, this exited 0; after, it exits non-zero
/// AND the `runs.status` column still reads `failed` (the run row is
/// marked before the bail).
#[cfg(unix)]
#[test]
fn analyze_failrun_exits_nonzero_with_run_row_marked_failed() {
    use std::os::unix::fs::symlink;

    let project_dir = tempfile::tempdir().unwrap();
    let plugin_dir = tempfile::tempdir().unwrap();

    clarion_bin()
        .args(["install", "--path"])
        .arg(project_dir.path())
        .assert()
        .success();

    // Put a `clarion-plugin-broken` on the synthetic PATH alongside a
    // malformed plugin.toml. Discovery will try to parse the toml and
    // collect the error; with no compliant plugins, FailRun fires.
    let plugin_bin = plugin_dir.path().join("clarion-plugin-broken");
    symlink("/bin/true", &plugin_bin).expect("symlink /bin/true");
    std::fs::write(
        plugin_dir.path().join("plugin.toml"),
        b"this is {not = valid toml @@@",
    )
    .expect("write malformed plugin.toml");

    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let new_path = std::env::join_paths(
        std::iter::once(plugin_dir.path().to_path_buf())
            .chain(std::env::split_paths(&current_path)),
    )
    .expect("join_paths");

    let out = clarion_bin()
        .args(["analyze"])
        .arg(project_dir.path())
        .env("PATH", &new_path)
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("failed"),
        "stderr should mention failure; got: {stderr}"
    );

    // The run row must still be marked `failed` — the FailRun WriterCmd
    // runs before the bail, so the DB state is consistent with the exit
    // code.
    let conn = Connection::open(project_dir.path().join(".clarion/clarion.db")).unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM runs ORDER BY started_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("query latest run status");
    assert_eq!(
        status, "failed",
        "run row must be marked 'failed' to stay consistent with exit code"
    );
}

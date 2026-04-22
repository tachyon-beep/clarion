//! WP2 Task 9 — end-to-end smoke test.
//!
//! Proves signoff A.2.8: the full Sprint 1 walking-skeleton pipeline works.
//!
//! Scenario:
//!   1. `clarion install` initialises `.clarion/clarion.db`.
//!   2. A `clarion-plugin-fixture` binary is placed on a synthetic `$PATH`
//!      alongside its `plugin.toml` (neighbour-discovery convention, L9).
//!   3. A single source file `demo.mt` is created in the project root.
//!   4. `clarion analyze` discovers the fixture plugin, spawns it,
//!      handshakes, calls `analyze_file` once, receives one entity, and
//!      persists it to the `entities` table.
//!
//! Asserts the full round-trip preserves entity identity: the persisted
//! row exactly matches the fixture plugin's declared emission
//! (`fixture:widget:demo.sample`).

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, fs};

use assert_cmd::Command;
use rusqlite::Connection;
use tempfile::TempDir;

fn clarion_bin() -> Command {
    Command::cargo_bin("clarion").expect("clarion binary")
}

/// Locate the `clarion-plugin-fixture` binary.
///
/// Tries `CARGO_BIN_EXE_clarion-plugin-fixture` first (set by cargo nextest
/// when `clarion-plugin-fixture` appears in `[dev-dependencies]`). Falls back
/// to the standard `target/{debug,release}/` search.
fn fixture_binary_path() -> PathBuf {
    if let Ok(path) = env::var("CARGO_BIN_EXE_clarion-plugin-fixture") {
        return PathBuf::from(path);
    }

    // Fallback: search target/ relative to the workspace root.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // clarion-cli is at crates/clarion-cli; workspace root is ../../
    let workspace_root = manifest_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("workspace root must exist");

    let target_dir =
        env::var("CARGO_TARGET_DIR").map_or_else(|_| workspace_root.join("target"), PathBuf::from);

    for profile in &["debug", "release"] {
        let candidate = target_dir.join(profile).join("clarion-plugin-fixture");
        if candidate.exists() {
            return candidate;
        }
    }

    panic!(
        "clarion-plugin-fixture binary not found. \
         Run `cargo build --workspace` before running this test. \
         Searched: {}",
        target_dir.display()
    );
}

/// Set up a synthetic `$PATH` directory containing:
/// - `clarion-plugin-fixture` executable (symlink to the real binary).
/// - `plugin.toml` manifest (copied from the core test fixtures).
///
/// Returns the temp dir (must stay alive for the duration of the test).
fn setup_plugin_dir(fixture_bin: &PathBuf) -> TempDir {
    let plugin_dir = TempDir::new().expect("create plugin tempdir");

    // Symlink the fixture binary into the dir under its expected name.
    let dest = plugin_dir.path().join("clarion-plugin-fixture");
    std::os::unix::fs::symlink(fixture_bin, &dest).expect("symlink clarion-plugin-fixture");

    // Verify the target is executable.
    let meta = fs::metadata(fixture_bin).expect("stat fixture binary");
    assert!(
        meta.permissions().mode() & 0o111 != 0,
        "fixture binary must be executable"
    );

    // Copy the `plugin.toml` fixture next to the binary (neighbor convention).
    let toml_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .unwrap()
        .join("clarion-core")
        .join("tests")
        .join("fixtures")
        .join("plugin.toml");
    let toml_dest = plugin_dir.path().join("plugin.toml");
    fs::copy(&toml_src, &toml_dest).expect("copy plugin.toml");

    plugin_dir
}

#[test]
fn wp2_e2e_smoke_fixture_plugin_round_trip() {
    // 1. Locate the fixture binary.
    let fixture_bin = fixture_binary_path();

    // 2. Create a synthetic $PATH directory with the plugin and manifest.
    let plugin_dir = setup_plugin_dir(&fixture_bin);

    // 3. Set up the project directory.
    let project_dir = TempDir::new().expect("create project tempdir");

    // 4. `clarion install` to initialise `.clarion/`.
    clarion_bin()
        .args(["install", "--path"])
        .arg(project_dir.path())
        .assert()
        .success();

    // 5. Place a source file the fixture plugin claims (`*.mt`).
    fs::write(
        project_dir.path().join("demo.mt"),
        b"widget demo.sample {}\n",
    )
    .expect("write demo.mt");

    // 6. Build a synthetic PATH: plugin_dir prepended to the current PATH.
    let current_path = env::var_os("PATH").unwrap_or_default();
    let new_path = env::join_paths(
        std::iter::once(plugin_dir.path().to_path_buf()).chain(env::split_paths(&current_path)),
    )
    .expect("join_paths");

    // 7. Run `clarion analyze` with the synthetic PATH.
    clarion_bin()
        .args(["analyze"])
        .arg(project_dir.path())
        .env("PATH", &new_path)
        .assert()
        .success();

    // 8. Verify the database — full round-trip identity assertions.
    let db_path = project_dir.path().join(".clarion/clarion.db");
    let conn = Connection::open(&db_path).expect("open db");

    // Assert 1 + 2: exactly one run row with status "completed".
    let (run_count, run_status): (i64, String) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(MAX(status), '') FROM runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query runs");

    assert_eq!(
        run_count, 1,
        "expected exactly one run row; got {run_count}"
    );
    assert_eq!(
        run_status, "completed",
        "run status must be 'completed'; got {run_status:?}"
    );

    // Assert 3: stats JSON reports entities_inserted = 1.
    let stats_raw: String = conn
        .query_row("SELECT stats FROM runs LIMIT 1", [], |row| row.get(0))
        .expect("query runs.stats");
    let stats: serde_json::Value =
        serde_json::from_str(&stats_raw).expect("stats column must be valid JSON");
    assert_eq!(
        stats["entities_inserted"],
        serde_json::Value::Number(1.into()),
        "stats must report entities_inserted = 1; got {stats_raw:?}"
    );

    // Assert 4: exactly one entity row.
    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .expect("query entities count");

    assert_eq!(
        entity_count, 1,
        "expected exactly one entity row; got {entity_count}"
    );

    // Asserts 5–8: the persisted row matches the fixture's declared emission.
    let (entity_id, entity_kind, entity_plugin_id, entity_name): (String, String, String, String) =
        conn.query_row(
            "SELECT id, kind, plugin_id, name FROM entities LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("query entity row");

    assert_eq!(
        entity_id, "fixture:widget:demo.sample",
        "entity id must be 'fixture:widget:demo.sample'; got {entity_id:?}"
    );
    assert_eq!(
        entity_kind, "widget",
        "entity kind must be 'widget'; got {entity_kind:?}"
    );
    assert_eq!(
        entity_plugin_id, "fixture",
        "entity plugin_id must be 'fixture'; got {entity_plugin_id:?}"
    );
    assert_eq!(
        entity_name, "demo.sample",
        "entity name must be 'demo.sample'; got {entity_name:?}"
    );
}

/// Regression for wp2 review-2 (clarion-978c8d6f15): crash-loop breaker is
/// wired into the production analyze path AND a single plugin crash no
/// longer tanks the whole run.
///
/// Scenario:
/// - `clarion-plugin-fixture` + its manifest in `plugin_dir_a` (extensions = mt)
/// - `clarion-plugin-broken` (symlink to /bin/true) + a manifest declaring
///   `plugin_id` "broken" and extensions = "bk" in `plugin_dir_b`
/// - Project root has `demo.mt` (fixture input) and `demo.bk` (broken input)
/// - Both plugin dirs prepended to PATH
///
/// Expected: `broken` fails handshake (no response on closed stdout), its
/// crash is recorded on the breaker, the run continues, and the fixture
/// plugin processes `demo.mt` successfully. The run resolves to `failed`
/// (exit 1, runs.status = 'failed') but the fixture's entity is persisted
/// — continue-past-crash preserves partial work.
#[test]
fn wp2_crash_in_one_plugin_does_not_prevent_other_plugins_from_running() {
    // 1. Locate the fixture binary.
    let fixture_bin = fixture_binary_path();

    // 2. plugin_dir_a: working fixture.
    let plugin_dir_a = setup_plugin_dir(&fixture_bin);

    // 3. plugin_dir_b: broken plugin pointing at /bin/true.
    let plugin_dir_b = TempDir::new().expect("create broken plugin dir");
    let broken_bin = plugin_dir_b.path().join("clarion-plugin-broken");
    std::os::unix::fs::symlink("/bin/true", &broken_bin).expect("symlink /bin/true");
    let broken_manifest = r#"
[plugin]
name = "clarion-plugin-broken"
plugin_id = "broken"
version = "0.1.0"
protocol_version = "1.0"
executable = "clarion-plugin-broken"
language = "broken"
extensions = ["bk"]

[capabilities.runtime]
expected_max_rss_mb = 256
expected_entities_per_file = 100
wardline_aware = false
reads_outside_project_root = false

[ontology]
entity_kinds = ["widget"]
edge_kinds = []
rule_id_prefix = "CLA-BROKEN-"
ontology_version = "0.1.0"
"#;
    fs::write(plugin_dir_b.path().join("plugin.toml"), broken_manifest)
        .expect("write broken plugin.toml");

    // 4. Set up project directory with one file per plugin extension.
    let project_dir = TempDir::new().expect("create project tempdir");
    clarion_bin()
        .args(["install", "--path"])
        .arg(project_dir.path())
        .assert()
        .success();
    fs::write(
        project_dir.path().join("demo.mt"),
        b"widget demo.sample {}\n",
    )
    .expect("write demo.mt");
    fs::write(project_dir.path().join("demo.bk"), b"// broken's input\n").expect("write demo.bk");

    // 5. PATH with BOTH plugin dirs.
    let current_path = env::var_os("PATH").unwrap_or_default();
    let new_path = env::join_paths(
        [
            plugin_dir_a.path().to_path_buf(),
            plugin_dir_b.path().to_path_buf(),
        ]
        .into_iter()
        .chain(env::split_paths(&current_path)),
    )
    .expect("join_paths");

    // 6. analyze must exit non-zero (a plugin crashed) but the run still
    //    processes the other plugin's files.
    let out = clarion_bin()
        .args(["analyze"])
        .arg(project_dir.path())
        .env("PATH", &new_path)
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("broken"),
        "stderr should name the crashed plugin; got: {stderr}"
    );

    // 7. Verify the DB: run = 'failed', entity from fixture IS persisted.
    //    `fail_run` writes the reason into stats.failure_reason (JSON).
    let conn = Connection::open(project_dir.path().join(".clarion/clarion.db")).unwrap();
    let (row_count, run_status, stats_raw): (i64, String, String) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(MAX(status), ''), COALESCE(MAX(stats), '') FROM runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("query runs");
    assert_eq!(row_count, 1, "expected exactly one run row");
    assert_eq!(
        run_status, "failed",
        "any-plugin-crash must still mark run as failed; got {run_status:?}"
    );
    let stats: serde_json::Value =
        serde_json::from_str(&stats_raw).expect("stats must be valid JSON");
    let failure_reason = stats["failure_reason"]
        .as_str()
        .expect("failure_reason must be a string");
    assert!(
        failure_reason.contains("broken"),
        "failure_reason should name the crashed plugin; got {failure_reason:?}"
    );

    // This is the behavioural assertion that matters: the fixture plugin's
    // entity is persisted even though `broken` crashed earlier in the run.
    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .expect("query entities count");
    assert_eq!(
        entity_count, 1,
        "fixture plugin's entity must still be persisted despite broken plugin's crash; got {entity_count}",
    );
    let entity_plugin_id: String = conn
        .query_row("SELECT plugin_id FROM entities LIMIT 1", [], |row| {
            row.get(0)
        })
        .expect("query entity plugin_id");
    assert_eq!(
        entity_plugin_id, "fixture",
        "surviving entity must be from the fixture plugin; got {entity_plugin_id:?}"
    );
}

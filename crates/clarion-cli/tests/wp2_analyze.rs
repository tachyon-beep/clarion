//! WP2 Task 8 integration test — `clarion analyze` with live plugin.
//!
//! Exercises the full `clarion analyze` command against a project directory
//! that has:
//! - A pre-initialised `.clarion/clarion.db` (via `clarion install`).
//! - The `clarion-plugin-fixture` binary on a synthetic `$PATH`.
//! - A `demo.mt` source file in the project root.
//!
//! Asserts: the command exits successfully, the `runs` table has exactly one
//! row with status `completed`, and the `entities` table has `entity_count > 0`.
//!
//! The fixture plugin emits one entity per `analyze_file` call
//! (`fixture:widget:demo.sample`), so one source file yields `entity_count == 1`.

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
fn wp2_analyze_with_fixture_plugin_produces_entities() {
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

    // 8. Verify the database.
    let db_path = project_dir.path().join(".clarion/clarion.db");
    let conn = Connection::open(&db_path).expect("open db");

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

    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .expect("query entities");

    assert!(
        entity_count > 0,
        "expected at least one entity; got {entity_count}"
    );
}

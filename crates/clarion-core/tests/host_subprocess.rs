//! T1 — subprocess happy path integration test.
//!
//! Spawns the `clarion-plugin-fixture` binary via [`PluginHost::spawn`],
//! performs the full handshake, issues one `analyze_file` for a fixture file,
//! receives one entity, shuts down cleanly, and asserts exit code 0.
//!
//! The fixture binary is located at runtime by searching the Cargo target
//! directory. This is necessary because `CARGO_BIN_EXE_*` is only available
//! for binaries in the same crate; cross-crate binary resolution requires
//! either `-Z bindeps` (unstable) or a runtime search.

use clarion_core::PluginHost;
use clarion_core::plugin::parse_manifest;

/// Path to the fixture plugin.toml — embedded at compile time.
const FIXTURE_MANIFEST_BYTES: &[u8] = include_bytes!("fixtures/plugin.toml");

/// Locate the `clarion-plugin-fixture` binary in the Cargo target directory.
///
/// Searches the standard Cargo output locations in order:
/// 1. `CARGO_BIN_EXE_clarion-plugin-fixture` env var (set by cargo nextest
///    when artifact deps are enabled — future use).
/// 2. `<target_dir>/debug/clarion-plugin-fixture` (default dev build).
/// 3. `<target_dir>/release/clarion-plugin-fixture` (release build).
///
/// Panics with a clear message if the binary is not found.
fn fixture_binary_path() -> std::path::PathBuf {
    // Check if an explicit path was provided (e.g. by a future artifact dep).
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_clarion-plugin-fixture") {
        return std::path::PathBuf::from(path);
    }

    // Locate the workspace target directory via CARGO_MANIFEST_DIR.
    // CARGO_MANIFEST_DIR for an integration test is the crate's directory.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // clarion-core is at crates/clarion-core; workspace root is ../../
    let workspace_root = manifest_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("workspace root must exist");

    // Try CARGO_TARGET_DIR override first, then the default `target/` directory.
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map_or_else(|_| workspace_root.join("target"), std::path::PathBuf::from);

    for profile in &["debug", "release"] {
        let candidate = target_dir.join(profile).join("clarion-plugin-fixture");
        if candidate.exists() {
            return candidate;
        }
    }

    panic!(
        "clarion-plugin-fixture binary not found. \
         Run `cargo build -p clarion-plugin-fixture` before running this test. \
         Searched in: {}",
        target_dir.display()
    );
}

/// Verify the fixture manifest parses correctly.
/// This catches schema mismatches before the subprocess test runs.
#[test]
fn fixture_manifest_parses_correctly() {
    let manifest = parse_manifest(FIXTURE_MANIFEST_BYTES).expect("fixture manifest must parse");
    assert_eq!(manifest.plugin.plugin_id, "fixture");
    assert_eq!(manifest.ontology.entity_kinds, vec!["widget"]);
    assert_eq!(manifest.ontology.rule_id_prefix, "CLA-FIXTURE-");
    assert!(
        !manifest.capabilities.runtime.reads_outside_project_root,
        "fixture manifest must not request reads_outside_project_root"
    );
}

/// T1: subprocess happy path.
///
/// Spawns the fixture plugin, completes the handshake, analyzes a real file,
/// receives one entity, shuts down, and asserts exit code 0.
#[test]
fn t1_subprocess_happy_path() {
    // 1. Parse the fixture manifest.
    let mut manifest =
        parse_manifest(FIXTURE_MANIFEST_BYTES).expect("fixture plugin.toml must be valid");

    // 2. Override the executable path to the compiled fixture binary.
    manifest.plugin.executable = fixture_binary_path()
        .to_str()
        .expect("fixture binary path must be valid UTF-8")
        .to_owned();

    // 3. Build a real project root containing the fixture sample file.
    let project_dir = tempfile::TempDir::new().expect("create tempdir");
    let sample_path = project_dir.path().join("sample.mt");
    std::fs::write(&sample_path, b"widget demo.sample {}\n").expect("write sample.mt");

    // 4. Spawn the plugin and complete the handshake.
    let (mut host, mut child) =
        PluginHost::spawn(manifest, project_dir.path()).expect("spawn must succeed");

    // 5. Analyze the fixture file.
    let entities = host
        .analyze_file(&sample_path)
        .expect("analyze_file must succeed");

    // 6. Assert: exactly one entity.
    assert_eq!(
        entities.len(),
        1,
        "fixture plugin must return exactly one entity per analyze_file; got {}",
        entities.len()
    );
    let entity = &entities[0];
    assert_eq!(
        entity.kind, "widget",
        "entity kind must be 'widget'; got {:?}",
        entity.kind
    );
    assert_eq!(
        entity.id.as_str(),
        "fixture:widget:demo.sample",
        "entity id must be 'fixture:widget:demo.sample'; got {:?}",
        entity.id.as_str()
    );

    // 7. Shut down cleanly.
    host.shutdown().expect("shutdown must succeed");

    // 8. Wait for the child and assert exit code 0.
    let status = child.wait().expect("wait for child process");
    assert!(
        status.success(),
        "fixture plugin must exit with code 0; got: {status:?}"
    );

    // 9. No unexpected findings.
    let findings = host.take_findings();
    assert!(
        findings.is_empty(),
        "no findings expected on happy path; got: {findings:?}"
    );
}

/// T9: handshake failure on a subprocess that exits before responding
/// returns `Err` promptly — the host does not hang on a closed stdout.
///
/// Points `executable` at `/bin/true` (or Windows equivalent), which exits
/// immediately. The host tries to read the initialize response from a closed
/// stdout and returns a transport error.
///
/// **What this test asserts**: `spawn()` returns `Err` and the whole call
/// completes well under 5 s. That's strictly a "did we hang?" probe — it
/// does NOT directly verify the zombie-reap behaviour added in commit
/// 0fcc57f (that fix is covered by code review of `host.rs::spawn`'s
/// `if let Err(e) = host.handshake()` block). Direct zombie observation
/// requires walking `/proc`, which is Linux-only and brittle across kernel
/// versions.
///
/// The earlier name `t9_handshake_failure_exits_cleanly_without_hanging`
/// overstated this — "exits cleanly" implied zombie-reap coverage.
#[test]
#[cfg(unix)]
fn t9_handshake_failure_on_immediate_exit_returns_err_promptly() {
    let mut manifest = parse_manifest(FIXTURE_MANIFEST_BYTES).expect("fixture manifest must parse");
    // `/bin/true` exists on all Unix systems, exits 0 without reading stdin.
    manifest.plugin.executable = "/bin/true".to_owned();

    let project_dir = tempfile::TempDir::new().expect("tmpdir");

    let start = std::time::Instant::now();
    let result = PluginHost::spawn(manifest, project_dir.path());
    let elapsed = start.elapsed();

    assert!(
        result.is_err(),
        "spawn must fail when executable exits before handshake response"
    );
    // Sanity: the handshake-failure path must not block. If reap lost a
    // waitpid, this would still return but a regression that swapped kill()
    // or wait() for a blocking read on the closed pipe would hang here.
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "handshake failure must return promptly; took {elapsed:?}"
    );
}

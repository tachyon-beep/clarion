//! Plugin-host supervisor.
//!
//! Implements ADR-021 §Layer 2 core-enforced minimums plus the ADR-022 ontology
//! boundary and UQ-WP2-11 identity-mismatch check.
//!
//! # Overview
//!
//! `PluginHost` is generic over `R: BufRead` and `W: Write` so unit tests can
//! drive it with an in-process mock without spawning a real subprocess.
//!
//! # Enforcement pipeline (per entity in `analyze_file` response)
//!
//! 1. **Ontology check** (ADR-022): `entity.kind` must be in
//!    `manifest.ontology.entity_kinds`. Violation → drop + finding; no kill.
//! 2. **Identity check** (UQ-WP2-11): `entity_id(plugin_id, kind, qualified_name)`
//!    must equal the returned `entity.id` string. Mismatch → drop + finding; no kill.
//! 3. **Jail check** (ADR-021 §2a): `entity.source.file_path` must canonicalise
//!    inside `project_root`. Escape → drop + finding; tick [`PathEscapeBreaker`].
//!    Breaker tripped → kill plugin, return [`HostError::PathEscapeBreakerTripped`].
//! 4. **Entity cap check** (ADR-021 §2c): run-cumulative count must stay ≤ 500k.
//!    Exceeded → kill plugin, return [`HostError::EntityCapExceeded`].
//!
//! # Memory limit
//!
//! On Linux, [`PluginHost::spawn`] calls [`apply_prlimit_as`] inside
//! `CommandExt::pre_exec` to set `RLIMIT_AS` before `exec()`. The closure body
//! only calls `setrlimit(2)`, which is async-signal-safe per POSIX.1-2017
//! §2.4.3. The `unsafe` block is the minimum required by the `pre_exec` API.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::entity_id::{EntityId, EntityIdError, entity_id};
use crate::plugin::jail::{JailError, jail_to_string};
use crate::plugin::limits::{
    BreakerState, CapExceeded, ContentLengthCeiling, DEFAULT_MAX_RSS_MIB, EntityCountCap,
    FINDING_DISABLED_PATH_ESCAPE, FINDING_ENTITY_CAP, FINDING_PATH_ESCAPE, PathEscapeBreaker,
    apply_prlimit_as, effective_rss_mib,
};
use crate::plugin::manifest::{Manifest, ManifestError};
use crate::plugin::protocol::{
    AnalyzeFileParams, ExitNotification, InitializeParams, InitializedNotification, ProtocolError,
    ResponseEnvelope, ResponsePayload, ShutdownParams, make_notification, make_request,
};
use crate::plugin::transport::{Frame, TransportError, read_frame, write_frame};

// ── Finding subcode constants ─────────────────────────────────────────────────

/// Emitted when a plugin emits an entity whose `kind` is not in the manifest's
/// `entity_kinds` list (ADR-022 ontology boundary).
pub const FINDING_UNDECLARED_KIND: &str = "CLA-INFRA-PLUGIN-UNDECLARED-KIND";

/// Emitted when a plugin emits an entity whose `id` string does not match the
/// expected `entity_id(plugin_id, kind, qualified_name)` (UQ-WP2-11).
pub const FINDING_ENTITY_ID_MISMATCH: &str = "CLA-INFRA-PLUGIN-ENTITY-ID-MISMATCH";

/// Emitted when the manifest contains a capability not supported in v0.1
/// (ADR-021 §Layer 1).
pub const FINDING_UNSUPPORTED_CAPABILITY: &str = "CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY";

// ── Wire entity types (Option A) ──────────────────────────────────────────────

/// Raw entity as received from the plugin wire.
///
/// Deserialised directly from the `entities` array in `AnalyzeFileResult`.
/// Surviving entities become [`AcceptedEntity`] values after validation.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawEntity {
    /// Three-segment entity ID: `plugin_id:kind:qualified_name`.
    pub id: String,
    /// Entity kind, e.g. `"function"`.
    pub kind: String,
    /// Canonical qualified name, e.g. `"auth.tokens.refresh"`.
    pub qualified_name: String,
    /// Source location.
    pub source: RawSource,
    /// Extra fields — accepted without interpretation.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Source location from the wire entity.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawSource {
    /// Absolute or project-relative path. Subject to the path jail.
    pub file_path: String,
    /// Extra source fields — accepted without interpretation.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// An entity that has passed all validation checks.
///
/// Returned by [`PluginHost::analyze_file`] for each entity that survived the
/// ontology, identity, jail, and cap checks.
#[derive(Debug, Clone)]
pub struct AcceptedEntity {
    /// Parsed and validated entity ID.
    pub id: EntityId,
    /// Kind (matches `manifest.ontology.entity_kinds`).
    pub kind: String,
    /// Canonical qualified name.
    pub qualified_name: String,
    /// Jail-canonicalised, UTF-8 source path.
    pub source_file_path: String,
    /// The original raw entity (for downstream consumers, e.g. WP1 writer).
    pub raw: RawEntity,
}

// ── Error types ───────────────────────────────────────────────────────────────

/// Operational failures returned to the caller of `PluginHost` methods.
#[derive(Debug, Error)]
pub enum HostError {
    /// Transport-layer failure (I/O or framing error).
    #[error("transport: {0}")]
    Transport(#[from] TransportError),

    /// Protocol violation (e.g. response id mismatch, error payload).
    #[error("protocol error: code={}, message={}", .0.code, .0.message)]
    Protocol(ProtocolError),

    /// Manifest capability check failed at handshake time.
    #[error("manifest validation at handshake: {0}")]
    Handshake(ManifestError),

    /// Path-jail error (non-escape variants, e.g. `Io` or `NonUtf8Path`).
    #[error("jail: {0}")]
    Jail(JailError),

    /// Run-cumulative entity cap exceeded; plugin was killed.
    #[error("entity cap exceeded")]
    EntityCapExceeded(#[source] CapExceeded),

    /// Path-escape circuit-breaker tripped; plugin was killed.
    #[error("path-escape breaker tripped; plugin killed")]
    PathEscapeBreakerTripped,

    /// JSON serialisation / deserialisation error.
    #[error("json: {0}")]
    Serde(#[from] serde_json::Error),

    /// Low-level I/O error not wrapped in a transport error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// Plugin spawn failed.
    #[error("plugin spawn failed: {0}")]
    Spawn(String),

    /// Entity ID construction error (malformed `plugin_id` or kind in manifest).
    #[error("entity id error: {0}")]
    EntityId(#[from] EntityIdError),
}

impl From<ManifestError> for HostError {
    fn from(e: ManifestError) -> Self {
        HostError::Handshake(e)
    }
}

/// Informational diagnostic accumulated during a host's lifetime.
///
/// Collected into `self.findings` on each enforcement action. Drained via
/// [`PluginHost::take_findings`]. Will eventually be persisted as ADR-004
/// Findings; for Sprint 1 they are collected only.
#[derive(Debug, Clone)]
pub struct HostFinding {
    /// Finding subcode, e.g. `"CLA-INFRA-PLUGIN-PATH-ESCAPE"`.
    pub subcode: &'static str,
    /// Human-readable message.
    pub message: String,
    /// Structured metadata (keys: `"offending_path"`, `"entity_id"`, etc.).
    pub metadata: BTreeMap<String, String>,
}

impl HostFinding {
    fn undeclared_kind(kind: &str, qualified_name: &str) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("kind".to_owned(), kind.to_owned());
        metadata.insert("qualified_name".to_owned(), qualified_name.to_owned());
        Self {
            subcode: FINDING_UNDECLARED_KIND,
            message: format!("entity kind {kind:?} is not declared in the manifest ontology"),
            metadata,
        }
    }

    fn entity_id_mismatch(got: &str, expected: &str) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("got".to_owned(), got.to_owned());
        metadata.insert("expected".to_owned(), expected.to_owned());
        Self {
            subcode: FINDING_ENTITY_ID_MISMATCH,
            message: format!("entity id mismatch: got {got:?}, expected {expected:?}"),
            metadata,
        }
    }

    fn path_escape(offending_path: &str) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("offending_path".to_owned(), offending_path.to_owned());
        Self {
            subcode: FINDING_PATH_ESCAPE,
            message: format!("entity source path escapes project root: {offending_path:?}"),
            metadata,
        }
    }

    fn disabled_path_escape() -> Self {
        Self {
            subcode: FINDING_DISABLED_PATH_ESCAPE,
            message: "path-escape circuit breaker tripped; plugin killed".to_owned(),
            metadata: BTreeMap::new(),
        }
    }

    fn entity_cap_exceeded_finding(cap: usize, would_reach: usize) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("cap".to_owned(), cap.to_string());
        metadata.insert("would_reach".to_owned(), would_reach.to_string());
        Self {
            subcode: FINDING_ENTITY_CAP,
            message: format!("entity cap {cap} would be exceeded (would reach {would_reach})"),
            metadata,
        }
    }

    fn unsupported_capability(msg: &str) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("detail".to_owned(), msg.to_owned());
        Self {
            subcode: FINDING_UNSUPPORTED_CAPABILITY,
            message: format!("manifest has unsupported capability: {msg}"),
            metadata,
        }
    }
}

// ── PluginHost ────────────────────────────────────────────────────────────────

/// Supervisor managing a single plugin connection.
///
/// Generic over `R: BufRead` and `W: Write` so tests can drive the host
/// in-process without a subprocess.
pub struct PluginHost<R, W>
where
    R: BufRead,
    W: Write,
{
    manifest: Manifest,
    project_root: PathBuf,
    reader: R,
    writer: W,
    ceiling: ContentLengthCeiling,
    entity_cap: EntityCountCap,
    path_breaker: PathEscapeBreaker,
    next_request_id: i64,
    findings: Vec<HostFinding>,
}

// ── Subprocess constructor ────────────────────────────────────────────────────

impl
    PluginHost<
        std::io::BufReader<std::process::ChildStdout>,
        std::io::BufWriter<std::process::ChildStdin>,
    >
{
    /// Spawn the plugin as a subprocess, apply `RLIMIT_AS` on Linux, perform
    /// the handshake, and return the live host alongside the child handle.
    ///
    /// # Errors
    ///
    /// Returns [`HostError::Spawn`] if the executable cannot be started, or a
    /// handshake error if the plugin fails `initialize` or the manifest fails
    /// `validate_for_v0_1`.
    pub fn spawn(
        manifest: Manifest,
        project_root: &Path,
    ) -> Result<(Self, std::process::Child), HostError> {
        let canonical_root = project_root
            .canonicalize()
            .map_err(|e| HostError::Spawn(format!("canonicalise project root: {e}")))?;

        let mut command = std::process::Command::new(&manifest.plugin.executable);
        command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        // SAFETY: `apply_prlimit_as` calls `setrlimit(2)` which is listed in
        // POSIX.1-2017 §2.4.3 as async-signal-safe. The `pre_exec` closure
        // runs in the forked child after `fork()` but before `exec()`, so
        // only the child's address-space limit is affected. No Rust allocation
        // or non-async-signal-safe functions are called inside the closure.
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            let rss_mib = effective_rss_mib(
                manifest.capabilities.runtime.expected_max_rss_mb,
                DEFAULT_MAX_RSS_MIB,
            );
            #[allow(unsafe_code)]
            unsafe {
                command.pre_exec(move || apply_prlimit_as(rss_mib));
            }
        }

        let mut child = command
            .spawn()
            .map_err(|e| HostError::Spawn(format!("spawn {}: {e}", manifest.plugin.executable)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| HostError::Spawn("no stdin handle".to_owned()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| HostError::Spawn("no stdout handle".to_owned()))?;

        let mut host = PluginHost {
            manifest,
            project_root: canonical_root,
            reader: std::io::BufReader::new(stdout),
            writer: std::io::BufWriter::new(stdin),
            ceiling: ContentLengthCeiling::DEFAULT,
            entity_cap: EntityCountCap::new(EntityCountCap::DEFAULT_MAX),
            path_breaker: PathEscapeBreaker::new_default(),
            next_request_id: 1,
            findings: Vec::new(),
        };

        host.handshake()?;

        Ok((host, child))
    }
}

// ── Generic methods ───────────────────────────────────────────────────────────

impl<R: BufRead, W: Write> PluginHost<R, W> {
    /// Construct a host around an arbitrary reader/writer pair.
    ///
    /// Does NOT call `handshake()` — the caller must do so explicitly after
    /// wiring up the other side (e.g. a mock plugin).
    ///
    /// # Errors
    ///
    /// Returns [`HostError::Io`] if `project_root` cannot be canonicalised.
    pub fn connect(
        manifest: Manifest,
        project_root: &Path,
        reader: R,
        writer: W,
    ) -> Result<Self, HostError> {
        let canonical_root = std::fs::canonicalize(project_root)?;
        Ok(PluginHost {
            manifest,
            project_root: canonical_root,
            reader,
            writer,
            ceiling: ContentLengthCeiling::DEFAULT,
            entity_cap: EntityCountCap::new(EntityCountCap::DEFAULT_MAX),
            path_breaker: PathEscapeBreaker::new_default(),
            next_request_id: 1,
            findings: Vec::new(),
        })
    }

    /// Perform the `initialize` → `initialized` handshake.
    ///
    /// Steps:
    /// 1. Send `initialize` request.
    /// 2. Read and validate the `initialize` response (id match, result variant).
    /// 3. Call `manifest.validate_for_v0_1()`. On failure: push finding, send
    ///    `shutdown` + `exit`, return error — no `initialized` is sent.
    /// 4. Send `initialized` notification.
    ///
    /// # Errors
    ///
    /// Returns [`HostError::Handshake`] if the manifest fails capability checks.
    pub fn handshake(&mut self) -> Result<(), HostError> {
        // Step 1: send initialize request.
        let id = self.next_id();
        let params = InitializeParams {
            protocol_version: "1.0".to_owned(),
            project_root: self.project_root.to_string_lossy().into_owned(),
        };
        let req = make_request("initialize", &params, id);
        let body = serde_json::to_vec(&req)?;
        write_frame(&mut self.writer, &Frame { body })?;

        // Step 2: read initialize response.
        let resp_frame = read_frame(&mut self.reader, self.ceiling)?;
        let resp: ResponseEnvelope = serde_json::from_slice(&resp_frame.body)?;
        if resp.id != id {
            return Err(HostError::Protocol(ProtocolError {
                code: -32_600,
                message: format!("response id {} does not match request id {id}", resp.id),
                data: None,
            }));
        }
        match &resp.payload {
            ResponsePayload::Result(_) => {}
            ResponsePayload::Error(e) => return Err(HostError::Protocol(e.clone())),
        }

        // Step 3: validate manifest capabilities (ADR-021 §Layer 1).
        if let Err(e) = self.manifest.validate_for_v0_1() {
            self.findings
                .push(HostFinding::unsupported_capability(&e.to_string()));
            // Graceful shutdown — plugin is alive but we will not use it.
            let _ = self.do_shutdown();
            return Err(HostError::Handshake(e));
        }

        // Step 4: send initialized notification.
        let note = make_notification("initialized", &InitializedNotification {});
        let body = serde_json::to_vec(&note)?;
        write_frame(&mut self.writer, &Frame { body })?;

        Ok(())
    }

    /// Send `analyze_file` for `path`, read and validate the response, and
    /// return the surviving entities.
    ///
    /// Each entity is processed through the four-stage validation pipeline.
    ///
    /// # Errors
    ///
    /// - [`HostError::PathEscapeBreakerTripped`] when >10 path escapes occur.
    /// - [`HostError::EntityCapExceeded`] when the run-cumulative cap is exceeded.
    /// - Transport / serde errors on wire failures.
    pub fn analyze_file(&mut self, path: &Path) -> Result<Vec<AcceptedEntity>, HostError> {
        let file_path = path.to_string_lossy().into_owned();
        let id = self.next_id();
        let params = AnalyzeFileParams { file_path };
        let req = make_request("analyze_file", &params, id);
        let body = serde_json::to_vec(&req)?;
        write_frame(&mut self.writer, &Frame { body })?;

        let resp_frame = read_frame(&mut self.reader, self.ceiling)?;
        let resp: ResponseEnvelope = serde_json::from_slice(&resp_frame.body)?;
        if resp.id != id {
            return Err(HostError::Protocol(ProtocolError {
                code: -32_600,
                message: format!(
                    "analyze_file response id {} does not match request id {id}",
                    resp.id
                ),
                data: None,
            }));
        }
        let result_val = match resp.payload {
            ResponsePayload::Result(v) => v,
            ResponsePayload::Error(e) => return Err(HostError::Protocol(e)),
        };

        let entities_raw: Vec<serde_json::Value> = result_val
            .get("entities")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let plugin_id = self.manifest.plugin.plugin_id.clone();
        let declared_kinds = self.manifest.ontology.entity_kinds.clone();
        let project_root = self.project_root.clone();

        let mut accepted = Vec::new();

        for raw_val in entities_raw {
            let raw: RawEntity = match serde_json::from_value(raw_val) {
                Ok(e) => e,
                Err(_) => continue, // malformed entity — skip silently
            };

            // 1. Ontology check (ADR-022).
            if !declared_kinds.contains(&raw.kind) {
                self.findings
                    .push(HostFinding::undeclared_kind(&raw.kind, &raw.qualified_name));
                continue;
            }

            // 2. Identity check (UQ-WP2-11).
            let expected_id = match entity_id(&plugin_id, &raw.kind, &raw.qualified_name) {
                Ok(eid) => eid,
                Err(e) => {
                    self.findings.push(HostFinding::entity_id_mismatch(
                        &raw.id,
                        &format!("<invalid: {e}>"),
                    ));
                    continue;
                }
            };
            if raw.id != expected_id.as_str() {
                self.findings.push(HostFinding::entity_id_mismatch(
                    &raw.id,
                    expected_id.as_str(),
                ));
                continue;
            }

            // 3. Jail check (ADR-021 §2a).
            let candidate = Path::new(&raw.source.file_path);
            let jailed = match jail_to_string(&project_root, candidate) {
                Ok(p) => p,
                Err(JailError::EscapedRoot { ref offending }) => {
                    let s = offending.to_string_lossy().into_owned();
                    self.findings.push(HostFinding::path_escape(&s));
                    let state = self.path_breaker.record_escape();
                    if state == BreakerState::Tripped {
                        self.findings.push(HostFinding::disabled_path_escape());
                        let _ = self.do_shutdown();
                        return Err(HostError::PathEscapeBreakerTripped);
                    }
                    continue;
                }
                Err(JailError::Io(_)) => {
                    // File does not exist — drop entity, don't kill.
                    self.findings
                        .push(HostFinding::path_escape(&raw.source.file_path));
                    continue;
                }
                Err(JailError::NonUtf8Path { ref offending }) => {
                    let s = offending.to_string_lossy().into_owned();
                    self.findings.push(HostFinding::path_escape(&s));
                    continue;
                }
            };

            // 4. Entity cap check (ADR-021 §2c).
            if let Err(e) = self.entity_cap.try_admit(1) {
                self.findings.push(HostFinding::entity_cap_exceeded_finding(
                    e.cap,
                    e.would_reach,
                ));
                let _ = self.do_shutdown();
                return Err(HostError::EntityCapExceeded(e));
            }

            accepted.push(AcceptedEntity {
                id: expected_id,
                kind: raw.kind.clone(),
                qualified_name: raw.qualified_name.clone(),
                source_file_path: jailed,
                raw,
            });
        }

        Ok(accepted)
    }

    /// Send `shutdown` request followed by the `exit` notification.
    ///
    /// # Errors
    ///
    /// Returns transport / serde errors if the shutdown exchange fails.
    pub fn shutdown(&mut self) -> Result<(), HostError> {
        self.do_shutdown()
    }

    /// Drain the accumulated findings, leaving the internal list empty.
    pub fn take_findings(&mut self) -> Vec<HostFinding> {
        std::mem::take(&mut self.findings)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn next_id(&mut self) -> i64 {
        let id = self.next_request_id;
        self.next_request_id += 1;
        id
    }

    fn do_shutdown(&mut self) -> Result<(), HostError> {
        let id = self.next_id();
        let req = make_request("shutdown", &ShutdownParams {}, id);
        let body = serde_json::to_vec(&req)?;
        write_frame(&mut self.writer, &Frame { body })?;

        let resp_frame = read_frame(&mut self.reader, self.ceiling)?;
        let _resp: ResponseEnvelope = serde_json::from_slice(&resp_frame.body)?;

        let note = make_notification("exit", &ExitNotification {});
        let body = serde_json::to_vec(&note)?;
        write_frame(&mut self.writer, &Frame { body })?;

        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tempfile::TempDir;

    use super::*;
    use crate::plugin::mock::MockPlugin;
    use crate::plugin::{AnalyzeFileParams, InitializeParams};

    // ── Manifest fixtures ─────────────────────────────────────────────────────

    fn compliant_manifest() -> Manifest {
        let toml = r#"
[plugin]
name = "mock-plugin"
plugin_id = "mock"
version = "0.1.0"
protocol_version = "1.0"
executable = "mock-plugin"
language = "mock"
extensions = ["mock"]

[capabilities.runtime]
expected_max_rss_mb = 256
expected_entities_per_file = 100
wardline_aware = false
reads_outside_project_root = false

[ontology]
entity_kinds = ["function"]
edge_kinds = []
rule_id_prefix = "CLA-MOCK-"
ontology_version = "0.1.0"
"#;
        crate::plugin::parse_manifest(toml.as_bytes()).expect("valid compliant manifest")
    }

    fn reads_outside_manifest() -> Manifest {
        let toml = r#"
[plugin]
name = "mock-plugin"
plugin_id = "mock"
version = "0.1.0"
protocol_version = "1.0"
executable = "mock-plugin"
language = "mock"
extensions = ["mock"]

[capabilities.runtime]
expected_max_rss_mb = 256
expected_entities_per_file = 100
wardline_aware = false
reads_outside_project_root = true

[ontology]
entity_kinds = ["function"]
edge_kinds = []
rule_id_prefix = "CLA-MOCK-"
ontology_version = "0.1.0"
"#;
        crate::plugin::parse_manifest(toml.as_bytes()).expect("valid reads-outside manifest")
    }

    // ── Full end-to-end helper ────────────────────────────────────────────────

    /// Wire a `PluginHost` to a `MockPlugin` end-to-end and drive the full
    /// handshake. After this call both are in the "Ready" state.
    ///
    /// Strategy: use a temporary "pristine" mock to generate the initialize
    /// response bytes (which the host needs to read during `handshake()`), then
    /// pass those bytes to the host's reader. After `handshake()` completes, the
    /// host's writer contains `[initialize_request | initialized_notification]`.
    /// We identify the boundary by computing the request length independently and
    /// forward only the initialized notification to `mock` (the test mock) so it
    /// transitions from `Initialized` to `Ready`.
    ///
    /// Returns `(host, project_dir)`.
    fn connect_and_handshake(
        manifest: Manifest,
        mock: &mut MockPlugin,
    ) -> (PluginHost<Cursor<Vec<u8>>, Vec<u8>>, TempDir) {
        let project_dir = TempDir::new().expect("tmpdir");

        // Step 1: use a fresh "response-only" mock to generate the initialize
        // response frame. This mock is separate from the test mock.
        let mut resp_mock = MockPlugin::new_compliant();
        let init_req = crate::plugin::protocol::make_request(
            "initialize",
            &InitializeParams {
                protocol_version: "1.0".to_owned(),
                project_root: project_dir.path().to_string_lossy().into_owned(),
            },
            1,
        );
        let init_req_body = serde_json::to_vec(&init_req).unwrap();
        write_frame(
            resp_mock.stdin(),
            &Frame {
                body: init_req_body.clone(),
            },
        )
        .unwrap();
        resp_mock.tick().expect("resp_mock tick for initialize");
        let init_resp_bytes = drain_mock_output(&mut resp_mock);

        // Step 2: also drive the test mock (mock) through initialize so it is in
        // Initialized state, ready to receive the initialized notification.
        write_frame(
            mock.stdin(),
            &Frame {
                body: init_req_body,
            },
        )
        .unwrap();
        mock.tick().expect("mock tick for initialize");
        drain_mock_output(mock); // consume mock's initialize response (we don't use it)

        // Step 3: build the host with the pre-generated initialize response.
        let reader = Cursor::new(init_resp_bytes);
        let writer: Vec<u8> = Vec::new();
        let mut host =
            PluginHost::connect(manifest, project_dir.path(), reader, writer).expect("connect");
        host.next_request_id = 1; // match the id we pre-sent (id=1)

        // Step 4: run handshake(). It reads the initialize response, validates
        // the manifest, then writes [initialize_request | initialized_notification]
        // into host.writer. We need to forward only the initialized notification
        // to mock.
        //
        // To find the boundary, compute the framed initialize_request length
        // independently (same bytes the host sends).
        let init_req_framed_len = {
            let mut buf: Vec<u8> = Vec::new();
            let init_req2 = crate::plugin::protocol::make_request(
                "initialize",
                &InitializeParams {
                    protocol_version: "1.0".to_owned(),
                    project_root: project_dir.path().to_string_lossy().into_owned(),
                },
                1,
            );
            let body = serde_json::to_vec(&init_req2).unwrap();
            write_frame(&mut buf, &Frame { body }).unwrap();
            buf.len()
        };

        host.handshake().expect("handshake must succeed");

        // host.writer = [initialize_req_frame | initialized_notification_frame]
        // Forward only the initialized notification.
        let initialized_bytes = host.writer[init_req_framed_len..].to_vec();
        mock.stdin().extend_from_slice(&initialized_bytes);
        mock.tick().expect("mock tick for initialized");

        (host, project_dir)
    }

    // ── T2: reads_outside_project_root refusal ────────────────────────────────

    /// T2: manifest with `reads_outside_project_root = true` is refused at
    /// handshake. Host emits `CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY`,
    /// sends `shutdown` + `exit`, and no `analyze_file` is dispatched.
    #[test]
    fn t2_reads_outside_project_root_refused_at_handshake() {
        let manifest = reads_outside_manifest();
        let project_dir = TempDir::new().expect("tmpdir");
        let mut mock = MockPlugin::new_compliant();

        // Prepare: build all mock responses the host will need:
        // initialize response, and shutdown response (for do_shutdown()).
        let mut all_responses: Vec<u8> = Vec::new();

        // initialize response
        {
            let req = crate::plugin::protocol::make_request(
                "initialize",
                &InitializeParams {
                    protocol_version: "1.0".to_owned(),
                    project_root: project_dir.path().to_string_lossy().into_owned(),
                },
                1,
            );
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("tick initialize");
            let end = mock.stdout().get_ref().len() as u64;
            all_responses.extend_from_slice(mock.stdout().get_ref());
            mock.stdout().set_position(end);
        }

        // shutdown response (id=2, since handshake used id=1)
        {
            let req = crate::plugin::protocol::make_request(
                "shutdown",
                &crate::plugin::protocol::ShutdownParams {},
                2,
            );
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("tick shutdown");
            let end = mock.stdout().get_ref().len() as u64;
            let start = mock.stdout().position();
            all_responses.extend_from_slice(
                &mock.stdout().get_ref()
                    [usize::try_from(start).unwrap()..usize::try_from(end).unwrap()],
            );
            mock.stdout().set_position(end);
        }

        let reader = Cursor::new(all_responses);
        let writer: Vec<u8> = Vec::new();
        let mut host =
            PluginHost::connect(manifest, project_dir.path(), reader, writer).expect("connect");
        host.next_request_id = 1;

        let err = host
            .handshake()
            .expect_err("handshake must fail for reads_outside=true");
        assert!(
            matches!(err, HostError::Handshake(_)),
            "error must be Handshake variant; got: {err:?}"
        );

        let findings = host.take_findings();
        assert!(
            !findings.is_empty(),
            "must have at least one finding after refusal"
        );
        assert!(
            findings
                .iter()
                .any(|f| f.subcode == FINDING_UNSUPPORTED_CAPABILITY),
            "must have CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY; got: {findings:?}"
        );

        // Verify that no analyze_file was sent: host writer should not contain
        // "analyze_file". (Writer holds bytes the host sent after the reader was built.)
        let written = String::from_utf8_lossy(&host.writer);
        assert!(
            !written.contains("analyze_file"),
            "analyze_file must not be sent after capability refusal; writer contained: {written}"
        );
    }

    // ── T3: ontology-boundary enforcement ────────────────────────────────────

    /// T3: plugin emits entity with `kind: "unknown"` not in manifest ontology.
    /// Host drops it and emits `CLA-INFRA-PLUGIN-UNDECLARED-KIND`.
    #[test]
    fn t3_undeclared_kind_is_dropped_with_finding() {
        let manifest = compliant_manifest(); // entity_kinds = ["function"]
        let mut mock = MockPlugin::new_undeclared_kind();
        let (mut host, project_dir) = connect_and_handshake(manifest, &mut mock);

        let sample = project_dir.path().join("sample.mock");
        std::fs::write(&sample, b"").unwrap();

        // Prepare: add analyze_file response to reader.
        // The mock is now in Ready state and will respond to analyze_file.
        {
            let req = crate::plugin::protocol::make_request(
                "analyze_file",
                &AnalyzeFileParams {
                    file_path: sample.to_string_lossy().into_owned(),
                },
                host.next_request_id,
            );
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("mock tick for analyze_file");
        }

        // Append analyze_file response to host reader.
        let end = mock.stdout().get_ref().len() as u64;
        let start = mock.stdout().position();
        let new_bytes = mock.stdout().get_ref()
            [usize::try_from(start).unwrap()..usize::try_from(end).unwrap()]
            .to_vec();
        mock.stdout().set_position(end);
        let pos_before = host.reader.position();
        let old_end = host.reader.get_ref().len() as u64;
        host.reader.get_mut().extend_from_slice(&new_bytes);
        if pos_before == old_end {
            host.reader.set_position(old_end);
        }

        let result = host
            .analyze_file(&sample)
            .expect("analyze_file must not error");

        assert!(
            result.is_empty(),
            "undeclared-kind entity must be dropped; got {} accepted",
            result.len()
        );
        let findings = host.take_findings();
        assert!(
            findings
                .iter()
                .any(|f| f.subcode == FINDING_UNDECLARED_KIND),
            "must have CLA-INFRA-PLUGIN-UNDECLARED-KIND; got: {findings:?}"
        );
    }

    // ── T4: identity-mismatch rejection ──────────────────────────────────────

    /// T4: plugin emits entity whose `id` doesn't match
    /// `entity_id(plugin_id, kind, qualified_name)`. Host drops it and emits
    /// `CLA-INFRA-PLUGIN-ENTITY-ID-MISMATCH`.
    #[test]
    fn t4_identity_mismatch_drops_entity_with_finding() {
        let manifest = compliant_manifest();
        let mut mock = MockPlugin::new_id_mismatch();
        let (mut host, project_dir) = connect_and_handshake(manifest, &mut mock);

        let sample = project_dir.path().join("sample.mock");
        std::fs::write(&sample, b"").unwrap();

        // Feed analyze_file response into reader.
        {
            let req = crate::plugin::protocol::make_request(
                "analyze_file",
                &AnalyzeFileParams {
                    file_path: sample.to_string_lossy().into_owned(),
                },
                host.next_request_id,
            );
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("tick analyze_file");
        }
        append_mock_output_to_host_reader(&mut mock, &mut host.reader);

        let result = host
            .analyze_file(&sample)
            .expect("analyze_file must not error");
        assert!(result.is_empty(), "id-mismatch entity must be dropped");
        let findings = host.take_findings();
        assert!(
            findings
                .iter()
                .any(|f| f.subcode == FINDING_ENTITY_ID_MISMATCH),
            "must have CLA-INFRA-PLUGIN-ENTITY-ID-MISMATCH; got: {findings:?}"
        );
    }

    // ── T5: path-jail drop-not-kill ───────────────────────────────────────────

    /// T5: plugin emits one entity with a source path that escapes the jail.
    /// Host drops the entity, emits `CLA-INFRA-PLUGIN-PATH-ESCAPE`, plugin
    /// stays alive (no kill error returned).
    #[test]
    fn t5_single_path_escape_drops_entity_plugin_survives() {
        let manifest = compliant_manifest();
        let mut mock = MockPlugin::new_escaping_path(1);
        let (mut host, project_dir) = connect_and_handshake(manifest, &mut mock);

        let sample = project_dir.path().join("sample.mock");
        std::fs::write(&sample, b"").unwrap();

        {
            let req = crate::plugin::protocol::make_request(
                "analyze_file",
                &AnalyzeFileParams {
                    file_path: sample.to_string_lossy().into_owned(),
                },
                host.next_request_id,
            );
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("tick analyze_file");
        }
        append_mock_output_to_host_reader(&mut mock, &mut host.reader);

        let result = host
            .analyze_file(&sample)
            .expect("analyze_file must not return error for 1 escape");
        assert!(result.is_empty(), "escaping-path entity must be dropped");
        let findings = host.take_findings();
        assert!(
            findings.iter().any(|f| f.subcode == FINDING_PATH_ESCAPE),
            "must have CLA-INFRA-PLUGIN-PATH-ESCAPE; got: {findings:?}"
        );
        assert!(
            !findings
                .iter()
                .any(|f| f.subcode == FINDING_DISABLED_PATH_ESCAPE),
            "breaker must NOT trip for a single escape"
        );
    }

    // ── T6: path-escape sub-breaker trip ──────────────────────────────────────

    /// T6: plugin emits 11 entities each with an escaping path. On the 11th
    /// the breaker trips; host kills the plugin and emits
    /// `CLA-INFRA-PLUGIN-DISABLED-PATH-ESCAPE`.
    #[test]
    fn t6_eleven_path_escapes_trip_breaker() {
        let manifest = compliant_manifest();
        let mut mock = MockPlugin::new_escaping_path(11);
        let (mut host, project_dir) = connect_and_handshake(manifest, &mut mock);

        let sample = project_dir.path().join("sample.mock");
        std::fs::write(&sample, b"").unwrap();

        // Prepare shutdown response for the mock too (do_shutdown() will be called).
        // The mock in Ready state will respond to shutdown.
        {
            let req = crate::plugin::protocol::make_request(
                "analyze_file",
                &AnalyzeFileParams {
                    file_path: sample.to_string_lossy().into_owned(),
                },
                host.next_request_id,
            );
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("tick analyze_file");
        }
        let analyze_response_bytes = drain_mock_output(&mut mock);

        // Also pre-generate the shutdown response that do_shutdown() will need.
        // do_shutdown() uses id = next_request_id + 1 (after analyze_file uses one id).
        let shutdown_id = host.next_request_id + 1;
        {
            let req =
                crate::plugin::protocol::make_request("shutdown", &ShutdownParams {}, shutdown_id);
            let body = serde_json::to_vec(&req).unwrap();
            write_frame(mock.stdin(), &Frame { body }).unwrap();
            mock.tick().expect("tick shutdown");
        }
        let shutdown_response_bytes = drain_mock_output(&mut mock);

        // Load both into the host reader in order: analyze_file response, then shutdown response.
        let mut all_bytes = analyze_response_bytes;
        all_bytes.extend_from_slice(&shutdown_response_bytes);
        let old_end = host.reader.get_ref().len() as u64;
        let pos_before = host.reader.position();
        host.reader.get_mut().extend_from_slice(&all_bytes);
        if pos_before == old_end {
            host.reader.set_position(old_end);
        }

        let err = host
            .analyze_file(&sample)
            .expect_err("11 escapes must return error");
        assert!(
            matches!(err, HostError::PathEscapeBreakerTripped),
            "error must be PathEscapeBreakerTripped; got: {err:?}"
        );
        let findings = host.take_findings();
        assert!(
            findings
                .iter()
                .any(|f| f.subcode == FINDING_DISABLED_PATH_ESCAPE),
            "must have CLA-INFRA-PLUGIN-DISABLED-PATH-ESCAPE; got: {findings:?}"
        );
    }

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn append_mock_output_to_host_reader(mock: &mut MockPlugin, host_reader: &mut Cursor<Vec<u8>>) {
        let new_bytes = drain_mock_output(mock);
        let old_pos = host_reader.position();
        let old_end = host_reader.get_ref().len() as u64;
        host_reader.get_mut().extend_from_slice(&new_bytes);
        if old_pos == old_end {
            host_reader.set_position(old_end);
        }
    }

    fn drain_mock_output(mock: &mut MockPlugin) -> Vec<u8> {
        let end = mock.stdout().get_ref().len() as u64;
        let start = mock.stdout().position();
        let bytes = mock.stdout().get_ref()
            [usize::try_from(start).unwrap()..usize::try_from(end).unwrap()]
            .to_vec();
        mock.stdout().set_position(end);
        bytes
    }
}

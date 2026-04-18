//! Plugin-host facade.
//!
//! Submodules are added per WP2 task:
//!   - `manifest`   — Task 1: `plugin.toml` parser + validator (L5, ADR-021/ADR-022).
//!   - `protocol`   — Task 2: JSON-RPC 2.0 typed envelopes + param/result structs (L4).
//!   - `transport`  — Task 2: LSP-style Content-Length framing (L4).
//!   - `jail`       — Task 4: path-jail enforcement (ADR-021 §2a).
//!   - `limits`     — Task 4: core-enforced ceilings and circuit-breakers (ADR-021 §2b–§2d).
//!   - `discovery`  — Task 5: `$PATH` scanning for `clarion-plugin-*` executables (L9, ADR-021 §L9).
//!   - `host`       — Task 6: plugin-host supervisor (ADR-021 §Layer 2, ADR-022, UQ-WP2-11).

pub mod discovery;
pub mod host;
pub mod jail;
pub mod limits;
pub mod manifest;
#[cfg(test)]
pub(crate) mod mock;
pub mod protocol;
pub mod transport;

pub use discovery::{DiscoveredPlugin, DiscoveryError, discover, discover_on_path};
pub use host::{AcceptedEntity, HostError, HostFinding, PluginHost};
pub use jail::{JailError, jail, jail_to_string};
pub use limits::{
    BreakerState, CapExceeded, ContentLengthCeiling, DEFAULT_MAX_RSS_MIB, EntityCountCap,
    FINDING_DISABLED_PATH_ESCAPE, FINDING_ENTITY_CAP, FINDING_FRAME_OVERSIZE, FINDING_OOM_KILLED,
    FINDING_PATH_ESCAPE, PathEscapeBreaker, apply_prlimit_as, effective_rss_mib,
};
pub use manifest::{Manifest, ManifestError, parse_manifest};
pub use protocol::{
    AnalyzeFileParams, AnalyzeFileResult, ExitNotification, InitializeParams, InitializeResult,
    InitializedNotification, JsonRpcVersion, NotificationEnvelope, ProtocolError, RequestEnvelope,
    ResponseEnvelope, ResponsePayload, ShutdownParams, ShutdownResult, make_notification,
    make_request,
};
pub use transport::{Frame, TransportError, read_frame, write_frame};

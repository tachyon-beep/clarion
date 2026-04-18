//! Plugin-host facade.
//!
//! Submodules are added per WP2 task:
//!   - `manifest`  — Task 1: `plugin.toml` parser + validator (L5, ADR-021/ADR-022).
//!   - `protocol`  — Task 2: JSON-RPC 2.0 typed envelopes + param/result structs (L4).
//!   - `transport` — Task 2: LSP-style Content-Length framing (L4).

pub mod manifest;
#[cfg(test)]
pub(crate) mod mock;
pub mod protocol;
pub mod transport;

pub use manifest::{Manifest, ManifestError, parse_manifest};
pub use protocol::{
    AnalyzeFileParams, AnalyzeFileResult, ExitNotification, InitializeParams, InitializeResult,
    InitializedNotification, JsonRpcVersion, NotificationEnvelope, ProtocolError, RequestEnvelope,
    ResponseEnvelope, ResponsePayload, ShutdownParams, ShutdownResult, make_notification,
    make_request,
};
pub use transport::{Frame, TransportError, read_frame, write_frame};

//! clarion-core — domain types, identifiers, and provider traits.
//!
//! This crate is dependency-light and contains no I/O. Storage and CLI
//! crates depend on it; it depends on neither.

pub mod entity_id;
pub mod llm_provider;
pub mod plugin;

pub use entity_id::{EntityId, EntityIdError, entity_id};
pub use llm_provider::{LlmProvider, NoopProvider};
pub use plugin::{
    // protocol (Task 2)
    AnalyzeFileParams,
    AnalyzeFileResult,
    ExitNotification,
    // transport (Task 2)
    Frame,
    InitializeParams,
    InitializeResult,
    InitializedNotification,
    JsonRpcVersion,
    // manifest (Task 1)
    Manifest,
    ManifestError,
    NotificationEnvelope,
    ProtocolError,
    RequestEnvelope,
    ResponseEnvelope,
    ResponsePayload,
    ShutdownParams,
    ShutdownResult,
    TransportError,
    make_notification,
    make_request,
    parse_manifest,
    read_frame,
    write_frame,
};

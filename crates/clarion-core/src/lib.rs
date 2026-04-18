//! clarion-core — domain types, identifiers, and provider traits.
//!
//! This crate is dependency-light and contains no I/O. Storage and CLI
//! crates depend on it; it depends on neither.

pub mod entity_id;
pub mod llm_provider;

pub use entity_id::{EntityId, EntityIdError, entity_id};
pub use llm_provider::{LlmProvider, NoopProvider};

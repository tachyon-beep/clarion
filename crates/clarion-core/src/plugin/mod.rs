//! Plugin-host facade.
//!
//! Submodules are added per WP2 task:
//!   - `manifest` — Task 1: `plugin.toml` parser + validator (L5, ADR-021/ADR-022).

pub mod manifest;

pub use manifest::{Manifest, ManifestError, parse_manifest};

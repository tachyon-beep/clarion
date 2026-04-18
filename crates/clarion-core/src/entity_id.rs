//! Entity-ID assembler.
//!
//! Per ADR-003 + ADR-022, every Clarion entity has a stable 3-segment ID:
//! `{plugin_id}:{kind}:{canonical_qualified_name}`.
//!
//! - `plugin_id` and `kind` must match the grammar `[a-z][a-z0-9_]*`.
//! - `canonical_qualified_name` is opaque to this assembler: its internal
//!   shape is the emitting plugin's concern (dotted qualnames for the
//!   Python plugin; content-addressed for core-minted file entities).
//! - No segment may contain a literal `:` — the separator is reserved.
//!   ADR-022's grammar precludes it in `plugin_id`/`kind`; `canonical_qualified_name`
//!   is checked at assembly time (UQ-WP1-07).

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(String);

impl EntityId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntityIdError {
    #[error("segment {field} empty")]
    EmptySegment { field: &'static str },

    #[error("segment {field} violates ADR-022 grammar [a-z][a-z0-9_]*: {value:?}")]
    GrammarViolation { field: &'static str, value: String },

    #[error("segment {field} contains reserved ':' separator: {value:?}")]
    SegmentContainsColon { field: &'static str, value: String },
}

/// Assemble an [`EntityId`] from its three segments.
///
/// `plugin_id` and `kind` are validated against the ADR-022 grammar.
/// `canonical_qualified_name` is opaque but may not contain `:`.
pub fn entity_id(
    plugin_id: &str,
    kind: &str,
    canonical_qualified_name: &str,
) -> Result<EntityId, EntityIdError> {
    validate_grammar("plugin_id", plugin_id)?;
    validate_grammar("kind", kind)?;
    validate_no_colon("canonical_qualified_name", canonical_qualified_name)?;
    if canonical_qualified_name.is_empty() {
        return Err(EntityIdError::EmptySegment {
            field: "canonical_qualified_name",
        });
    }
    Ok(EntityId(format!(
        "{plugin_id}:{kind}:{canonical_qualified_name}"
    )))
}

fn validate_grammar(field: &'static str, value: &str) -> Result<(), EntityIdError> {
    if value.is_empty() {
        return Err(EntityIdError::EmptySegment { field });
    }
    validate_no_colon(field, value)?;
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        // Unreachable: emptiness is checked above, but the defensive branch
        // avoids any panic path and satisfies clippy::unwrap_in_result.
        return Err(EntityIdError::EmptySegment { field });
    };
    if !first.is_ascii_lowercase() {
        return Err(EntityIdError::GrammarViolation {
            field,
            value: value.to_owned(),
        });
    }
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(EntityIdError::GrammarViolation {
                field,
                value: value.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_no_colon(field: &'static str, value: &str) -> Result<(), EntityIdError> {
    if value.contains(':') {
        return Err(EntityIdError::SegmentContainsColon {
            field,
            value: value.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_level_function() {
        let id = entity_id("python", "function", "demo.hello").unwrap();
        assert_eq!(id.as_str(), "python:function:demo.hello");
    }

    #[test]
    fn class_method() {
        let id = entity_id("python", "function", "demo.Foo.bar").unwrap();
        assert_eq!(id.as_str(), "python:function:demo.Foo.bar");
    }

    #[test]
    fn nested_function_uses_python_locals_marker() {
        let id = entity_id("python", "function", "demo.outer.<locals>.inner").unwrap();
        assert_eq!(id.as_str(), "python:function:demo.outer.<locals>.inner");
    }

    #[test]
    fn core_reserved_file_kind() {
        // The file-entity canonical_qualified_name shape is core-file-discovery's
        // concern (per detailed-design.md §2:229). Sprint 1 only tests the
        // assembler's concatenation; `src/demo.py` is a stand-in.
        let id = entity_id("core", "file", "src/demo.py").unwrap();
        assert_eq!(id.as_str(), "core:file:src/demo.py");
    }

    #[test]
    fn core_reserved_subsystem_kind() {
        let id = entity_id("core", "subsystem", "a1b2c3d4").unwrap();
        assert_eq!(id.as_str(), "core:subsystem:a1b2c3d4");
    }

    #[test]
    fn rejects_empty_plugin_id() {
        assert_eq!(
            entity_id("", "function", "demo.hello"),
            Err(EntityIdError::EmptySegment { field: "plugin_id" }),
        );
    }

    #[test]
    fn rejects_empty_kind() {
        assert_eq!(
            entity_id("python", "", "demo.hello"),
            Err(EntityIdError::EmptySegment { field: "kind" }),
        );
    }

    #[test]
    fn rejects_empty_qualified_name() {
        assert_eq!(
            entity_id("python", "function", ""),
            Err(EntityIdError::EmptySegment {
                field: "canonical_qualified_name",
            }),
        );
    }

    #[test]
    fn rejects_uppercase_plugin_id() {
        assert!(matches!(
            entity_id("Python", "function", "demo.hello"),
            Err(EntityIdError::GrammarViolation {
                field: "plugin_id",
                ..
            })
        ));
    }

    #[test]
    fn rejects_digit_prefixed_kind() {
        assert!(matches!(
            entity_id("python", "1function", "demo.hello"),
            Err(EntityIdError::GrammarViolation { field: "kind", .. })
        ));
    }

    #[test]
    fn rejects_hyphen_in_kind() {
        assert!(matches!(
            entity_id("python", "func-tion", "demo.hello"),
            Err(EntityIdError::GrammarViolation { field: "kind", .. })
        ));
    }

    #[test]
    fn rejects_colon_in_qualified_name() {
        assert!(matches!(
            entity_id("python", "function", "demo:hello"),
            Err(EntityIdError::SegmentContainsColon {
                field: "canonical_qualified_name",
                ..
            })
        ));
    }

    #[test]
    fn rejects_colon_in_plugin_id() {
        // Defence in depth: grammar check rejects this, but the colon
        // check fires first and produces a more descriptive error.
        let err = entity_id("py:thon", "function", "demo.hello").unwrap_err();
        assert!(matches!(
            err,
            EntityIdError::SegmentContainsColon {
                field: "plugin_id",
                ..
            }
        ));
    }

    #[test]
    fn entity_id_serialises_as_string() {
        let id = entity_id("python", "function", "demo.hello").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"python:function:demo.hello\"");
    }
}

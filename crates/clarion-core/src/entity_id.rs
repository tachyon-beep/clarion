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

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct EntityId(String);

impl EntityId {
    /// Returns the entity ID as a string slice in its canonical 3-segment form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for EntityId {
    type Err = EntityIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        match parts.as_slice() {
            [plugin_id, kind, canonical_qualified_name] => {
                entity_id(plugin_id, kind, canonical_qualified_name)
            }
            _ => Err(EntityIdError::MalformedId {
                value: s.to_owned(),
            }),
        }
    }
}

impl<'de> serde::Deserialize<'de> for EntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let s = String::deserialize(deserializer)?;
        s.parse::<EntityId>().map_err(D::Error::custom)
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

    #[error("EntityId must have exactly 3 colon-separated segments, got: {value:?}")]
    MalformedId { value: String },
}

/// Assemble an [`EntityId`] from its three segments.
///
/// `plugin_id` and `kind` are validated against the ADR-022 grammar
/// (`[a-z][a-z0-9_]*`). `canonical_qualified_name` is opaque but may not
/// contain `:`.
///
/// # Errors
///
/// - [`EntityIdError::EmptySegment`] if any segment is empty.
/// - [`EntityIdError::GrammarViolation`] if `plugin_id` or `kind` does not
///   match the ADR-022 grammar.
/// - [`EntityIdError::SegmentContainsColon`] if any segment contains `:`
///   (colon is reserved as the segment separator; UQ-WP1-07).
pub fn entity_id(
    plugin_id: &str,
    kind: &str,
    canonical_qualified_name: &str,
) -> Result<EntityId, EntityIdError> {
    validate_grammar("plugin_id", plugin_id)?;
    validate_grammar("kind", kind)?;
    if canonical_qualified_name.is_empty() {
        return Err(EntityIdError::EmptySegment {
            field: "canonical_qualified_name",
        });
    }
    validate_no_colon("canonical_qualified_name", canonical_qualified_name)?;
    Ok(EntityId(format!(
        "{plugin_id}:{kind}:{canonical_qualified_name}"
    )))
}

/// Validate that a string matches the ADR-022 identifier grammar `[a-z][a-z0-9_]*`.
///
/// Used by both the entity-ID assembler and the manifest parser to enforce a
/// single canonical check — no divergent copies.
pub(crate) fn validate_kind_grammar(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
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

    #[derive(Debug, serde::Deserialize)]
    struct FixtureRow {
        plugin_id: String,
        kind: String,
        canonical_qualified_name: String,
        expected_entity_id: String,
    }

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

    #[test]
    fn parse_roundtrip_via_from_str() {
        use std::str::FromStr;
        let id = entity_id("python", "function", "demo.hello").unwrap();
        let parsed = EntityId::from_str(id.as_str()).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn from_str_rejects_fewer_than_three_segments() {
        use std::str::FromStr;
        let err = EntityId::from_str("python:function").unwrap_err();
        assert!(matches!(err, EntityIdError::MalformedId { .. }));
    }

    #[test]
    fn from_str_rejects_empty_segments_via_underlying_validator() {
        use std::str::FromStr;
        // splitn(3, ':') on "::foo" yields ["", "", "foo"] — empty plugin_id
        let err = EntityId::from_str("::demo.hello").unwrap_err();
        assert!(matches!(
            err,
            EntityIdError::EmptySegment { field: "plugin_id" }
        ));
    }

    #[test]
    fn deserialize_validates_through_from_str() {
        // Valid input round-trips.
        let id: EntityId = serde_json::from_str("\"python:function:demo.hello\"").unwrap();
        assert_eq!(id.as_str(), "python:function:demo.hello");
    }

    #[test]
    fn deserialize_rejects_invalid_ids() {
        // An unstructured string must fail deserialisation now (pre-fix, it
        // would silently deserialise into a corrupt EntityId).
        let result: Result<EntityId, _> = serde_json::from_str("\"notanid\"");
        assert!(
            result.is_err(),
            "expected custom deserialiser to reject non-3-segment input"
        );
    }

    #[test]
    fn shared_fixture_byte_for_byte_parity() {
        // L2 byte-for-byte parity proof (WP3 Task 5 / UQ-WP3-08): this
        // test and `plugins/python/tests/test_entity_id.py::test_matches_shared_fixture`
        // consume the same `fixtures/entity_id.json` at the workspace root.
        // Divergence on either side fails CI. Retroactively earns the
        // signoff A.1.4 proof (WP1 ticked it against the fixture before
        // the file existed — WP3 Task 5 is where it lands).
        let fixture_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/entity_id.json");
        let contents = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|err| panic!("read fixture {}: {err}", fixture_path.display()));
        let rows: Vec<FixtureRow> =
            serde_json::from_str(&contents).expect("fixture parses as Vec<FixtureRow>");
        assert!(
            rows.len() >= 20,
            "fixture must have at least 20 rows; got {}",
            rows.len()
        );
        for row in &rows {
            let actual = entity_id(&row.plugin_id, &row.kind, &row.canonical_qualified_name)
                .unwrap_or_else(|err| panic!("row {row:?} failed to assemble: {err}"));
            assert_eq!(
                actual.as_str(),
                row.expected_entity_id,
                "mismatch on row {row:?}"
            );
        }
    }
}

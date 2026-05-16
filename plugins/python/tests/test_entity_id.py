"""Unit tests for the 3-segment EntityId assembler (WP3 Task 4).

Task 5 replaces the Python-side expected-string literals with rows from
the shared ``fixtures/entity_id.json`` — which WP1's Rust tests will
also consume — for the byte-for-byte L2 parity proof.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from clarion_plugin_python.entity_id import (
    EmptySegmentError,
    GrammarViolationError,
    SegmentContainsColonError,
    entity_id,
)

# Repo root is four parents up from this test file:
# plugins/python/tests/test_entity_id.py → ... → /repo
_REPO_ROOT = Path(__file__).resolve().parents[3]
_FIXTURE_PATH = _REPO_ROOT / "fixtures" / "entity_id.json"


def test_module_level_function_id() -> None:
    assert entity_id("python", "function", "demo.hello") == "python:function:demo.hello"


def test_class_method_id() -> None:
    assert entity_id("python", "function", "demo.Foo.bar") == "python:function:demo.Foo.bar"


def test_nested_function_id_carries_locals_marker() -> None:
    assert (
        entity_id("python", "function", "demo.outer.<locals>.inner")
        == "python:function:demo.outer.<locals>.inner"
    )


def test_core_file_id() -> None:
    assert entity_id("core", "file", "src/demo.py") == "core:file:src/demo.py"


def test_core_subsystem_id() -> None:
    assert entity_id("core", "subsystem", "a1b2c3d4") == "core:subsystem:a1b2c3d4"


def test_rejects_empty_plugin_id() -> None:
    with pytest.raises(EmptySegmentError) as exc_info:
        entity_id("", "function", "demo.hello")
    assert exc_info.value.field == "plugin_id"


def test_rejects_empty_kind() -> None:
    with pytest.raises(EmptySegmentError) as exc_info:
        entity_id("python", "", "demo.hello")
    assert exc_info.value.field == "kind"


def test_rejects_empty_qualified_name() -> None:
    with pytest.raises(EmptySegmentError) as exc_info:
        entity_id("python", "function", "")
    assert exc_info.value.field == "canonical_qualified_name"


def test_rejects_uppercase_plugin_id() -> None:
    with pytest.raises(GrammarViolationError) as exc_info:
        entity_id("Python", "function", "demo.hello")
    assert exc_info.value.field == "plugin_id"


def test_rejects_digit_prefixed_kind() -> None:
    with pytest.raises(GrammarViolationError) as exc_info:
        entity_id("python", "1function", "demo.hello")
    assert exc_info.value.field == "kind"


def test_rejects_hyphen_in_kind() -> None:
    with pytest.raises(GrammarViolationError) as exc_info:
        entity_id("python", "func-tion", "demo.hello")
    assert exc_info.value.field == "kind"


def test_rejects_colon_in_qualified_name() -> None:
    with pytest.raises(SegmentContainsColonError) as exc_info:
        entity_id("python", "function", "demo:hello")
    assert exc_info.value.field == "canonical_qualified_name"


def test_rejects_colon_in_plugin_id() -> None:
    """Colon check fires before grammar check (defence in depth, matches Rust)."""
    with pytest.raises(SegmentContainsColonError) as exc_info:
        entity_id("py:thon", "function", "demo.hello")
    assert exc_info.value.field == "plugin_id"


def test_matches_shared_fixture() -> None:
    """UQ-WP3-08: byte-for-byte L2 parity with the Rust assembler.

    The same ``fixtures/entity_id.json`` is consumed by
    ``crates/clarion-core/src/entity_id.rs::tests::shared_fixture_byte_for_byte_parity``.
    If this test or the Rust test disagrees on any row, the ID scheme has
    drifted between languages — the cross-product identity join (ADR-018)
    would break silently. CI fails both sides in lockstep.
    """
    with _FIXTURE_PATH.open() as fh:
        fixture = json.load(fh)
    rows = fixture["entities"]
    assert len(rows) >= 20, f"fixture must have >=20 entity rows, got {len(rows)}"
    for row in rows:
        actual = entity_id(
            row["plugin_id"],
            row["kind"],
            row["canonical_qualified_name"],
        )
        assert actual == row["expected_entity_id"], f"mismatch for row {row!r}"


def test_matches_shared_contains_edge_fixture() -> None:
    """B.3 cross-language parity for contains-edge wire shape (ADR-026).

    Both Rust and Python read the same fixture rows and construct a
    contains-edge dict from (parent_id, child_id), asserting byte-for-byte
    equality with ``expected_wire``. Catches drift in the wire shape (e.g.
    one side accidentally adding ``source_byte_*`` keys).
    """
    with _FIXTURE_PATH.open() as fh:
        fixture = json.load(fh)
    edges = fixture["contains_edges"]
    assert len(edges) >= 3, f"fixture must have >=3 contains-edge rows, got {len(edges)}"
    for row in edges:
        wire = {
            "kind": "contains",
            "from_id": row["parent_id"],
            "to_id": row["child_id"],
        }
        assert wire == row["expected_wire"], f"mismatch for edge row {row!r}"
        # No source_byte_* fields per ADR-026 decision 3.
        assert "source_byte_start" not in wire
        assert "source_byte_end" not in wire

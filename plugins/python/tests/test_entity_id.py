"""Unit tests for the 3-segment EntityId assembler (WP3 Task 4).

Task 5 replaces the Python-side expected-string literals with rows from
the shared ``fixtures/entity_id.json`` — which WP1's Rust tests will
also consume — for the byte-for-byte L2 parity proof.
"""

from __future__ import annotations

import pytest

from clarion_plugin_python.entity_id import (
    EmptySegmentError,
    GrammarViolationError,
    SegmentContainsColonError,
    entity_id,
)


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

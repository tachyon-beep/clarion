"""Unit tests for the AST → function-entity extractor (WP3 Task 4)."""

from __future__ import annotations

from typing import TYPE_CHECKING

from clarion_plugin_python.extractor import (
    _module_source_range,
    extract,
    module_dotted_name,
)

if TYPE_CHECKING:
    import pytest


def test_empty_file_yields_one_module_entity() -> None:
    """B.2 Q1 supersession of Sprint-1 UQ-WP3-11: empty file produces one module entity, not [].

    The function-extraction part of UQ-WP3-11 still holds: zero *function* entities.
    """
    entities = extract("", "empty.py")
    assert len(entities) == 1
    assert entities[0]["kind"] == "module"
    assert entities[0].get("parse_status") == "ok"
    function_entities = [e for e in entities if e["kind"] == "function"]
    assert function_entities == []


def test_whitespace_only_file_yields_one_module_entity() -> None:
    """Whitespace + comment-only file → one module entity (parse_status='ok'), zero functions."""
    entities = extract("\n\n# just a comment\n", "empty.py")
    assert len(entities) == 1
    assert entities[0]["kind"] == "module"
    assert entities[0].get("parse_status") == "ok"


def test_module_level_function() -> None:
    entities = extract("def hello():\n    pass\n", "demo.py")
    function_entities = [e for e in entities if e["kind"] == "function"]
    assert len(function_entities) == 1
    entity = function_entities[0]
    assert entity["id"] == "python:function:demo.hello"
    assert entity["kind"] == "function"
    assert entity["qualified_name"] == "demo.hello"
    assert entity["source"]["file_path"] == "demo.py"
    assert entity["source"]["source_range"]["start_line"] == 1
    assert entity["source"]["source_range"]["start_col"] == 0


def test_class_method() -> None:
    entities = extract("class Foo:\n    def bar(self):\n        pass\n", "demo.py")
    function_entities = [e for e in entities if e["kind"] == "function"]
    assert len(function_entities) == 1
    assert function_entities[0]["id"] == "python:function:demo.Foo.bar"


def test_nested_function_emits_both_outer_and_inner() -> None:
    entities = extract("def outer():\n    def inner():\n        pass\n", "demo.py")
    function_ids = {e["id"] for e in entities if e["kind"] == "function"}
    assert function_ids == {
        "python:function:demo.outer",
        "python:function:demo.outer.<locals>.inner",
    }


def test_async_function() -> None:
    entities = extract("async def aloha():\n    pass\n", "demo.py")
    function_entities = [e for e in entities if e["kind"] == "function"]
    assert len(function_entities) == 1
    assert function_entities[0]["id"] == "python:function:demo.aloha"


def test_nested_class_method() -> None:
    source = "class Outer:\n    class Inner:\n        def method(self):\n            pass\n"
    entities = extract(source, "demo.py")
    function_entities = [e for e in entities if e["kind"] == "function"]
    assert len(function_entities) == 1
    assert function_entities[0]["id"] == "python:function:demo.Outer.Inner.method"


def test_syntax_error_emits_degraded_module_entity_and_logs_to_stderr(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """UQ-WP3-02 + B.2 Q1: SyntaxError files now emit a degraded module entity (was: empty list)."""
    result = extract("def :", "broken.py")
    assert len(result) == 1
    assert result[0]["kind"] == "module"
    assert result[0].get("parse_status") == "syntax_error"
    captured = capsys.readouterr()
    assert "broken.py" in captured.err


def test_src_prefix_stripped() -> None:
    """UQ-WP3-05: `src/pkg/module.py` → dotted module `pkg.module`."""
    entities = extract("def hello():\n    pass\n", "src/pkg/module.py")
    fn = next(e for e in entities if e["kind"] == "function")
    assert fn["qualified_name"] == "pkg.module.hello"


def test_init_py_collapsed_to_package_name() -> None:
    """UQ-WP3-06: `pkg/__init__.py` → dotted `pkg` (not `pkg.__init__`).

    ``source.file_path`` stays as the literal file path; the dotted module
    used for qualified_name is the package name only.
    """
    entities = extract("def pkg_helper():\n    pass\n", "pkg/__init__.py")
    fn = next(e for e in entities if e["kind"] == "function")
    assert fn["qualified_name"] == "pkg.pkg_helper"
    assert fn["source"]["file_path"] == "pkg/__init__.py"


def test_module_prefix_path_decouples_file_path_and_dotted_prefix() -> None:
    """server passes absolute file_path + relativised module_prefix_path."""
    entities = extract(
        "def hello():\n    pass\n",
        "/tmp/proj/demo.py",
        module_prefix_path="demo.py",
    )
    fn = next(e for e in entities if e["kind"] == "function")
    assert fn["source"]["file_path"] == "/tmp/proj/demo.py"
    assert fn["id"] == "python:function:demo.hello"
    assert fn["qualified_name"] == "demo.hello"


def test_module_dotted_name_helper() -> None:
    assert module_dotted_name("demo.py") == "demo"
    assert module_dotted_name("src/demo.py") == "demo"
    assert module_dotted_name("pkg/__init__.py") == "pkg"
    assert module_dotted_name("src/pkg/mod.py") == "pkg.mod"
    assert module_dotted_name("src/pkg/sub/mod.py") == "pkg.sub.mod"


def test_source_range_end_fields_populated() -> None:
    entities = extract("def f():\n    pass\n", "d.py")
    fn = next(e for e in entities if e["kind"] == "function")
    source_range = fn["source"]["source_range"]
    assert source_range["start_line"] == 1
    assert source_range["start_col"] == 0
    assert source_range["end_line"] == 2
    assert source_range["end_col"] >= 0


def test_module_source_range_no_trailing_newline() -> None:
    """File ending without `\\n` still produces correct end_line.

    `"a\\nb"` has one newline → end_line = 2.
    """
    rng = _module_source_range("a\nb")
    assert rng == {"start_line": 1, "start_col": 0, "end_line": 2, "end_col": 0}


def test_module_source_range_crlf() -> None:
    """CRLF-terminated file produces same end_line as LF (count('\\n') handles both)."""
    rng = _module_source_range("a\r\nb\r\n")
    # Two `\n`s → end_line = 3 (one past the last terminator).
    assert rng == {"start_line": 1, "start_col": 0, "end_line": 3, "end_col": 0}


def test_module_source_range_empty_string() -> None:
    """Empty source → end_line = 1 (count is 0; +1)."""
    rng = _module_source_range("")
    assert rng == {"start_line": 1, "start_col": 0, "end_line": 1, "end_col": 0}


def test_module_entity_emitted_for_every_call() -> None:
    """Q1: every analyze produces exactly one module entity."""
    entities = extract("def hello():\n    pass\n", "demo.py")
    module_entities = [e for e in entities if e["kind"] == "module"]
    assert len(module_entities) == 1
    module = module_entities[0]
    assert module["id"] == "python:module:demo"
    assert module["kind"] == "module"
    assert module["qualified_name"] == "demo"
    assert module["source"]["file_path"] == "demo.py"
    assert module["source"]["source_range"] == {
        "start_line": 1,
        "start_col": 0,
        "end_line": 3,  # "def hello():\n    pass\n" → 2 newlines + 1
        "end_col": 0,
    }
    assert module.get("parse_status") == "ok"


def test_module_entity_for_empty_file() -> None:
    """Q1: empty file produces one module entity (parse_status='ok' since ast.parse('') succeeds)."""
    entities = extract("", "empty.py")
    assert len(entities) == 1
    module = entities[0]
    assert module["kind"] == "module"
    assert module["id"] == "python:module:empty"
    assert module["source"]["source_range"] == {
        "start_line": 1,
        "start_col": 0,
        "end_line": 1,
        "end_col": 0,
    }
    assert module.get("parse_status") == "ok"


def test_module_entity_for_init_py_collapses_to_package() -> None:
    """`pkg/__init__.py` produces module entity at `python:module:pkg`."""
    entities = extract("", "pkg/__init__.py")
    assert len(entities) == 1
    module = entities[0]
    assert module["id"] == "python:module:pkg"
    assert module["qualified_name"] == "pkg"


def test_module_entity_for_syntax_error_file(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Q1: syntax-error file emits one module entity with parse_status='syntax_error'."""
    entities = extract("def :", "broken.py")
    assert len(entities) == 1, "syntax-error file emits only the module entity"
    module = entities[0]
    assert module["kind"] == "module"
    assert module["id"] == "python:module:broken"
    assert module.get("parse_status") == "syntax_error"
    # Source range covers the broken file (formula is uniform across parse_status values).
    assert module["source"]["source_range"] == {
        "start_line": 1,
        "start_col": 0,
        "end_line": 1,  # no `\n` in "def :"
        "end_col": 0,
    }
    captured = capsys.readouterr()
    assert "broken.py" in captured.err
    assert "syntax error" in captured.err


def test_top_level_init_py_skipped_with_stderr(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Top-level `__init__.py` (no package name) returns [] + one stderr line.

    `module_dotted_name("__init__.py")` returns "" (the empty stem case).
    Emitting an entity with empty qualified_name would crash the entity-ID
    assembler at crates/clarion-core/src/entity_id.rs:97-101.
    """
    entities = extract("def helper():\n    pass\n", "__init__.py")
    assert entities == []
    captured = capsys.readouterr()
    assert "__init__.py" in captured.err
    assert "top-level __init__.py has no package name" in captured.err

"""Unit tests for the AST → function-entity extractor (WP3 Task 4)."""

from __future__ import annotations

from typing import TYPE_CHECKING

from clarion_plugin_python.extractor import extract, module_dotted_name

if TYPE_CHECKING:
    import pytest


def test_empty_file_yields_zero_entities() -> None:
    """UQ-WP3-11: an empty .py file has zero functions (host must tolerate)."""
    assert extract("", "empty.py") == []


def test_whitespace_only_file_yields_zero_entities() -> None:
    assert extract("\n\n# just a comment\n", "empty.py") == []


def test_module_level_function() -> None:
    entities = extract("def hello():\n    pass\n", "demo.py")
    assert len(entities) == 1
    entity = entities[0]
    assert entity["id"] == "python:function:demo.hello"
    assert entity["kind"] == "function"
    assert entity["qualified_name"] == "demo.hello"
    assert entity["source"]["file_path"] == "demo.py"
    assert entity["source"]["source_range"]["start_line"] == 1
    assert entity["source"]["source_range"]["start_col"] == 0


def test_class_method() -> None:
    entities = extract("class Foo:\n    def bar(self):\n        pass\n", "demo.py")
    assert len(entities) == 1
    assert entities[0]["id"] == "python:function:demo.Foo.bar"


def test_nested_function_emits_both_outer_and_inner() -> None:
    entities = extract("def outer():\n    def inner():\n        pass\n", "demo.py")
    ids = {e["id"] for e in entities}
    assert ids == {
        "python:function:demo.outer",
        "python:function:demo.outer.<locals>.inner",
    }


def test_async_function() -> None:
    entities = extract("async def aloha():\n    pass\n", "demo.py")
    assert len(entities) == 1
    assert entities[0]["id"] == "python:function:demo.aloha"


def test_nested_class_method() -> None:
    source = "class Outer:\n    class Inner:\n        def method(self):\n            pass\n"
    entities = extract(source, "demo.py")
    assert len(entities) == 1
    assert entities[0]["id"] == "python:function:demo.Outer.Inner.method"


def test_syntax_error_yields_empty_list_and_logs_to_stderr(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """UQ-WP3-02: SyntaxError files are skipped + logged, not fatal."""
    result = extract("def :", "broken.py")
    assert result == []
    captured = capsys.readouterr()
    assert "broken.py" in captured.err


def test_src_prefix_stripped() -> None:
    """UQ-WP3-05: `src/pkg/module.py` → dotted module `pkg.module`."""
    entities = extract("def hello():\n    pass\n", "src/pkg/module.py")
    assert entities[0]["qualified_name"] == "pkg.module.hello"


def test_init_py_collapsed_to_package_name() -> None:
    """UQ-WP3-06: `pkg/__init__.py` → dotted `pkg` (not `pkg.__init__`).

    ``source.file_path`` stays as the literal file path; the dotted module
    used for qualified_name is the package name only.
    """
    entities = extract("def pkg_helper():\n    pass\n", "pkg/__init__.py")
    assert entities[0]["qualified_name"] == "pkg.pkg_helper"
    assert entities[0]["source"]["file_path"] == "pkg/__init__.py"


def test_module_prefix_path_decouples_file_path_and_dotted_prefix() -> None:
    """server passes absolute file_path + relativised module_prefix_path."""
    entities = extract(
        "def hello():\n    pass\n",
        "/tmp/proj/demo.py",
        module_prefix_path="demo.py",
    )
    assert entities[0]["source"]["file_path"] == "/tmp/proj/demo.py"
    assert entities[0]["id"] == "python:function:demo.hello"
    assert entities[0]["qualified_name"] == "demo.hello"


def test_module_dotted_name_helper() -> None:
    assert module_dotted_name("demo.py") == "demo"
    assert module_dotted_name("src/demo.py") == "demo"
    assert module_dotted_name("pkg/__init__.py") == "pkg"
    assert module_dotted_name("src/pkg/mod.py") == "pkg.mod"
    assert module_dotted_name("src/pkg/sub/mod.py") == "pkg.sub.mod"


def test_source_range_end_fields_populated() -> None:
    entities = extract("def f():\n    pass\n", "d.py")
    source_range = entities[0]["source"]["source_range"]
    assert source_range["start_line"] == 1
    assert source_range["start_col"] == 0
    assert source_range["end_line"] == 2
    assert source_range["end_col"] >= 0

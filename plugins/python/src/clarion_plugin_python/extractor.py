"""AST → function-entity extractor for the Python plugin (WP3 Task 4).

Walks a parsed Python file and emits one entity per ``FunctionDef`` /
``AsyncFunctionDef`` (Sprint 1 scope). Class, decorator, module, and
import/call edge emission is WP3-feature-complete scope and deliberately
out of band here.

Entity shape matches the Rust fixture plugin's wire layout
(``crates/clarion-plugin-fixture/src/main.rs``)::

    {
        "id": "python:function:...",
        "kind": "function",
        "qualified_name": "pkg.module.func",
        "module_path": "pkg/module.py",
        "source_range": {
            "start_line": 1, "start_col": 0,
            "end_line": 3, "end_col": 4,
        },
    }

``qualified_name`` is the dotted module prefix joined to Python's own
``__qualname__`` (reconstructed per L7). ``module_path`` is an entity
*property*, not part of the ID (ADR-003).

Behaviour:

- Empty or comment-only file → empty list (UQ-WP3-11).
- ``SyntaxError`` during ``ast.parse`` → empty list + one stderr log line
  (UQ-WP3-02). The run continues; WP4-era findings can later attach a
  ``CLA-PY-SYNTAX-ERROR`` annotation.
- Paths starting with ``src/`` have the prefix stripped (UQ-WP3-05).
- ``pkg/__init__.py`` files yield qualified_names rooted at ``pkg``
  (not ``pkg.__init__``) — UQ-WP3-06.
"""

from __future__ import annotations

import ast
import sys
from pathlib import PurePosixPath
from typing import Any

from clarion_plugin_python.entity_id import entity_id
from clarion_plugin_python.qualname import reconstruct_qualname

_PLUGIN_ID = "python"
_KIND = "function"


def module_dotted_name(module_path: str) -> str:
    """Derive the dotted module prefix from a root-relative source path.

    Rules:
    - Leading ``src/`` is stripped (UQ-WP3-05).
    - The ``.py`` suffix is dropped.
    - ``__init__`` filenames collapse to their containing package
      (UQ-WP3-06: ``pkg/__init__.py`` → ``pkg``).
    - Path separators become ``.``.

    ``module_path`` itself remains unchanged; it's stored on the entity
    as a property so WP4 can still find the file on disk.
    """
    parts = list(PurePosixPath(module_path).parts)
    if parts and parts[0] == "src":
        parts = parts[1:]
    if parts:
        last = parts[-1]
        if last.endswith(".py"):
            stem = last[:-3]
            if stem == "__init__":
                parts = parts[:-1]
            else:
                parts[-1] = stem
    return ".".join(parts)


def extract(source: str, module_path: str) -> list[dict[str, Any]]:
    """Return a list of function entities extracted from ``source``."""
    try:
        tree = ast.parse(source)
    except SyntaxError as exc:
        sys.stderr.write(
            f"clarion-plugin-python: skipping {module_path}: syntax error at "
            f"line {exc.lineno}: {exc.msg}\n",
        )
        return []

    dotted_module = module_dotted_name(module_path)
    entities: list[dict[str, Any]] = []
    _walk(tree, [tree], dotted_module, module_path, entities)
    return entities


def _walk(
    node: ast.AST,
    parents: list[ast.AST],
    dotted_module: str,
    module_path: str,
    out: list[dict[str, Any]],
) -> None:
    for child in ast.iter_child_nodes(node):
        if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)):
            out.append(_build_entity(child, parents, dotted_module, module_path))
        _walk(child, [*parents, child], dotted_module, module_path, out)


def _build_entity(
    node: ast.FunctionDef | ast.AsyncFunctionDef,
    parents: list[ast.AST],
    dotted_module: str,
    module_path: str,
) -> dict[str, Any]:
    python_qualname = reconstruct_qualname(node, parents)
    qualified_name = f"{dotted_module}.{python_qualname}" if dotted_module else python_qualname
    end_line = node.end_lineno if node.end_lineno is not None else node.lineno
    end_col = node.end_col_offset if node.end_col_offset is not None else node.col_offset
    return {
        "id": entity_id(_PLUGIN_ID, _KIND, qualified_name),
        "kind": _KIND,
        "qualified_name": qualified_name,
        "module_path": module_path,
        "source_range": {
            "start_line": node.lineno,
            "start_col": node.col_offset,
            "end_line": end_line,
            "end_col": end_col,
        },
    }

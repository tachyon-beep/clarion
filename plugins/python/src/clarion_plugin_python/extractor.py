"""AST → function-entity extractor for the Python plugin (WP3 Task 4).

Walks a parsed Python file and emits one entity per ``FunctionDef`` /
``AsyncFunctionDef`` (Sprint 1 scope). Class, decorator, module, and
import/call edge emission is WP3-feature-complete scope and deliberately
out of band here.

Entity shape matches the Rust host's ``RawEntity`` + ``RawSource``
contract (``crates/clarion-core/src/plugin/host.rs:132-154``)::

    {
        "id": "python:function:...",
        "kind": "function",
        "qualified_name": "pkg.module.func",
        "source": {
            "file_path": "pkg/module.py",
            "source_range": {
                "start_line": 1, "start_col": 0,
                "end_line": 3, "end_col": 4,
            },
        },
    }

``source.file_path`` lands in the host's path jail (canonicalised +
checked against ``project_root``); any other source-side fields flow
through ``RawSource.extra`` (serde flatten) and are bounded by
``MAX_ENTITY_EXTRA_BYTES`` (64 KiB). ``qualified_name`` is the dotted
module prefix joined to Python's own ``__qualname__`` (reconstructed
per L7). The file_path passed on the wire may be absolute (what the
host sent) while the prefix used for qualified-name dotting can be the
relativised form — the two are decoupled via ``extract``'s
``module_prefix_path`` kwarg.

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


def extract(
    source: str,
    file_path: str,
    *,
    module_prefix_path: str | None = None,
) -> list[dict[str, Any]]:
    """Return a list of function entities extracted from ``source``.

    ``file_path`` lands in each entity's ``source.file_path`` verbatim.
    ``module_prefix_path`` (default: same as ``file_path``) is the path
    whose dotted form prefixes every entity's ``qualified_name`` — callers
    can supply a project-relative path here while keeping ``file_path``
    absolute so the host's path jail validates the original path.
    """
    try:
        tree = ast.parse(source)
    except SyntaxError as exc:
        sys.stderr.write(
            f"clarion-plugin-python: skipping {file_path}: syntax error at "
            f"line {exc.lineno}: {exc.msg}\n",
        )
        return []

    prefix_source = module_prefix_path if module_prefix_path is not None else file_path
    dotted_module = module_dotted_name(prefix_source)
    entities: list[dict[str, Any]] = []
    _walk(tree, [tree], dotted_module, file_path, entities)
    return entities


def _walk(
    node: ast.AST,
    parents: list[ast.AST],
    dotted_module: str,
    file_path: str,
    out: list[dict[str, Any]],
) -> None:
    for child in ast.iter_child_nodes(node):
        if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)):
            out.append(_build_entity(child, parents, dotted_module, file_path))
        _walk(child, [*parents, child], dotted_module, file_path, out)


def _build_entity(
    node: ast.FunctionDef | ast.AsyncFunctionDef,
    parents: list[ast.AST],
    dotted_module: str,
    file_path: str,
) -> dict[str, Any]:
    python_qualname = reconstruct_qualname(node, parents)
    qualified_name = f"{dotted_module}.{python_qualname}" if dotted_module else python_qualname
    end_line = node.end_lineno if node.end_lineno is not None else node.lineno
    end_col = node.end_col_offset if node.end_col_offset is not None else node.col_offset
    return {
        "id": entity_id(_PLUGIN_ID, _KIND, qualified_name),
        "kind": _KIND,
        "qualified_name": qualified_name,
        "source": {
            "file_path": file_path,
            "source_range": {
                "start_line": node.lineno,
                "start_col": node.col_offset,
                "end_line": end_line,
                "end_col": end_col,
            },
        },
    }

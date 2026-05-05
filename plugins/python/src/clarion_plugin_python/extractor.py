"""AST → entity extractor for the Python plugin (Sprint 2 / B.2).

Walks a parsed Python file and emits one ``module`` entity per file plus
one ``function`` entity per ``FunctionDef`` / ``AsyncFunctionDef``. Class
entity emission joins in the next plan task; decorator, import, and
call-edge emission is later WP3-feature-complete scope.

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

Behaviour (B.2 §3 Q1 supersedes Sprint-1 UQ-WP3-11 for module entities):

- Every analyzed file produces exactly one ``module`` entity. Empty
  files and comment-only files emit one with ``parse_status="ok"``.
  Zero *function* entities for empty files still holds (UQ-WP3-11).
- ``SyntaxError`` during ``ast.parse`` → one degraded module entity
  with ``parse_status="syntax_error"`` plus one stderr log line
  (UQ-WP3-02). The run continues; WP4-era findings can later attach a
  ``CLA-PY-SYNTAX-ERROR`` annotation.
- Top-level ``__init__.py`` (where the dotted module name resolves to
  ``""``) is skipped with stderr; the entity-ID assembler rejects an
  empty ``canonical_qualified_name``.
- Paths starting with ``src/`` have the prefix stripped (UQ-WP3-05).
- ``pkg/__init__.py`` files yield qualified_names rooted at ``pkg``
  (not ``pkg.__init__``) — UQ-WP3-06.

Module-entity ``source_range`` is a whole-file cover with ``end_col=0``
as a sentinel for module entities only — class and function entities
carry real ``ast.*.end_col_offset`` data, so consumers must NOT infer
column semantics by analogy across kinds.
"""

from __future__ import annotations

import ast
import sys
from pathlib import PurePosixPath
from typing import Literal, NotRequired, TypedDict

from clarion_plugin_python.entity_id import entity_id
from clarion_plugin_python.qualname import reconstruct_qualname

_PLUGIN_ID = "python"


class SourceRange(TypedDict):
    start_line: int
    start_col: int
    end_line: int
    end_col: int


class EntitySource(TypedDict):
    file_path: str
    source_range: SourceRange


class RawEntity(TypedDict):
    """Wire shape matching the Rust host's RawEntity contract.

    ``parse_status`` is set on module entities only and rides through the
    host's ``serde(flatten) extra`` map. Class and function entities omit
    it; the field is ``NotRequired`` to keep mypy --strict happy.
    """

    id: str
    kind: str  # "function" | "class" | "module"; not narrowed to keep extension cheap.
    qualified_name: str
    source: EntitySource
    parse_status: NotRequired[Literal["ok", "syntax_error"]]


def _module_source_range(source: str) -> SourceRange:
    """Whole-file cover for module entities (Q4 resolution, B.2 §3 Q4).

    Uniform formula regardless of ``parse_status``: ``end_line =
    source.count('\\n') + 1``, ``end_col = 0``. The ``end_col = 0`` value
    is a sentinel for module entities only — it means "end-of-file," NOT
    "column 0 of the last line." Class and function entities use real
    ``ast.*.end_col_offset`` data; consumers must not infer column
    semantics by analogy across kinds.
    """
    return {
        "start_line": 1,
        "start_col": 0,
        "end_line": source.count("\n") + 1,
        "end_col": 0,
    }


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


def _build_module_entity(
    source: str,
    dotted_module: str,
    file_path: str,
    parse_status: Literal["ok", "syntax_error"],
) -> RawEntity:
    """Build the per-file module entity (Q1 + Q4 resolutions)."""
    return {
        "id": entity_id(_PLUGIN_ID, "module", dotted_module),
        "kind": "module",
        "qualified_name": dotted_module,
        "source": {
            "file_path": file_path,
            "source_range": _module_source_range(source),
        },
        "parse_status": parse_status,
    }


def extract(
    source: str,
    file_path: str,
    *,
    module_prefix_path: str | None = None,
) -> list[RawEntity]:
    """Return a list of entities extracted from ``source``.

    Always emits exactly one module entity (B.2 Q1) prepended to the
    result; functions and classes follow. ``file_path`` lands in each
    entity's ``source.file_path`` verbatim. ``module_prefix_path``
    (default: same as ``file_path``) is the path whose dotted form
    prefixes every entity's ``qualified_name`` — callers can supply a
    project-relative path here while keeping ``file_path`` absolute so
    the host's path jail validates the original path.
    """
    prefix_source = module_prefix_path if module_prefix_path is not None else file_path
    dotted_module = module_dotted_name(prefix_source)

    # Top-level __init__.py would resolve to "" — entity_id() rejects that
    # (crates/clarion-core/src/entity_id.rs:97-101). Skip with stderr.
    if not dotted_module:
        sys.stderr.write(
            f"clarion-plugin-python: skipping {file_path}: "
            f"top-level __init__.py has no package name\n",
        )
        return []

    try:
        tree = ast.parse(source)
    except SyntaxError as exc:
        sys.stderr.write(
            f"clarion-plugin-python: skipping {file_path}: syntax error at "
            f"line {exc.lineno}: {exc.msg}\n",
        )
        return [_build_module_entity(source, dotted_module, file_path, "syntax_error")]

    entities: list[RawEntity] = [_build_module_entity(source, dotted_module, file_path, "ok")]
    _walk(tree, [tree], dotted_module, file_path, entities)
    return entities


def _walk(
    node: ast.AST,
    parents: list[ast.AST],
    dotted_module: str,
    file_path: str,
    out: list[RawEntity],
) -> None:
    for child in ast.iter_child_nodes(node):
        match child:
            case ast.FunctionDef() | ast.AsyncFunctionDef():
                out.append(_build_function_entity(child, parents, dotted_module, file_path))
            case ast.ClassDef():
                out.append(_build_class_entity(child, parents, dotted_module, file_path))
        _walk(child, [*parents, child], dotted_module, file_path, out)


def _build_function_entity(
    node: ast.FunctionDef | ast.AsyncFunctionDef,
    parents: list[ast.AST],
    dotted_module: str,
    file_path: str,
) -> RawEntity:
    python_qualname = reconstruct_qualname(node, parents)
    qualified_name = f"{dotted_module}.{python_qualname}" if dotted_module else python_qualname
    end_line = node.end_lineno if node.end_lineno is not None else node.lineno
    end_col = node.end_col_offset if node.end_col_offset is not None else node.col_offset
    return {
        "id": entity_id(_PLUGIN_ID, "function", qualified_name),
        "kind": "function",
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


def _build_class_entity(
    node: ast.ClassDef,
    parents: list[ast.AST],
    dotted_module: str,
    file_path: str,
) -> RawEntity:
    """Build a class entity. Uses real ast.end_lineno/end_col_offset (not the module sentinel).

    Class methods continue to emit as ``function`` entities (per
    detailed-design.md:67); no separate ``method`` kind. Nested classes
    nest in the qualname per ``reconstruct_qualname`` (no ``<locals>``
    between class names).
    """
    python_qualname = reconstruct_qualname(node, parents)
    qualified_name = f"{dotted_module}.{python_qualname}" if dotted_module else python_qualname
    end_line = node.end_lineno if node.end_lineno is not None else node.lineno
    end_col = node.end_col_offset if node.end_col_offset is not None else node.col_offset
    return {
        "id": entity_id(_PLUGIN_ID, "class", qualified_name),
        "kind": "class",
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

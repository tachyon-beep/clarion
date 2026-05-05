"""Round-trip self-test: plugin analyses its own source (WP3 Task 8).

Drives the *installed* ``clarion-plugin-python`` entry-point binary
(not ``sys.executable -m``) so the pip-install entry point is exercised
end-to-end. The plugin's own ``extractor.py`` is the analysis target; the
test asserts the module's public API functions appear in the returned
entity list with the expected 3-segment L2 EntityId shape.
"""

from __future__ import annotations

import json
import subprocess
import sysconfig
from pathlib import Path
from typing import IO, Any

import pytest


def _encode_frame(payload: dict[str, Any]) -> bytes:
    body = json.dumps(payload).encode("utf-8")
    header = f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
    return header + body


def _read_frame(stream: IO[bytes]) -> dict[str, Any]:
    headers: dict[str, str] = {}
    while True:
        line = stream.readline()
        if not line:
            msg = "EOF before headers terminator"
            raise RuntimeError(msg)
        if line in (b"\r\n", b"\n"):
            break
        name, _, value = line.decode("ascii").rstrip("\r\n").partition(":")
        headers[name.strip().lower()] = value.strip()
    length = int(headers["content-length"])
    body = stream.read(length)
    parsed: dict[str, Any] = json.loads(body)
    return parsed


def _locate_binary() -> Path:
    scripts = Path(sysconfig.get_path("scripts"))
    binary = scripts / "clarion-plugin-python"
    if not binary.exists():
        pytest.skip(
            f"clarion-plugin-python not at {binary}; "
            "install with `pip install -e plugins/python[dev]`",
        )
    return binary


def test_round_trip_self_analysis() -> None:  # noqa: PLR0915 - by-kind invariants are flat asserts
    """Plugin → analyze_file on its own extractor.py → expected entities appear."""
    binary = _locate_binary()

    # plugins/python/src is the package root; using it as project_root lets
    # the plugin relativise extractor.py to `clarion_plugin_python/extractor.py`,
    # whose dotted module name is `clarion_plugin_python.extractor`.
    plugin_src = Path(__file__).resolve().parents[1] / "src"
    target = plugin_src / "clarion_plugin_python" / "extractor.py"
    assert target.is_file(), f"target source not found at {target}"

    proc = subprocess.Popen(  # noqa: S603 - invoking our own installed entry point
        [str(binary)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        assert proc.stdin is not None
        assert proc.stdout is not None

        # Handshake.
        proc.stdin.write(
            _encode_frame(
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocol_version": "1.0",
                        "project_root": str(plugin_src),
                    },
                },
            ),
        )
        proc.stdin.flush()
        init_response = _read_frame(proc.stdout)
        assert init_response["id"] == 1
        assert init_response["result"]["name"] == "clarion-plugin-python"

        proc.stdin.write(
            _encode_frame({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
        )
        proc.stdin.flush()

        # Analyze extractor.py.
        proc.stdin.write(
            _encode_frame(
                {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "analyze_file",
                    "params": {"file_path": str(target)},
                },
            ),
        )
        proc.stdin.flush()
        response = _read_frame(proc.stdout)
        assert response["id"] == 2

        entities = response["result"]["entities"]
        function_entities = [e for e in entities if e["kind"] == "function"]
        module_entities = [e for e in entities if e["kind"] == "module"]
        class_entities = [e for e in entities if e["kind"] == "class"]
        function_ids = {e["id"] for e in function_entities}

        # Invariants — no exact totals (those become merge-conflict generators
        # the moment someone adds a private helper to extractor.py).
        assert len(module_entities) == 1, "exactly one module entity per analyzed file"
        assert module_entities[0]["id"] == "python:module:clarion_plugin_python.extractor"
        assert module_entities[0].get("parse_status") == "ok"

        # Public extractor API must be present.
        assert "python:function:clarion_plugin_python.extractor.module_dotted_name" in function_ids
        assert "python:function:clarion_plugin_python.extractor.extract" in function_ids
        # Private walker is a FunctionDef too, so it emits.
        assert "python:function:clarion_plugin_python.extractor._walk" in function_ids
        # B.2 renamed `_build_entity` → `_build_function_entity` and added
        # `_build_class_entity` + `_build_module_entity` (and `_module_source_range`).
        assert (
            "python:function:clarion_plugin_python.extractor._build_function_entity" in function_ids
        )
        assert "python:function:clarion_plugin_python.extractor._build_class_entity" in function_ids
        assert (
            "python:function:clarion_plugin_python.extractor._build_module_entity" in function_ids
        )

        # extractor.py defines its wire-shape TypedDicts at module level
        # (SourceRange, EntitySource, RawEntity); these are AST ClassDefs
        # and so emit as `class` entities. Subset assertion only —
        # exhaustive enumeration would be brittle.
        class_ids = {e["id"] for e in class_entities}
        assert "python:class:clarion_plugin_python.extractor.SourceRange" in class_ids
        assert "python:class:clarion_plugin_python.extractor.EntitySource" in class_ids
        assert "python:class:clarion_plugin_python.extractor.RawEntity" in class_ids

        # Every entity carries the absolute source.file_path we sent
        # (project_root relativisation only affects the qualified_name prefix).
        for entity in entities:
            assert entity["source"]["file_path"] == str(target)

        # Graceful shutdown.
        proc.stdin.write(
            _encode_frame({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": {}}),
        )
        proc.stdin.flush()
        _read_frame(proc.stdout)  # shutdown ack
        proc.stdin.write(_encode_frame({"jsonrpc": "2.0", "method": "exit"}))
        proc.stdin.flush()
        proc.stdin.close()
        assert proc.wait(timeout=5) == 0
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait(timeout=2)

from __future__ import annotations

import shutil
import stat
import sys
import textwrap
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

from clarion_plugin_python.pyright_session import (
    FINDING_PYRIGHT_CALL_RESOLUTION_TIMEOUT,
    FINDING_PYRIGHT_INIT_TIMEOUT,
    FINDING_PYRIGHT_INSTALL_FAILURE,
    FINDING_PYRIGHT_POISON_FRAME,
    FINDING_PYRIGHT_RESTART,
    FINDING_PYRIGHT_UNAVAILABLE,
    LspTimeoutError,
    PyrightSession,
)

if TYPE_CHECKING:
    from collections.abc import Sequence

    from clarion_plugin_python.call_resolver import Finding


@pytest.fixture(scope="session")
def pyright_langserver() -> str:
    venv_candidate = Path(sys.executable).parent / "pyright-langserver"
    if venv_candidate.exists():
        return str(venv_candidate)
    resolved = shutil.which("pyright-langserver")
    if resolved is None:
        pytest.skip("pyright-langserver is not installed")
    return resolved


def _write_module(tmp_path: Path, source: str, name: str = "demo.py") -> Path:
    path = tmp_path / name
    path.write_text(textwrap.dedent(source).lstrip(), encoding="utf-8")
    return path


def _finding_codes(result_findings: Sequence[Finding]) -> set[str]:
    return {str(finding["subcode"]) for finding in result_findings}


@pytest.mark.pyright
def test_pyright_session_resolves_direct_call(tmp_path: Path, pyright_langserver: str) -> None:
    module = _write_module(
        tmp_path,
        """
        def callee():
            pass

        def caller():
            callee()
        """,
    )

    with PyrightSession(tmp_path, executable=pyright_langserver) as session:
        result = session.resolve_calls(
            module,
            ["python:function:demo.caller", "python:function:demo.callee"],
        )

    assert result.edges == [
        {
            "kind": "calls",
            "from_id": "python:function:demo.caller",
            "to_id": "python:function:demo.callee",
            "confidence": "resolved",
            "source_byte_start": result.edges[0]["source_byte_start"],
            "source_byte_end": result.edges[0]["source_byte_end"],
        },
    ]
    assert result.edges[0]["source_byte_start"] < result.edges[0]["source_byte_end"]
    assert result.pyright_query_latency_ms[0] > 0
    assert result.unresolved_call_sites_total == 0


@pytest.mark.pyright
def test_pyright_session_ambiguous_dict_dispatch(tmp_path: Path, pyright_langserver: str) -> None:
    module = _write_module(
        tmp_path,
        """
        from collections.abc import Callable

        def alpha() -> None:
            pass

        def beta() -> None:
            pass

        handlers: dict[str, Callable[[], None]] = {"a": alpha, "b": beta}

        def caller(key: str) -> None:
            handlers[key]()
        """,
    )

    with PyrightSession(tmp_path, executable=pyright_langserver) as session:
        result = session.resolve_calls(
            module,
            [
                "python:function:demo.alpha",
                "python:function:demo.beta",
                "python:function:demo.caller",
            ],
        )

    edge = next(edge for edge in result.edges if edge["from_id"] == "python:function:demo.caller")
    assert edge["confidence"] == "ambiguous"
    assert edge["to_id"] == "python:function:demo.alpha"
    assert edge["properties"]["candidates"] == [
        "python:function:demo.alpha",
        "python:function:demo.beta",
    ]


@pytest.mark.pyright
def test_pyright_session_ambiguous_determinism(tmp_path: Path, pyright_langserver: str) -> None:
    module = _write_module(
        tmp_path,
        """
        from collections.abc import Callable

        def beta() -> None:
            pass

        def alpha() -> None:
            pass

        handlers: dict[str, Callable[[], None]] = {"b": beta, "a": alpha}

        def caller(key: str) -> None:
            handlers[key]()
        """,
    )
    function_ids = [
        "python:function:demo.alpha",
        "python:function:demo.beta",
        "python:function:demo.caller",
    ]

    with PyrightSession(tmp_path, executable=pyright_langserver) as first:
        first_edge = first.resolve_calls(module, function_ids).edges[0]
    with PyrightSession(tmp_path, executable=pyright_langserver) as second:
        second_edge = second.resolve_calls(module, function_ids).edges[0]

    assert first_edge == second_edge
    assert first_edge["to_id"] == "python:function:demo.alpha"
    assert first_edge["properties"]["candidates"] == [
        "python:function:demo.alpha",
        "python:function:demo.beta",
    ]


@pytest.mark.pyright
def test_pyright_session_restart_on_crash(tmp_path: Path, pyright_langserver: str) -> None:
    module = _write_module(
        tmp_path,
        """
        def callee():
            pass

        def caller():
            callee()
        """,
    )

    with PyrightSession(tmp_path, executable=pyright_langserver) as session:
        assert session.resolve_calls(module, ["python:function:demo.caller"]).edges
        session.kill_for_test()
        result = session.resolve_calls(module, ["python:function:demo.caller"])

    assert result.edges
    assert FINDING_PYRIGHT_RESTART in _finding_codes(result.findings)


@pytest.mark.pyright
def test_pyright_session_restart_cap(tmp_path: Path, pyright_langserver: str) -> None:
    module = _write_module(
        tmp_path,
        """
        def callee():
            pass

        def caller():
            callee()
        """,
    )

    with PyrightSession(
        tmp_path,
        executable=pyright_langserver,
        max_restarts_per_run=0,
    ) as session:
        assert session.resolve_calls(module, ["python:function:demo.caller"]).edges
        session.kill_for_test()
        poisoned = session.resolve_calls(module, ["python:function:demo.caller"])
        continued = session.resolve_calls(module, ["python:function:demo.caller"])

    assert poisoned.edges == []
    assert FINDING_PYRIGHT_POISON_FRAME in _finding_codes(poisoned.findings)
    assert poisoned.unresolved_call_sites_total == 1
    assert continued.edges == []
    assert continued.unresolved_call_sites_total == 1


def _write_executable(tmp_path: Path, body: str) -> Path:
    script = tmp_path / "fake_langserver.py"
    script.write_text(body, encoding="utf-8")
    script.chmod(script.stat().st_mode | stat.S_IXUSR)
    return script


def test_pyright_session_init_timeout(tmp_path: Path) -> None:
    script = _write_executable(
        tmp_path,
        "#!/usr/bin/env python3\nimport time\ntime.sleep(60)\n",
    )
    module = _write_module(tmp_path, "def caller():\n    print('x')\n")

    with PyrightSession(tmp_path, executable=str(script), init_timeout_secs=0.05) as session:
        result = session.resolve_calls(module, ["python:function:demo.caller"])

    assert result.edges == []
    assert FINDING_PYRIGHT_INIT_TIMEOUT in _finding_codes(result.findings)


def test_pyright_session_unavailable_binary_missing(tmp_path: Path) -> None:
    module = _write_module(tmp_path, "def caller():\n    print('x')\n")

    with PyrightSession(tmp_path, executable="clarion-missing-pyright") as session:
        result = session.resolve_calls(module, ["python:function:demo.caller"])

    assert result.edges == []
    assert result.unresolved_call_sites_total == 1
    assert FINDING_PYRIGHT_UNAVAILABLE in _finding_codes(result.findings)


def test_pyright_session_install_failure(tmp_path: Path) -> None:
    module = _write_module(tmp_path, "def caller():\n    print('x')\n")

    with PyrightSession(
        tmp_path,
        executable=sys.executable,
        install_check=lambda _: False,
    ) as session:
        result = session.resolve_calls(module, ["python:function:demo.caller"])

    assert result.edges == []
    assert result.unresolved_call_sites_total == 1
    assert FINDING_PYRIGHT_INSTALL_FAILURE in _finding_codes(result.findings)


class TimeoutSession(PyrightSession):
    def _request(self, method: str, params: dict[str, object], timeout_secs: float) -> object:
        if method == "callHierarchy/outgoingCalls":
            raise LspTimeoutError(method)
        return super()._request(method, params, timeout_secs)


@pytest.mark.pyright
def test_pyright_session_call_resolution_timeout(tmp_path: Path, pyright_langserver: str) -> None:
    module = _write_module(
        tmp_path,
        """
        def callee():
            pass

        def caller():
            callee()
        """,
    )

    with TimeoutSession(tmp_path, executable=pyright_langserver) as session:
        result = session.resolve_calls(module, ["python:function:demo.caller"])

    assert result.edges == []
    assert FINDING_PYRIGHT_CALL_RESOLUTION_TIMEOUT in _finding_codes(result.findings)


def test_pyright_session_stderr_drain(tmp_path: Path) -> None:
    script = _write_executable(
        tmp_path,
        textwrap.dedent(
            """
            #!/usr/bin/env python3
            import json
            import sys

            sys.stderr.write("x" * 131072)
            sys.stderr.flush()

            def read_frame():
                headers = {}
                while True:
                    line = sys.stdin.buffer.readline()
                    if line in (b"", b"\\r\\n"):
                        return None
                    name, value = line.decode("ascii").strip().split(":", 1)
                    headers[name.lower()] = value.strip()
                    if sys.stdin.buffer.readline() == b"\\r\\n":
                        break
                return json.loads(sys.stdin.buffer.read(int(headers["content-length"])))

            def write_frame(message):
                body = json.dumps(message).encode("utf-8")
                sys.stdout.buffer.write(
                    b"Content-Length: " + str(len(body)).encode("ascii") + b"\\r\\n\\r\\n"
                )
                sys.stdout.buffer.write(body)
                sys.stdout.buffer.flush()

            while True:
                frame = read_frame()
                if frame is None:
                    break
                method = frame.get("method")
                if method == "initialize":
                    write_frame({"jsonrpc": "2.0", "id": frame["id"], "result": {}})
                elif method == "textDocument/prepareCallHierarchy":
                    write_frame({"jsonrpc": "2.0", "id": frame["id"], "result": []})
                elif method == "callHierarchy/outgoingCalls":
                    write_frame({"jsonrpc": "2.0", "id": frame["id"], "result": []})
                elif method == "shutdown":
                    write_frame({"jsonrpc": "2.0", "id": frame["id"], "result": {}})
                elif method == "exit":
                    break
            """,
        ).lstrip(),
    )
    module = _write_module(tmp_path, "def caller():\n    print('x')\n")

    with PyrightSession(tmp_path, executable=str(script), init_timeout_secs=1.0) as session:
        result = session.resolve_calls(module, ["python:function:demo.caller"])

    assert result.edges == []
    assert session.stderr_thread_alive is False

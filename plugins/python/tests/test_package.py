"""Task 1 smoke test: the package imports cleanly and the bootstrap main() exits 0."""

from __future__ import annotations

import clarion_plugin_python
from clarion_plugin_python.__main__ import main


def test_package_version_matches_pyproject() -> None:
    assert clarion_plugin_python.__version__ == "0.1.0"


def test_main_returns_zero() -> None:
    assert main() == 0

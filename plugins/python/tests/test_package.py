"""Package-level smoke test: version string is pinned to the pyproject value."""

from __future__ import annotations

import clarion_plugin_python


def test_package_version_matches_pyproject() -> None:
    assert clarion_plugin_python.__version__ == "0.1.0"

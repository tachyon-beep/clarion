"""Unit tests for the L8 Wardline probe (WP3 Task 6).

Each case stubs ``importlib.import_module`` inside the probe module to
simulate the absent / in-range / out-of-range states without requiring
the real ``wardline`` package to be present or absent in the test
environment.
"""

from __future__ import annotations

from types import SimpleNamespace
from typing import TYPE_CHECKING

from clarion_plugin_python.wardline_probe import probe

if TYPE_CHECKING:
    import pytest


def _install_fake_import(
    monkeypatch: pytest.MonkeyPatch,
    *,
    wardline_module: object | None,
    registry_module: object | None,
) -> None:
    """Replace ``importlib.import_module`` as seen by wardline_probe."""

    def fake_import(name: str) -> object:
        if name == "wardline.core.registry":
            if registry_module is None:
                msg = "no wardline.core.registry"
                raise ImportError(msg)
            return registry_module
        if name == "wardline":
            if wardline_module is None:
                msg = "no wardline"
                raise ImportError(msg)
            return wardline_module
        msg = f"unexpected import: {name}"
        raise ImportError(msg)

    # String target bypasses mypy's re-export check on
    # `clarion_plugin_python.wardline_probe.importlib`.
    monkeypatch.setattr(
        "clarion_plugin_python.wardline_probe.importlib.import_module",
        fake_import,
    )


def test_probe_absent_when_registry_import_fails(monkeypatch: pytest.MonkeyPatch) -> None:
    _install_fake_import(monkeypatch, wardline_module=None, registry_module=None)
    assert probe("0.1.0", "0.2.0") == {"status": "absent"}


def test_probe_enabled_when_version_in_range(monkeypatch: pytest.MonkeyPatch) -> None:
    fake_wardline = SimpleNamespace(__version__="0.1.5")
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "enabled", "version": "0.1.5"}


def test_probe_at_lower_bound_is_enabled(monkeypatch: pytest.MonkeyPatch) -> None:
    """Lower bound is inclusive."""
    fake_wardline = SimpleNamespace(__version__="0.1.0")
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "enabled", "version": "0.1.0"}


def test_probe_at_upper_bound_is_out_of_range(monkeypatch: pytest.MonkeyPatch) -> None:
    """Upper bound is exclusive."""
    fake_wardline = SimpleNamespace(__version__="0.2.0")
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "version_out_of_range", "version": "0.2.0"}


def test_probe_above_upper_bound_is_out_of_range(monkeypatch: pytest.MonkeyPatch) -> None:
    fake_wardline = SimpleNamespace(__version__="0.3.0")
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "version_out_of_range", "version": "0.3.0"}


def test_probe_absent_when_version_attribute_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    fake_wardline = SimpleNamespace()  # no __version__
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "absent"}


def test_probe_absent_when_version_is_not_a_string(monkeypatch: pytest.MonkeyPatch) -> None:
    fake_wardline = SimpleNamespace(__version__=123)
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "absent"}


def test_probe_absent_when_version_is_not_valid_semver(monkeypatch: pytest.MonkeyPatch) -> None:
    fake_wardline = SimpleNamespace(__version__="not-a-version")
    fake_registry = SimpleNamespace(REGISTRY={})
    _install_fake_import(
        monkeypatch,
        wardline_module=fake_wardline,
        registry_module=fake_registry,
    )
    assert probe("0.1.0", "0.2.0") == {"status": "absent"}

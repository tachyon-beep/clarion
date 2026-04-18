# WP3 — Python Plugin v0.1 Baseline (Sprint 1)

**Status**: DRAFT — blocked-by WP2
**Anchoring design**: [detailed-design.md §1 (Plugin implementation — Python specifics)](../../clarion/v0.1/detailed-design.md#1-plugin-implementation-detail), [system-design.md §2](../../clarion/v0.1/system-design.md#2-core--plugin-architecture)
**Accepted ADRs**: [ADR-018](../../clarion/adr/ADR-018-identity-reconciliation.md), [ADR-022](../../clarion/adr/ADR-022-core-plugin-ontology.md)
**Predecessor**: [WP2](./wp2-plugin-host.md).
**Blocks**: the Sprint 1 walking-skeleton demo.

---

## 1. Scope (Sprint 1 narrow)

WP3 in Sprint 1 ships the smallest real Python plugin that makes the walking skeleton
pass. It extracts **functions only** (module-level and class-method), produces
canonical names that match WP1's L2 format exactly, speaks WP2's JSON-RPC protocol,
and makes one real pass at the Wardline `REGISTRY` integration surface (even if the
pass is a graceful-no-op probe).

Feature-completeness for the Python plugin — every entity kind, every edge, every
`CLA-PY-*` rule — is **not** Sprint 1. That's the WP3-feature-complete sprint,
reached after WP4 exists and can consume richer plugin output.

**In scope for Sprint 1:**

- Python package `clarion-plugin-python` installable via `pip install -e
  plugins/python/` from the monorepo.
- `plugin.toml` manifest matching WP2's L5 schema.
- `clarion-plugin-python` executable (entry point) on `$PATH` after install.
- JSON-RPC server speaking WP2's L4 method set: `initialize`, `initialized`,
  `analyze_file`, `shutdown`, `exit`.
- `ast`-based function extraction: for a `.py` file, emit one entity per
  module-level function and per class method, each with:
  - `kind: "function"` (declared in `[ontology].entity_kinds` per ADR-022).
  - `id` — the 3-segment `EntityId` (L2) = `python:function:{qualified_name}`
    where `{qualified_name}` is the dotted module-relative name produced per
    L7. Format matches WP1's Rust `entity_id()` byte-for-byte via the shared
    fixture (Task 5).
  - `qualified_name` — the language-native dotted name per L7 (Python-side
    producer of the `canonical_qualified_name` segment).
  - `module_path` — the source-relative file path (entity **property**, not
    part of the ID per ADR-003).
  - `source_range` — line/column start and end.
- L7 qualname production format (see §2).
- L8 Wardline `REGISTRY` direct-import probe — attempt import, capture version,
  log outcome, continue regardless (graceful no-op if Wardline absent; see
  UQ-WP3-03 for the "fully wire" vs "probe-only" decision).
- `sys.stdout` discipline: plugin initialisation swaps `sys.stdout` for a
  JSON-RPC-only writer to prevent stray `print()` from corrupting framing
  (WP2 UQ-WP2-08 resolution).
- Round-trip self-test: plugin runs against its own source tree and asserts
  specific expected entities appear.

**Explicitly out of scope for Sprint 1:**

- Classes, decorators, module entities — deferred to WP3 feature-complete.
- Imports edge extraction — deferred.
- Calls edge extraction — deferred.
- Dynamic imports (`importlib`, `__import__`) — deferred; deliberately not solved
  in v0.1 per the out-of-scope list in `../v0.1-plan.md`.
- All `CLA-PY-*` findings — WP4 consumes, WP3-feature-complete emits.
- Type-inference, dataflow, taint — NG-05, never in scope for Clarion.
- Multi-file incremental analysis — WP6/WP7 territory.

## 2. Lock-in callouts

### L7 — Python qualified-name production format

**What locks**: the exact function producing the `canonical_qualified_name`
segment (the third segment of L2's 3-segment `EntityId`) per ADR-003. The
plugin prepends `python:function:` to form the full ID; WP1's Rust
`entity_id()` must produce byte-identical output from the same inputs (proven
via the shared fixture — Task 5).

**Format** (Sprint 1 subset — only functions; extend in later sprints as more kinds
ship):

- Module-level function `def hello(): ...` in `demo.py` → qualified name
  `demo.hello` → full `EntityId` `python:function:demo.hello`.
- Nested function `def outer(): def inner(): ...` in `demo.py` → qualified
  name `demo.outer.<locals>.inner` (matches Python's own `__qualname__`
  combined with the module dotted path).
- Class method `class Foo: def bar(self): ...` in `demo.py` → qualified name
  `demo.Foo.bar`.
- Async function → same as sync; the `async` keyword does not change the name.
- Lambdas → `<lambda>` — but Sprint 1 skips lambdas; they're deferred.

The rule "module dotted path + Python's own `__qualname__` semantics, joined
by `.`" is the spec. Sprint 1's implementation reconstructs `__qualname__`
from the AST (Python's `__qualname__` is only available at runtime
post-definition, which this plugin's static analyser isn't doing). Tests
assert against hand-written expectations derived from Python's documented
`__qualname__` rules. Module dotted path is derived from the root-relative
`module_path` via `detailed-design.md §1` Python normalisation (including
`src/` prefix stripping; UQ-WP3-05).

**Why now**: `↗` ADR-018 — Wardline stores qualnames produced by the same rule
and uses this segment (not the full 3-segment `EntityId`) as its Clarion-side
join key. Divergence means Filigree's triage state can't join Clarion's
entities to Wardline's annotations. This is the single most important
cross-product alignment in Sprint 1.

**Downstream impact**:
- WP1's L2 `entity_id()` concatenates `python`, `function`, and this
  segment. Shared fixture (Task 5) is the byte-for-byte parity proof.
- `↗` Wardline must produce the same qualnames. If Wardline's rule diverges, either
  Wardline changes or Clarion adds translation. ADR-018 names direct-production as
  the default; WP3's tests are the executable spec for what "match Wardline" means.

**Mitigation if Wardline diverges**: discovered via the first real cross-check (either
a Sprint 2 WP9 test against real Wardline annotations, or a manual spot-check during
this WP). Response is documented in ADR-018 — the translator route — not a WP3
change.

### L8 — Wardline `REGISTRY` import + version-pin protocol

**What locks**: the import path (`from wardline.core.registry import REGISTRY`) and
the version-pin syntax used in the plugin's `plugin.toml` (or a dedicated
`wardline_compat` field).

**Symbol verification** (2026-04-18, pre-sprint check): both symbols exist in
the Wardline source at this sprint's start and can be relied on:

- `wardline.core.registry.REGISTRY` — declared at
  `wardline/src/wardline/core/registry.py:55` as a `MappingProxyType[str, RegistryEntry]`.
- `wardline.__version__` — re-exported from `wardline/src/wardline/__init__.py:3`
  (sourced from `wardline._version`).

Both are usable today; UQ-WP3-03 resolves to "fully wire" (no stub-only
fallback).

**Sprint 1 pin approach**:

- Manifest field: `[integrations.wardline]` section with `min_version = "0.1.0"` and
  `max_version = "0.2.0"` (semver range). Exact numbers set when Wardline's current
  version is checked.
- Plugin startup probe:
  1. Attempt `import wardline.core.registry`. If `ImportError`, record `wardline
     absent` in the handshake response's `capabilities` field and proceed.
  2. If import succeeds, read `wardline.__version__`. If outside declared range,
     record `wardline version out of range` and proceed with the integration
     disabled (no hard fail — WP3 still emits entities, just without the
     REGISTRY cross-check).
  3. If in range, capture `REGISTRY` for later use. Sprint 1 does not actually
     call into `REGISTRY` for any entity — that's WP3-feature-complete work.
     Sprint 1 only proves the import + version-pin handshake works end-to-end.

**Why now**: the *protocol* is what locks. Once the plugin ships to any user who
also has Wardline installed, changing the probe shape or the version-pin syntax is
visible behaviour. The `REGISTRY` symbol itself is already named in ADR-018;
WP3 commits the plugin's consumption pattern.

**Cross-product implication**: `↗` **Wardline should not rename
`wardline.core.registry.REGISTRY` or drop `wardline.__version__` without a
coordinated Clarion-side release**. ADR-018 already states this; WP3 makes it a
real dependency. Same-author note: this is within-scope Wardline discipline, not
cross-team coordination — but the constraint is real.

**Sprint 1 decision** (resolved — see UQ-WP3-03 in §5): fully wire the
import. Symbol existence was verified pre-sprint
(`wardline/src/wardline/core/registry.py:55`,
`wardline/src/wardline/__init__.py:3`), so the probe runs against a real
`pip install wardline` in the dev venv rather than stubbing behind a
`CLARION_WARDLINE_ENABLED` env var. This locks L8 completely — the
exercised lock-in is the honest one.

## 3. File decomposition

```
/plugins/python/
  pyproject.toml                  # package metadata, entry-point: clarion-plugin-python
  plugin.toml                     # L5 manifest
  README.md                       # install + dev notes
  src/
    clarion_plugin_python/
      __init__.py
      __main__.py                 # entry point; runs the JSON-RPC server loop
      server.py                   # JSON-RPC framing + dispatch
      extractor.py                # ast visitor producing entities (L7)
      qualname.py                 # __qualname__ reconstruction (L7)
      entity_id.py                # L2 3-segment EntityId assembler matching WP1
      wardline_probe.py           # L8 import probe
      stdout_guard.py             # sys.stdout discipline (WP2 UQ-WP2-08)
  tests/
    test_qualname.py              # matches L7 __qualname__ expectations
    test_entity_id.py             # matches WP1 shared fixture byte-for-byte
    test_extractor.py             # ast → entities
    test_server.py                # JSON-RPC framing + dispatch
    test_wardline_probe.py        # probe behaviour with and without wardline
    test_round_trip.py            # plugin runs against its own source
    fixtures/                     # .py files with expected-entity YAML
```

The plugin is its own pip-installable package — deliberately separate from the Rust
workspace. This is Sprint 1's commitment to "plugins are independent artefacts,
not core-vendored code" per ADR-022.

## 4. External dependencies being locked

### Cross-product dependency locked by Sprint 1

- **Wardline** — import path `wardline.core.registry.REGISTRY` and version attribute
  `wardline.__version__`. See L8.

### Python-side package dependencies

Minimal. `pyproject.toml` declares:

- `python_requires = ">=3.11"` (UQ-WP3-04 — proposal; revisit Task 1).
- No runtime deps beyond the standard library for Sprint 1. `ast`, `json`, `sys`,
  `os`, `pathlib` are all stdlib.
- Dev deps: `pytest`, `pytest-cov`, `ruff`, `mypy`.
- Optional dep: `wardline` (declared in `[project.optional-dependencies] integrations`).
  The plugin works without Wardline; declaring it optional allows `pip install
  clarion-plugin-python[integrations]` to pull Wardline when desired.

## 5. Unresolved questions

- **UQ-WP3-01** — **Qualname for nested class methods**: `class A: class B: def c():`.
  Python's `__qualname__` gives `A.B.c`. Confirm the L7 rule matches this without
  edge cases. **Proposal**: yes, follow `__qualname__` exactly; add the case as a
  test fixture. **Resolution by**: Task 3.
- **UQ-WP3-02** — **How does the plugin handle syntax errors in the source file?**
  `ast.parse()` raises `SyntaxError`. Options: (a) skip the file + log + emit zero
  entities for that file; (b) fail the run. **Proposal**: (a) — skip + log. Unusable
  files should not abort analysis; WP4 may later attach a finding. **Resolution
  by**: Task 4.
- **UQ-WP3-03** — **Fully wire Wardline import in Sprint 1 or stub?** ~~open~~
  — **resolved as "fully wire"**. Pre-sprint symbol check (see L8) confirmed
  `wardline.core.registry.REGISTRY` and `wardline.__version__` exist in the
  current Wardline source (`src/wardline/core/registry.py:55` and
  `src/wardline/__init__.py:3`). The probe can run against a real
  `pip install wardline` in the dev venv, which is the honest lock-in.
  **Resolved**: Task 6.
- **UQ-WP3-04** — **Minimum Python version**: **Resolved — 3.11**. Picked
  for `ast.unparse` availability and better error messages; 3.12 raises the
  install barrier without a Sprint 1 payoff. Clarion users are developers
  with reasonable Python versions available. **Resolved**: Task 1.
- **UQ-WP3-05** — **Module-path normalisation**: the `module_path` entity
  property and the derivation of the dotted-module prefix for L7's
  `canonical_qualified_name` are both rooted at the analysis root (the arg
  passed to `clarion analyze`). Does WP3 receive the root explicitly in
  `analyze_file` params, or is each path already root-relative from the
  host? **Proposal**: the host passes root-relative paths after jail
  normalisation (WP2 L6); WP3 does not re-canonicalise. Cross-check with WP2
  Task 6 implementation. **Resolution by**: Task 4.
- **UQ-WP3-06** — **Handling of `__init__.py` module-path**: should the module-path
  for entities in `pkg/__init__.py` be `pkg/__init__.py` or `pkg`? Proposal:
  `pkg/__init__.py` (the literal file path); `pkg` is semantic module naming that
  WP4 can synthesise if needed. Simplicity wins: file path is unambiguous.
  **Resolution by**: Task 4.
- **UQ-WP3-07** — **Type-annotation functions (`typing.overload`, protocol methods)**:
  do they get emitted like regular functions? **Proposal**: yes — they're still
  `def`-bound names; WP4 can later add a `CLA-PY-OVERLOAD` rule if useful.
  **Resolution by**: Task 3.
- **UQ-WP3-08** — **Byte-for-byte `EntityId` parity strategy**: how do we
  maintain parity with WP1's Rust implementation? Option (a): both
  implementations read the same spec (ADR-003) and rely on tests. Option (b):
  ship a shared test fixture file (JSON) with input triples + expected
  outputs; both implementations' test suites consume it. **Proposal**: (b) —
  a shared `fixtures/entity_id.json` file at the repo root. Each row contains
  `{plugin_id, kind, canonical_qualified_name, expected_entity_id}`. Exact
  same inputs, exact same expected outputs; divergence fails CI on both
  sides. **Resolution by**: Task 5.
- **UQ-WP3-09** — **Plugin logging destination**: stderr for free-form (per WP2
  UQ-WP2-07 resolution) or a file under `.clarion/logs/`? **Proposal**: stderr;
  core forwards to tracing; `.clarion/logs/` is a Sprint 2+ decision. **Resolution
  by**: Task 2.
- **UQ-WP3-10** — **Testing infrastructure**: **Resolved — pytest + ruff**.
  Mypy adoption deferred until the plugin grows enough to benefit.
  **Resolved**: Task 1.
- **UQ-WP3-11** — **What does the plugin return for an empty `.py` file (zero
  functions)?** An empty `entities` array. Confirm WP2's host handles this without
  tripping any alert. **Resolution by**: Task 4.
- **UQ-WP3-12** — **How does the plugin identify itself in the `initialize`
  handshake?** Proposal: return `{"name": "clarion-plugin-python", "version": "0.1.0",
  "ontology_version": "0.1.0"}` matching the manifest. Host cross-checks against
  the manifest and fails handshake if mismatched. **Resolution by**: Task 2.

## 6. Task ledger

### Task 1 — Python package skeleton

**Files**:
- Create `/plugins/python/pyproject.toml`
- Create `/plugins/python/src/clarion_plugin_python/__init__.py`
- Create `/plugins/python/src/clarion_plugin_python/__main__.py`
- Create `/plugins/python/README.md`
- Create `/plugins/python/tests/__init__.py`

Steps:

- [ ] Write `pyproject.toml` with `project.name = "clarion-plugin-python"`, `requires-python = ">=3.11"` (UQ-WP3-04), `project.scripts.clarion-plugin-python = "clarion_plugin_python.__main__:main"`, no runtime deps, dev deps `pytest` + `ruff`.
- [ ] Write `__main__.py` with a `main()` that prints `clarion-plugin-python 0.1.0\n` to stderr and exits 0 (so `pip install -e .` produces a verifiable binary).
- [ ] `pip install -e plugins/python` and verify `which clarion-plugin-python` returns a path and running it exits 0.
- [ ] Commit: `feat(wp3): Python plugin package skeleton`.

### Task 2 — JSON-RPC server loop + stdout discipline

**Files**:
- Create `/plugins/python/src/clarion_plugin_python/stdout_guard.py`
- Create `/plugins/python/src/clarion_plugin_python/server.py`
- Modify `/plugins/python/src/clarion_plugin_python/__main__.py`

Steps:

- [ ] In `stdout_guard.py`: on import, replace `sys.stdout` with an object whose `.buffer` is the real stdout bytes stream and whose `.write` raises. Provide a context manager `jsonrpc_output()` that yields the real bytes stream. This enforces WP2 UQ-WP2-08.
- [ ] In `server.py`: implement Content-Length frame read/write from `sys.stdin.buffer` / the real stdout bytes stream. Implement a dispatch loop handling `initialize`, `initialized`, `analyze_file`, `shutdown`, `exit` by method name; each dispatches to a handler function.
- [ ] Handlers for `initialize` (returns `{"name": "clarion-plugin-python", "version": "0.1.0", "ontology_version": "0.1.0"}`, UQ-WP3-12) and `shutdown` (returns `null`, then the next loop iteration exits on `exit`). `analyze_file` returns `{"entities": []}` for now; filled in later tasks.
- [ ] Failing integration test in `test_server.py`: spin up the server in a subprocess, send an `initialize` frame, receive a response with the expected shape.
- [ ] Implement and verify. Commit: `feat(wp3): L4-compatible JSON-RPC server + stdout guard`.

### Task 3 — Qualname reconstruction (L7)

**Files**:
- Create `/plugins/python/src/clarion_plugin_python/qualname.py`
- Create `/plugins/python/tests/test_qualname.py`

Steps:

- [ ] Failing tests in `test_qualname.py`: module-level function, nested function (`outer.<locals>.inner`), class method (`Foo.bar`), nested class method (`Outer.Inner.method`), async function (same as sync). Include UQ-WP3-01 and UQ-WP3-07 cases.
- [ ] Implement `qualname.py`: `def reconstruct(node: ast.AST, parents: list[ast.AST]) -> str` — walks the parent chain; adds `.<locals>.` between function-parent and child; joins with `.` for class-parent.
- [ ] Run tests; expect pass.
- [ ] Commit: `feat(wp3): L7 qualname reconstruction matching __qualname__ semantics`.

### Task 4 — Extractor (ast → entities)

**Files**:
- Create `/plugins/python/src/clarion_plugin_python/extractor.py`
- Create `/plugins/python/src/clarion_plugin_python/entity_id.py`
- Create `/plugins/python/tests/test_extractor.py`
- Create `/plugins/python/tests/fixtures/` with sample `.py` files + expected-entity YAML

Steps:

- [ ] In `entity_id.py`, implement the 3-segment assembler per ADR-003: `def entity_id(plugin_id: str, kind: str, canonical_qualified_name: str) -> str`. Sprint 1's byte-for-byte match with WP1's Rust is covered by Task 5's shared fixture.
- [ ] In `extractor.py`, `def extract(source: str, module_path: str) -> list[dict]`:
  - `ast.parse(source)`; on `SyntaxError`, log to stderr and return `[]` (UQ-WP3-02).
  - Walk the tree; for each `ast.FunctionDef` / `ast.AsyncFunctionDef`, emit one entity.
  - Each entity: `{id: "python:function:...", kind: "function", qualified_name: ..., module_path: ..., source_range: {start_line, start_col, end_line, end_col}}`. `id` is built by `entity_id("python", "function", qualified_name)`.
- [ ] Failing tests: fixture `empty.py` → 0 entities (UQ-WP3-11); fixture `simple.py` with module-level `def hello()` → one entity with `id == "python:function:simple.hello"`; fixture `nested.py` with class methods and nested functions; fixture `syntax_error.py` → 0 entities + stderr log.
- [ ] Implement; run; expect pass.
- [ ] Commit: `feat(wp3): function extractor with L2 EntityId production`.

### Task 5 — Shared `EntityId` fixture (WP1 parity)

**Files**:
- Create `/fixtures/entity_id.json` at repo root
- Modify `/crates/clarion-core/src/entity_id.rs` tests to consume it
- Modify `/plugins/python/tests/test_entity_id.py` to consume it

Steps:

- [ ] Write the fixture: JSON array of objects each with `plugin_id`, `kind`, `canonical_qualified_name`, and `expected_entity_id` fields. At least 20 rows covering the representative cases from ADR-003 (both plugin-emitted `python:function:*` rows and core-reserved `core:file:*`/`core:subsystem:*` rows).
- [ ] Add a test in `clarion-core` that loads the fixture at test time, runs `entity_id()` for each row, asserts equality. Same-shaped test in `plugins/python` (the Python side concatenates its three segments; assertion is on the final string).
- [ ] Run both test suites; expect pass in both.
- [ ] Commit: `test(wp3): shared EntityId fixture (UQ-WP3-08 resolution)`.

### Task 6 — Wardline probe (L8)

**Files**:
- Create `/plugins/python/src/clarion_plugin_python/wardline_probe.py`
- Create `/plugins/python/tests/test_wardline_probe.py`

Steps:

- [ ] Failing tests:
  - `test_probe_absent`: stub `sys.modules` to raise `ImportError` for `wardline.core.registry`; probe returns `{"status": "absent"}`.
  - `test_probe_present_in_range`: stub to return a module with `__version__ = "0.1.5"` and a `REGISTRY` attr; probe returns `{"status": "enabled", "version": "0.1.5"}`.
  - `test_probe_out_of_range`: stub to return `__version__ = "0.3.0"` (above `max_version`); probe returns `{"status": "version_out_of_range", "version": "0.3.0"}`.
- [ ] Implement `probe(min_version: str, max_version: str) -> dict` using `importlib.import_module` in a `try/except ImportError`. Use `packaging.version.Version` for comparison (add `packaging` as a dep — first runtime dep; accepted because the alternative is hand-parsing semver).
- [ ] Wire probe result into the `initialize` handshake response's `capabilities` field.
- [ ] Run; expect pass.
- [ ] Commit: `feat(wp3): L8 Wardline REGISTRY probe with version pinning`.

### Task 7 — `plugin.toml` manifest + `analyze_file` end-to-end

**Files**:
- Create `/plugins/python/plugin.toml`
- Modify `/plugins/python/src/clarion_plugin_python/server.py` `analyze_file` handler

Steps:

- [ ] Write `plugin.toml` matching WP2 L5 schema: `[plugin]` (name, version, protocol_version, executable, `language = "python"`, `extensions = ["py"]`), `[capabilities]` (RSS 512MB, 300s runtime, 10MB frame ceiling, 100k entity cap), `[ontology]` (kinds = `["function"]`, edge_kinds = `[]`, `rule_id_prefix = "CLA-PY-"`, `ontology_version = "0.1.0"`), `[integrations.wardline]` (`min_version = "0.1.0"`, `max_version = "0.2.0"`).
- [ ] Arrange installation to place `plugin.toml` where WP2's discovery (L9) finds it: at install-prefix `share/clarion/plugins/clarion-plugin-python/plugin.toml`. Using `tool.setuptools` or `hatch` data-file declarations in `pyproject.toml`. Verify after `pip install -e .` the file is discoverable.
- [ ] Modify `analyze_file` handler: read the requested path, run `extractor.extract()`, return `{"entities": [...]}`.
- [ ] Commit: `feat(wp3): plugin.toml manifest + analyze_file wired to extractor`.

### Task 8 — Round-trip self-test

**Files**:
- Create `/plugins/python/tests/test_round_trip.py`

Steps:

- [ ] Test: spawn the installed plugin as a subprocess; complete handshake; call `analyze_file` on `plugins/python/src/clarion_plugin_python/extractor.py`; assert the returned entities include specific expected ones (the `extract` function itself, etc.); shutdown cleanly.
- [ ] Commit: `test(wp3): round-trip self-test against plugin's own source`.

### Task 9 — Sprint 1 walking-skeleton end-to-end

**Files**:
- Create `/tests/e2e/sprint_1_walking_skeleton.sh` (shell integration test)

Steps:

- [ ] Write the test as a shell script or `bats` test that runs the [README §3 demo script](./README.md#3-demo-script-sprint-1-close-proof) commands in sequence, asserting each step's expected output.
- [ ] Run locally end-to-end.
- [ ] Wire into CI (if CI exists; if not, document the manual run).
- [ ] Commit: `test(sprint-1): walking-skeleton end-to-end demo script`.

## 7. ADR triggers

None in Sprint 1. ADR-018 and ADR-022 are already Accepted. **However**, WP3 stress-
tests the L8 version-pin protocol; if the probe's shape turns out to be insufficient
(e.g., semver-range is the wrong abstraction), a ADR-018 amendment may be needed —
not a new ADR.

## 8. Exit criteria

WP3 is done for Sprint 1 when all of:

- The walking-skeleton demo script (README §3) passes end-to-end on a clean machine.
- L7 (qualname) and L8 (Wardline probe) each have passing positive and negative tests.
- The shared `EntityId` fixture (`fixtures/entity_id.json`) passes in both the
  Rust (`clarion-core::entity_id`) and Python (`test_entity_id.py`) test suites.
- Round-trip self-test passes.
- Every UQ-WP3-* is marked resolved in §5.
- `pip install -e plugins/python` works on a clean Python 3.11 venv and
  `clarion-plugin-python` is on `$PATH`.

See also [`signoffs.md` Tier A](./signoffs.md#tier-a--sprint-1-close-walking-skeleton).

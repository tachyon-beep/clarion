# WP2 — Plugin Host and Hybrid Authority (Sprint 1)

**Status**: DRAFT — blocked-by WP1
**Anchoring design**: [system-design.md §2 (Core/Plugin Architecture)](../../clarion/v0.1/system-design.md#2-core--plugin-architecture), [detailed-design.md §1 (Plugin implementation detail)](../../clarion/v0.1/detailed-design.md#1-plugin-implementation-detail)
**Accepted ADRs**: [ADR-002](../../clarion/adr/ADR-002-plugin-transport-json-rpc.md), [ADR-021](../../clarion/adr/ADR-021-plugin-authority-hybrid.md), [ADR-022](../../clarion/adr/ADR-022-core-plugin-ontology.md)
**Predecessor**: [WP1](./wp1-scaffold.md).
**Blocks**: WP3.

---

## 1. Scope (Sprint 1 narrow)

WP2 in Sprint 1 delivers the minimum plugin-host machinery the walking skeleton needs:
spawn one plugin subprocess, exchange a handshake, issue one `analyze_file` request,
receive entity output, shut the plugin down cleanly. All four ADR-021 core-enforced
minimums are implemented to spec — this is non-negotiable in Sprint 1 because those
enforcement points are what lock L6 and determine the plugin API contract.

**In scope for Sprint 1:**

- Content-Length-framed JSON-RPC 2.0 transport over subprocess stdin/stdout.
- Plugin-host module that spawns a plugin process, performs the handshake, issues
  requests, and supervises lifecycle.
- Plugin manifest schema (`plugin.toml`) per ADR-022, with a Rust parser that
  validates a manifest file and returns a typed `Manifest` value.
- Core-enforced minimums per ADR-021:
  - **Path jail** — canonicalise paths; any path escaping the analysis root is rejected
    before being passed to the plugin and after being returned from the plugin.
  - **Content-Length ceiling** — hard limit on a single frame's payload size;
    exceeding kills the plugin.
  - **Per-run entity-count cap** — total entities accepted per run bounded; exceeding
    halts the run with a dedicated error.
  - **prlimit RSS limit** — on Linux, `prlimit`-on-spawn applies an RSS ceiling drawn
    from the manifest's declared `max_rss_mb`.
- Ontology-boundary enforcement per ADR-022: the host rejects any emitted entity
  kind, edge kind, or rule-ID not declared in the plugin manifest.
- Crash-loop breaker: per-plugin crash count over a rolling window; trip-condition per
  ADR-002. For Sprint 1's single invocation, this is mostly scaffolding; a unit test
  proves the breaker trips. Respawn logic is implemented but the walking-skeleton demo
  does not exercise it (one analyze run = one spawn).
- In-process mock plugin used by tests.
- Wiring WP1's `clarion analyze` to discover and spawn plugins.

**Explicitly out of scope for Sprint 1:**

- Multi-plugin orchestration. Sprint 1 hosts one plugin at a time. Multi-plugin is
  NG-09 for v0.1.
- Streaming responses. All requests are unary (one request → one response).
- Dynamic plugin loading during `serve`. `analyze` spawns-per-run; `serve` is WP8.
- Plugin sandboxing beyond ADR-021's minimums (no seccomp, no namespace isolation).

## 2. Lock-in callouts

### L4 — JSON-RPC method set + Content-Length framing

**What locks**: the over-the-wire protocol between core and any plugin, per
[ADR-002](../../clarion/adr/ADR-002-plugin-transport-json-rpc.md) and
[detailed-design.md §1](../../clarion/v0.1/detailed-design.md#1-plugin-implementation-detail).

**Sprint 1 method set** (minimum viable for walking skeleton):

| Method | Direction | Purpose |
|---|---|---|
| `initialize` | Core → plugin | Handshake; exchanges protocol version + plugin manifest summary |
| `initialized` | Core → plugin (notification) | Signals the plugin may begin |
| `analyze_file` | Core → plugin | Per-file entity extraction; returns an array of entity objects |
| `shutdown` | Core → plugin | Graceful stop request; plugin must reply then exit |
| `exit` | Core → plugin (notification) | Forceful termination notification after `shutdown` reply |

**Methods deliberately deferred** (added in later sprints, not Sprint 1):
`file_list` (ADR-021 §2a path-jail target — deferred because Sprint 1 is
one-file-at-a-time via `analyze_file`; the jail helper is still written to
ADR-021 spec, ready for file_list in Tier B), `resolve_imports`,
`get_call_graph`, any incremental `analyze` variants. These are WP3's
feature-complete surface, not walking-skeleton.

**Framing**: Content-Length header + `\r\n\r\n` + JSON body. Exactly the LSP framing
shape. Content-Length ceiling is an L6 concern.

**Why now**: every future plugin (Rust plugin, Go plugin, etc.) speaks this.
Changing framing later = breaking every plugin implementation.

**Downstream impact**:
- WP3 implements the plugin side against this spec.
- `↗` No direct cross-product touch in Sprint 1, but if Wardline ever becomes a
  Clarion plugin (not planned for v0.1), it inherits this protocol.

### L5 — `plugin.toml` manifest schema

**What locks**: the schema of the manifest file every plugin must provide, per
[ADR-022](../../clarion/adr/ADR-022-core-plugin-ontology.md). WP2 ships the Rust
parser + validator; WP3 ships the first real manifest.

**Schema (Sprint 1)**:

```toml
[plugin]
name = "clarion-plugin-python"           # package name; informational (hyphens OK, human-readable)
plugin_id = "python"                      # identifier fed to entity_id(); must match [a-z][a-z0-9_]* (ADR-022)
version = "0.1.0"                         # semver
protocol_version = "1.0"                  # matches ADR-002 version
executable = "clarion-plugin-python"      # command on PATH (see L9)
language = "python"                       # informational tag
extensions = ["py"]                       # file extensions this plugin claims

[capabilities.runtime]
# Declarations, not enforcements. Per ADR-021 §Layer 1, these values describe
# the plugin's expected envelope; the core uses them for sanity-warnings and
# to pick a floor no stricter than the plugin requested. The four Layer-2
# minimums (Content-Length ceiling, entity-count cap, RSS limit, path jail)
# are core-enforced with fixed defaults a plugin cannot raise — see §L6.
expected_max_rss_mb = 512                 # plugin's own RSS estimate; effective prlimit = min(this, core default 2 GiB)
expected_entities_per_file = 5000         # triggers CLA-INFRA-PLUGIN-ENTITY-OVERRUN-WARNING well before the 500k hard cap (warning impl deferred — see Task 4)
wardline_aware = true                     # plugin reads wardline.core.registry.REGISTRY (WP3 L8)
reads_outside_project_root = false        # opt-out declaration; v0.1 refuses `true` at initialize (see Task 1 + Task 6)

[ontology]
entity_kinds = ["function", "class", "module", "decorator"]
edge_kinds = ["imports", "calls", "decorates", "contains"]
rule_id_prefix = "CLA-PY-"                # every emitted rule-ID must start with this; must match ADR-022 grammar (see Task 1)
ontology_version = "0.1.0"                # bump when entity/edge/rule set changes
```

**Reserved edge kinds**. Plugins may list the four core-reserved edge kinds
(`contains`, `guides`, `emits_finding`, `in_subsystem`) in `edge_kinds` to
signal they emit them, binding themselves to the core's semantics per
ADR-022. The semantics of those four kinds are fixed across all plugins;
plugins may not redefine them. Sprint 1's manifest schema carries no
per-edge-kind semantic annotations, so this compliance is automatic —
a manifest listing `contains` in `edge_kinds` parses successfully (Task 1
test).

**Rule-ID namespace**. Per ADR-022, `CLA-INFRA-*` is core-only (pipeline
findings), `CLA-FACT-*` is shared (any plugin or the core), and
`CLA-{PLUGIN_ID_UPPERCASE}-*` is reserved to that plugin. A manifest
declaring `rule_id_prefix = "CLA-INFRA-"` or `"CLA-FACT-"` is rejected at
parse (Task 1); emission-time enforcement of off-namespace rule IDs
(`CLA-INFRA-RULE-ID-NAMESPACE`) is deferred to the findings-emitting Tier B
sprint — Sprint 1's walking skeleton emits no findings, so the RPC-time
check has nothing to fire against.

**Manifest-validation finding codes surfaced by this WP**:

- `CLA-INFRA-MANIFEST-MALFORMED` — kind strings or `rule_id_prefix` fail
  ADR-022's identifier grammar (`[a-z][a-z0-9_]*` for kinds,
  `CLA-[A-Z]+(-[A-Z0-9]+)+` for rule-ID prefixes). Rejected at `initialize`;
  plugin fails to start.
- `CLA-INFRA-MANIFEST-RESERVED-KIND` — manifest declares `file`, `subsystem`,
  or `guidance` in `entity_kinds`. Rejected at `initialize`.
- `CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY` — manifest declares
  `reads_outside_project_root = true`; v0.1 has no mechanism to allow it.
  Rejected at `initialize`.
- `CLA-INFRA-RULE-ID-NAMESPACE` — manifest declares a reserved
  `rule_id_prefix` (`CLA-INFRA-` or `CLA-FACT-`). Rejected at parse.
  *Emission-time* enforcement (plugin emits a rule ID outside its namespace)
  is deferred to the findings-emitting Tier B sprint.
- `CLA-INFRA-PLUGIN-ENTITY-OVERRUN-WARNING` — per-file entity count exceeds
  `expected_entities_per_file`. Implementation deferred to the
  catalog-emitting Tier B sprint (Sprint 1 is one file per invocation; the
  warning has no useful surface area yet).

**Shape note — TOML vs YAML**. WP2 specifies the manifest in TOML to match
the `plugins.toml` convention used elsewhere in detailed-design §1;
ADR-021 §Layer 1 and detailed-design §1 show the same data in YAML. The
field names, types, and semantics are identical — the on-disk encoding is
a WP2 local decision, called out here so a future reader comparing docs
doesn't treat the TOML shape as drift.

**Why now**: this schema is the core/plugin ontology boundary. Once plugins author
manifests against it, schema changes become breaking.

**Downstream impact**:
- WP3's `plugin.toml` is the first instance.
- Every later plugin uses this schema.
- WP6 cache key (ADR-007) includes `ontology_version` — the field's name and
  semantic are locked here.

### L6 — Core-enforced minimums

**What locks**: the shape and enforcement points of the four ADR-021 minimums.
Plugins cannot opt out; plugin authors rely on these running for every run.

**Enforcement points** (defaults and subcodes per
[ADR-021](../../clarion/adr/ADR-021-plugin-authority-hybrid.md) §2):

| Minimum | Where enforced | Default | Behaviour on violation |
|---|---|---|---|
| Path jail (ADR-021 §2a) | Every path the plugin returns on `analyze_file` responses (file_path in source range, evidence anchors) | canonicalise via `std::fs::canonicalize`, reject if outside `project_root` | **Drop** the offending entity/edge/finding on **first offense** (do NOT kill plugin — path escape is more often a correctness bug than a live attack). Emit `CLA-INFRA-PLUGIN-PATH-ESCAPE` with `metadata.clarion.offending_path`. A dedicated sub-breaker counts repeats: **>10 path-escapes in 60s → plugin killed**, `CLA-INFRA-PLUGIN-DISABLED-PATH-ESCAPE`. |
| Content-Length ceiling (ADR-021 §2b) | Every inbound JSON-RPC frame from the plugin | **8 MiB** per frame (floor 1 MiB) | Framing parser refuses the frame before deserialising; plugin killed (SIGTERM → SIGKILL if non-responsive); emit `CLA-INFRA-PLUGIN-FRAME-OVERSIZE` with observed vs ceiling bytes. Crash-loop counter increments. |
| Entity-count cap (ADR-021 §2c) | Cumulative across all plugin-emitted `entity + edge + finding` records within one run | **500,000** combined records (floor 10,000) | In-flight batch flushed; plugin killed; run enters partial-results state; emit `CLA-INFRA-PLUGIN-ENTITY-CAP`. |
| Per-plugin RSS limit (ADR-021 §2d) | On spawn | **2 GiB** virtual-memory cap (`RLIMIT_AS`, floor 512 MiB) | `prlimit(RLIMIT_AS)` on Linux / `setrlimit` on macOS applied via `pre_exec`. Process killed by OS on exceed; core detects `WIFSIGNALED && WTERMSIG == 9` and emits `CLA-INFRA-PLUGIN-OOM-KILLED`. |

**Sprint 1 scope note — `file_list` deferred**. ADR-021 §2a also specifies
path-jail enforcement on `file_list` RPC returns. Sprint 1's walking skeleton
operates one file at a time via `analyze_file` only (see WP3 §1 "In scope");
the `file_list` RPC + its jail enforcement point is deferred to the
catalog-emitting Tier B sprint. This is deliberate — Sprint 1's jail tests
(§signoffs A.2.3 + §A.2) cover `analyze_file` response paths, not
`file_list` returns. When `file_list` lands, the same `jail.rs` helper from
Task 4 is reused; no re-design.

**Ceilings hierarchy**: the manifest's `capabilities` values are upper bounds
*the plugin declares for itself*. The core applies ADR-021's absolute
ceilings independently; the effective ceiling is `min(manifest, core)`.
Core ceiling config keys live under `clarion.yaml:plugin_limits.*` but
Sprint 1 hard-codes the ADR-021 defaults above (config surface deferred to
WP6).

**Why now**: enforcement semantics are what ADR-021 is for. Getting them right and
uniform across plugins is the point of the hybrid-authority model. Changing a
ceiling from "response-drop" to "run-halt" later forces re-testing every plugin
against the new behaviour.

**Downstream impact**:
- WP3 tests must cover both "plugin stays under ceiling" and "plugin exceeds
  ceiling, host kills it" paths.
- Every future plugin author assumes these semantics and writes against them.

### L9 — Plugin discovery convention

**What locks**: how the core finds plugin binaries at `clarion analyze` time.

**Three candidate conventions** (UQ-WP2-01 resolves this):

- **A. PATH-based**: look up `executable` from manifest on `$PATH` (like `git`
  finds `git-foo` subcommands). Pro: zero configuration, distro-native. Con:
  installation is user-dependent.
- **B. Explicit plugin dir**: a `~/.config/clarion/plugins/<plugin-name>/plugin.toml`
  layout. Pro: explicit, discoverable. Con: bespoke install step.
- **C. Config-listed paths**: `clarion.yaml` has `[[plugins]] manifest = "path"`.
  Pro: project-local plugin overrides. Con: requires config before `analyze`.

**Proposal**: **A with a fallback to B**. `clarion analyze` discovers plugins by
scanning `$PATH` for executables matching `clarion-plugin-*`, then loading each
one's `plugin.toml` from `<install-prefix>/share/clarion/plugins/<plugin-name>/plugin.toml`
(or next to the binary, whichever is found first). This matches the `git`
subcommand idiom and is the lowest-friction path for the WP3 Python plugin which
is pip-installable.

**Why now**: every plugin author builds their install story around this. Changing
the convention later breaks installation docs and packaging.

**Downstream impact**:
- WP3's `pip install -e plugins/python` must produce an executable on `$PATH`
  plus a manifest findable via this convention. See WP3 §"File decomposition"
  for the exact packaging.

## 3. File decomposition

Within `clarion-core` (new modules):

```
/crates/clarion-core/src/
  plugin/
    mod.rs                 # re-exports; the plugin-host facade
    transport.rs           # Content-Length framing; JSON-RPC frame encode/decode
    protocol.rs            # typed request/response structs for every L4 method
    manifest.rs            # plugin.toml parser + validator (L5)
    host.rs                # supervisor: spawn, handshake, request-response loop, shutdown
    jail.rs                # path-jail helper (L6)
    limits.rs              # Content-Length ceiling + entity-count cap + prlimit wiring (L6)
    discovery.rs           # plugin discovery (L9) — PATH scan + manifest load
    breaker.rs             # crash-loop breaker
    mock.rs                # in-process mock plugin (test-only; `#[cfg(test)]`)
```

The decision to put plugin support in `clarion-core` (rather than a new crate) keeps
the plugin types close to the domain types they produce. If that becomes unwieldy
later, splitting to `clarion-plugin-host` is a mechanical refactor.

`clarion-cli` gets a small update:

```
/crates/clarion-cli/src/
  analyze.rs               # modified: discover plugins, spawn, iterate files, persist entities
```

## 4. External dependencies being locked

New workspace dependencies introduced by WP2:

| Purpose | Candidate | Notes |
|---|---|---|
| TOML parsing | `toml` (serde-compatible) | Manifest parsing |
| JSON-RPC framing | hand-rolled over `serde_json` | Keeps dependency surface small; see UQ-WP2-02 |
| Async runtime | `tokio` (locked by ADR-011; WP1 already adopted) | WP2 reuses the same runtime — no separate `if adopted` branch |
| prlimit syscall | `nix` or `rustix` | `RLIMIT_AS` wrapper; Linux-only enforcement in Sprint 1 (see L6 §UQ-WP2-06) |

**No cross-sibling Rust-side deps in Sprint 1.** Wardline integration is Python-side
(WP3).

## 5. Unresolved questions

- **UQ-WP2-01** — **Plugin discovery convention (L9)**: proposal is PATH + manifest
  beside binary; see §2. **Resolution by**: Task 5.
- **UQ-WP2-02** — **JSON-RPC library choice**: hand-rolled over `serde_json` vs
  `jsonrpsee` (async, batteries-included) vs `jsonrpc-core` (mature but older).
  Hand-rolled wins on dep-surface; `jsonrpsee` wins on feature set (batching,
  bidirectional notifications). Walking skeleton uses unidirectional unary → hand-roll
  is enough. **Proposal**: hand-roll. **Resolution by**: Task 2.
- **UQ-WP2-03** — **Path jail semantics**: does canonicalisation follow symlinks? If
  yes, a symlink pointing outside the analysis root is rejected. If no, a symlink
  *within* the root that resolves outside is silently admitted. **Proposal**: yes,
  follow symlinks; reject-on-escape. **Resolution by**: Task 4.
- **UQ-WP2-04** — **Content-Length ceiling default**: ~~open~~ —
  **resolved by ADR-021 §2b**. Default ceiling is **8 MiB** per frame,
  floor 1 MiB, config key `clarion.yaml:plugin_limits.max_frame_bytes`
  (config surface deferred to WP6; Sprint 1 hard-codes the 8 MiB default).
  On exceed, the framing parser refuses the frame before deserialising,
  the plugin is killed (SIGTERM → SIGKILL if non-responsive), and
  `CLA-INFRA-PLUGIN-FRAME-OVERSIZE` is emitted. **Resolved**: Task 4.
- **UQ-WP2-05** — **Entity-count cap: cap per file or per run?** ~~open~~ —
  **resolved by ADR-021 §2c**. Per-run cumulative cap on
  `entity + edge + finding` notifications combined. Default **500,000**,
  floor 10,000, config key `clarion.yaml:plugin_limits.max_records_per_run`
  (config surface deferred to WP6; Sprint 1 hard-codes the 500k default).
  On exceed: current in-flight batch flushed, plugin killed, run enters
  partial-results state, `CLA-INFRA-PLUGIN-ENTITY-CAP` emitted.
  **Resolved**: Task 4.
- **UQ-WP2-06** — **prlimit on non-Linux**: ADR-021 §2d names both paths
  (`prlimit(RLIMIT_AS)` on Linux, `setrlimit(RLIMIT_AS)` on macOS — both
  POSIX). Sprint 1 scope is **Linux-only** per
  [WP1 §1 "Explicitly out of scope"](./wp1-scaffold.md#1-scope-sprint-1-narrow),
  so the macOS path described in ADR-021 is out of scope *for Sprint 1
  implementation* even though it's in scope *for the ADR*. Do we
  `#[cfg(target_os = "linux")]` the enforcement or compile an error?
  **Proposal**: `#[cfg]`-gate the Linux implementation; on non-Linux, log
  a warning once and proceed without the limit (the ADR-021 §2d macOS
  path lands when Sprint N adds macOS support). **Resolution by**: Task 4.
- **UQ-WP2-07** — **Shape of plugin non-entity output**: does the plugin write progress
  updates to stderr (free-form, the host just tees it to `tracing::info!`) or via JSON-RPC
  notifications (`$/progress`)? Walking skeleton doesn't need progress, but the
  convention is a lock-in-by-omission if not decided. **Proposal**: stderr is
  free-form and forwarded to tracing; progress notifications are deferred. Plugins
  that need structured progress add it in a later sprint. **Resolution by**: Task 3.
- **UQ-WP2-08** — **Plugin stdout discipline**: plugins must use stdout for JSON-RPC
  only. Stray `print()` statements in a Python plugin will corrupt framing. How do
  we enforce? **Proposal**: document in the WP3 plugin-author guide; the Python
  plugin bootstraps by replacing `sys.stdout` with a non-writable wrapper during
  initialisation. Not a core enforcement; plugin-level discipline. **Resolution by**:
  Task 3 (documented in plugin-author docs).
- **UQ-WP2-09** — **Manifest hot-reload**: should the host re-read the manifest on
  each analyze run, or cache it across runs within one `serve` session? Sprint 1 only
  has `analyze`, so always-reload is simplest. **Proposal**: always-reload in Sprint 1;
  revisit at WP8. **Resolution by**: Task 2.
- **UQ-WP2-10** — **Crash-loop breaker parameters**: ~~open~~ —
  **resolved by ADR-002 + ADR-021 §Layer 3**. General breaker:
  **>3 crashes in 60s** → plugin disabled, `CLA-INFRA-PLUGIN-DISABLED-CRASH-LOOP`.
  Path-escape sub-breaker (ADR-021 §2a): **>10 escapes in 60s** → plugin
  killed, `CLA-INFRA-PLUGIN-DISABLED-PATH-ESCAPE`. Sprint 1 hard-codes both
  thresholds; config surface deferred to WP6. **Resolved**: Task 7.
- **UQ-WP2-11** — **What happens if the plugin returns an `id` that doesn't
  match the 3-segment L2 format?** **Proposal**: host validates by
  reconstructing the `EntityId` from the entity's `plugin_id` (known — the
  emitting plugin), `kind`, and `qualified_name` fields and comparing against
  the returned `id`; mismatch = drop entity + emit
  `CLA-INFRA-PLUGIN-ENTITY-ID-MISMATCH`. This is the ontology-boundary
  enforcement (ADR-022) extended to the identity format (ADR-003).
  **Resolution by**: Task 6.

## 6. Task ledger

### Task 1 — Manifest parser (L5)

**Files**:
- Create `/crates/clarion-core/src/plugin/mod.rs`
- Create `/crates/clarion-core/src/plugin/manifest.rs`
- Modify `/crates/clarion-core/src/lib.rs` to `pub mod plugin;`

Steps:

- [ ] Define `Manifest`, `Capabilities`, `CapabilitiesRuntime`, `Ontology` structs mirroring the L5 schema. Use `serde` derive. `Capabilities.runtime` is the ADR-021 §Layer 1 sub-struct carrying `expected_max_rss_mb`, `expected_entities_per_file`, `wardline_aware`, `reads_outside_project_root`.
- [ ] Write failing tests:
  - Positive: parse a valid `plugin.toml` fixture and assert all fields populated, including `capabilities.runtime.*`.
  - Positive (F5 / ADR-022 §Core owns): manifest listing a core-reserved edge kind (`contains`) in `edge_kinds` parses successfully — plugins bind to core semantics by listing the kind, they do not redefine it.
  - Negative: missing `[plugin].name` returns a clear error.
  - Negative: `expected_max_rss_mb = 0` rejected (must be > 0).
  - Negative: `entity_kinds = []` rejected (must declare at least one).
  - Negative (ADR-022 identifier grammar): an entity kind not matching `[a-z][a-z0-9_]*` (e.g. `Function`, `func-tion`, `1function`) is rejected with `CLA-INFRA-MANIFEST-MALFORMED`.
  - Negative (ADR-022 identifier grammar): a `rule_id_prefix` not matching `CLA-[A-Z]+(-[A-Z0-9]+)+` (e.g. `PY-`, `cla-py-`, `CLA-py-`) is rejected with `CLA-INFRA-MANIFEST-MALFORMED`.
  - Negative (ADR-022 reserved kinds): a manifest declaring `file`, `subsystem`, or `guidance` in `entity_kinds` is rejected with `CLA-INFRA-MANIFEST-RESERVED-KIND`.
  - Negative (ADR-022 namespace registry): `rule_id_prefix = "CLA-INFRA-"` rejected with `CLA-INFRA-RULE-ID-NAMESPACE` (core-only namespace).
  - Negative (ADR-022 namespace registry): `rule_id_prefix = "CLA-FACT-"` rejected with `CLA-INFRA-RULE-ID-NAMESPACE` (shared namespace; plugins must use their own).
  - Negative (ADR-021 §Layer 1): a manifest declaring `reads_outside_project_root = true` produces a validator result that the supervisor (Task 6) surfaces at `initialize` as `CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY`. Task 1's test asserts the validator flags the manifest; Task 6's test asserts the handshake rejection path.
- [ ] Run tests; expect failures.
- [ ] Implement `pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, ManifestError>`. Include the two ADR-022 grammar regexes, the reserved-entity-kind list (`file`, `subsystem`, `guidance`), and the reserved-prefix list (`CLA-INFRA-`, `CLA-FACT-`).
- [ ] Run tests; expect pass.
- [ ] Commit: `feat(wp2): L5 plugin manifest parser and validator (ADR-021 §Layer 1 + ADR-022 grammar)`.

### Task 2 — JSON-RPC transport (L4)

**Files**:
- Create `/crates/clarion-core/src/plugin/transport.rs`
- Create `/crates/clarion-core/src/plugin/protocol.rs`

Steps:

- [ ] In `protocol.rs`, define typed request/response structs for `initialize`, `initialized`, `analyze_file`, `shutdown`, `exit`. Use `#[serde(tag = "method", content = "params")]` on an enum or separate structs keyed off method name.
- [ ] In `transport.rs`, implement `read_frame(reader: &mut impl BufRead) -> Result<Frame>` and `write_frame(writer: &mut impl Write, frame: &Frame) -> Result<()>`. A `Frame` is `(Content-Length, JSON bytes)`.
- [ ] Write failing round-trip tests: encode a frame → decode it → assert equality. Include edge cases: exact Content-Length boundary, trailing data beyond Content-Length treated as next frame's start.
- [ ] Write failing ceiling test: reading a frame with Content-Length above the configured ceiling returns `FrameTooLarge` without consuming the body.
- [ ] Run tests; expect failures.
- [ ] Implement round-trip and ceiling.
- [ ] Run tests; expect pass.
- [ ] Commit: `feat(wp2): L4 JSON-RPC Content-Length transport`.

### Task 3 — In-process mock plugin (test harness)

**Files**:
- Create `/crates/clarion-core/src/plugin/mock.rs`

Steps:

- [ ] Implement a mock plugin as a struct that owns a pair of pipes (or duplex channel) standing in for subprocess stdio. Provide `MockPlugin::new_compliant()` that returns one entity for every `analyze_file` call. Provide `MockPlugin::new_crashing()` that exits after the handshake. Provide `MockPlugin::new_oversize()` that responds with a frame larger than the Content-Length ceiling.
- [ ] Unit test that the compliant mock completes a handshake through the transport.
- [ ] Commit: `feat(wp2): in-process mock plugin test harness`.

### Task 4 — Core-enforced minimums (L6)

**Files**:
- Create `/crates/clarion-core/src/plugin/jail.rs`
- Create `/crates/clarion-core/src/plugin/limits.rs`

Steps:

- [ ] In `jail.rs`, implement `pub fn jail(root: &Path, candidate: &Path) -> Result<PathBuf, JailError>`. Canonicalise both via `std::fs::canonicalize` (follows symlinks per UQ-WP2-03); assert `canonical_candidate.starts_with(canonical_root)`. Return a typed `JailError::EscapedRoot { offending: PathBuf }` on violation — the *caller* decides whether to drop the record or kill the plugin (path-jail policy per ADR-021 §2a is drop-entity-not-plugin on first offense; see Task 6).
- [ ] Failing tests: a path inside the root is admitted; a path via `..` that escapes is rejected with `EscapedRoot`; a symlink inside the root pointing outside is rejected (UQ-WP2-03 resolution); a non-existent path is rejected.
- [ ] Implement; run; expect pass.
- [ ] In `limits.rs`, implement:
  - `ContentLengthCeiling` with **8 MiB default** per ADR-021 §2b, consulted by transport.rs (refactor transport.rs to take a `&ContentLengthCeiling` in Task 2's ceiling test).
  - `EntityCountCap` with **500,000 default** per ADR-021 §2c; `try_admit(delta: usize) -> Result<(), CapExceeded>` tracks cumulative `entity + edge + finding` across the run.
  - `PathEscapeBreaker` with ADR-021 §2a threshold (**>10 escapes in 60s**) — rolling counter consumed by Task 6's host when a `JailError::EscapedRoot` is observed on a plugin response.
  - `apply_prlimit_as(max_rss_mib: u64)` using `nix::sys::resource::setrlimit` inside `CommandExt::pre_exec` (pre-exec fork path) — applies `RLIMIT_AS` per ADR-021 §2d with **2 GiB default**. Effective limit = `min(manifest.capabilities.runtime.expected_max_rss_mb, core_default)`. `#[cfg(target_os = "linux")]`-gated (UQ-WP2-06); on non-Linux, log-once warning.
  - **Deferred**: `CLA-INFRA-PLUGIN-ENTITY-OVERRUN-WARNING` (ADR-021 §Consequences/Negative) — the per-file sanity warning fired when a plugin exceeds its declared `capabilities.runtime.expected_entities_per_file`. Sprint 1's walking skeleton is one file per invocation, so the warning has no useful surface area; implementation lands in the catalog-emitting Tier B sprint alongside multi-file discovery. Documented here so the subcode and declaration field stay wired end-to-end.
- [ ] Tests for each; commit.
- [ ] Commit: `feat(wp2): L6 core-enforced minimums — path jail, ceilings, prlimit (ADR-021 defaults)`.

### Task 5 — Plugin discovery (L9)

**Files**:
- Create `/crates/clarion-core/src/plugin/discovery.rs`

Steps:

- [ ] Write failing test: discovery finds a mock `clarion-plugin-*` binary on a test `$PATH` and loads its manifest from the expected location beside it.
- [ ] Implement: scan `$PATH` for entries matching `clarion-plugin-*`; for each, look for `plugin.toml` next to the binary first, fall back to `<install-prefix>/share/clarion/plugins/<name>/plugin.toml`.
- [ ] Run; expect pass.
- [ ] Commit: `feat(wp2): L9 plugin discovery convention (PATH + neighboring manifest)`.

### Task 6 — Plugin-host supervisor

**Files**:
- Create `/crates/clarion-core/src/plugin/host.rs`

Steps:

- [ ] Failing integration test: using a real subprocess (a tiny Rust binary in `tests/fixtures/` that speaks the protocol), `PluginHost::spawn(manifest, root)` completes a handshake, issues one `analyze_file` for a fixture, receives entities, and shuts down cleanly. Assert plugin exit code 0.
- [ ] Failing test (ADR-021 §Layer 1): a plugin whose manifest declares `capabilities.runtime.reads_outside_project_root = true` is refused at `initialize`; the host emits `CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY` and the plugin process is terminated before any `analyze_file` is issued. v0.1 has no mechanism to allow this capability.
- [ ] Failing test: ontology-boundary enforcement (ADR-022) — the fixture plugin emits an entity with `kind: "unknown"` not in the manifest; host drops it and emits `CLA-INFRA-PLUGIN-UNDECLARED-KIND`.
- [ ] Failing test: identity-mismatch rejection (UQ-WP2-11) — fixture plugin emits an entity whose `id` doesn't match `entity_id(plugin_id, kind, qualified_name)`; host drops it.
- [ ] Failing test: path-jail drop-not-kill (ADR-021 §2a) — fixture plugin emits an `analyze_file` response with a `source.file_path` that canonicalises outside `project_root`. Host drops the entity, emits `CLA-INFRA-PLUGIN-PATH-ESCAPE`, and the plugin remains alive for the next request.
- [ ] Failing test: path-escape sub-breaker (ADR-021 §2a) — fixture plugin emits 11 escaping paths within 60s; on the 11th, the host kills the plugin and emits `CLA-INFRA-PLUGIN-DISABLED-PATH-ESCAPE`.
- [ ] Implement `host.rs`:
  - Spawn subprocess with `std::process::Command`, stdin/stdout piped.
  - Apply `apply_prlimit_as` (from Task 4) inside `CommandExt::pre_exec` before `exec`, using `min(manifest.capabilities.runtime.expected_max_rss_mb, core_default = 2 GiB)`.
  - Perform handshake: send `initialize`, await response; send `initialized` notification.
  - **Before** sending `initialized`: if `manifest.capabilities.runtime.reads_outside_project_root == true`, emit `CLA-INFRA-MANIFEST-UNSUPPORTED-CAPABILITY`, send `shutdown` + `exit`, and return a host error to the caller. Do not dispatch any `analyze_file` requests. (ADR-021 §Layer 1: v0.1 has no mechanism to allow this capability.)
  - Provide `PluginHost::analyze_file(path: &Path) -> Result<Vec<Entity>>` that:
    - Runs the request-side path through the jail (jail error on request = host error returned to caller; no plugin involvement).
    - Sends request, awaits response.
    - For each returned entity/edge/finding: run its `source.file_path` and evidence-anchor paths through the jail. On `EscapedRoot`, drop the record, emit `CLA-INFRA-PLUGIN-PATH-ESCAPE`, and tick the `PathEscapeBreaker` counter. If the breaker trips, kill the plugin and emit `CLA-INFRA-PLUGIN-DISABLED-PATH-ESCAPE`.
    - Validate each surviving entity: ontology kind (ADR-022), `EntityId` reconstruction match.
    - Returns surviving entities.
  - On drop, send `shutdown` + `exit` + wait (with timeout).
- [ ] Run all Task 6 tests; expect pass.
- [ ] Commit: `feat(wp2): plugin-host supervisor with ADR-021 enforcement + ADR-022 ontology`.

### Task 7 — Crash-loop breaker

**Files**:
- Create `/crates/clarion-core/src/plugin/breaker.rs`

Steps:

- [ ] Failing test: using `MockPlugin::new_crashing()`, attempt to spawn and run the plugin N times in a rolling window; on the Nth failure, the breaker trips and refuses further spawn attempts for the configured cooldown.
- [ ] Implement per-ADR-002 parameters (hard-coded Sprint 1 per UQ-WP2-10).
- [ ] Run; expect pass.
- [ ] Commit: `feat(wp2): crash-loop breaker`.

### Task 8 — Wire `clarion analyze` to use the plugin host

**Files**:
- Modify `/crates/clarion-cli/src/analyze.rs`

Steps:

- [ ] Modify `clarion analyze`:
  - On start: discover plugins (Task 5).
  - For each discovered plugin, spawn (Task 6), iterate the source tree, call `analyze_file` per matching file (match against the manifest's `[plugin].extensions` field).
  - Persist returned entities via the writer-actor (WP1 Task 6).
  - On plugin error or cap hit, mark run as failed with diagnostic.
- [ ] Failing integration test: using the mock plugin fixture, `clarion analyze fixtures/demo.py` produces a run with `entity_count > 0`.
- [ ] Run; expect pass.
- [ ] Commit: `feat(wp2): wire clarion analyze to plugin host`.

### Task 9 — WP2 end-to-end smoke test

**Files**:
- Create `/crates/clarion-cli/tests/wp2_e2e.rs`

Steps:

- [ ] Integration test using the fixture mock-plugin binary: `clarion install` + `clarion analyze fixture_dir/` produces a completed run with the mock's expected entity persisted.
- [ ] Commit: `test(wp2): end-to-end smoke with mock plugin`.

## 7. ADR triggers

None in Sprint 1. ADR-002, ADR-021, ADR-022 are already Accepted and cover the WP.

## 8. Exit criteria

WP2 is done for Sprint 1 when all of:

- L4 (JSON-RPC method set + transport), L5 (manifest parser), L6 (each of the four
  minimums), L9 (discovery) each have ≥1 passing positive test and ≥1 passing
  negative test.
- `clarion analyze` with the mock plugin on a fixture produces persisted entities
  in the DB.
- Every UQ-WP2-* is marked resolved in this doc's §5.
- `cargo test --workspace` passes.

See also [`signoffs.md` Tier A](./signoffs.md#tier-a--sprint-1-close-walking-skeleton).

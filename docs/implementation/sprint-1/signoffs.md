# Clarion Sprint 1 — Sign-off Ladder

**Status**: DRAFT — becomes the closing gate for Sprint 1
**Scope**: [WP1](./wp1-scaffold.md), [WP2](./wp2-plugin-host.md), [WP3](./wp3-python-plugin.md)
**Read-with**: [`README.md`](./README.md), [`../v0.1-plan.md`](../v0.1-plan.md)

This document is the closing gate for Sprint 1. Sprint 1 is closed when every box in
**Tier A** is ticked. Tiers B and C are included so Sprint 2+ can extend this ladder
without rewriting the doc; they are not Sprint 1's gate.

Each tick must be accompanied by a verifiable artefact (a commit hash, a test-run
log, an ADR file, or a documentation update). Self-attested ticks without a pointer
are not sufficient.

---

## Tier A — Sprint 1 close (walking skeleton)

Every box below MUST be ticked for Sprint 1 to close. Lock-in ticks carry a
`locked on <YYYY-MM-DD>` stamp once ratified; after that date, revisiting the
locked design requires a follow-up ADR and cross-WP impact analysis.

### A.1 Storage layer (WP1)

- [ ] **A.1.1** — `cargo build --workspace --release` succeeds on a clean Linux checkout. Proof: CI log or commit hash.
- [ ] **A.1.2** — `cargo test --workspace` passes. Proof: test run log.
- [ ] **A.1.3** — **L1 locked**: migration file `0001_initial_schema.sql` contains every table, virtual table, trigger, generated column, and view from [detailed-design.md §3](../../clarion/v0.1/detailed-design.md#3-storage-implementation): tables `entities`, `entity_tags`, `edges`, `findings`, `summary_cache`, `runs`, `schema_migrations`; virtual table `entity_fts` (FTS5); triggers `entities_ai`, `entities_au`, `entities_ad`; generated columns `entities.priority` + `ix_entities_priority`, `entities.git_churn_count` + `ix_entities_churn`; view `guidance_sheets`. Proof: migration file commit; verification via `sqlite3 < migrations/0001_initial_schema.sql` against a fresh DB produces the expected schema; `schema_apply` integration test (WP1 Task 3) passes all assertions. _Locked on ______._
- [ ] **A.1.4** — **L2 locked**: `entity_id()` Rust assembler produces the 3-segment `{plugin_id}:{kind}:{canonical_qualified_name}` form per ADR-003 + ADR-022 and passes all rows in `/fixtures/entity_id.json`. Proof: passing test in `clarion-core`. _Locked on ______._
- [ ] **A.1.5** — **L3 locked**: `WriterCmd` enum and per-N-batch writer-actor shipped; per-command ack, batch-boundary commit, rollback on `FailRun` each have a passing test. Proof: tests in `clarion-storage`. _Locked on ______._
- [ ] **A.1.6** — `clarion install` in a fresh tempdir produces `.clarion/{clarion.db, config.json, .gitignore}` **plus** a `clarion.yaml` stub at the project root (per [detailed-design.md §File layout](../../clarion/v0.1/detailed-design.md#file-layout); `.clarion/` holds internal state, `clarion.yaml` is the user-edited config). Proof: integration test passing.
- [ ] **A.1.7** — `clarion install` refuses to overwrite an existing `.clarion/` without `--force`. Proof: negative integration test passing.
- [ ] **A.1.8** — `clarion analyze .` in a plugin-less scratch dir produces a `runs` row with status `skipped_no_plugins`. Proof: integration test passing.
- [ ] **A.1.9** — **ADR-005 authored** and moved from backlog to Accepted in [`../../clarion/adr/README.md`](../../clarion/adr/README.md). Proof: ADR file commit.
- [ ] **A.1.10** — Every UQ-WP1-* marked resolved in [`wp1-scaffold.md §5`](./wp1-scaffold.md#5-unresolved-questions). Proof: doc commit showing resolution state.

### A.2 Plugin host (WP2)

- [ ] **A.2.1** — **L4 locked**: JSON-RPC method set (`initialize`, `initialized`, `analyze_file`, `shutdown`, `exit`) + Content-Length framing round-trip tested. Proof: tests in `clarion-core::plugin::transport`. _Locked on ______._
- [ ] **A.2.2** — **L5 locked**: `plugin.toml` schema parsed and validated; rejects manifests missing required fields. Proof: tests in `clarion-core::plugin::manifest`. _Locked on ______._
- [ ] **A.2.3** — **L6 locked**: path jail (drop-not-kill on first offense per ADR-021 §2a; >10 escapes/60s sub-breaker), 8 MiB Content-Length ceiling, 500k per-run entity-count cap, 2 GiB `RLIMIT_AS` each have both positive and negative tests passing. Jail coverage is **`analyze_file` response paths only** — `file_list` RPC and its jail enforcement point are deferred to Tier B per WP2 §L4 and §L6. Proof: tests in `clarion-core::plugin::jail` and `clarion-core::plugin::limits`. _Locked on ______._
- [ ] **A.2.4** — **L9 locked**: plugin discovery finds `clarion-plugin-*` binaries on `$PATH` and loads neighboring `plugin.toml`. Proof: test in `clarion-core::plugin::discovery`. _Locked on ______._
- [ ] **A.2.5** — Ontology-boundary enforcement drops entities whose `kind` is not in the manifest's `[ontology].entity_kinds`. Proof: host integration test with a fixture plugin emitting an out-of-ontology entity.
- [ ] **A.2.6** — Identity-mismatch enforcement (UQ-WP2-11 resolution) drops entities whose `id` doesn't match `entity_id(plugin_id, kind, qualified_name)`. Proof: host integration test.
- [ ] **A.2.7** — Crash-loop breaker trips after the configured number of crashes in the configured window. Proof: test with `MockPlugin::new_crashing`.
- [ ] **A.2.8** — `clarion analyze` with the fixture mock plugin produces ≥1 persisted entity. Proof: `wp2_e2e` integration test.
- [ ] **A.2.9** — Every UQ-WP2-* marked resolved in [`wp2-plugin-host.md §5`](./wp2-plugin-host.md#5-unresolved-questions). Proof: doc commit.

### A.3 Python plugin (WP3)

- [ ] **A.3.1** — `pip install -e plugins/python` on a clean Python 3.11 venv places `clarion-plugin-python` on `$PATH`. Proof: install log.
- [ ] **A.3.2** — **L7 locked**: qualname reconstruction matches the documented rules for module-level, nested, class, async, and nested-class cases. Proof: `test_qualname.py` passing. _Locked on ______._
- [ ] **A.3.3** — **L8 locked**: Wardline probe returns the three documented states (`absent`, `enabled`, `version_out_of_range`). The handshake `capabilities` field carries the probe result. Proof: `test_wardline_probe.py` + `test_server.py` passing. _Locked on ______._
- [ ] **A.3.4** — Shared fixture `/fixtures/entity_id.json` passes in both `clarion-core` (Rust `entity_id()`) and `plugins/python` (Python `entity_id()`) test suites. Proof: both test runs green. **This is L2+L7 byte-for-byte alignment proof.**
- [ ] **A.3.5** — Round-trip self-test passes: plugin extracts entities from its own source and the host persists them. Proof: `test_round_trip.py` passing.
- [ ] **A.3.6** — Syntax-error files are skipped with a stderr log; the run continues (UQ-WP3-02). Proof: integration test with `syntax_error.py` fixture.
- [ ] **A.3.7** — Every UQ-WP3-* marked resolved in [`wp3-python-plugin.md §5`](./wp3-python-plugin.md#5-unresolved-questions). Proof: doc commit.

### A.4 End-to-end walking skeleton

- [ ] **A.4.1** — The [README §3 demo script](./README.md#3-demo-script-sprint-1-close-proof) runs end-to-end on a clean machine and each step produces the documented output. Proof: shell/bats test passing + demo-log attached to the closing commit.
- [ ] **A.4.2** — `sqlite3 .clarion/clarion.db "select id, kind from entities;"` after the demo returns `python:function:demo.hello|function` (per the locked 3-segment L2 format). Proof: demo log.
- [ ] **A.4.3** — No regression in pre-existing Clarion tests (there are none yet, but this box stays for later sprints' sanity). Proof: test log.

### A.5 Cross-product stance

- [ ] **A.5.1** — Sprint 1 has introduced **no changes** in the Filigree repo. Proof: Filigree `git log --since=<sprint-1-start>` shows no sprint-attributable commits.
- [ ] **A.5.2** — Sprint 1 has introduced **no changes** in the Wardline repo — only a pinned dependency on existing names (`wardline.core.registry.REGISTRY`, `wardline.__version__`). Proof: Wardline `git log --since=<sprint-1-start>` shows no sprint-attributable commits.
- [ ] **A.5.3** — L8 version-pin range (`min_version`, `max_version` in `plugin.toml`) is compatible with the current Wardline version at Sprint 1 close. Proof: probe returns `enabled` against `pip install wardline` in the dev venv.
- [ ] **A.5.4** — Any drift between Clarion's L7 qualname format and what Wardline's REGISTRY uses is documented (the first pass may uncover divergence). Proof: either "no divergence" note in the closing commit or an opened ADR-018 amendment ticket.

### A.6 Documentation hygiene

- [ ] **A.6.1** — [`../v0.1-plan.md`](../v0.1-plan.md) WP1/WP2/WP3 sections updated to reflect actual Sprint 1 narrower scope (Sprint 2+ scope clearly deferred). Proof: doc commit.
- [ ] **A.6.2** — [`../../clarion/adr/README.md`](../../clarion/adr/README.md) shows ADR-005 moved to Accepted. Proof: doc commit.
- [ ] **A.6.3** — [`README.md`](./README.md) §4 "Lock-in summary" table has every L-row marked with the `locked on <date>` stamp. Proof: doc commit.

---

## Tier B — Catalog-emitting (post-Sprint-1)

Tier B is the next natural milestone after the walking skeleton. These checkboxes
are **not** required to close Sprint 1 — they live here so the path forward is
visible. Sprint 2 may split B across multiple sprints.

- [ ] **B.1** — Phase 0 (discovery): `clarion analyze` walks a directory tree and dispatches one `analyze_file` call per matching file per plugin. Proof: integration test with a multi-file fixture.
- [ ] **B.2** — Python plugin emits **classes** and **module** entities in addition to functions. Ontology manifest updated accordingly.
- [ ] **B.3** — Python plugin emits **contains** edges (module → function, class → method).
- [ ] **B.4** — `catalog.json` is rendered after an analyze run, listing entities and edges. Proof: file produced, schema matches detailed-design §3.
- [ ] **B.5** — Per-subsystem markdown files rendered. (Subsystem detection may be naive — single flat subsystem for Tier B; WP4 clustering fills this in.)
- [ ] **B.6** — Demo extended: against the elspeth-slice fixture, `catalog.json` lists ≥95% of the Python classes and functions visible in the source (manually verified).
- [ ] **B.7** — No change in the Filigree or Wardline repos — Tier B is still Clarion-only work.

## Tier C — WP3 feature-complete

Tier C reaches WP3's full scope from `../v0.1-plan.md`. Checkboxes included for
ladder completeness; not required to close Sprint 1 or Sprint 2.

- [ ] **C.1** — Every Python entity kind from [detailed-design.md §1](../../clarion/v0.1/detailed-design.md#1-plugin-implementation-detail) emitted (functions, classes, methods, module-level variables, decorators, modules).
- [ ] **C.2** — Every Python edge kind emitted (`imports`, `calls`, `decorates`, `contains`, `inherits`).
- [ ] **C.3** — Import resolution: relative and absolute Python imports resolved to canonical entity IDs. Dynamic imports out of scope per NG-05-adjacent rule.
- [ ] **C.4** — Call-graph precision: intra-module calls resolved; inter-module calls resolved when import resolution succeeded; unresolved calls emitted as best-effort with a confidence marker.
- [ ] **C.5** — Every `CLA-PY-*` rule in [detailed-design.md §5](../../clarion/v0.1/detailed-design.md#5-pipeline--rule-catalogue-and-example-run) has positive and negative fixture coverage.
- [ ] **C.6** — Round-trip test against the full elspeth-slice passes and meets the plugin manifest's declared `max_rss_mb`.
- [ ] **C.7** — Identity reconciliation with Wardline exercised end-to-end on a real fixture (this is where L7/L8 divergence gets caught in practice, if any).

---

## Sign-off meta

- **Sprint 1 close requires**: every Tier A box ticked; for each L-row ticked, a
  `locked on <YYYY-MM-DD>` stamp present in this doc AND in [`README.md`](./README.md) §4.
- **Who signs**: the sprint owner (John Morrissey) — same author for all three Loom
  products, so no cross-team coordination needed. Sign-off proof is a single commit
  that updates this doc with all boxes ticked and the `locked on` stamps filled in.
- **Post-close**: open Sprint 2 as a new subfolder `docs/implementation/sprint-2/`
  following the same structure. Tier B and Tier C checkboxes move to the Sprint 2
  sign-off doc (or be split as Sprint 2's scope requires); the Tier A ladder here
  stays frozen as the historical record of Sprint 1's close state.

## References

- [`README.md`](./README.md) — sprint orientation and demo script.
- [`wp1-scaffold.md`](./wp1-scaffold.md), [`wp2-plugin-host.md`](./wp2-plugin-host.md), [`wp3-python-plugin.md`](./wp3-python-plugin.md) — per-WP plans.
- [`../v0.1-plan.md`](../v0.1-plan.md) — high-level plan for all 11 WPs.
- [`../../clarion/v0.1/plans/v0.1-scope-commitments.md`](../../clarion/v0.1/plans/v0.1-scope-commitments.md) — scope memo.

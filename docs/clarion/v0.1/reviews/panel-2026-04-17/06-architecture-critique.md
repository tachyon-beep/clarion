# Clarion v0.1 — Architectural Critique

**Assessor**: architecture-critic (panel 2026-04-17)
**Mode**: docs-only, pre-implementation
**Sources**: requirements.md, system-design.md, detailed-design.md, ADR-001..004, reviews/design-review.md, reviews/integration-recon.md, suite/loom.md
**Overall soundness**: **3 / 5 — Acceptable. No critical architecture defects, but scope is ambitious and three load-bearing assumptions are under-defended.**

---

## 1. Ambition vs capacity — HIGH

**Clarion v0.1 is trying to ship roughly six products in one release.** Count the v0.1-first concerns in `requirements.md`:

- Catalog producer with federated ontology (`REQ-CATALOG-01..07`)
- Multi-tier LLM orchestrator: budget preflight, per-level policy, profiles (`REQ-CONFIG-*`)
- Structured briefings: schema validation, four detail levels, prompt-injection defence (`REQ-BRIEFING-01..06`)
- Guidance system: composition algorithm, export/import, staleness, Wardline auto-derivation (`REQ-GUIDANCE-01..06`)
- SARIF v2.1.0 → Filigree translator as permanent suite feature (`REQ-FINDING-04`)
- MCP server with cursor sessions, 40+ tools, persistence (`REQ-MCP-01..06`)
- HTTP API with token auth, ETag caching, Prometheus metrics (`REQ-HTTP-01..04`)
- Filigree registry-backend integration, confirmed by recon as **schema surgery in a sibling repo** (`recon §2.2`)
- Wardline 42-decorator ingest, three-way identity reconciliation, fingerprint/exceptions parsing (`REQ-INTEG-WARDLINE-01..06`)

`NFR-PERF-01` (≤60 min wall-clock) and `NFR-COST-01` (`$15 ± 50%`) are unbenchmarked — both came from the design's own example run.

**Impact**: The "v0.1" label suggests a validating minimum. This is not that. A realistic v0.1 ships the catalog, the Python plugin, SQLite storage, and a local finding writer — deferring registry displacement, SARIF translator, MCP consult, HTTP auth, and Wardline-derived guidance to v0.2.

**Recommendation**: Anything whose acceptance test requires a mock of a sibling API that doesn't exist yet (`CON-FILIGREE-02`, `recon §2.6`) is a v0.2 dependency, not v0.1.

---

## 2. Architecture fitness — MEDIUM

Rust + JSON-RPC plugins + SQLite is defensible: single binary (`NFR-OPS-01`), subprocess isolation preserves Principle 3, SQLite gives the commit-DB story (`NFR-OPS-03`). These are coherent.

**But**:

- **ADR-001 barely justifies Rust.** The principal rejection of Go reads "the primary author directed Rust for v0.1" (`ADR-001:24,44`). That is preference, not engineering. Go meets every stated requirement; the ADR acknowledges "higher contribution and recruiting bar" (`:73`) without weighing it. For a solo-author project this is the load-bearing language choice, resting on 20 lines of alternatives analysis.

- **Premature architecture smells**:
  - `REQ-MCP-03` commits to 15 exploration-elimination shortcuts before any agent has used the MCP surface.
  - `§5 Prompt caching strategy` pre-designs four-segment cache layering; `NFR-COST-02` commits to ≥95% hit rate after stabilisation — a guess.
  - `REQ-INTEG-FILIGREE-03` makes Clarion *the* Filigree file registry via a config flag that does not exist in Filigree (`recon §2.2`: zero matches for `registry_backend`).

The architecture is probably correct in shape but over-committed in detail. The more concrete the pre-code design becomes, the more the first prototype will invalidate it.

---

## 3. JSON-RPC plugin transport — MEDIUM

Content-Length framing is a reasonable default. The failure modes are under-specified.

`ADR-002` picks right framing (binary-safe, LSP-style, resumable). The 100-message mpsc (`system-design.md §2`) gives coarse backpressure. **Under-specified**:

- **Latency budget**: every `analyze_file` is a subprocess round-trip with JSON serialisation both ways. No per-message latency target.
- **Reverse-flow backpressure**: core-to-plugin (`build_prompt` during consult) has no stated bound.
- **Binary payloads**: source-range content with non-UTF-8 bytes is unspecified; base64 encoding is implied nowhere.
- **Streaming granularity**: a module with ~500 entities streams ~500 JSON parses. No batching envelope.
- **Magic thresholds**: crash-loop "3 crashes in 60s", "abort at >10% file crash rate" (`§6`) are unexplained.

**Recommendation**: Write a plugin-protocol section naming both-direction latency targets, backpressure, binary encoding, batching.

---

## 4. Entity-ID scheme — MEDIUM (improvement over pre-restructure)

Symbolic canonical IDs (`ADR-003`) fix the worst problem. File paths demoted to properties. The pre-restructure path-embedded design was CRITICAL per `design-review.md §2.1`; that fix is real.

**What remains**:

- **Pure symbol renames still detach every cross-tool reference** (`ADR-003:82`). `EntityAlias` deferred. `REQ-ANALYZE-04` emits `CLA-FACT-ENTITY-DELETED` — that is visibility, not reconciliation.
- **Three identity schemes coexist** (Clarion `EntityId`, Wardline `qualname`, Wardline exception-register `location` — `system-design.md §3`, `recon §2.5`). Clarion owns the translation (`REQ-HTTP-02`), which is correct per Loom doctrine (`loom.md §6`). Reconciliation depends on Wardline's transient `module_file_map` (`recon §2.4`). If Wardline hasn't run, `resolve` silently returns `heuristic` — and the design doesn't specify what callers should do with heuristic resolution.
- **Re-exports produce alias proliferation**: `alias_of` edges (`REQ-PLUGIN-05`) should share summary-cache with definition site; the docset does not demonstrate the invariant.

Under-specified for federated use, not over-engineered. `EntityAlias` is load-bearing — schedule it before cross-tool references accumulate, not after.

---

## 5. Finding exchange format — MEDIUM

`ADR-004` makes the honest choice: Filigree-native intake over SARIF, because per `recon §2.1` that's what Filigree actually ingests.

**Three concerns**:

1. **`metadata.clarion.*` nesting is not a suite convention.** Wardline will nest under `metadata.wardline_properties.*`; Shuttle will invent a third. Loom has no shared vocabulary for finding metadata. The design chose right for v0.1 and punted on the standard.

2. **SARIF compatibility ≠ SARIF interoperability.** Clarion imports SARIF (`REQ-FINDING-04`) but never emits it. One-way. Third-party SARIF consumers cannot read Clarion findings.

3. **Severity-enum drift across the suite is unresolved.** Three vocabularies in `recon §2.5`. `REQ-FINDING-03` maps Clarion→Filigree one way only.

**Recommendation**: Write a one-page "Loom Finding Exchange" spec in `docs/suite/` pinning the `metadata.{tool}_properties.*` convention and severity mapping. Three tools silently diverging here is the failure mode `loom.md §5` warns against.

---

## 6. Load-bearing assumptions — HIGH

Decisions with the highest "if wrong, rewrite" cost, ranked by evidence weakness:

| Decision | Evidence | Rewrite cost |
|---|---|---|
| SQLite + writer-actor under commit-DB workload | `§4`, `CON-SQLITE-01` | High. Shadow-DB alternative (ADR-011) is a different consistency model for consumers, not a drop-in. |
| Prompt-caching as the cost-control mechanism | `CON-ANTHROPIC-01`, `§5` | High. Cache hit <80% at elspeth → `$15` budget blows out; plugin `build_prompt` needs refactor (design admits this at `§5 Honest portability framing`). |
| Leiden clustering drives subsystem-synthesis quality | `§6 Phase 3`, backlog ADR-006 | High. ~50 Opus calls per run ride on cluster quality. No acceptance criterion for cluster quality. |
| Elspeth is market-representative | `NFR-SCALE-01` | Medium. v0.1 is Python-only; next customer shape is unknown. |
| Filigree-as-finding-broker is the right federation shape | `REQ-FINDING-03`, `§9` | Medium. Clarion is building the suite's cross-tool fabric (Wardline has no HTTP client, `recon §2.6`). If siblings later build intake paths, this becomes plumbing. |

Common theme: mechanisms whose failure modes aren't measured. Unavoidable at docs-only stage, but the design treats speculative numbers (±50% cost, ≥95% hit rate, ≤60 min) as acceptance criteria rather than hypotheses. A prototype should displace these before the rest of the architecture is frozen.

---

## 7. Missing architectural concerns

- **Concurrency — interaction gap**: writer-actor + read pool (`§4`) covers storage. `NFR-SCALE-03` claims 16 reads handle "one consult agent + one Wardline puller." Multiple consult agents + summary-cache writes during consult + analyze writer: not modelled.
- **Versioning — incomplete**: `NFR-COMPAT-01..03` pins Filigree `_schema`, Wardline REGISTRY, Anthropic SDK. **Missing**: JSON-RPC plugin-protocol version, manifest version, DB schema migrations for users with `.clarion/clarion.db` committed to git across Clarion versions.
- **Observability — breadth without depth**: Prometheus + stats.json + structured logs + compat-report are good. Missing: tracing (OpenTelemetry, span propagation across plugin boundary), sampling strategy for `log.jsonl` (elspeth = ~2,000 LLM calls, large log).
- **Performance budget — claimed, not designed**: 100ms MCP init, 50ms p95 (`NFR-PERF-02`). Source unspecified. No per-tool breakdown. No cold-start vs warm.
- **Git-committable DB is an unsolved social problem**: merge-helper is "last-writer-wins on `updated_at`" (`§4`). Two developers' divergent summaries: the helper silently drops one. For a team paying $15/run, losing half the runs in merges is real money.

---

## 8. Second-order effects

**Makes easy**: new-language plugins in any host language; new entity/edge kinds without core changes (Principle 3); local-only operation (`CON-LOCAL-01`); LLM spend audit (`NFR-OBSERV-02`); namespaced rule IDs (`REQ-FINDING-02`).

**Makes hard**:

- **Multi-repo/multi-project**. `.clarion/` is per-project; shared-library entities across services have no mechanism.
- **Incremental analyze**. One-file change re-runs graph completion, clustering, subsystem synthesis. Content-hash caching reduces LLM cost, not phase cost.
- **Provider diversity**. `CON-ANTHROPIC-01` acknowledges: second provider without caching refactor is v0.3+.
- **Live-collaboration**. Git-committable SQLite is not a server. Simultaneous analyze runs are not supported.
- **Wardline vocabulary coordination**. Every new Wardline decorator requires a Clarion plugin release until v0.2 inverts this (`design-review.md §8`).
- **Replacing Clarion in the suite**. Once Filigree uses `registry_backend: clarion`, removing Clarion requires a Filigree migration. Gravitational pull toward "Clarion always running" — the soft centralisation `loom.md §5` warns against. Shadow-registry fallback exists; the pull remains.

---

## Confidence, Risk, Gaps, Caveats

**Confidence**: High on structural findings — requirements, system-design, and ADRs are detailed enough to assess. Low on cost/latency numbers (`NFR-COST-01`, `NFR-PERF-01`, `NFR-COST-02`) — unbenchmarked; should not be acceptance criteria.

**Risk**: Biggest risk is *aggregate commitment* at docs-only stage. Every measured reality (cache hit rates, subsystem cluster quality, elspeth wall-clock) will push against frozen design. The "Revision 5" provenance shows diminishing returns on further docs iteration without code.

**Gaps**: Did not exhaustively read detailed-design (49k tokens); some concerns may resolve there. Elspeth's actual shape (import graph density, decorator density) is unknown to me. No prior prototype measurements referenced.

**Caveats**: Paper-on-paper review. Prototype measurements will invalidate some findings. A v0.1 shipping smaller scope with measured numbers could be a 4 at code-complete.

---

## Three architectural risks most likely to bite in the first 6 months

1. **SQLite + writer-actor breaks under sustained concurrent `analyze` + `serve` on elspeth-scale DBs.** The design already names shadow-DB swap as an alternative (`§4`, ADR-011), which tells you the authors know the writer-actor model may not hold. First customer running `clarion serve` during a fresh `analyze` in Phase 4 will hit WAL growth, checkpoint starvation, and busy-timeout errors within the first week. Mitigation is a consistency-model change for consumers, not a tuning pass.

2. **Filigree `registry_backend` integration blocks v0.1 ship or forces a scope cut.** `REQ-INTEG-FILIGREE-03` depends on a Filigree-side flag that doesn't exist (`recon §2.2`, `CON-FILIGREE-02`). 5-8 hot files plus FK rework across a sibling tool. Either Clarion waits on Filigree, writes the Filigree patch itself (cross-repo, not in scope), or ships in shadow-registry mode — in which case the "Clarion owns structural truth" story isn't realised until v0.2. The design admits this via fallback; in practice, shadow mode means the integration story is marketing.

3. **Cost budget blows past `$15 ± 50%` on elspeth and invalidates the profile presets.** `NFR-COST-01`/`NFR-COST-03` are unmeasured; four-segment prompt caching is a hypothesis. When the first full run costs $40 at 60% hit rate, `budget / default / deep` become decorative, preflight loses credibility, and the "enterprise at lack of scale" pitch (`requirements.md:35`) takes a direct hit. This is the risk most likely to make the first customer abandon the tool before the catalog proves its value.

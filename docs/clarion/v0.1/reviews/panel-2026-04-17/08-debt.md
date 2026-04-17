# Clarion v0.1 — Pre-Implementation Debt Catalog

**Panel date**: 2026-04-17
**Scope**: debt embedded in the v0.1 design docset *before any code exists*.
**Summary**: **5 critical, 11 major, 10 minor** (26 items). Sources: `requirements.md`, `system-design.md`, `detailed-design.md`, `adr/README.md`, ADR-001 through ADR-004, `reviews/integration-recon.md`.

---

## Critical (blocks implementation)

| ID | Location | Kind | Interest rate | Suggested fix |
|---|---|---|---|---|
| **DEBT-C1 — 16 unauthored ADRs on the P0/P1 hot path** | `adr/README.md` backlog; `system-design.md:1179-1201` | missing-decision | Very high. Eight are P0 and load-bearing (ADR-006 clustering; ADR-007 cache key; ADR-011 writer-actor; ADR-014 registry backend; ADR-015 Wardline emission; ADR-016 observation transport; ADR-017 severity/dedup; ADR-018 identity reconciliation). Each is one sentence of rationale — alternatives, consequences, and rejection arguments unwritten. Without them, the first implementer litigates each in PR threads. | Author the 8 P0 ADRs as standalone files before Phase 1 code lands. |
| **DEBT-C2 — Filigree `registry_backend` is a hard dependency on work that does not exist** | `CON-FILIGREE-02`; `system-design.md:908-914, 1149-1152`; `integration-recon.md §2.2, §3.6` | unjustified deferral | Very high. Recon proved this is schema surgery (4 NOT-NULL FKs, 3 auto-create paths, ~5-8 hot files in Filigree). Requirements already downgrade to "shadow-registry mode" — meaning the headline "Clarion owns structural truth" is not delivered in v0.1. No Filigree-side design, no named owner, no cadence. | Author a Filigree-side `RegistryProtocol` design (joint, not Clarion-only). Until it lands, rewrite Clarion docs to say "Clarion owns the entity catalog; Filigree shadows the file mapping." Drop the aspirational framing. |
| **DEBT-C3 — Observation transport has no committed path** | `REQ-INTEG-FILIGREE-02`; `system-design.md:916-922`; `integration-recon.md §3.7, §4.8` | placeholder | High. "HTTP preferred, MCP fallback" names two unwritten implementations: Filigree has no `POST /api/v1/observations`; Clarion has no MCP client. Fallback doubles Clarion's transport surface (Rust binary spawning Python MCP subprocess) with no auth, failure-mode, or lifecycle spec. If the HTTP endpoint slips, Clarion inherits permanent MCP-client baggage. | Author ADR-016 with a Filigree owner/date. Define fallback failure modes and subprocess auth posture. |
| **DEBT-C4 — Summary-cache key is specified but not proven** | `REQ-BRIEFING-03`; `NFR-COST-02`; ADR-007 (backlog) | speculation-dressed-as-decision | High. The ≥95% stabilised hit rate and ±50% cost estimator both assume the 5-tuple key over-invalidates rarely and under-invalidates never. No invalidation matrix exists; TTL-backstop default of 180 days is unjustified. If the key over-invalidates, the `$15 ± 50%` elspeth budget is fiction. | Author ADR-007 with a full invalidation matrix (edit-type × entity-scope) and a spike on `elspeth-slice` before `NFR-COST-01/03` are locked. |
| **DEBT-C5 — Wardline-to-Filigree flow is assigned to Clarion but never decided** | `REQ-FINDING-04`, `REQ-INTEG-WARDLINE-05`; `system-design.md:957-971`; `integration-recon.md §3.5` | unjustified deferral | High. Wardline has zero HTTP client code today; CI uploads SARIF to GitHub Security, not Filigree. Clarion v0.1 absorbs the bridge via `clarion sarif import`. Severity mapping and rule-ID round-trip (ADR-017) are P0 backlog. | Author ADR-015 + ADR-017 jointly. Size the Wardline bridge inside v0.1. Confirm with Wardline owner that native emission is genuinely v0.2, not perpetually slipping. |

---

## Major (implementation hits it in weeks 1-4)

| ID | Location | Kind | Interest rate | Suggested fix |
|---|---|---|---|---|
| **DEBT-M1 — Clustering algorithm undecided; v0.1 "vendors ~400 lines of Leiden"** | `detailed-design.md:1644`; ADR-006 | hand-wave | Clustering drives Phase-6 Opus calls (most expensive phase). "Vendors a minimal Leiden or pulls a crate if one exists by implementation time" is a coin flip deferred to the engineer. | Spike two candidates + hand-roll; author ADR-006 with measured modularity on `elspeth-slice`. |
| **DEBT-M2 — Wardline tier vocabulary mismatch** | `integration-recon.md §3.4`; detailed-design glossary | inconsistency | Wardline uses `INTEGRAL/ASSURED/GUARDED/EXTERNAL_RAW`; earlier drafts invented `T1..T4`. Correction absorbed in some places, not all. Briefing controlled-vocabulary depends on consistency. | Grep for `T1..T4` in tier contexts; normalise to Wardline's actual names. |
| **DEBT-M3 — Three identity schemes, no reconciliation spec** | `REQ-INTEG-WARDLINE-06`; `integration-recon.md §4.15` | missing-decision | Clarion EntityId, Wardline `qualname`, and exception-register `location` string are three strings for the same element. `/entities/resolve` is named as the oracle; the algorithm and confidence-band criteria are not. | Author ADR-018 with per-scheme resolution matrix and fixture tests. |
| **DEBT-M4 — Tautological acceptance criteria pervasive** | `REQ-CATALOG-01`, `REQ-CATALOG-05`, `REQ-PLUGIN-04`, others | placeholder | `≥1 entity per known kind` is satisfied by any working-or-broken implementation. Pattern lets quality regress under time pressure. | Replace per-requirement verification lines with concrete counts/coverage/ratios. Borrow from `NFR-SCALE-01`'s style. |
| **DEBT-M5 — `mark_unseen=true` dedup is an explicit "v0.1 compromise"** | `REQ-FINDING-06`; `system-design.md:902-906` | unjustified deferral | Finding-flap on within-file moves; triage state lost on move. 30-day prune default unjustified. | Either commit Filigree v0.2 server-side per-entity dedup with a date, or synthesise entity-qualified `rule_id` so dedup has entity granularity today. |
| **DEBT-M6 — Controlled-vocabulary seed list does not exist** | `REQ-BRIEFING-02`; `NFR-SEC-02` | placeholder | `patterns`/`antipatterns`/`risk.tag` vocabulary is load-bearing for injection defence and comparability, yet no seed list exists. The novel-tag detector ships without knowing what "novel" means. | Write 50-100 seed tags as an appendix before Phase 4. Review adversarially. |
| **DEBT-M7 — ~12 "default N, configurable" thresholds with no rationale** | `REQ-GUIDANCE-05` (50/20), `REQ-MCP-06` (1h), `detailed-design.md:141, 914, 971, 1000, 1632` (30s, 0.5, 0.1, 100-msg) | hand-wave | Every threshold is a guess wrapped in a config key. First real user hits a wrong default. "Just tune it" becomes the support answer. | One-line rationale per threshold; commit fixtures measuring the tail so defaults are empirical. |
| **DEBT-M8 — Class-decoration "Clarion augmentation" undefined** | `integration-recon.md §4.6`; `REQ-PLUGIN-06` | missing-decision | Clarion detects class decorators Wardline itself doesn't. Rule-ID namespace and briefing/guidance precedence when augmentation disagrees with Wardline-authoritative are undefined. | Define `CLA-AUG-*` namespace and precedence rules. |
| **DEBT-M9 — Preflight cost estimator has no methodology** | `NFR-COST-03`; system-design §5 | placeholder | ±50% is stated; technique ("per-entity-level heuristics") is hand-waved. First calibration is elspeth with no corrective data. | Publish estimator algorithm in ADR-007 or sibling; cross-check on a non-elspeth fixture. |
| **DEBT-M10 — Compat report covers probes but not remediation** | `REQ-INTEG-FILIGREE-05`, `NFR-OBSERV-04`; system-design §11 | placeholder | 8 degraded modes in §11 each need operator-action text. Report tells you what's missing, not what to do. | Populate each fallback-table row with a specific operator action. |
| **DEBT-M11 — Wardline auto-derived guidance "edit preservation" undefined** | `REQ-GUIDANCE-04` | speculation-dressed-as-decision | `source: wardline_derived_overridden` is the preservation mechanism but the detection semantics (hash? diff? timestamp?) are unspecified. Foreseeable: regeneration clobbers user edits on a subtle formatting change. | Specify detection (normalised-text hash); write a fixture test. |

---

## Minor (can live with it, but cheap to fix now)

| ID | Location | Kind | Interest rate | Suggested fix |
|---|---|---|---|---|
| **DEBT-m1 — TTL defaults (180/90 days) unjustified** | `REQ-BRIEFING-03/04` | placeholder | Low magic numbers. | Add rationale or mark arbitrary. |
| **DEBT-m2 — "Typical briefings ~half these values"** | `REQ-BRIEFING-06` | hand-wave | Low. Expectation without measurement. | Drop or replace with measured number post-integration. |
| **DEBT-m3 — Entity-ID length/collision bound unspecified** | ADR-003; `REQ-CATALOG-06` | missing-decision | Low-medium. Canonical names can be arbitrarily long (generics, protocols); MCP arg budgets are not. | Cap length or specify hash-fallback past N chars. |
| **DEBT-m4 — "Hardcoded registry mirror" generation mechanism unspecified** | `REQ-INTEG-WARDLINE-01` | placeholder | Low. Who writes `wardline_registry_v<pin>.py`? On what schedule? | Define: auto-generated at Clarion release time from `REGISTRY.dump()`. |
| **DEBT-m5 — SARIF property-bag forward compat** | `integration-recon.md §4.1` | placeholder | Low. 44 `wardline.*` keys pass through; behaviour on a 45th is "best effort." | Policy: unknown keys preserved; emit finding on new-key-first-seen. |
| **DEBT-m6 — Plugin manifest schema version missing** | `REQ-PLUGIN-02` | missing-decision | Low. No `manifest_version` field breaks old plugins silently later. | Add `manifest_version: 1`. |
| **DEBT-m7 — "Use project-scoped key" is operator prose, not enforced** | `system-design.md:1098-1102` | hand-wave | Low. Real risk (personal key charged for team DB). | Preflight warning: detect `sk-ant-` vs `sk-proj-` shapes. |
| **DEBT-m8 — `clarion check-auth --from wardline` not catalogued** | `system-design.md:1013` | placeholder | Low. CLI command mentioned in passing. | Add to the CLI command table. |
| **DEBT-m9 — TLS "terminate at reverse proxy" has no NG-** | `system-design.md:1015` | unjustified deferral | Low-medium. Acceptable posture but will be re-litigated without a non-goal. | Promote to `NG-26: No native TLS in v0.1`. |
| **DEBT-m10 — Dirty-tree `commit_ref` handling unreconciled with Wardline** | `integration-recon.md §4.13` | missing-decision | Low. `REQ-CATALOG-07` strips `-dirty`; Wardline SARIF preserves it. | One-line reconciliation. |

---

## Pattern observations

- **P0 ADR backlog is the dominant shape of the debt.** 8 of 16 unauthored ADRs are P0. The docset is not "undecided"; it's "decided at one-sentence resolution." Authoring canonical ADRs is the highest-leverage paydown.
- **Cross-tool recon did its job.** Integration-recon surfaced real gaps; requirements now carry `CON-*` / `NG-*` entries. Remaining debt: those constraints were recorded but the remediation work wasn't scheduled.
- **Threshold culture.** A dozen magic numbers (30s, 50 commits, 16 connections, 0.5 drift, 30-day prune, 100-msg backpressure, 180-day TTL, 2GB DB, ±50% estimator, $15 budget). None critical, all speculative — they form a debt class on their own.

## Limitations

- System-design read fully; detailed-design sampled via grep (not full-read). Thresholds buried in uncommented code blocks may be missing.
- I did not cross-check against `reviews/design-review.md`; some items may double-count with findings there.
- Severity is my classification; DEBT-C4 and DEBT-M5 are the likeliest to flip category.

## Confidence / risk / caveats

- **High confidence**: DEBT-C1, C2, C3, C5 (directly evidenced by integration recon + ADR backlog).
- **Medium confidence**: DEBT-C4, M1, M3, M7 (inferred load-bearing-ness from prose).
- **Lower confidence**: minor items — some are judgement calls on inclusion.

**Top risks of not paying this down before coding**:
1. Implementation-time ADR litigation — 8 P0 decisions resolved in PR threads.
2. Cost-model fiction — DEBT-C4 + M9 together mean `$15 ± 50%` is speculation; first real user may see $40+.
3. Permanent cross-tool fragility — DEBT-C2 + C3 + C5 are infrastructure Clarion quietly absorbs. If those dependencies slip, Clarion ships a narrower product than its headline claims.

**Caveats**: debt estimates assume stated v0.1 scope; if scope contracts, several major items drop to minor. "Interest rate" is qualitative — quantifying needs spikes this review did not run.

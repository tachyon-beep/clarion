# Post-Commitment Work Brief — Clarion v0.1

**Status**: ready for execution
**Date opened**: 2026-04-18
**Audience**: a fresh Claude Code session (or human engineer) picking up Clarion v0.1 rework after the scope-commitment memo
**Expected duration**: 2–3 days of focused writing + a half-day cost-model spike

This is a self-contained handoff brief. Read it before touching anything. All referenced paths are relative to `/home/john/clarion/`.

---

## 1. Context (60 seconds)

Clarion is a security-analysis / code-archaeology product in the Loom suite (Clarion + Filigree + Wardline, plus a proposed Shuttle). The suite is single-author — one person maintains all three running products — so "cross-team coordination" does not apply to Filigree or Wardline work.

A 10-agent panel review completed on 2026-04-17 and produced an executive synthesis and 10 specialist reviews under `docs/clarion/v0.1/reviews/panel-2026-04-17/`. The panel's conclusion was **conditional-go with rework gates**.

The three scope commitments the panel said only the author could decide are now decided. They are captured in **`docs/clarion/v0.1/plans/v0.1-scope-commitments.md`** with status `DECIDED`. The decisions are not open for re-litigation in this work.

**Committed decisions, terse form:**

- **Q1**: Minimal-core v0.1, carving in `registry_backend`. Deferred to v0.2: Wardline→Filigree SARIF bridge, observation HTTP transport, summary cache beyond in-memory, HTTP write API.
- **Q2**: Filigree `registry_backend` is a v0.1 Clarion commitment. Filigree-side implementation is within-scope (same author owns Filigree).
- **Q3**: Plugin authority model is hybrid — plugin declares capabilities in its manifest, core enforces minimum-safe controls (path jail, Content-Length ceiling, entity-count cap, per-plugin `ulimit` RSS).

If anything you read in this brief looks inconsistent with those commitments, trust the commitments.

---

## 2. What has already been done (do not redo)

Day 1 of the post-commitment plan landed before this brief was written. The following edits are in place:

- `docs/suite/loom.md` §5 — failure test broadened to three coupling modes; new "v0.1 asterisks" subsection naming the SARIF triangle and the plugin `REGISTRY` import with retirement conditions.
- `docs/suite/briefing.md` — data-flow row 4 marked v0.2; "What Clarion v0.1 ships" rewritten as minimal-core; "What the suite needs" split into v0.1 commitments and v0.2 deferrals.
- `docs/clarion/v0.1/system-design.md` §12 — canonical ADR table with rationale summaries. ADR-021 and ADR-022 added. Priority promotions (006, 007, 011, 013 → P0 per panel) noted in Writing Cadence.
- `docs/clarion/v0.1/detailed-design.md` §11 — narrowed to a "where captured in this design" cross-reference table (no drift surface).
- `docs/clarion/adr/ADR-001-rust-for-core.md` — amended: requirements cited (NFR-OPS-04, CON-SQLITE-01, REQ-PLUGIN-*, NFR-COST-*, CON-LOCAL-01); Go steelmanned with concrete technical trade-offs.
- `docs/clarion/adr/README.md` — ADR-021 and ADR-022 added to backlog; commitment memo linked.
- `docs/clarion/v0.1/README.md` — commitment memo linked under supporting docs.

Do not re-edit `loom.md` §5 or the briefing scope language. Do not re-litigate ADR-001. Do not re-open Q1/Q2/Q3.

---

## 3. Work remaining

Three blocks of work, listed in recommended order. Blocks 2 and 3 can overlap.

### Block A — ADR writing sprint (2–3 days, ~11 ADRs)

Author each ADR as a standalone markdown file in `docs/clarion/adr/` following the format established by the four existing authored ADRs (`ADR-001-rust-for-core.md` through `ADR-004-finding-exchange-format.md`). The format is: **Status / Date / Deciders / Context / Summary / Context / Decision / Alternatives Considered / Consequences / Related Decisions / References**.

Stay tight. Each ADR should be 60–120 lines. Alternatives must be real (steelman them); consequences must include both sides; "Related Decisions" must be reciprocal (if ADR-X links ADR-Y, update ADR-Y to link back).

When an ADR is authored:
1. Update `docs/clarion/adr/README.md` — move the row from "Backlog" to "Authored ADRs".
2. Update `docs/clarion/v0.1/system-design.md` §12 — change Status from "To author" to "Accepted".

Author in this suggested order (dependencies first):

**ADR-014 — Filigree `registry_backend` flag and pluggable `RegistryProtocol`** (P0). The most downstream-load-bearing decision. Joint Clarion+Filigree deliverable in v0.1. Decide: protocol shape (trait or equivalent), what Filigree's `registry_backend: clarion` delegates, the `registry_backend: local` default-fallback that keeps Filigree solo-usable, schema-compatibility pin. Consider: whether the decision is a v0.1-only shape or a v0.2-extensible shape. Related: ADR-004, ADR-008 (superseded), ADR-017, ADR-018, ADR-022.

**ADR-022 — Core / plugin ontology ownership boundary** (P0, new). Principle 3 requires "Clarion observes, Wardline enforces" but the analogue at the core ↔ plugin boundary is unstated. Decide: plugins own language-specific entity/edge kinds in their manifest; core validates shape (kind is a well-formed identifier, required edges have the shape the core expects) without embedding language-specific ontology. Spell out what "validates shape without embedding ontology" means in practice. Related: ADR-002, ADR-003, ADR-006, ADR-021.

**ADR-021 — Plugin authority model: hybrid** (P0, new). Plugin declares capabilities in manifest (entity kinds, max memory, expected runtime, Wardline-aware yes/no). Core enforces minimum-safe controls: path jail refusing plugin-returned paths outside project root; per-run entity-count cap; Content-Length ceiling on every JSON-RPC frame; per-plugin RSS limit applied via `ulimit` at spawn. Not a full sandbox. Decide: the exact set of core-enforced minimums; the manifest schema changes needed; how violations are surfaced (kill + `CLA-INFRA-PLUGIN-VIOLATION` finding). Related: ADR-002, ADR-022, system-design §10 threat-model table.

**ADR-018 — Identity reconciliation** (P0). Clarion maintains the qualname ↔ EntityId translation layer. Wardline owns its own qualnames. Clarion's plugin imports `wardline.core.registry.REGISTRY` at startup with version pinning via `REGISTRY_VERSION`. Decide: initialization-coupling boundary (plugin-level, not core-level — cite `loom.md` §5 v0.1 asterisks); pin mismatch behaviour (warn-and-best-effort vs refuse). Related: ADR-002, ADR-003, ADR-015, ADR-021.

**ADR-015 — Wardline→Filigree emission ownership** (P0). Default committed position: v0.1 ships Clarion-side SARIF translator; Wardline gains a native Filigree emitter in v0.2; Clarion's translator retires at that point. *Optional promotion*: if the Wardline-native-emitter spike (Block C) shows the emitter is cheap, promote to v0.1 and delete the asterisk in `loom.md` §5. Decide: the v0.1 position (translator or native) and the retirement condition (named in v0.2 terms). Related: ADR-004, ADR-017, ADR-019.

**ADR-017 — Severity mapping + rule-ID round-trip + dedup** (P0). Clarion internal severity (`INFO/WARN/ERROR/CRITICAL/NONE`) maps to Filigree wire (`critical/high/medium/low/info`). Round-trip preserves internal severity via `metadata.clarion.internal_severity`. Dedup in v0.1 uses `mark_unseen=true` on ingest. Decide: the precise mapping table (including how `NONE` maps); rule-ID namespace reservation per REQ-FINDING-02 (`CLA-PY-*` for Python-plugin structural findings, `CLA-INFRA-*` for pipeline failures — fix the `CLA-INFRA-PARSE-ERROR` vs `CLA-PY-PARSE-ERROR` inconsistency while you're here, flagged in `04-self-sufficiency.md` Issue 7). Related: ADR-004, ADR-015, ADR-019.

**ADR-016 — Observation transport** (P0). v0.1 Clarion emits observations via MCP tool call (spawn `filigree mcp` from Clarion). v0.2 Filigree adds `POST /api/v1/observations` and Clarion switches to HTTP. Decide: the v0.1 MCP-spawn mechanics and the v0.2 retirement trigger. Related: ADR-014.

**ADR-011 — Writer-actor concurrency model** (P0 promoted from P1). Single writer actor + per-N-files transactions; `--shadow-db` flag for zero-stale-read scenarios. Decide: the concurrency model commitment, the `--shadow-db` trigger conditions, the reader pool cardinality. Call out the SQLite-concurrency assumption as unvalidated (Block C spike). Related: ADR-001, ADR-005.

**ADR-006 — Clustering algorithm** (P0 promoted from P1). Leiden on imports + calls subgraph with Louvain fallback. Decide: the specific edge definition (weight? direction?); the "fallback to Louvain if Leiden edge-reference library is unstable" condition; the cluster-quality acceptance criterion if any. Related: ADR-003, ADR-022.

**ADR-007 — Summary cache key design** (P0 promoted from P1). Key is `(entity_id, content_hash, prompt_template_id, model_tier, guidance_fingerprint)` + TTL backstop + churn-eager invalidation. Decide: precise hashing scope (whole file? entity slice?); TTL default; "churn-eager" trigger. This ADR's assumptions drive the cost-model spike in Block C. Related: ADR-001.

**ADR-013 — Pre-ingest secret scanner** (P0 promoted from P1). Pre-ingest redaction via `detect-secrets`; LLM-dispatch block if not clean. Decide: the exact block-on-unclean behaviour; opt-out flag and its audit consequence; secret-kind coverage. Related: ADR-004, ADR-021 (plugin boundary implications).

### Block B — HTTP auth default flip (~4 hours)

The v0.1 HTTP auth default must not be `auth: none`. Threat-model risk 9 (`09-threat-model.md` T-02).

1. Decide the new default. Candidates: (a) Unix domain socket at `.clarion/socket` with mode 0600; (b) auto-minted token written to `.clarion/auth.token` with mode 0600 on first `clarion serve`. Recommend (a) since it also closes the `CAP_NET_BIND_SERVICE`-equivalent attack surface entirely, with (b) as the fallback when UDS is unavailable (Windows, remote-over-SSH scenarios).
2. Update **`system-design.md` §10** — rewrite the "Loopback is not a security boundary" paragraph to describe the new default posture.
3. Update **`detailed-design.md` §7** — describe the UDS path, permission mode, and the `clarion check-auth --from wardline` ergonomic for CI plumbing.
4. Update / author **ADR-012 (Token auth in v0.1)** — the existing backlog summary says "opt-in default". Rewrite as "authenticated-or-not-listening by default". This is the lowest-cost ADR to author because the decision is now already made.
5. Add a `CLA-INFRA-HTTP-AUTH-DISABLED` finding that emits if the operator explicitly flips the default to `none`.

### Block C — Cost-model spike + optional Wardline-emitter spike (0.5–1.5 days)

Two throw-away spikes, runnable independently. Both inform ADR content; neither blocks ADR writing.

**C1 — Cost-model spike** (half day). `NFR-COST-01` ($15 ± 50% per run), `NFR-COST-02` (≥95% cache hit after stabilisation), `NFR-PERF-01` (≤60 min wall-clock) are currently design estimates with no real data. Run a minimal end-to-end cost estimation against `elspeth-slice` (the smallest viable elspeth subset). Produce a real number. Acceptable outcomes: confirm the design numbers, or downgrade any number whose envelope is wrong. Update **`requirements.md` NFR-COST-* rationale** with the spike result, and append a one-paragraph summary to the v0.1-scope-commitments memo under a new "Validation" section.

**C2 — Wardline-native-emitter spike** (1 day, optional but high-value). Investigate whether adding a native `POST /api/v1/scan-results` emitter in Wardline is a small piece of work or a significant refactor. If small (< 1 day): promote ADR-015 to v0.1, delete the v0.1 asterisk in `loom.md` §5 that covers the SARIF triangle, update the briefing's "Wardline-sourced findings" data-flow row from "*v0.2*" to current, retire the SARIF translator's "v0.1 path" framing. If large: leave ADR-015 as the committed v0.2 deferral. Record the decision basis in ADR-015 itself.

---

## 4. Exit criteria

All of the following must be true before the docset is considered "ready for implementation":

- [ ] All 11 ADRs authored and Accepted
- [ ] `adr/README.md` moved authored rows out of Backlog
- [ ] `system-design.md` §12 shows the authored ADRs as Accepted
- [ ] HTTP auth default flipped in `system-design.md` §10, `detailed-design.md` §7, and ADR-012
- [ ] Cost-model spike result recorded in `requirements.md` NFR-COST-* and the commitment memo's Validation section
- [ ] Wardline-native-emitter spike decision recorded in ADR-015 (either way)
- [ ] The commitment memo's sign-off section reflects completion

---

## 5. Constraints and guardrails

- **Do not re-open Q1, Q2, or Q3.** If a new case for re-opening them surfaces while authoring ADRs, stop and raise it explicitly rather than silently drifting.
- **Do not rewrite `loom.md` §5 or the briefing.** Those edits landed in Day 1 and reflect the committed scope. Quote them where useful; do not restate them.
- **Preserve existing ADR register / voice**. Existing ADRs 002-004 are the template for register and structure. Amended ADR-001 is the template for the "steelman alternatives" pattern.
- **Panel reviews are historical**. `docs/clarion/v0.1/reviews/panel-2026-04-17/` is the reference material that produced the commitments. Cite it when helpful; do not re-run it.
- **Loom suite is single-author**. When Clarion design requires Filigree or Wardline work, document it as a within-scope prerequisite, not a cross-team dependency.
- **File edits over new docs**. Where possible, edit existing design docs rather than creating new ones. The exception is new ADRs (one file per decision) and the spike result files.

---

## 6. Canonical references

- **Scope commitments (the decisions)**: `docs/clarion/v0.1/plans/v0.1-scope-commitments.md`
- **Executive synthesis (the rationale)**: `docs/clarion/v0.1/reviews/panel-2026-04-17/00-executive-synthesis.md`
- **Panel reviews**: `docs/clarion/v0.1/reviews/panel-2026-04-17/01-*.md` through `11-*.md` and `reader-journals/`
- **Suite doctrine**: `docs/suite/loom.md` (note v0.1 asterisks subsection in §5)
- **Suite briefing**: `docs/suite/briefing.md`
- **Requirements**: `docs/clarion/v0.1/requirements.md`
- **System design** (canonical ADR rationale table in §12): `docs/clarion/v0.1/system-design.md`
- **Detailed design** (implementer navigation table in §11): `docs/clarion/v0.1/detailed-design.md`
- **ADR folder**: `docs/clarion/adr/`

Start with ADR-014. Its decision shape unlocks ADR-017, ADR-022, and the rest.

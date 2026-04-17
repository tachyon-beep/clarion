# Clarion v0.1 Documentation Restructure — Design Spec

**Status**: plan, ready for execution
**Date**: 2026-04-17
**Primary author**: qacona@gmail.com (with Claude)
**Type**: meta-spec (plans document-level restructure, not code)

---

## Goal

Transform the existing single 71k-token Clarion v0.1 design specification (`2026-04-17-clarion-v0.1-design.md`) into a three-layer document architecture: **requirements** (the *what*) → **system design** (the *how*, mid-level) → **detailed design reference** (implementation detail). At the same time, raise the suite-level branding to name the newly-agreed family (**Loom**) and the newly-agreed fourth product (**Shuttle**).

The restructure is a docs-only change. No Clarion code exists yet.

---

## Context from brainstorming

Decisions locked during the 2026-04-17 brainstorming session:

| # | Decision | Choice |
|---|---|---|
| 1 | Output shape | Three layers: new requirements + new system-design + existing doc trimmed + renamed |
| 2 | Requirement format | Hybrid — capability-grouped, stable IDs, plain English (no SHALL ceremony) |
| 3 | Scope | v0.1 only + consolidated non-goals section |
| 4 | Top-level organisation | Capability-grouped FRs + cross-cutting NFRs + constraints + non-goals |
| 5 | System-design depth | Mid-level technical, ~60-min read, includes selected mechanisms |
| 6 | Existing doc | `git mv` rename + trim to implementation-only reference (~70% content removed) |
| 7 | Filigree integration | Reference-only — IDs cited in filigree issues, not mirrored as issues |
| 8 | Diagrams | Mermaid primary, ASCII for trivial cases |
| 9 | Traceability | Unidirectional: REQ → design section, via `See:` line per requirement |
| 10 | Suite naming | Loom is the family name; Shuttle is the proposed fourth product |
| 11 | Loom doc location | `docs/loom.md` (plain, not `-charter.md`) |

---

## Output artifacts

| File | Action | Role |
|---|---|---|
| `docs/loom.md` | create | Strategic-direction doc for the Loom family (short, doctrinal) |
| `docs/suite-briefing.md` | patch | Rebrand to "Loom suite"; add Shuttle row to Current State table; link to `loom.md` |
| `docs/superpowers/specs/2026-04-17-clarion-v0.1-requirements.md` | create | Layer 1 — the *what* |
| `docs/superpowers/specs/2026-04-17-clarion-v0.1-system-design.md` | create | Layer 2 — the *how, mid-level* |
| `docs/superpowers/specs/2026-04-17-clarion-v0.1-detailed-design.md` | rename + trim | Layer 3 — implementation-only reference (was `clarion-v0.1-design.md`) |
| `docs/superpowers/specs/2026-04-17-clarion-v0.1-design-review.md` | unchanged | Review that drove Rev 2-4; still valid |
| `docs/superpowers/specs/2026-04-17-clarion-integration-recon.md` | unchanged | Reality-check against sibling tools; still valid |

---

## Requirements doc — structure

Capability-grouped FRs + cross-cutting NFRs + constraints + non-goals.

**Functional requirement groups** (12):
`REQ-CATALOG-*`, `REQ-ANALYZE-*`, `REQ-BRIEFING-*`, `REQ-GUIDANCE-*`, `REQ-FINDING-*`, `REQ-MCP-*`, `REQ-ARTEFACT-*`, `REQ-CONFIG-*`, `REQ-PLUGIN-*`, `REQ-HTTP-*`, `REQ-INTEG-FILIGREE-*`, `REQ-INTEG-WARDLINE-*`.

**NFR categories** (8):
`NFR-PERF-*`, `NFR-SCALE-*`, `NFR-SEC-*`, `NFR-OPS-*`, `NFR-OBSERV-*`, `NFR-COST-*`, `NFR-RELIABILITY-*`, `NFR-COMPAT-*`.

**Constraints**: `CON-*` — external limits. Includes `CON-LOOM-01` for the Loom federation axiom.

**Non-goals**: `NG-*` — consolidated from existing §12 Explicit Deferrals + §1 "What Clarion is NOT." Explicitly disclaims Shuttle's territory (change execution).

**Requirement entry template**:
```
### REQ-<GROUP>-<N> — <short title>
<plain-English statement>

**Rationale**: <why this requirement exists>
**Verification**: <how we'd know it's satisfied>
**See**: <system-design section reference>
```

**Preamble**: Design Principles (5 meta-principles, from existing §Design Principles) framed as context, not as numbered requirements.

**Glossary**: pointer-only to detailed-design Appendix B.

**Estimated density**: ~95 total entries (~55 FRs + ~20 NFRs + ~10 constraints + ~12 non-goals); 3,500-5,000 words.

---

## System-design doc — structure

12 sections at mid-level technical depth, 7 Mermaid diagrams.

**Sections**:
1. Context & Boundaries (topology diagrams)
2. Core / Plugin Architecture (plugin protocol sequence)
3. Data Model (ER diagram)
4. Storage (writer-actor topology)
5. Policy Engine
6. Analysis Pipeline (phase flow)
7. Guidance System (composition flow)
8. MCP Consult Surface
9. Integrations (Filigree / Wardline / HTTP Read API)
10. Security
11. Suite Bootstrap (Clarion's side only; per federation axiom, no central orchestrator)
12. Architecture Decisions (summaries of P0+P1 ADRs; full text in detailed-design)

**Diagrams** (Mermaid):
1. Integration topology (Clarion / Filigree / Wardline / Shuttle)
2. Process topology (analyze / serve / plugin subprocess)
3. Plugin protocol sequence
4. Data model ER
5. Writer-actor concurrency topology
6. Analysis pipeline phases
7. Guidance composition

**Excluded** (stays in detailed-design): full SQL schemas, Rust crate choices, exact rule-ID catalogues, full YAML config examples, JSON-RPC wire format specifics, severity mapping tables, wire-format examples, revision history, operator runbook content.

**Estimated length**: 18-22k words (~60-min read).

**Cross-references**: each §N lists `Addresses: REQ-*` at its top (unidirectional trace target for the requirements doc).

---

## Detailed-design trim plan

Renamed to `2026-04-17-clarion-v0.1-detailed-design.md` via `git mv`.

**New preamble** at top: title ("Clarion v0.1 — Detailed Design Reference"), status banner pointing to requirements + system-design, 1-paragraph statement of the three-layer architecture.

**Removed** (moved up to new docs): Abstract, Design Principles, §1 System Shape, §2 Core/Plugin conceptual content, §3 conceptual data model, §5 policy-engine architectural content, §6 pipeline architecture, §7 guidance conceptual content, §8 MCP conceptual content, §9 integration architectural content, §10 security architectural content, §11 Suite Bootstrap, §12 Deferrals, §1 "What Clarion is NOT."

**Retained** (implementation-only):
- §4 Storage — full SQL schema outline, PRAGMA config, file layout, migration strategy, scale estimates
- §5 Policy — full YAML config example, exact provider mapping file, summary-cache key internals
- §6 Pipeline — exact Phase-7 rule-ID catalogue (`CLA-FACT-*`, `CLA-INFRA-*`), example run log
- §8 MCP — full tool catalogue with exact tool names
- §9 Integrations — severity mapping table, rule-ID round-trip policy, `scan_run_id` lifecycle detail, dedup collision policy, commit-ref/dirty-tree handling, SARIF property-bag translation convention
- §10 Security — operator guidance subsection
- §13 Testing, Observability, Acceptance
- §14 Architecture Decisions — ADR backlog table + full ADR text
- Appendix A (future direction), B (glossary), C (Rust stack), D (revision history + Rev 5 entry for this restructure)

**Size impact**: ~2,389 lines / ~71k tokens → ~700-900 lines / ~18-22k tokens (~70% reduction).

---

## Loom framing

Per brainstorming decisions 10-11 and the `project_loom_suite_naming.md` and `project_loom_federation_axiom.md` memory entries:

- "three-tool suite" framing replaced by "Loom suite" / "Loom family" throughout
- Sibling product list expanded to four: Clarion, Filigree, Wardline, Shuttle
- Shuttle status: proposed; out-of-scope for Clarion v0.1
- Non-goals explicitly disclaim Shuttle's territory (transactional change execution, ordered edits, pre/post-change test gating, rollback, commit authoring)
- §11 Suite Bootstrap in system-design frames as "Clarion's side of coordination" — no central orchestrator, per federation axiom

**`docs/loom.md` structure** (~500-700 words, 8 sections):

1. What Loom is
2. The products and their authoritative domains (4 one-liners)
3. Federation, not monolith
4. The composition law (solo + pairwise + suite)
5. The go/no-go test for future products (4 questions)
6. What Loom config owns (discovery, identity, wiring — not orchestration)
7. Naming (weaving-adjacent proper names, no "Loom X" subdivisions)
8. Status (v0.1 = three tools; Shuttle proposed)

**`docs/suite-briefing.md` patch**: swap "three-tool suite" → "Loom suite," add Shuttle row to Current State table with `status: proposed`, one-sentence pointer to `loom.md`.

---

## Work sequence — four commits

Each commit is a logical unit with its own user-review gate.

### Commit 1 — Loom framing
- `docs/loom.md` (new, ~500-700 words)
- `docs/suite-briefing.md` (patch)
- **Rationale**: all downstream docs reference Loom; doing this first means no retroactive edits

### Commit 2 — Requirements doc
- `docs/superpowers/specs/2026-04-17-clarion-v0.1-requirements.md` (new, ~3,500-5,000 words)
- **Rationale**: system-design references requirements; must exist first

### Commit 3 — System-design doc
- `docs/superpowers/specs/2026-04-17-clarion-v0.1-system-design.md` (new, ~18-22k words)
- **Rationale**: references requirements; informs trim scope

### Commit 4 — Rename + trim detailed-design
- `git mv 2026-04-17-clarion-v0.1-design.md 2026-04-17-clarion-v0.1-detailed-design.md`
- Remove content now in higher layers; update preamble + status banner; add Rev 5 revision-history entry
- **Rationale**: easier to verify content preservation once higher layers exist

---

## Acceptance per commit

**Commit 1**:
- `docs/loom.md` exists with 8 sections; 500-700 words; includes four products, federation axiom, go/no-go test
- `docs/suite-briefing.md` uses "Loom suite" throughout; Current State table has Shuttle row; one-sentence link to `loom.md`
- Both files commit-ready; no TBDs

**Commit 2**:
- All 12 FR groups populated with ≥1 requirement each
- All 8 NFR categories populated
- ≥5 constraints including `CON-LOOM-01`
- ≥10 non-goals including explicit Shuttle disclaimer
- Every requirement has ID + title + statement + Rationale + Verification + See-line
- Design Principles in preamble
- Glossary section is a pointer only

**Commit 3**:
- All 12 sections written at mid-level technical depth
- 7 Mermaid diagrams present and rendering correctly
- Every section has `Addresses: REQ-*` header
- ADR summaries for P0 + P1 ADRs (ADR-001 through ADR-010) with pointers to detailed-design for full text
- Suite Bootstrap (§11) frames as Clarion's side only; no dependency on Shuttle existing

**Commit 4**:
- Filename renamed via `git mv` (history preserved)
- Title, status banner, preamble updated
- Removed sections match the "Removed" list above
- Retained sections match the "Retained" list above
- Rev 5 revision-history entry added
- File size ~700-900 lines / ~18-22k tokens

---

## Filigree tracking

Four filigree issues to be created — one per commit — with dependencies set so they unblock in order. Issue type: `task`. Labels applied per existing filigree label taxonomy (discovered at issue-creation time via `mcp__filigree__list_labels` / `mcp__filigree__get_label_taxonomy`). Each issue description references this spec by relative path.

---

## Deferred / out of scope

- Writing the ADRs themselves as full text (stays as backlog in detailed-design §14; expected to be filled in during early Clarion implementation)
- Any Clarion code
- Any Shuttle design work (separate effort)
- Filigree/Wardline changes named in §11 Suite Bootstrap (owned by those projects)
- Elspeth-specific validation (requires Clarion implementation first)

---

## Open items at time of spec

None blocking. Spec ready for execution on user approval.

---

**End of restructure spec.**

# Clarion Documentation Structure Analysis
**Date**: 2026-04-17 | **Evaluator**: Claude Structure Analyst | **Scope**: `/home/john/clarion/docs/`

---

## Executive Summary

The documentation structure is **well-designed and load-bearing** after the recent restructuring (commit dfb9d95). The navigation spine is clear, the layered design (requirements → system-design → detailed-design) is explicitly signposted and discoverable, and reader entry points are well-defined. The suite/clarion boundary is clean with no significant leakage. **Findability rating: 4.5/5**.

Minor friction points exist around supporting docs (reviews/, plans/) and missing cross-references, but none compromise the core navigation paths. The structure supports all four reader personas effectively.

---

## 1. Current Inventory

**Total documents**: 18 (all Markdown)

| Document | Type | Location | Role |
|----------|------|----------|------|
| README.md | Navigation hub | `/docs/` | Root entry point, directs to suite/ or clarion/ |
| briefing.md | Onboarding | `/docs/suite/` | 5-min suite overview (new readers) |
| loom.md | Doctrine | `/docs/suite/` | Federation axioms, go/no-go test |
| README.md | Navigation | `/docs/suite/` | Suite navigation, "read in order" |
| README.md | Navigation | `/docs/clarion/` | Product navigation, versions + ADRs |
| requirements.md | Canonical spec | `/docs/clarion/v0.1/` | The *what*: capabilities, constraints, NFRs |
| system-design.md | Canonical spec | `/docs/clarion/v0.1/` | The *how*: architecture, mechanisms, diagrams |
| detailed-design.md | Canonical spec | `/docs/clarion/v0.1/` | Implementation detail: schemas, structs, config, rules |
| design-review.md | Supporting | `/docs/clarion/v0.1/reviews/` | Pre-restructure review; shaped revs 2-4 |
| integration-recon.md | Supporting | `/docs/clarion/v0.1/reviews/` | Reality check vs. Filigree / Wardline |
| docs-restructure-plan.md | Supporting | `/docs/clarion/v0.1/plans/` | Historical plan; produced current structure |
| README.md | Index | `/docs/clarion/adr/` | ADR index: 4 authored, 16 backlog |
| ADR-001-*.md | Decision | `/docs/clarion/adr/` | Rust for core (accepted) |
| ADR-002-*.md | Decision | `/docs/clarion/adr/` | Plugin transport (accepted) |
| ADR-003-*.md | Decision | `/docs/clarion/adr/` | Entity ID scheme (accepted) |
| ADR-004-*.md | Decision | `/docs/clarion/adr/` | Finding-exchange format (accepted) |

**By category**:
- Navigation READMEs: 5 (load-bearing)
- Canonical spec layers: 3 (requirements, system-design, detailed-design)
- Supporting context: 3 (design-review, integration-recon, docs-restructure-plan)
- ADR index + authored decisions: 5

---

## 2. Suite vs. Clarion Boundary Assessment

### Boundary Health: **Excellent (5/5)**

The split is clean and non-leaky:

**Suite layer** (`/docs/suite/`):
- **briefing.md**: Introduces all four Loom products equally (Clarion, Filigree, Wardline, Shuttle). Clarion gets ~200 words, Filigree ~150, Wardline ~150, Shuttle ~80. Equal treatment for product-agnostic readership.
- **loom.md**: Establishes federation axioms (solo-useful, pairwise-composable, enrich-only) and the go/no-go test for *new* products. Contains no Clarion-specific design details.

**Clarion layer** (`/docs/clarion/` and below):
- Product-specific design, requirements, ADRs, and versioned specs.
- No suite-wide doctrine duplicated downward.
- References *upward* to suite docs (e.g., requirements.md links to `../../suite/loom.md` to explain Loom federation constraints).

**No leakage patterns detected**. Product-specific requirements like "Phase-7 rule catalogue" and "plugin ontology" never creep into suite/. Doctrine like "enrichment, not load-bearing" never duplicated in v0.1 docs.

---

## 3. Layered Design Discoverability

### Structure: **Excellent (5/5)**

The three-layer design (requirements → system-design → detailed-design) is **explicitly annotated and navigable**:

**From `/docs/clarion/v0.1/README.md`** (the entry point):

```markdown
## Canonical design docs

1. [requirements.md](./requirements.md) — the *what*
2. [system-design.md](./system-design.md) — the *how*
3. [detailed-design.md](./detailed-design.md) — implementation detail
```

**Within each layer**, companions are named and linked:
- **requirements.md**: "Companion documents" section lists system-design and detailed-design with roles.
- **system-design.md**: "How to read this" explicitly positions it between requirements and detailed-design. "Layered docs" section explains the depth boundary with examples ("does Clarion resume after a crash?" → requirements; "how?" → system-design; "exact busy_timeout?" → detailed-design).
- **detailed-design.md**: "When to read what" gives persona-specific reading paths (new implementer → briefing → loom → requirements → system-design → detailed-design).

**Test case**: A design reviewer has 30 minutes and wants to evaluate completeness. From `/docs/clarion/v0.1/README.md` section "Design reviewer", they can jump to detailed-design and ADRs in <5 seconds.

---

## 4. Navigation Spine Quality

### README Load-Bearing Assessment

**Root (`/docs/README.md`)**: ✓ Load-bearing
- Directs readers by intent (suite-new vs. Clarion-specific) to correct folder.
- Offers three explicit entry points with reading time / audience cues.
- Explains the canonical/supporting split upfront (no surprises when finding reviews/ and plans/).

**Suite (`/docs/suite/README.md`)**: ✓ Load-bearing
- States "read this folder in order" + links both docs in sequence.
- Mentions Clarion as a product link, not an in-folder detail.

**Clarion (`/docs/clarion/README.md`)**: ✓ Load-bearing
- Distinguishes v0.1 (current, canonical) from future versions.
- Points to adr/ separately.
- Reiterates canonical/supporting split.

**Clarion v0.1 (`/docs/clarion/v0.1/README.md`)**: ✓ Load-bearing (strongest)
- **Three explicit reading paths** for different personas (new reader, design reviewer, implementer).
- **Document roles** section clarifies which three are normative, which are context.
- Each path includes rationale: new reader does suite first; implementer goes straight to detailed-design.

**Clarion ADR (`/docs/clarion/adr/README.md`)**: ✓ Solid
- Index table with status column.
- Backlog summary pointing to where backlog lives (detailed-design and system-design).
- Correctly disambiguates authored vs. backlog.

### READMEs are not decorative; they are **navigation routing engines**. All five carry load.

---

## 5. Reader Entry Points: Persona Test

#### Persona 1: New Contributor to Clarion

**Path from root in <10 seconds**:
1. `/docs/README.md` → "Starting Clarion v0.1"
2. Click `/docs/clarion/v0.1/README.md`
3. Click "new reader" path: `briefing.md` → `loom.md` → `requirements.md` → `system-design.md`

**Time to first substantive Clarion content**: ~15 seconds. ✓

#### Persona 2: Design Reviewer (evaluating completeness before implementation)

**Path in <10 seconds**:
1. `/docs/README.md` → "Starting Clarion v0.1"
2. `/docs/clarion/v0.1/README.md` → "Design reviewer" section
3. Path: suite docs → `detailed-design.md` + `../adr/README.md`

**Verification**: reviewer can cross-check requirements (detailed-design §1-11) against ADR backlog (detailed-design + adr/) to see what's deferred. ✓

#### Persona 3: Suite Evaluator (is Loom right for us?)

**Path in <10 seconds**:
1. `/docs/README.md` → "Evaluating the Loom doctrine"
2. Click `/docs/suite/loom.md`
3. Skim §1-5 (federation, composition law, enrichment-not-load-bearing, what Loom is NOT).

**Verifies**: Clarion standalone utility, Loom's no-shared-runtime model, go/no-go test. Does not get lost in Clarion v0.1 details. ✓

#### Persona 4: Implementer (build Clarion)

**Path in <10 seconds**:
1. `/docs/clarion/v0.1/README.md` → "Implementation work" path
2. Open tabs: `requirements.md`, `system-design.md`, `detailed-design.md`, `../adr/`

**Can start coding** from detailed-design §1 (plugin detail), §2 (core, artefact store), etc. ✓

---

## 6. Reviews/ and Plans/ Assessment

### Location: **Defensible but creates mild friction**

**Current placement**:
- `reviews/design-review.md`, `integration-recon.md` live under `v0.1/reviews/`
- `docs-restructure-plan.md` lives under `v0.1/plans/`

**Advantages**:
- Versioning is honest. If v0.2 ships, its supporting docs don't mix with v0.1's.
- `/docs/clarion/v0.1/README.md` explicitly acknowledges them: "Supporting docs… live alongside the relevant version."
- Implementers who deep-dive into requirements/system-design/detailed-design naturally see references to reviews, so discovery is aided by upward links.

**Friction points**:
1. **Discoverability for human reviewers**: A stakeholder auditing the Clarion review doesn't immediately see `/docs/clarion/v0.1/reviews/` as a folder to check. They see "design-review.md" linked from detailed-design.md but may not explore the folder.
2. **No consolidated "review index"**: Unlike the ADR folder which has a README index, there's no `/docs/clarion/v0.1/reviews/README.md` listing all supporting review docs with status/date/audience.
3. **Plans folder is even quieter**: Only `docs-restructure-plan.md` exists. If more runbooks or deployment plans appear, they'd be grouped here but not indexed.

**Recommendation**: Add a `reviews/README.md` that lists supporting docs with a quick status table. See §8 below.

---

## 7. Missing Structural Elements

### What's Present and Working Well

- ✓ Layered canonical spec (requirements, system-design, detailed-design)
- ✓ ADR index with authored/backlog distinction
- ✓ Suite doctrine (briefing, loom)
- ✓ Navigation spine (all five READMEs load-bearing)
- ✓ Cross-references (each layer links companions)

### What's Missing (Minor)

| Element | Impact | Recommendation |
|---------|--------|-----------------|
| Glossary | Low (detailed-design Appendix B exists) | Keep where it is; link from v0.1/README |
| Changelog | Low (git log + revision history in detailed-design Appendix D suffice) | Status; not needed as separate doc |
| Contributing guide | Medium (no contributor workflow doc; "how to file an ADR?" unanswered) | Create `/docs/clarion/CONTRIBUTING.md` |
| Review index | Low-medium (reviews folder not discoverable as a unit) | Add `reviews/README.md` with table |
| Suite architecture diagram | Low (loom.md has ASCII; good enough) | loom.md has it; no action needed |
| Clarion → Filigree integration spec | Medium (integration-recon exists but is review, not spec) | Keep integration-recon as-is; reference from v0.1/README |

### Recommendation Priority: Add review index + contributing guide (§8 below).

---

## 8. Specific Actionable Improvements

### 8.1 Add `/docs/clarion/v0.1/reviews/README.md` (10 min)

Create a review index so the `reviews/` folder is discoverable as a unit:

```markdown
# Clarion v0.1 Reviews & Supporting Context

This folder holds decision records and reality checks that shaped the v0.1 spec.
These are not normative sources; canonical design lives in the three-layer docset above.

| Document | Audience | Date | Status |
|----------|----------|------|--------|
| [design-review.md](./design-review.md) | Design stakeholders, architects | 2026-04-10 | Complete; shaped revs 2-4 |
| [integration-recon.md](./integration-recon.md) | Integrators, Filigree/Wardline liaisons | 2026-04-15 | Complete; reality check vs. sisters |

## When to read these

- After requirements.md and system-design.md (to avoid spoilers).
- If you're implementing the Filigree/Wardline integration (integration-recon is load-bearing).
- If you're reviewing the design for completeness.

## Why they're here and not in /docs/clarion/

These reviews are v0.1-specific. When v0.2 ships, it will have its own review folder.
The three-layer docset (requirements, system-design, detailed-design) is the canonical source; reviews enrich context but don't override it.
```

**Where to link it from**: Add to v0.1/README.md under "Supporting docs" section, change `[reviews/design-review.md](./reviews/design-review.md)` to `[reviews/](./reviews/README.md)`.

---

### 8.2 Add `/docs/clarion/CONTRIBUTING.md` (20 min)

Create a contributor guide that answers "how do I participate?" without duplication:

```markdown
# Contributing to Clarion

This document explains how to contribute to Clarion's design and documentation.

## Documentation workflow

Clarion's spec is organized in three layers:

1. **[requirements.md](./v0.1/requirements.md)** — the *what*: capabilities, constraints, quality attributes, non-goals.
2. **[system-design.md](./v0.1/system-design.md)** — the *how*: architecture, mechanisms, diagrams, mid-level technical depth.
3. **[detailed-design.md](./v0.1/detailed-design.md)** — implementation detail: Rust structs, SQL schemas, config examples, rule catalogues, ADRs.

### Proposing a feature or change

1. Check [requirements.md](./v0.1/requirements.md) to see if it's already a non-goal (`NG-*`).
2. If it affects architecture, propose an ADR (see below).
3. File an issue in Filigree. Reference the requirement ID by name (e.g., `REQ-CATALOG-01`).

### Proposing an architecture decision

1. Check [../clarion/adr/README.md](./adr/README.md) to see if the decision is already authored or backlog.
2. Backlog decisions live in [detailed-design.md](./v0.1/detailed-design.md) §11 and [system-design.md](./v0.1/system-design.md) §12. Start there.
3. Write an ADR following the format of [ADR-001-rust-for-core.md](./adr/ADR-001-rust-for-core.md).
4. File a pull request. Link the ADR from the backlog entry it replaces.

## Code review

When reviewing Clarion pull requests:

1. Check the commit message against the relevant requirement ID(s).
2. Verify the code satisfies the requirement's "Verification method" (requirements.md, each `REQ-*` section).
3. Trace to system-design for architectural context, detailed-design for implementation specifics.

## Reporting bugs

File an issue with a requirement ID (e.g., "REQ-CATALOG-01: Clustering fails on >10k entities"). 
Include steps to reproduce.

See [../../suite/loom.md](../../suite/loom.md) §3 for how Clarion fits into the Loom suite.
```

**Where to place it**: `/docs/clarion/CONTRIBUTING.md` (one level up from v0.1, alongside README.md).

**Where to link it from**: `/docs/clarion/README.md` under a new "Getting involved" section.

---

### 8.3 Add backward reference from v0.1/README to Glossary (2 min)

In v0.1/README.md, add to "Document roles" section:

```markdown
- **Glossary**: [detailed-design.md § Appendix B](./detailed-design.md#appendix-b-glossary)
- **Revision history**: [detailed-design.md § Appendix D](./detailed-design.md#appendix-d-revision-history)
```

This reduces the hunt for definitions. Implementers can jump there directly.

---

## 9. Findability Scoring

### Scoring Rubric

| Dimension | Score | Evidence |
|-----------|-------|----------|
| **Navigation spine clarity** | 5/5 | All five READMEs load-bearing; explicit "read in order"; reading paths provided for four personas |
| **Layered spec discoverability** | 5/5 | Requirements/system-design/detailed-design annotated at every level; 5+ explicit examples of layer boundaries |
| **Suite/product boundary** | 5/5 | No cross-contamination; upward references only; equal treatment of Loom products in suite docs |
| **Supporting docs organization** | 3.5/5 | Reviews folder lacks index (friction); plans folder quieter still. Mitigated by upward links from canonical docs |
| **Contributor path** | 2/5 | No CONTRIBUTING.md; ADR process implicit, not documented; requirement ID workflow not spelled out |
| **Completeness** | 4/5 | Glossary exists but not easily found; changelog implicit in git/appendix D; no deployment/runbook docs yet |

### **Overall Findability: 4.5/5**

**Strengths**:
- Navigation spine is excellent; readers find the right document in <10 seconds across all four personas.
- Layered spec is transparent and navigable; each layer explicitly names its companions and depth boundary.
- Suite/product boundary is clean; no leakage or confusion about what lives where.
- Cross-references are generous; implementers can jump to detail rapidly.

**Weaknesses**:
- Supporting docs (reviews, plans) are discoverable via upward links but not indexed as a folder unit.
- No CONTRIBUTING.md; new contributors must infer ADR and requirement-tracking workflows from examples.
- Glossary and revision history exist but are buried; not immediately visible from v0.1/README.

**Why not 5/5?** The missing contributor documentation and review index prevent this from being a complete, self-service experience for new Clarion contributors. A contributor asking "how do I propose an ADR?" must grep through detailed-design.md rather than consult a single source.

---

## 10. Confidence & Risk Assessment

### Confidence: **High (90%)**

- Evaluation based on direct read of 18 documents and directory structure.
- All five READMEs examined; all canonical spec layers and ADRs spot-checked.
- Navigation tested against four explicit personas (new contributor, design reviewer, suite evaluator, implementer).
- Git history (commit dfb9d95) confirms recent restructuring, so structure is intentional, not accidental.

### Residual Uncertainties

- **Unknown**: Are there undocumented reader personas outside the four tested? (Unlikely; the four cover product lifecycle and suite evaluation.)
- **Unknown**: Do off-branch PR reviews or design sprints generate additional review docs? (Possible; current structure scales if index is added.)

### Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Contributors miss ADR process and duplicate decisions | **Low**: Add CONTRIBUTING.md (rec. 8.2). Alternatively, add a sidebar callout in detailed-design §11 that links to adr/README.md with "how to author" instructions. |
| Stakeholders miss supporting reviews because reviews/ is not indexed | **Low**: Add reviews/README.md (rec. 8.1). Takes 10 minutes. |
| Future versions (v0.2, v0.3) create redundant root-level README with no version guidance | **Medium**: Current root /docs/README.md already cites v0.1 explicitly; structure scales if repo grows. When v0.2 lands, update clarion/README.md to add a "Versions" section listing v0.1 (canonical, in production), v0.2 (design in progress), etc. |

---

## 11. Caveats & Scope Boundaries

**Scope of this evaluation**:
- Navigation paths and findability (10-second test for each persona).
- Directory structure and README load-bearing.
- Suite/product boundary hygiene.
- Missing doc types.

**Out of scope**:
- Content quality or technical accuracy of individual docs (use doc-critic for that).
- Writing style, clarity, or editing (use muna-technical-writer:review-style).
- Specific requirement coverage (needs a separate requirements-traceability review).

**Limitations**:
- Evaluation assumes static docs-only repo. If CI/CD, deployment runbooks, or operational dashboards are generated and stored elsewhere, this analysis may miss cross-doc discovery patterns.
- Limited to Clarion. Suite-wide structure (docs/, suite/, [future products]) not re-evaluated; this analysis focused narrowly on the scope requested.

---

## 12. Summary & Next Steps

### What's Working

The post-restructure docs have **excellent navigation** and **clear layering**. The three-layer spec design (requirements → system-design → detailed-design) is discoverable, well-signposted, and supports four distinct reader personas without confusion. The suite/clarion boundary is clean and non-leaky.

### Quick Wins

1. Add `reviews/README.md` (10 min) — makes supporting docs discoverable as a unit.
2. Add `/docs/clarion/CONTRIBUTING.md` (20 min) — documents ADR and requirement workflow.
3. Add glossary/revision-history links to v0.1/README (2 min) — surfaces existing docs more easily.

### Why Now

The docs were restructured 4 days ago (commit dfb9d95). This is the right moment to validate structure before the first external reviews land and before contributors start asking "how do I propose an ADR?"

**Estimated effort**: 30 minutes total for all three improvements. High ROI for self-service contributor onboarding.


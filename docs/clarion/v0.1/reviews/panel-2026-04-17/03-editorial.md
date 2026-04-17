# Editorial Review — Clarion Docset
**Panel date**: 2026-04-17
**Reviewer**: editorial agent (register consistency, voice, audience fit)
**Scope**: `docs/suite/briefing.md`, `docs/suite/loom.md`, `docs/clarion/v0.1/requirements.md`,
`docs/clarion/v0.1/system-design.md`, `docs/clarion/v0.1/detailed-design.md`, ADR-001–004

---

## Confidence Assessment

**High.** Each document is long enough (ADRs are the shortest, but structurally uniform) for reliable register detection. All markers are internally consistent; no ambiguous edge cases.

## Register Map

| Document | Detected Register | Appropriate? |
|---|---|---|
| `loom.md` | Doctrine / manifesto | Yes — explicitly a founding charter |
| `briefing.md` | Technical reference (orientation) | Mostly yes; see §3 below |
| `requirements.md` | Technical-precise (policy/spec) | Yes |
| `system-design.md` | Technical-precise (architecture) | Yes |
| `detailed-design.md` | Technical-precise (implementation reference) | Yes |
| ADR-001–004 | Technical decision record | Yes, and consistent across all four |

---

## Per-Document Findings

### 1. `loom.md` — Doctrine / Manifesto

**Register**: Authoritative declarative. Section headings carry real normative weight ("Federation, not monolith"; "Enrichment, not load-bearing"). First-person plural absent; third-person declarative throughout. The load-bearing sentence in §5 is explicitly called out as such: "enrichment, not load-bearing. If that principle is ever compromised, the rest collapses."

**Fit**: Strong. Voice is consistent with a founding charter that will outlive v0.1. Register is justified by the doc's stated purpose.

**One register slip**: §2 contains a parenthetical aside that drifts into product marketing:
> "Notably, Wardline's 'configuration' is the source code itself plus the adjacent declarations — it does not have a separate authoritative config store."

This is an implementation detail, not a doctrine statement. It reads like a footnote from the design doc rather than from the charter. A doctrine doc should speak in principles; implementation specifics belong in the briefing.

---

### 2. `briefing.md` — The Audience Fit Problem

**Register**: Technical reference (orientation layer).

**Fit**: Partial. The briefing is well-organised and factually dense — appropriate for an engineering audience arriving cold. But the stated audience is "engineers, reviewers, or stakeholders new to the Loom suite," and two of those three need different things:

- The "What Clarion v0.1 ships" subsection reads internally, not outwardly. It describes what the *suite* needs from sibling tools before Clarion can ship — prerequisite asks against Filigree and Wardline — but frames this as if the reader is already tracking the release plan:
  > "Because Clarion is the work that weaves the fabric, several changes land in the sibling tools as prerequisites"

  A newcomer has no map for "weaves the fabric" yet. This section assumes the reader has already absorbed loom.md §5.

- The fabric diagram (ASCII box-and-arrow) is good for engineers but loses non-technical reviewers / stakeholders immediately. The "Where to read next" table at the close is excellent; the problem is that stakeholders have no hint before that table that they can stop reading after the one-paragraph version.

**Highest-ROI fix (see §5, Fix A)**: A one-line navigation cue at the top of the "How they interact" section: "If you are a non-technical stakeholder, the summary above is sufficient — the sections below are for engineering readers."

---

### 3. `requirements.md` — Strong Register Fit with One Recurrent Tic

**Register**: Technical-precise policy (spec). Requirements carry IDs, verification methods, rationale sections, and `See:` traceability lines. Voice is declarative and impersonal throughout.

**Fit**: Strong. The document knows what it is. The three-layer framing (`what` / `how` / `implementation-level`) is stated early and the document stays inside its lane.

**One recurrent voice issue**: several rationale sections shift into first-person motivational language that belongs in the design principles preamble, not in per-requirement rationale. Example from REQ-ANALYZE-06:
> "Silent fallbacks make debugging impossible and gradually erode trust — operators stop believing the catalog because 'Clarion sometimes skips files for reasons I can't see.'"

The quoted end-user complaint is vivid and persuasive, but it is persuasive register, not specification register. Rationale sections should explain *why the requirement exists in architectural terms*; the quoted inner monologue belongs in user-story documentation or the design-principles section. This pattern recurs in REQ-BRIEFING-01, REQ-CATALOG-03, and REQ-MCP-01. It does not undermine accuracy, but it weakens the register consistency a specification reader expects.

---

### 4. `system-design.md` — Clean Technical-Precise Register with Minor Mixed-Audience Moments

**Register**: Technical-precise (architecture). Each section opens with an `Addresses:` requirements trace; diagrams use Mermaid; vocabulary is stable and consistent.

**Fit**: Strong. The document is clearly the mid layer and refers up to requirements and down to detailed-design correctly.

**One slip worth flagging**: §1 "What Clarion is NOT" reads as a defensive bullet list in a tone more suited to a product FAQ than a system design document:
> "Clarion is not a linter (NG-01, Wardline's territory), not a workflow tracker (NG-02, Filigree's territory), not an IDE (NG-03)..."

The non-goals list is appropriate in a requirements doc. Restating it in the system design, in the same FAQ-list style, is register duplication. The system design should make the same point by drawing the boundary in positive architectural terms (which it does elsewhere — boundary contracts in §1 are excellent). The "What Clarion is NOT" block imports a requirements-doc move into the wrong layer.

---

### 5. `detailed-design.md` — Consistent Reference Register

**Register**: Technical-precise (implementation reference). Code blocks, Rust struct definitions, YAML examples, table enumerations. Voice is entirely impersonal.

**Fit**: Strong. The document's own preamble section "What moved up" correctly narrates the restructure — the self-awareness is register-appropriate for a reference layer that wants to orient the implementer.

**No significant register issues.** The only observation is cosmetic: the preamble's "When to read what" uses a question-and-answer format ("Answering 'what does Clarion guarantee?' → Requirements") that is slightly warmer in tone than the rest of the document, but this is justified as explicit wayfinding and does not constitute register drift.

---

### 6. ADR-001 through ADR-004 — Voice Consistency

**Register**: Technical decision record. All four share the same structure: Summary, Context, Decision, Alternatives Considered (pros/cons/why rejected), Consequences (positive/negative/neutral), Related Decisions, References.

**Fit**: Strong. The voice is consistent across all four. Section headers are identical in structure; the level of detail in "Alternatives Considered" is calibrated similarly (two or three options each, brief bullet lists). No ADR drifts into marketing or hedges a decision in a way that undermines the record.

**One structural note, not a register issue**: ADR-001's "Alternatives Considered — Alternative 1 (Go) — Why rejected" is unusually frank:
> "the primary author directed Rust for v0.1, and the rest of the design already assumes Rust-native ecosystem choices."

This is honest and unambiguous, which is the right instinct for an ADR. But it does mean the Go rejection is recorded as a directive rather than a technical judgment. That is a completeness observation for the decision record, not an editorial problem — the register is correct.

---

## Tone and Voice Consistency Across the Suite

The first-person plural ("we") appears in:
- `requirements.md` preamble: "These requirements respect the Loom federation axiom"
- `system-design.md`: used sparingly in narrative sections
- ADRs: "We will implement...", "We will use..." — this is the appropriate ADR convention

The voice shift between the doctrine docs (loom.md) and the spec docs (requirements, system-design) is intentional and appropriate — one is charter, the other is specification. The shift is not jarring because each document declares its audience and purpose at the top.

**One weak seam**: `briefing.md` uses the same first-person-plural voice as the spec docs but is positioned before them in the reading order. The effect is that a newcomer reading the briefing encounters spec-layer voice ("Clarion v0.1 delivers the cross-tool protocols") before encountering the doctrine that justifies it. This is a sequencing issue, not a register violation in any single document.

---

## Risk Assessment

**Low editorial risk overall.** The docset has a coherent register strategy: doctrine → orientation → requirements → design → implementation-reference. The ADRs are the most internally consistent layer. The three issues flagged below are polish items, not structural failures.

## Information Gaps

- `loom.md` and `briefing.md` were both recently revised. The reviewer has not seen prior versions and cannot assess whether current register drift is improvement or regression.
- No review of the README or the reviews layer (`design-review.md`, `integration-recon.md`) was requested; those may have their own register profile.

## Caveats

- Register fit assessments treat the stated audience and purpose headers on each document as authoritative. If those headers are themselves aspirational (i.e., the actual readers differ from the stated audience), fit ratings may be generous.

---

## Three Highest-ROI Editorial Fixes

### Fix A — `briefing.md` line 69 ("How they interact" section heading)
**Problem**: Stakeholder readers arrive at a data-flow diagram and sibling-tool integration details with no warning that this section is for engineers only. The briefing effectively stops being a briefing and becomes a design summary without signalling the transition.
**Fix**: Add a navigation note at the top of "How they interact": one sentence indicating that the section below is engineering depth and stakeholders can skip to "Principles that shape the suite."
**ROI**: High — the suite briefing is the entry point for the widest audience. Improving its navigability serves every reader who isn't already an engineer on the project.

### Fix B — `requirements.md` rationale sections (REQ-ANALYZE-06, REQ-BRIEFING-01, REQ-MCP-01, REQ-CATALOG-03)
**Problem**: Rationale sections shift into persuasive-narrative voice — quoted user inner monologues, motivational phrasing — that belongs in design-principles documentation, not in a requirements specification. This does not affect technical accuracy but weakens the spec register a requirements reader expects.
**Fix**: Trim rationale to architectural consequence statements. The motivational material can stay in the design-principles preamble (which already exists and is the right place for it).
**ROI**: High — requirements docs are reference material that implementers and reviewers re-read. Register consistency reduces cognitive friction on every subsequent read.

### Fix C — `system-design.md` §1 "What Clarion is NOT"
**Problem**: The bullet list restates non-goals from requirements in a FAQ-list register that is out of place in an architecture document. The system design already draws the boundary positively through its integration-contracts and boundary-conditions sections; the "NOT" list is redundant and imports requirements-doc register into the wrong layer.
**Fix**: Remove or collapse the "What Clarion is NOT" block to a single sentence and a cross-reference to the requirements NG-* entries. The positive architectural framing already present in §1 does the same work better.
**ROI**: Medium-high — system-design is a document implementers consult repeatedly. Removing the register intrusion keeps the architectural section focused and improves the document's internal coherence.

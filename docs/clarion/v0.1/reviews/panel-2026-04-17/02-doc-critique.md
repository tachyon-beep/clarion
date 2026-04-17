# Documentation Critique — Clarion v0.1 Canonical Design Docs

**Reviewer**: Documentation Critic Agent (claude-sonnet-4-6)
**Date**: 2026-04-17
**Scope**: requirements.md (1126 lines), system-design.md (1216 lines), detailed-design.md (1772 lines); skimmed briefing.md

---

## Summary

| Doc | Structure | Clarity | Completeness | Audience Fit | Rating |
|---|---|---|---|---|---|
| requirements.md | Good | Good | Good | Good | 4/5 |
| system-design.md | Good | Needs Work | Needs Work | Good | 3/5 |
| detailed-design.md | Good | Good | Needs Work | Good | 3.5/5 |

No document has unresolved TBDs or marketing tone. The chief failure modes are: (1) a cross-layer duplication problem that will cause drift; (2) opaque terms introduced without definition at point of use; (3) an incomplete rule catalogue that an implementer cannot work from.

---

## requirements.md — Rating 4/5

### First 200 words

Clear. The preamble in 19 lines establishes document type, layer role, ID contract, Loom positioning, and design principles. A reader knows exactly what they hold and where to look next. No clarity issue here.

### Structure

Well-structured. Each requirement follows an identical template (statement, rationale, verification, See) — highly scannable. The Non-Goals section (NG-01 through NG-25) is the most complete such section in the docset and deserves commendation. Sections are balanced; no orphan subsections.

**Minor structural issue**: The Glossary section (line 41) says "See detailed-design.md Appendix B." This is a forward reference: a reader of requirements first, who hits an undefined term on line 65, must know to skip to a 1,772-line document. Consider adding a one-sentence "terms in plain English" note in the preamble, or at least numbering the pointer by line.

### Jargon / acronyms

Most terms are well-handled. Three exceptions:

| Location | Term | Issue |
|---|---|---|
| requirements.md:87 | `subsystem` | First use. Not defined until detailed-design Appendix B. A reader of requirements alone cannot parse "Clarion clusters the entity graph and emits one subsystem entity per cluster" without knowing what subsystem means here. |
| requirements.md:150 | `REQ-BRIEFING-01` references `EntityBriefing schema` | Schema is not referenced or described until detailed-design §2. Forward reference with no hint. |
| requirements.md:195 | `guidance_fingerprint` | Used in the cache key description before the guidance system section (REQ-GUIDANCE-*) has been read. |

### Unresolved TBDs / hand-wavy claims

None. Every requirement has a concrete verification condition. Impressive for a doc at this stage.

### Missing sections

**Minor**: There is no requirements traceability matrix between NFR-* and functional REQ-*. For example, NFR-PERF-01 (wall-clock budget) is clearly driven by REQ-ANALYZE-02 (parallelism), but the link is implicit. An implementer building phase parallelism has to infer which NFRs they are satisfying.

### Prose quality

Active voice throughout. Short paragraphs. Rationale blocks occasionally run long (5-7 sentences) but are not unclear.

### Must fix before implementation starts

1. **[requirements.md:41]** Glossary forward reference. Either add inline definitions for the five most-used terms (entity, subsystem, briefing, guidance, finding) in the preamble, or note the specific Appendix B line range so a reader can jump there.
2. **[requirements.md:87]** `subsystem` first use at REQ-CATALOG-05 has no definition. The concept drives several other requirements (REQ-BRIEFING-06, REQ-MCP-02). One sentence of definition at first use would eliminate the forward-reference problem.
3. **[requirements.md:195]** `guidance_fingerprint` is a load-bearing cache key concept used here before it is explained. Either add a parenthetical — "(a hash of composed guidance sheets; see REQ-GUIDANCE-02 for the composition algorithm)" — or reorder so REQ-GUIDANCE-* precedes REQ-BRIEFING-03.

---

## system-design.md — Rating 3/5

### First 200 words

Clear and well-framed. The layered-docs orientation ("A reader asking 'does Clarion resume after a crash?' looks in requirements; 'how does it resume?' looks here") is genuinely useful. The Loom framing is appropriate at this level.

### Structure

**Issue**: Section 12 (Architecture Decisions) is a mirror of the ADR backlog already in detailed-design §11. Both tables must be kept in sync manually. This is not a hygiene issue — the design explicitly acknowledges "Any changes to the ADR list must be applied to both tables" (system-design.md:1175). That acknowledgement is a warning sign, not a remedy. Two tables that must be manually synced will diverge. The duplication is the structural flaw; the instruction to sync is its symptom.

**Issue**: The Glossary section at system-design.md:1210 is a stub that says "Terms are defined in detailed-design Appendix B." A system-design reader who doesn't open detailed-design cannot look up a term. At minimum, the stub should reproduce the 10-15 most-used terms rather than delegating entirely.

**Issue**: The `Addresses:` headers on each section (e.g., "Addresses: REQ-CATALOG-01, REQ-CATALOG-03...") are valuable for traceability, but several sections address requirements that appear in different groups without cross-links. For example, §5 (Policy Engine) addresses `NFR-COST-01 through NFR-COST-03` but those NFR-COST-* IDs are not listed in §5's `Addresses:` header (system-design.md:477 — it shows NFR-COST-01/02/03 only; the budget / caching NFRs listed there are fine, but `NFR-COST-03` is missing from the header). Minor, but traceability is stated as load-bearing.

### Jargon / undefined terms

**Problem**: `Leiden algorithm` is introduced at system-design.md (§6 Phase 3 diagram) and system-design.md:186 without definition or citation. It is simply used. An implementer unfamiliar with community-detection algorithms has no indication this is a graph-clustering method. One parenthetical — "(Leiden: a community-detection algorithm for weighted graphs)" — would suffice.

**Problem**: `RecordingProvider` (system-design.md:529) is introduced in the LLM provider abstraction section as if it is an established term. It is described much later in detailed-design §4 as a test harness. A system-design reader sees "Adding a second provider... Adding one without caching... `RecordingProvider` for replay" and has no definition of RecordingProvider.

**Problem**: `scope lens` (system-design.md:760) is introduced with four variants but the `Taint` lens variant defers to v0.2 in a parenthetical. Fine, but the table then references `Taint` in the lens enum without the deferral qualifier, making it appear v0.1-complete.

### Unresolved TBDs

None in the conventional sense. However, system-design.md §5 (Policy Engine) references "See ADR-011" for the writer-actor decision, but ADR-011 has status "To author" (not yet written). An implementer following that reference hits a stub. This is not a TBD in the text, but it is an unresolved gap in the decision record the text relies on.

### Missing sections

**Substantive gap**: There is no error handling section at the system-design level. §6 has a failure & degradation table, but it covers pipeline failures only. There is no system-level discussion of: what happens when the SQLite writer actor's channel fills (backpressure overflow); what Clarion does when `clarion serve` cannot bind its port; or what happens when a session write fails mid-tool-call. The detailed-design covers some of this, but a system-design should at least establish the principles (fail-fast vs. degrade, which errors are surfaced to the caller vs. swallowed as findings).

**Missing**: No upgrade/migration story at the system level. The detailed-design has a migration strategy (§3), but the system-design says nothing about what a user does when Clarion's schema changes between versions. This belongs at the system-design layer.

### Prose quality

Several sections have prose accumulation — particularly §9 (Integrations), which is partly prose and partly repetition of what is already covered in §3 (Data Model). The wire-format example JSON appears in both the system-design (§9) and the detailed-design (§2/§7) with minor differences between versions. This will diverge. Pick one canonical home.

### Must fix before implementation starts

1. **[system-design.md:1172-1206 and detailed-design.md:1483-1513]** Dual ADR tables. Collapse into a single table in one document and point to it from the other. The design explicitly flags this as a sync-hazard; removing the duplication eliminates the hazard.
2. **[system-design.md:1210-1213]** Glossary stub. Reproduce the 12-15 most-used terms inline. The current stub requires opening a separate 1,772-line document to look up common vocabulary while reading architecture.
3. **[system-design.md:800-830 and detailed-design.md:312-344]** Duplicate wire-format JSON. The two examples are slightly different (system-design omits `supports`/`supported_by`; detailed-design includes them). One will be wrong when the format changes. Designate detailed-design §2/§7 as canonical; system-design §9 should reference it, not reproduce it.

---

## detailed-design.md — Rating 3.5/5

### First 200 words

Excellent. The preamble clearly states what moved up (to requirements / system-design) and what stayed here. The "When to read what" section is a genuine navigation aid. A reader picking up this document knows exactly what they will and will not find.

### Structure

Well-structured overall. The section numbering is clean (1-11, no gaps after the restructure). The appendices are genuinely appendix-grade material (glossary, Rust stack, revision history).

**Issue**: Detailed-design §11 (ADR backlog) and system-design §12 are identical tables. See must-fix above.

**Issue**: The rule catalogue in §5 is incomplete. The preamble promises "exact Phase-7 rule ID catalogues with thresholds" (detailed-design.md:22). The actual Phase-7 catalogue (§5) lists only three rules: `CLA-FACT-TIER-SUBSYSTEM-MIXING`, `CLA-FACT-ENTITY-DELETED`, `CLA-FACT-SUBSYSTEM-TIER-UNANIMOUS`. But the requirements and system-design reference many more rule IDs scattered through the text: `CLA-PY-PARSE-ERROR`, `CLA-PY-TIMEOUT`, `CLA-PY-PARTIAL-PARSE`, `CLA-INFRA-PLUGIN-CRASH`, `CLA-INFRA-LLM-ERROR`, `CLA-INFRA-BUDGET-WARNING`, `CLA-INFRA-BUDGET-EXCEEDED`, `CLA-INFRA-ANALYSIS-ABORTED`, `CLA-INFRA-PLUGIN-DISABLED-CRASH-LOOP`, `CLA-INFRA-BRIEFING-INVALID`, `CLA-INFRA-FILIGREE-UNAVAILABLE`, `CLA-INFRA-WARDLINE-REGISTRY-MIRRORED`, `CLA-SEC-SECRET-DETECTED`, `CLA-SEC-UNREDACTED-SECRETS-ALLOWED`, `CLA-INFRA-TOKEN-STORAGE-DEGRADED`, and others. There is no single place to find the complete rule catalogue, which is exactly what the preamble promises. An implementer building the finding emission layer must grep through all three documents to assemble the list.

**Issue**: The plugin manifest YAML example (detailed-design.md:60-136) shows `rules: - { id: CLA-PY-STRUCTURE-001, ... } # ... etc.` at line 128. The `# ... etc.` comment is the only occurrence of a hand-wavy stub in the entire docset. It should be replaced with the complete v0.1 rule list.

### Jargon / undefined terms

The glossary in Appendix B is the best-executed part of the docset — 40+ terms with precise definitions. The only gap: `wirte-actor` is defined in Appendix B, but `RecordingProvider` (used in §4 and §7) is not listed in the glossary despite being a non-obvious implementation-specific term.

### Unresolved TBDs

**Substantive**: detailed-design.md:1532 states "Appendix A — Future direction: unified storage layer." This is a forward-looking note (fine), but the last line says "Revisit when the integration patterns have enough real-world runway to inform the unification design" with no trigger condition or owner. Not blocking for implementation, but it floats without resolution criteria.

**Mild**: The `clarion.yaml` example at detailed-design.md:948 shows `server_url: "stdio:filigree-mcp"` for Filigree integration. But the "Preferred" transport established in system-design §9 is `POST /api/v1/observations` (HTTP), not MCP-stdio. The config example shows the fallback path as if it is the default. This is a factual inconsistency an implementer would need to resolve.

### Missing sections

**Gap**: No data-lifecycle or data-retention section. The detailed-design specifies what goes into the DB, how the DB is committed, and how it is merged. It does not specify: when old runs are pruned from `runs/<run_id>/`; when `unseen_in_latest` findings are removed from the local store; what "clarion db vacuum" or equivalent looks like for teams with years of accumulated runs. The `--prune-unseen` flag is mentioned but never fully specified.

**Gap**: The acceptance criteria in §10 (detailed-design.md:1435) list 8 functional, 4 quality, 4 operational, and 8 ecosystem criteria. The ecosystem criteria (mock Filigree, SARIF corpus, pin tests) list specific test-harness deliverables but assign no owner and cite no deadline condition. For an implementation guide, "who writes the mock Filigree server" is a real question. This is out of scope for a design doc, but the acceptance criteria give the impression of completeness without naming the delivery owner.

### Prose quality

Active voice throughout; the revision history (Appendix D) is concise and useful. One style note: the sections on concurrency (§3 Concurrency, ~300 words) repeat the same explanation of "why not a single write transaction" twice in two consecutive paragraphs (detailed-design.md:769 and 826). Merge into one.

### Must fix before implementation starts

1. **[detailed-design.md:22 vs §5]** Rule catalogue is incomplete despite the preamble promising it is exhaustive. Add a dedicated §5.1 "Complete v0.1 rule ID catalogue" table with: rule ID, phase emitted, severity, kind (Defect/Fact/etc.), brief description. Every rule ID scattered across all three documents belongs here.
2. **[detailed-design.md:948, `server_url: "stdio:filigree-mcp"`]** The `clarion.yaml` example shows the fallback transport for Filigree observations, not the preferred HTTP transport. Fix the example or add a comment distinguishing the preferred path from the fallback: `# preferred: POST /api/v1/observations (HTTP); fallback shown below`.
3. **[detailed-design.md:128, `# ... etc.`]** The plugin manifest YAML example uses `# ... etc.` for the rules list. The preamble promises exact rule catalogues. Replace the stub with the complete v0.1 Python plugin rule list.

---

## Cross-cutting issues

### Briefing.md / loom.md — skim findings

briefing.md is well-executed as an entry-point document. One issue: the "How they interact" ASCII diagram (briefing.md:73-101) shows Clarion sending findings directly to Filigree and Wardline sending SARIF to "scan import" via Clarion, but the diagram omits the triage-state read-back flow from Filigree to Clarion (which is specified in REQ-BRIEFING-05). The diagram is incomplete in a way that understates Clarion's dependency on Filigree.

### Content duplication risk across the layer

The wire-format JSON example appears in system-design §9 and detailed-design §7 with minor differences. The severity mapping table appears in system-design §9, detailed-design §7, and requirements §REQ-FINDING-03 descriptions. The Phase-7 structural findings description appears in system-design §6, detailed-design §5, and requirements §REQ-ANALYZE-05. This is the docset's most significant maintenance risk. A change to any of these (e.g., a new severity level) requires edits in three places. The revision history (Appendix D) already records one instance where the wire format was wrong in Rev 2 (Rev 3 corrected `properties`→`metadata`). A second such error will be harder to catch with three copies in play.

**Recommendation**: Pick one canonical home for each of these shared items and use cross-reference sentences in the other documents. Wire-format lives in detailed-design §7; system-design §9 says "see detailed-design §7 for the exact wire format." Phase-7 rule descriptions live in detailed-design §5; requirements §REQ-ANALYZE-05 says "full rule catalogue in detailed-design §5."

---

## Confidence Assessment

**Overall Confidence**: High

| Finding | Confidence | Basis |
|---|---|---|
| Dual ADR tables will diverge | High | Verified at system-design.md:1175 and detailed-design.md:1487; design explicitly flags sync requirement |
| Rule catalogue incomplete | High | Verified: preamble promise at detailed-design.md:22; §5 lists 3 rules; grep across all three docs found 20+ rule IDs outside §5 |
| Glossary stub delegates to another doc | High | Verified: system-design.md:1210-1213 |
| Wire-format duplication | High | Verified: system-design.md:849-882 and detailed-design.md:312-344; minor differences present |
| `clarion.yaml` MCP fallback shown as default | High | Verified: detailed-design.md:948 and preferred transport specified in system-design.md:919-921 |
| `# ... etc.` stub in plugin manifest | High | Verified: detailed-design.md:128 |
| Data-lifecycle / pruning story absent | Moderate | No relevant section found in any of the three docs; `--prune-unseen` mentioned at requirements.md:329 without full specification |

---

## Risk Assessment

**Documentation risk for implementation**: Medium-High
**Reversibility**: Easy — all issues are additive corrections or consolidations; no rework of design decisions

| Risk | Severity | Likelihood | Mitigation |
|---|---|---|---|
| Dual ADR tables diverge silently during implementation | High | High | Merge into one table in one document |
| Incomplete rule catalogue causes implementer to miss finding IDs | High | Medium | Add exhaustive catalogue to detailed-design §5 |
| Wire-format copies diverge after a Filigree schema change | Medium | High | Single canonical copy; cross-references elsewhere |
| Glossary delegation causes implementer to skip definition lookup | Low | Medium | Reproduce top-15 terms in system-design |

---

## Information Gaps

1. [ ] **Intended update protocol**: It is not stated how the three-layer docset is to be updated during implementation. If a design decision changes, must all three docs be updated? Who reviews cross-layer consistency?
2. [ ] **loom.md not reviewed in depth**: Only skimmed. If loom.md contains constraints that contradict any of the three Clarion docs, those contradictions would not appear in this review.
3. [ ] **README.md (docs/clarion/v0.1/README.md)**: The briefing.md references this as the reading-order entry point. This review did not assess it; if it is out of date with the restructured docset it would mislead new readers.

---

## Caveats and Required Follow-ups

### Before relying on this analysis

- [ ] Verify the rule-catalogue completeness finding by grepping all three documents for `CLA-` prefixed strings and comparing against detailed-design §5.
- [ ] Confirm the `clarion.yaml` example ambiguity is an error and not an intentional default by checking against the ADR for observation transport.

### Assumptions made

- The docset is intended as an authoritative implementation guide, not a working draft. Issues are rated at that bar.
- System-design §12 and detailed-design §11 are meant to be identical; any current differences are authoring drift, not intentional divergence.

### Limitations

- Technical accuracy of the design decisions (e.g., whether Leiden clustering is the right algorithm, whether the dedup policy is correct) is outside this review's scope.
- Wardline and Filigree internals referenced in the docs (e.g., whether `wardline.core.registry.REGISTRY` actually exists with the claimed interface) are not verified here.

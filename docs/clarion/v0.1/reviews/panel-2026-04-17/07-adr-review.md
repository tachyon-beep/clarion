# Panel Review 2026-04-17 — ADR Set Review

**Reviewer**: architecture-decision-reviewer
**Scope**: ADR-001 through ADR-004 plus the backlog in `adr/README.md`
**Context**: Docs-only repo, pre-implementation. Maturity-appropriate rigor here is "enough to prevent later regret," not CMMI Level 3 formalism.

## Overall posture

The four authored ADRs are coherent and the set is internally consistent: ADR-001 (Rust) is the premise, ADR-002 (subprocess JSON-RPC) is what lets ADR-001 stay narrow, ADR-003 (symbolic IDs) is independent of language choice, and ADR-004 (Filigree-native intake) is the only one genuinely forced by external reality (integration-recon). The set reads as a snapshot of decisions already made, not decisions being made. That is fine — but it means the ADRs are thinner on genuine alternatives than a from-scratch evaluation would be, and that thinness is most visible in ADR-001.

Cross-ADR consistency is clean. No contradictions. The `Related Decisions` links are reciprocal. ADR-002 correctly inherits Rust and correctly isolates the plugin boundary so that a later language reversal on ADR-001 would not cascade.

## ADR-by-ADR

### ADR-001 — Rust for the core

**Verdict: accept-with-amendments.**

Honest in an unusual way: it openly says "primary author directed Rust" and says the rest of the design already assumes Rust. That is accurate, and pretending otherwise would be worse. But two amendments are needed.

1. **Name the requirement, not just the posture.** "Local-first and operationally lightweight" is a principle, not a requirement. The actual requirement links exist — single-binary distribution (system-design §1), subprocess supervision (REQ-PLUGIN-01), SQLite (NFR-PERF), no runtime install — and the ADR should cite them. Without that, a future reader cannot test the decision.
2. **Strengthen the Go rejection.** "Weaker fit for SQLite ergonomics" and "less alignment with author's direction" are thin. Go would meet the single-binary and subprocess-supervision requirements. The honest reason to prefer Rust is author fluency plus ecosystem (axum/rusqlite/tokio). Say that. Don't strawman Go.

Resume-driven-design smell: borderline. Rust *is* fashionable for CLI tools, and "primary author directive" is the kind of phrasing that normally triggers rejection. It survives here because (a) the requirements genuinely fit Rust's strengths, (b) Go would also fit and is acknowledged, and (c) the ADR is self-aware about the directive nature. The phrase "Not subject to alternatives analysis" in system-design §12 should be softened; it reads as defensive.

Testability: add a 12-month review criterion. Suggested: if contributor onboarding time exceeds N weeks, or if >20% of implementation effort is fighting the borrow checker on non-core paths, revisit.

### ADR-002 — Plugin transport: Content-Length framed JSON-RPC

**Verdict: accept.**

The strongest ADR of the four. It responds to a specific defect flagged in the design review (framing ambiguity), names three genuine alternatives (newline-delimited, embedded Python, Wasm) with honest trade-offs, and the chosen option is clearly the lowest-risk fit for v0.1 with a named escape hatch (Wasm later). Consequences section honestly flags stdout/stderr hygiene as a plugin-author burden — that is a real cost, not a sales pitch.

Minor: cite REQ-PLUGIN-01 explicitly. The ADR refers to "the design review identified framing ambiguity as a real defect" but the requirement itself already pins Content-Length framing, which is a stronger citation.

Testability: if plugin authors routinely violate stdout hygiene in practice, the decision is fine but needs tooling support (a plugin-test harness that catches stdout pollution).

### ADR-003 — Entity IDs use symbolic canonical names

**Verdict: accept.**

Tied to a named requirement (REQ-CATALOG-06, symbolic IDs) and to a concrete failure mode (path-embedded IDs rot on file moves). Alternatives are genuine: path-embedded (simpler, fails the stability bar) and full alias tracking in v0.1 (correct but over-scoped). The "manual repair in v0.1, EntityAlias in v0.2" split is an appropriate phasing decision, not a cop-out.

One gap: the ADR does not explicitly address the case the requirements hint at — Python re-exports and `__init__.py` surface names where "definition site wins" is not always obvious. Add a one-line consequence: "Re-export ambiguity resolved by plugin-declared canonical rule; plugins own this semantics."

Testability: measure how often `--repair-aliases` is invoked after routine refactors in the validating codebase. If that count is high, EntityAlias needs to come forward from v0.2.

### ADR-004 — Filigree-native intake as v0.1 finding exchange format

**Verdict: accept.**

The only ADR in the set that is genuinely forced by external evidence (integration-recon). Alternatives are real (direct SARIF, SARIF-lite suite format) and both are honestly rejected on grounds of external reality or centralisation drift. The `metadata.clarion.*` nesting convention is tied to verified Filigree behaviour, which is the right kind of grounding.

Gap: the ADR does not name which requirement it serves. REQ-INTEG-FILIGREE-01 is the obvious anchor and should be cited. Also add an explicit link to REQ-FINDING-04 (the SARIF translator survives as a permanent path) so the "SARIF isn't dead, it's just not the primary contract" point has a requirement hook.

Testability: if Filigree adds a native SARIF ingest in v0.2+, does ADR-004 need to be revisited? The ADR should say "no — SARIF-as-import survives regardless, and the native intake is still the lowest-risk v0.1 contract" or equivalent.

## Backlog triage

The backlog is mostly the right set of decisions. Priorities broadly track load-bearingness: ADR-014 through ADR-018 are all P0 and all tied to integration-recon findings, which is the correct signal. A few observations:

- **ADR-011 (writer-actor)** is P1 but is a concurrency-model decision that, if wrong, is expensive to reverse. Consider promoting to P0 or at least authoring before first `clarion analyze` implementation lands.
- **ADR-010 (MCP as first-class surface)** is P2 but the lock-in risk it names is strategic, not tactical. P2 is defensible given v0.3+ review is planned, but the ADR should be authored before external API stability is promised.
- **ADR-013 (pre-ingest secret scanner)** is correctly P1 but reads like a P0 to me: shipping without it means the first real user leaks secrets to Anthropic. The requirement (NFR-SEC-01) appears to already treat this as a hard dependency. Reconcile: if it is a hard dependency, ADR-013 is P0.

## Highest-priority missing ADR

**ADR on the core/plugin ontology ownership boundary** (how plugins declare entity kinds and edge kinds; how the core validates without embedding ontology).

This is Principle 3 of the requirements — "plugin owns ontology; core owns algorithms" — and it shapes ADR-002's RPC surface, ADR-003's ID format (the `{kind}` component), REQ-PLUGIN-02's manifest semantics, and ADR-006's clustering subgraph definition. It is not in the backlog at all. Every current backlog item implicitly assumes a working answer. Authoring it is cheap now and expensive later: once the first plugin ships, the ontology-registration contract is frozen by precedent rather than by design.

Secondary gap: no ADR covers the **failure/degraded-mode semantics of the plugin subprocess** (crash, hang, protocol violation). ADR-002 covers framing; it does not cover lifecycle. REQ-ANALYZE-06 ("no silent fallbacks") implies a policy that is not yet written down as a decision.

## Confidence / risk / gaps

- **Review confidence**: HIGH on the four authored ADRs; MEDIUM on backlog triage (I have not read every backlog summary in detail).
- **Decision risk**: LOW for ADR-002/003/004; LOW-MEDIUM for ADR-001 (reversibility is poor but fit is real); MEDIUM for the un-authored ontology-boundary decision flagged above.
- **Information gaps**: I inspected requirements.md and system-design.md via targeted greps rather than full read; the detailed-design.md was not consulted.
- **Caveats**: "accept" verdicts are conditional on the amendments landing. The set is pre-implementation — the real test of these ADRs is the first time one of them is reversed, and how cleanly that reversal can happen.

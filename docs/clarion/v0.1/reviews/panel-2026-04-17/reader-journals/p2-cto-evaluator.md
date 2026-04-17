# Reader Journal — p2-cto-evaluator (Marcus)

**Date:** 2026-04-17
**Documents read:** briefing.md (full), loom.md (§1–§6, §9; skipped §7 and §8 per persona)
**Total reading time (estimated):** ~12 minutes

---

## Mood Journal

### briefing.md — status table first

First move: scan the page for a status table. Took about thirty seconds. Found it under "Current state" — four rows, honest. Filigree: built, in use. Wardline: built, in use. Clarion: designed only. Shuttle: proposed, no design.

Good. They put the table in. That's already more self-aware than most early-stage project docs. But the lede — the one-paragraph version at the top — reads "three independent tools that enrich one another." That's a three-product claim. The table tells me it's two-plus-one-on-paper-plus-one-concept. Those are not the same thing, and the one-paragraph version is the first thing a skimmer reads before they find the status table.

I don't think this is deceptive. I think the author wrote the vision paragraph at vision altitude and the table at ground altitude, and didn't notice the gap between them. But I notice it, because I've sent docs to my board that had exactly this structure and had to walk it back in Q&A. The correct lede here is: "Two tools in production, one in design, one proposed." Lead with what's real.

That said — the table is there, the language is unambiguous ("designed, not yet built"), and Shuttle is explicitly flagged as "no design yet." That's a point in the project's favor. The authors aren't hiding the ball. They're just framing before grounding.

### briefing.md — product descriptions

I read the Clarion description because Clarion is what's not built yet and that's what I care about. The pitch is coherent: LLM-agent-facing catalog, answers structural questions from batch analysis so agents don't have to re-explore at query time. Fine. The "code-archaeology catalog" label is a bit evocative for my taste but I understand the problem it's solving. MCP integration is smart — agents are already using MCP as a tool surface, so meeting them there is the right call.

Filigree and Wardline I skimmed. They're already live, they have working CLIs, and the CLAUDE.md in this repo uses Filigree actively. That's not vaporware. Wardline's 38-annotation-across-17-groups thing is detailed in a way that suggests real implementation rather than spec. Someone actually built this.

Shuttle I basically skipped. Proposed, no design. Not relevant to my evaluation today.

### briefing.md — the interaction model

Glanced at the fabric diagram. My senior engineer would need to read this carefully; I'm just checking whether the integration model looks conceptually sane. It does. The flows are documented, they're narrow (file reads and POST calls, not shared databases), and the translation layer lives in Clarion, which is the product that owns the catalog. That's the right place to put it. The entity ID scheme — Clarion mints the identifiers, others reference them — is a sensible authority model.

One thing caught my eye: the table under "Data flows" lists "Entity state | Clarion | Wardline (v0.2+)" with the note "Wardline currently re-scans." That's an honest flag that one of the described integration flows isn't live yet. Appreciated.

### briefing.md — principles

Four principles. Read them. They are not platitudes — each one has a boundary condition. "Clarion observes, Wardline enforces" is a clear division of responsibility that would resolve an actual design question. "Findings are facts, not just errors" has implications for schema. "Each tool is independently useful" maps directly to the thing I care about: can my team adopt one piece before committing to the whole suite? "Local-first, single-binary, git-committable state" is a strong stance — it tells me there's no surprise cloud dependency to audit.

These read like principles that have already made someone say no to something. That's the test.

### briefing.md — what the suite needs from siblings for Clarion to ship

This section is actually the most useful thing in the document and it's near the bottom. It lists concrete changes required in Filigree and Wardline as prerequisites for Clarion v0.1 to ship — pluggable registry backend, stable REGISTRY_VERSION, an HTTP observation endpoint. Clarion ships with degraded-mode fallbacks if siblings aren't ready (`--no-filigree`, `--no-wardline`).

This is mature. This tells me the team has thought about dependency sequencing, not just architecture. This is the section a senior engineer would want to see: "here is what integration actually costs in the other codebases." Most project docs of this type handwave this and say "and then we integrate." This one names the specific asks.

---

### loom.md — §3 (Federation, not monolith)

One paragraph, bolded founding law. Good. The anti-pattern it names — "a lightweight glue layer that quietly becomes the real system of record, reducing sibling products to thin clients" — is exactly the failure mode I've seen in three prior integrations at previous companies. The fact that they name it explicitly tells me someone has seen it happen before and is designing against it. This is not naivete.

The phrase "stealth-monolith failure mode" is precise. That's what happens. You call it a bus, it becomes a broker, it becomes the database, and now nothing works without the thing you said was optional. The fact that this section exists — and that it was written as a bolded founding law, not buried in a footnote — suggests a deliberate architectural stance rather than a retrospective rationalization.

### loom.md — §4 (Composition law)

Solo mode, pair mode, suite mode. The hard rule: "A product that only works when all siblings are present is a feature of a monolith wearing modular clothing." That's quotable. The mode framing is useful because it gives a concrete test I can apply to a design decision. If Clarion in solo mode has no respectable use case, §4 flags it. That's a constraint that could actually reject something.

I want to know whether this constraint was tested against Clarion specifically — whether someone wrote down "Clarion solo mode use case: X" and had to defend it. The briefing says `clarion analyze` and `clarion serve` are the invocations, and the catalog itself is useful without Filigree or Wardline. I can see it. The solo-mode story is coherent: you get a structural map of your codebase, answerable via MCP, with no other tool required. That stands up.

### loom.md — §5 (Enrichment, not load-bearing)

This is the load-bearing section for my evaluation. The failure test is clear: if removing a sibling changes the *meaning* of another product's data — not its richness, but its meaning — federation has collapsed. The examples are concrete and each one passes the test on paper.

The Filigree example is the easiest one to believe: you can file a ticket, work it, and close it without Clarion installed. That's obviously true. The Wardline example is also clean: SARIF output doesn't care whether Filigree is downstream.

The Clarion example is the one I scrutinize. "Clarion builds its catalog whether Wardline is present or not. Wardline's annotations enrich Clarion's entity metadata with trust-tier and policy-semantic information, but Clarion's structural truth is independent of Wardline's policy truth." I accept this on first reading. But I want someone to test it: run Clarion against a codebase with no Wardline config and see whether the catalog is coherent or riddled with blank trust-tier fields that make the output look incomplete to the user. "Works" and "useful" are different bars. §5 is about meaning, not usefulness — so if blank trust-tier fields are just absent enrichment rather than semantic corruption, it passes. My senior engineer should verify this when evaluating the detailed design.

The "Why this matters" sub-section closes with the sharpest sentence in either document: "The moment one product needs another to make sense of its own data, the composition law becomes dishonest — standalone mode works only because the sibling is still running somewhere, and the illusion of modularity collapses the first time deployment doesn't match." That's not a platitude. That's a failure mode written from experience.

### loom.md — §6 (What Loom is NOT)

Explicit negatives. No shared runtime, no shared config, no central store, no system of record for cross-product state, no identity reconciliation service, no capability negotiation bus. Six clean "not this" statements. I find this section more useful than the positive definitions because it forecloses the sneaky compromises. The final test formulation — "if the proposal introduces something that would need to be running or present for the suite to work, it violates federation" — is the kind of thing you put on the wall in a design review.

I read this and thought: this is someone who has been in an architecture review where a well-meaning engineer said "couldn't we just add a shared config service?" This section is the rehearsed answer.

### loom.md — §7 and §8

Skipped, as intended.

### loom.md — §9 (Status)

Same table as briefing.md, slightly terser. Consistent. Good.

---

## Key Finding

The one thing Marcus takes away that the author probably didn't notice they did:

**The doctrine is not aspirational — it is defensive.** loom.md §3, §5, and §6 are not describing a vision of how the suite will work; they are foreclosing specific failure modes that the author has clearly encountered before. The "stealth-monolith" framing, the "enrichment not load-bearing" failure test, the explicit "What Loom is NOT" negations — these read as a set of pre-commitments against pressures the authors expect to face from contributors who will reasonably ask "wouldn't it be simpler to share X?" The doctrine is a defense against the project's own future contributors more than it is a pitch to new adopters. That is actually a strong signal. Projects that write their doctrine as a forward-looking vision tend to drift when implementation pressure hits. Projects that write their doctrine as a series of named failure modes they have decided to prohibit tend to hold the line longer. Marcus would not surface this reading to the authors — they probably know it — but it changes his confidence level in the architecture's durability.

---

## Unanswered Questions

**1. What is the solo-mode validation story for Clarion?**
The doc says Clarion works without Filigree and Wardline, and the claim is coherent in principle. But Clarion is not built yet. Who is responsible for verifying that solo-mode Clarion is genuinely useful, not just syntactically independent? The "first customer is elspeth" framing suggests there will be a real validation. But I'd want to know: is there a written test for solo-mode that someone will run before v0.1 ships, or is this architectural axiom accepted on faith until the integration test?

**2. What does Clarion v0.1 look like on a codebase without Wardline annotations?**
The enrichment model means trust-tier fields will be absent if Wardline isn't present. That's fine semantically. But is the user experience coherent? A catalog that presents blank trust-tier metadata for every entity might look broken even if it's technically correct. Is there a degraded-mode output that makes absence legible, not just absent?

**3. Who owns the cross-product integration test surface?**
The briefing lists specific changes required in Filigree and Wardline for Clarion to ship. Those are cross-team asks. What's the coordination model? Is there an integration test suite that validates the Wardline SARIF → Clarion translator → Filigree ingestion path end-to-end? Or is each team validating their own piece bilaterally? This is the sequencing risk for a small-team context.

**4. When does "designed, not yet built" become a concern?**
Clarion has a design doc set but no implementation. The design itself seems sound based on what's been reviewed here, but design docs can drift from implementation reality. Is there a commitment cadence — sprint-level, milestone-level — that says when Clarion will produce working code that can be tested against elspeth? I'm not worried about vaporware in the bad-faith sense, but I want to know whether this is "designed and building imminently" or "designed and building when bandwidth permits."

**5. What is the actual adoption cost for a team that wants to start with Filigree only?**
The solo-mode principle says each tool stands alone. Filigree is already live. If my team started with just Filigree and decided later to add Wardline, then later Clarion — what does that incremental path look like in practice? Are the integration protocols versioned and backward-compatible? The briefing mentions `--no-filigree` and `--no-wardline` degraded modes for Clarion, which is promising. But I'd want the sibling side: when Filigree gets the `registry_backend` change that Clarion requires, does that change break existing Filigree setups, or is it additive?

# Reader Journal — p4-oss-contributor (Dev)
**Date**: 2026-04-17
**Reading order**: loom.md first, briefing.md second (hunting claims before product names)

---

## Mood Journal

### loom.md §1 — What Loom is

First paragraph and I'm already parsing the tagline: "enterprise-grade code governance on small teams." That combination of words has been used to sell me at least a dozen Slack integrations and CI plugins that shipped one point release and went dark. I'm going to keep reading, but the alarm is already ticking.

The metaphor explanation is unnecessary. I know what a loom does. But I notice the author feels the need to immediately clarify that "there is nothing called Loom to install, deploy, or keep running." That's a telling defensive move on sentence two. Either someone already asked the obvious question, or the author has been burned before. Probably both. This is actually a good sign — it means they've thought about the failure mode, not just the vision.

Four products listed. Three built (allegedly), one "proposed and not yet in flight." Shuttle gets named here without context. Reading loom.md without briefing.md first, I genuinely do not know what Shuttle does from §1. The phrase "transactional scoped change execution" is jargon stacked on jargon. File that for §7.

### loom.md §2 — Authoritative domains

Clarion: code archaeology, entity catalog. Filigree: issue tracking, workflow lifecycle. Wardline: trust policy enforcement. These are clean separations. I've seen messier domain splits in projects with ten engineers and a dedicated architect. The bounded-context thinking is real here, not decorative.

Shuttle's description in §2 is where I start actually stress-testing: "receives a scoped change intent, binds it to files or entities, orders the edits, applies them incrementally with pre- and post-change checks, rolls back on failure, and lints / commits / emits telemetry on success." Okay. But the sentence that follows is the one that matters: "It does not plan, triage, or reason about the code it is editing." So Shuttle receives intents but does not generate them. Who generates them? The domain description implies Filigree holds work state and Clarion understands structure. If Shuttle executes changes and those changes must be "scoped" before Shuttle sees them — scoped by what? The product description for Shuttle is actually a description of a capability *hosted inside* the workflow that Filigree and Clarion jointly enable. I'm already suspicious this fails go/no-go criterion 2.

### loom.md §3 — Federation, not monolith

"This is the founding architectural law." Bold claim. The paragraph lays out the failure mode correctly: "stealth-monolith failure mode: a 'lightweight glue layer' that quietly becomes the real system of record." I've watched this happen with a monitoring sidecar in my own tool's ecosystem — everyone started pointing their configs at the sidecar endpoint, and suddenly the sidecar was the thing that had to be up. The author has named this precisely.

What I'm looking for now is whether the rest of the document actually enforces this law or just names it. The law is only as strong as the tests that apply it.

### loom.md §4 — The composition law

Solo mode, pair mode, suite mode. Suite mode must never be mandatory. This is a clean formulation and I agree with it architecturally. The claim "pairwise composability is a hard rule, not an aspiration" is exactly what I want to see — the weakest form of this kind of architecture doc says "should" everywhere, and this one says "hard rule."

But: hard rules need enforcement mechanisms. Where's the test harness? Where's the integration test that actually exercises a product in solo mode and verifies its outputs aren't semantically degraded? These aren't rhetorical questions; they're the difference between a doctrine doc and a policy.

### loom.md §5 — Enrichment, not load-bearing

This is the best section in the document. The failure test is concrete: "If removing a sibling product changes the *meaning* of another product's own data, Loom has centralised too far." The emphasis on *meaning* versus *convenience* is exactly the right distinction. A lot of "optional" integrations are load-bearing in disguise — they're technically removable but the product's UX degrades from coherent to confusing without them.

The Filigree example holds up. The Wardline example holds up. The Clarion example holds up.

The Shuttle example is where my earlier suspicion gets confirmed: "Shuttle, if built, would execute changes whether any sibling is present. Sibling tools enrich its telemetry (which Filigree ticket? which Clarion entity? which Wardline policy?) but are never required for a change to apply or roll back."

On its face this sounds like a passing result. But read it again. What is Shuttle executing in solo mode? "Scoped change intents." Where do scoped change intents come from? The doc doesn't say. If the only realistic source of a scoped change intent is a Filigree issue plus Clarion entity context — if that's the actual workflow the tool is designed for — then Shuttle in solo mode is a transaction wrapper with no transaction planner. That's not a product; it's a library function waiting for a caller.

The telemetry framing is suspicious too. "Which Filigree ticket? Which Clarion entity? Which Wardline policy?" are described as optional enrichment. But for the tool to emit *any* meaningful provenance — to be useful as a change-execution record — you need at least one of those. An execution record that says "I edited these files" with no context about why or under what authority is not a transactional record; it's a git commit with extra steps.

### loom.md §6 — What Loom is NOT

Good defensive section. The "capability negotiation bus" exclusion is specifically useful — I've seen federated architectures get corrupted by exactly that abstraction. Each product probing others directly via their own surfaces is the right call.

The test at the end of §6 is important: "if the proposal introduces something that would need to be *running* or *present* for the suite to work, it violates federation." I would apply this test to Shuttle's intent source. Does Shuttle need Filigree running to receive intents in the typical case? If yes, that's a violation. The document doesn't answer this because Shuttle has no design document yet.

### loom.md §7 — The go/no-go test

Here's where I came to do real work.

The four criteria are:

1. Authoritative for one narrowly bounded thing
2. Useful by itself
3. Forms a sensible story with each existing product one-to-one
4. Full suite is better because of it, without making others incomplete in its absence

Let me run Shuttle against all four.

**Criterion 1 — authoritative for one narrowly bounded thing**: "Transactional execution record of a code change." That scope *sounds* narrow. But "execution record" implies you also own the pre-change state, the post-change state, the rollback log, the lint output, and the commit hash. That's not one thing; that's a ledger. Still, arguably coherent as a single domain. I'll give this a conditional pass — it depends on design decisions not yet made.

**Criterion 2 — useful by itself**: This is where Shuttle fails, or nearly fails. Shuttle "receives a scoped change intent." In solo mode, where does the intent come from? A hand-written JSON file? An ad hoc CLI flag? The document does not answer this. The voice in my head says: "Shuttle in solo mode is a deployment script that has read its own architecture document." If the intended invocation path is always `filigree issue → clarion entity context → shuttle execute`, then Shuttle is not useful by itself in any meaningful sense. It is a pipeline stage dressed as a product.

The document's own §5 Shuttle example essentially admits this by framing the solo-mode value as "apply and roll back" — transactional safety on top of what? On top of a change plan you already had. Where did you get the change plan? The doc is silent.

**Criterion 3 — sensible story with each existing product one-to-one**: Shuttle + Filigree: plausible (execute the changes associated with a ticket). Shuttle + Clarion: plausible (execute changes scoped to a specific entity or module). Shuttle + Wardline: this is the interesting pair. "Execute a change that respects trust tier boundaries" is a coherent story, but it requires Wardline's semantics to be consulted during execution. Is that enrichment or load-bearing? If Shuttle executes a change that promotes code to INTEGRAL tier and Wardline isn't present to validate, did the change succeed or fail? The doc doesn't say, because there's no Shuttle design document.

**Criterion 4 — suite is better because of it, without making others incomplete**: Conditional yes. If Shuttle ships well, the suite gains something real. But "without making others incomplete in its absence" has an uncomfortable implication: the suite is currently described as v0.1 = Clarion + Filigree + Wardline. That suite must be complete without Shuttle. The briefing calls Shuttle "proposed" and Clarion "designed, not yet built." So the suite being evaluated for v0.1 is Filigree + Wardline, with Clarion pending. The go/no-go test for Shuttle is therefore pre-emptive — it doesn't gate Shuttle joining v0.1, it gates Shuttle ever joining. That's fine, but the test is being applied to a design that doesn't exist yet (§9 says "separate design effort when prioritised"). Applying go/no-go to a non-design is a placeholder at best. It signals intent without providing evidence.

Overall go/no-go verdict on Shuttle: criterion 2 is suspect, criterion 3 requires a design document to evaluate, and criterion 4 is unanswerable until criterion 2 is resolved. The document is not dishonest about this — it explicitly calls Shuttle "proposed" and "not in flight." But the §5 defense of Shuttle's solo-mode validity is premature reassurance dressed as analysis.

### loom.md §8–9 — Naming and Status

Naming rationale is fine, I don't care about names. §9 status table is honest and I appreciate it. "Designed; implementation not yet started" for Clarion is more candid than most design docs manage.

---

### briefing.md — Consistency Check

Reading briefing.md after loom.md. The question is whether the product descriptions in briefing.md are consistent with the doctrine claims in loom.md, or whether they quietly smuggle in dependencies that loom.md's federation axiom would prohibit.

**Clarion description** — "Consult-mode LLM agents query Clarion through MCP tools so they never need to spawn an explore-agent." Fine. This is the product doing its job. No integration dependency described as mandatory.

**Data flows table** — This is where I find the most tension. Look at the flow labeled "Entity state": "Clarion → Wardline (v0.2+): Clarion HTTP read API; Wardline currently re-scans." So Wardline currently re-scans independently, which is good (solo-mode viable). But in v0.2+, Wardline reads Clarion. That's fine if enrichment only — but the note says "Wardline currently re-scans" which implies Wardline's correctness today does not depend on Clarion. In v0.2+, if Wardline stops re-scanning and only reads Clarion's entity state, the enrichment-not-load-bearing principle is violated. The briefing doesn't flag this risk.

**Clarion's v0.1 asks from siblings** — "Filigree: a pluggable `registry_backend` so Clarion can own the file registry." Wait. Clarion owning the file registry means Filigree's file tracking depends on Clarion's catalog. If Clarion is absent, what happens to Filigree's `registry_backend`? The briefing says Clarion ships with "degraded-mode fallbacks (`--no-filigree`, `--no-wardline`)" — but the question runs the other direction. What happens to Filigree in `--no-clarion`? If Filigree's registry backend silently breaks or degrades to an incoherent state without Clarion, that's a load-bearing dependency in the direction loom.md most explicitly prohibits.

**"Bootstrapping the suite fabric"** — The briefing uses this phrase to describe Clarion v0.1's scope. loom.md §3 explicitly says "there is no Loom runtime, no Loom config layer, and no Loom store." But Clarion is described as building the "cross-tool protocols that Filigree and Wardline don't yet speak." If those protocols, once landed, become the connective tissue that other products depend on, then Clarion becomes the fabric and "no Loom runtime" becomes a naming convention, not an architectural reality. The federation axiom survives only if the protocols Clarion bootstraps are consumed optionally by siblings, not required.

The briefing is mostly consistent with loom.md. The tensions are at the edges, and they're the right kind of tension — things that need design decisions, not things that have already been decided badly. But the file-registry ask and the v0.2+ entity-state flow deserve explicit federation-axiom reviews before they land.

---

## Key Finding

The loom.md federation axiom is genuine architectural thinking — the failure test in §5 is precisely formulated, the §6 exclusion list is comprehensive, and the solo/pair/suite composition law is clean. But the document pre-emptively defends Shuttle's solo-mode viability without actually establishing it: the claim that Shuttle "would execute changes whether any sibling is present" is asserted, not argued, and it sidesteps the foundational question of where scoped change intents originate in solo mode. The go/no-go test in §7 is well-designed as a gate but is being applied to a product with no design document, making criterion 2 (useful by itself) unverifiable rather than passed. Meanwhile, briefing.md introduces two concrete integration patterns — the `registry_backend` ownership transfer and the v0.2+ Wardline-reads-Clarion entity state flow — that have not been reviewed against the enrichment-not-load-bearing principle and could quietly invert the dependency direction that loom.md most explicitly prohibits.

---

## Unanswered Questions (contributor-oriented)

1. **Where do scoped change intents come from in Shuttle solo mode?** If the answer is "hand-authored JSON" or "a CLI flag", that technically satisfies criterion 2 but nobody would actually use it that way. If the answer is "Filigree issues always", then Shuttle hasn't passed criterion 2 and the go/no-go test should return a no until a design exists that establishes genuine solo-mode utility.

2. **What is the federation-axiom test procedure for new integrations?** loom.md §5 states the failure test in prose, but there is no described process for applying it when a new cross-product data flow is proposed. Who reviews it? Is there a checklist? For a contributor wanting to add a new integration point — say, a Clarion-to-Wardline entity state read — what is the acceptance criteria for "this is enrichment, not load-bearing"?

3. **What is the stability contract on Wardline's annotation registry?** briefing.md says Clarion's Python plugin "directly imports" `wardline.core.registry.REGISTRY` at plugin startup. If Wardline changes its registry structure, Clarion's plugin breaks. Is there a documented API surface here, a versioned interface, or is this currently an informal coupling? For a contributor building a non-Python language plugin for Clarion, understanding this boundary matters a lot.

4. **How does the `registry_backend` pluggability work if Clarion is absent?** The briefing says Filigree needs a "pluggable `registry_backend` so Clarion can own the file registry." What does Filigree use when Clarion is not present — a default no-op backend, a built-in fallback, or does the feature simply not appear in the UI? The federation axiom requires that Filigree's data remains semantically coherent without Clarion; the answer here determines whether that requirement is met.

5. **What does "designed, not yet built" mean for contribution entry points?** Clarion has a requirements doc, a system design, and a detailed design. There is no implementation. For an external contributor wanting to evaluate whether to invest time: is there a contribution guide, a defined first-milestone scope, or a list of deliberate deferral decisions that would help a contributor know what is in-scope for v0.1? The design docs exist but there is no stated "here is how to get started if you want to build this" path.

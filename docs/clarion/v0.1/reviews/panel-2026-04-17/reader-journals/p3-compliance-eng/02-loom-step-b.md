# Journal Entry: loom.md
## Step B — In-reading and post-reading reactions

**Date:** 2026-04-17

---

### §1-2: What Loom is / The products

"Loom is a family name and a composition doctrine — not a platform, not a shared runtime, not a store, and not a broker."

Fine. The anti-centralization stance is stated clearly and early. §2 gives me the bounded domain list I needed: Filigree owns "finding triage state" explicitly. That is the sentence I was looking for in briefing.md and didn't find stated this cleanly. So: Filigree is authoritative for triage state. The question is whether that authority is durable when upstream translation changes.

---

### §3: Federation, not monolith

"The rule protects against the stealth-monolith failure mode: a 'lightweight glue layer' that quietly becomes the real system of record."

They have named the failure mode I was watching for. That is encouraging. The question is whether Clarion's translation layer — which the briefing describes as the single point responsible for three identity schemes — is itself the stealth monolith in embryo. It is not load-bearing *yet* because Clarion is not built. When it ships, the translation layer will be the thing every finding's lineage depends on. Naming the failure mode does not prevent it.

---

### §4: The composition law

Reading quickly. This is product strategy. The three modes (solo, pair, suite) are coherent. The pairwise composability rule is a good gate. My annotation: "this is the design-time test, not the runtime test." Passing the composition law at design time does not guarantee the translation layer stays clean at runtime. I am noting this and moving on.

---

### §5: Enrichment, not load-bearing — slow read

This is the section I came for.

"A sibling product may enrich another product's view, but it must never be required for that product's semantics to make sense."

Now the failure test: "If removing a sibling product changes the *meaning* of another product's own data, Loom has centralised too far."

I am going to hold this test up against the SARIF translation path I identified in briefing.md.

Wardline emits a finding. Clarion translates it. Filigree stores it. Now: if I remove Clarion from the picture — if Clarion is down, or its translator has a bug, or a version bump changes its output — what happens to the Filigree finding record?

Option A: Wardline eventually gets a native Filigree emitter (the briefing mentions this as a future goal — "eventually, a native emitter to Filigree so Clarion's SARIF translator can retire"). In that world, Clarion's translator is a temporary bridge, and its removal does not change the meaning of anything already in Filigree — it just means new findings stop arriving until the native emitter ships.

Option B (the current state): Wardline has no native Filigree emitter. Clarion is the only path from Wardline to Filigree. In this state, removing Clarion does not change the *meaning* of findings already in Filigree — those records are already stored. But it stops new findings from arriving, and any in-progress triage that relied on Clarion's entity-ID cross-references now has dangling references. The Filigree triage record is semantically intact (it knows "this finding is suppressed, triaged by X on date Y") but the structural context (which Clarion entity does this map to?) is unavailable.

Does that fail the §5 test? The doctrine says: "Sibling absence may reduce convenience or automation; it must not alter semantics." Dangling entity cross-references — is that altered semantics or reduced convenience? I would argue it is a semantic degradation. The Filigree record says "this finding maps to entity `python:class:auth.tokens::TokenManager`" and if Clarion is absent, that claim is unverifiable. The data is not incoherent, but it is unauditable without Clarion. For compliance purposes, "unauditable without a sibling tool" is close enough to "altered semantics" to flag.

**But loom.md's concrete examples do not address this case.** The Wardline example in §5 is: "Wardline enforces trust policy whether Filigree is ingesting findings or not. Findings reach Wardline's own SARIF output regardless of whether a downstream triage system exists." That is about Wardline's *own* operation being independent. It does not address what happens to Filigree's finding records when the Clarion translator — which is the only conduit for Wardline→Filigree in v0.1 — changes or is absent.

The Clarion example in §5 is: "Wardline's annotations enrich Clarion's entity metadata with trust-tier and policy-semantic information, but Clarion's structural truth is independent of Wardline's policy truth." This is about Clarion's independence from Wardline — correct, but again not addressing the translation path.

**The §5 failure test does not fully answer my lifecycle question.** It answers a different question: "do the products work alone?" Yes. But my question is: "when finding identity crosses the Clarion translation boundary, who is responsible for that translation's fidelity over time, and is that responsibility durable?" loom.md's answer is implicitly: Clarion is responsible (§6: "the product that *cares* does the translation"). But what happens when Clarion's translation changes? The doctrine does not say. The enrichment principle says Clarion's translation enriches the finding but should not be load-bearing for Filigree's semantics. But in v0.1, the translation *is* load-bearing for getting Wardline findings into Filigree at all.

There is a gap between the doctrine and the v0.1 implementation. The doctrine is right. The v0.1 implementation violates it in one specific, acknowledged, temporary way (the SARIF translator as bridge). The question is whether the gap is managed — is there a schema-compatibility contract that makes the violation bounded and traceable? The briefing says Filigree needs "a published schema-compatibility contract." That is the ask, not the answer.

---

### §6: What Loom is NOT

"Finding lifecycle lives in Filigree. Entity identity lives in Clarion. Policy baselines live in Wardline."

Clean statement of authority. Good.

"When cross-scheme translation is needed — e.g. Wardline qualname → Clarion entity ID — the product that *cares* does the translation, because that product is the one whose authority needs it."

"The product that cares does the translation." In practice this means Clarion owns the translation and Clarion is the single point of authority for cross-scheme identity. That is the governance risk I flagged in my briefing.md reading. loom.md names it as a deliberate design choice, not an oversight. I can work with a deliberate choice. What I need is: what are the versioning commitments on Clarion's translation layer, and what is the migration story when those change?

---

### §7: The go/no-go test

Skimming. Product strategy. The four gates are coherent. "Is it useful by itself?" maps to the composition law. Nothing new for data governance.

---

### Post-reading summary

loom.md's §5 failure test — "removing a sibling changes the meaning of another product's data" — partially answers my lifecycle question but does not fully close it.

What it answers: Filigree's finding records are semantically self-contained once posted. Removing Clarion or Wardline after ingest does not corrupt the triage state. The data remains coherent. This is the correct design and the doctrine correctly names it.

What it does not answer: what happens to finding identity when Clarion's translator changes its output across versions? If Clarion v0.1 maps Wardline rule `WD-001` to Filigree finding ID `F-1234` with metadata `{wardline_properties: {ruleId: "WD-001", ...}}`, and Clarion v0.2 changes the normalization logic so the same Wardline scan produces a different `scan_source` fingerprint or different metadata keys, does Filigree recognize the v0.2 finding as the same finding as the v0.1 finding? Or does it create a new record, leaving the old triage state orphaned?

The doctrine says Filigree owns triage lifecycle. The doctrine does not say what the deduplication key is for findings across translator versions. That is a schema design question that lives below the doctrine level — but it is precisely the question that determines whether SOC 2 audit trail continuity survives a Clarion version bump.

**Mood after reading:** The doctrine is well-constructed and the federation principle is the right model. My residual concern is not with the doctrine — it is with the gap between the doctrine's maturity and the v0.1 implementation's maturity. The doctrine says enrichment, not load-bearing. v0.1's SARIF translator is load-bearing. The briefing acknowledges this as temporary. I believe them. But "temporary" without a durability contract on the finding identity scheme is a compliance gap, not just a technical debt item.

# Reader Journal — p3-compliance-eng (Yasmin)
## Loom suite docs: briefing.md + loom.md
**Date:** 2026-04-17
**Persona:** Security / Compliance Engineer, SOC 2 Type II regulated SaaS

---

## Mood Journal

### briefing.md — The one-paragraph version

"Three independent tools that enrich one another through narrow additive protocols. Each is fully authoritative in its domain."

Good framing. "Fully authoritative in its domain" is a testable claim. I flagged it immediately to hold it against the data-flow section. The "without any shared runtime, store, or orchestrator" statement is the kind of assertion that is easy to put in a paragraph and hard to maintain in an implementation. I came in skeptical; the tools that claim no shared state tend to develop sync mechanisms that are shared stores with different branding.

### briefing.md — The product descriptions

Clarion is listed as authoritative for "structural / factual findings." Filigree is authoritative for "finding lifecycle, triage history." These two claims are adjacent and the boundary between them is unstated. My read: Clarion owns finding emission, Filigree owns finding lifecycle after ingest. That is a workable division, but it needs to be explicit — because the thing that sits between them is Clarion's SARIF translator, and that translator is currently the only path from Wardline to Filigree.

Wardline's fingerprint baseline is notable: a per-function fingerprint means there is a stable external identifier for each enforced function. That identifier needs to survive Clarion's translation intact. Whether it does is not stated here.

### briefing.md — Data-flow table (close read)

This is where I slowed down.

Row 1 (Wardline manifest → Clarion, file read at `clarion analyze`): no version handshake declared in the table. The briefing later acknowledges Wardline needs to provide "a stable `REGISTRY_VERSION`" — so the gap is known, but the table shows the happy path and the gap is footnoted elsewhere. That is an accurate representation of an incomplete integration, and I appreciate the honesty, but I note it.

Row 2 (Annotation vocabulary, direct import at plugin startup): Clarion's Python plugin takes a direct runtime dependency on Wardline's Python package. That is a tight coupling that the "each tool is independently usable" principle is supposed to prevent. The degraded-mode fallbacks acknowledge this; the table does not surface it. Tables that show only happy paths are a compliance documentation smell.

Row 4 (Wardline SARIF → Clarion translator → Filigree): this is the row I came for. Three steps, two transformations. My question at this row: what is the unit of identity that links the Filigree finding record back to the original Wardline SARIF finding? What does Clarion's translator preserve, and what does it normalize away?

The briefing's answer is in the identity section: "Filigree preserves the `metadata` dict verbatim, so Clarion's richer fields and Wardline's SARIF property-bag extensions survive ingest under namespaced keys (`metadata.clarion.*`, `metadata.wardline_properties.*`)." This is the most important sentence in the document for my purposes. Verbatim metadata preservation is the right design. If the full Wardline SARIF property bag is passed through, including the `ruleId` and location fields, then the lineage is traceable even if Filigree assigns its own primary key. The audit trail holds.

The qualifier that concerns me: "Wardline's SARIF property-bag extensions survive." Does "extensions" mean the full SARIF output including `ruleId`, `level`, and `physicalLocation`? Or does it mean only the non-standard fields in the property bag, with the standard SARIF top-level fields normalized into Clarion-native fields and potentially not preserved verbatim? If the translator normalizes SARIF `level` into a Clarion `severity` field and drops the original, then Filigree's record cannot be reconciled against the raw Wardline SARIF without going back through Clarion. That is a lineage gap.

### briefing.md — Identity and the shared vocabulary

"The suite has three concurrent identity schemes — Clarion EntityId, Wardline qualname, Wardline exception-register location string — Clarion maintains the translation layer; neither sibling tool takes on that responsibility."

Three identity schemes, one translator. Honest. Also: a significant single point of governance responsibility. If Clarion's translation layer changes in v0.2 — a revised EntityId minting algorithm, a changed qualname normalization rule — what happens to Filigree records created under v0.1? The Filigree primary key is Filigree-assigned and stable. The Clarion EntityId stored in cross-references may become stale. The briefing does not address this.

### briefing.md — Principles section

"Findings are facts, not just errors." This is the framing I want. A finding as an immutable fact about a point-in-time scan, with mutable lifecycle state layered on top. The compliance implication: suppressing a finding should not erase it — it should add a triage record (suppressed, by whom, when, why) while the underlying fact persists. The briefing implies this design; it does not state it explicitly. Whether Filigree implements findings as immutable facts with mutable state wrappers, or as mutable records where suppression overwrites state, matters enormously for audit trail durability.

The degraded-mode fallback (`--no-filigree`, Clarion writes findings to local JSONL) is good engineering and also a compliance gap in regulated environments. Local JSONL is not auditable in the same way as Filigree's tracked store. If Clarion is ever run without Filigree in a SOC 2 scope, those findings are outside the audit trail. This needs to be a documented operational constraint, not just a graceful-degradation footnote.

### loom.md — §1-3

§2 gives me the clean statement I needed: Filigree owns "work state and workflow lifecycle" and "finding triage state." That is clearer than anything in briefing.md. The federation principle in §3 names the stealth-monolith failure mode explicitly — "a lightweight glue layer that quietly becomes the real system of record." That is the failure mode I was watching for. Naming it does not prevent it, but it shows the authors know what they are defending against.

### loom.md — §4 (Composition law)

Product strategy. The three modes (solo, pair, suite) are a useful design gate. My annotation to myself: this is the design-time test, not the runtime test. A product can pass all three modes at design time and still produce a translation layer that becomes load-bearing at runtime. The composition law is necessary but not sufficient for the data-governance properties I care about.

### loom.md — §5 (Enrichment, not load-bearing) — slow read

This is the section I came for. The principle: "A sibling product may enrich another product's view, but it must never be required for that product's semantics to make sense." The failure test: "If removing a sibling product changes the *meaning* of another product's own data, Loom has centralised too far."

I worked the Wardline example carefully against this test.

The §5 concrete examples address the wrong axis. The Wardline example says: "Wardline enforces trust policy whether Filigree is ingesting findings or not." True — that is about Wardline's own independence. The Clarion example says: "Wardline's annotations enrich Clarion's entity metadata... but Clarion's structural truth is independent." Also true. But neither example addresses the thing I was testing: what happens to Filigree's finding records when the Clarion translator — currently the only conduit for Wardline→Filigree — changes or produces different output?

The §5 failure test partially answers my lifecycle question. It answers the static case: once a finding is in Filigree, removing Clarion does not change the meaning of the stored triage record. Filigree's authority over triage state is real and durable post-ingest. That is the correct design and the doctrine correctly names it.

What it does not answer is the dynamic case: when Clarion's translator changes its output across versions, does Filigree recognize a finding from the new version as the same finding as one from the old version? The deduplication key for findings — what it is, who defines it, whether it is stable across translator versions — is the question that determines whether SOC 2 audit trail continuity survives a Clarion upgrade. That question is below the doctrine level, but the doctrine's silence on it is a gap.

I also notice: the §5 principle states that Clarion's enrichment of Wardline data "must never be required for that product's semantics to make sense." But in v0.1, Clarion's translator is not enriching Wardline — it is *translating* Wardline output for ingest into Filigree. That is a different operation. Enrichment is additive annotation on top of existing semantics. Translation is a format conversion that determines what arrives in the target store. Clarion's SARIF translator is not enrichment in the §5 sense; it is a pipeline stage. The doctrine's enrichment principle does not directly govern it, and that is a gap in the doctrine's coverage.

### loom.md — §6 (What Loom is NOT)

"When cross-scheme translation is needed — the product that *cares* does the translation, because that product is the one whose authority needs it."

This is the doctrine's rationale for Clarion owning the translation layer. It is principled. My concern is not with the principle but with the versioning commitment: Clarion translates because it owns the catalog. When the catalog's entity ID scheme evolves, the translation changes. The doctrine does not address what obligations Clarion has to previously-emitted translations — whether old EntityIds remain valid, whether Filigree records created under old translations remain coherent. That is a schema-compatibility governance question, not answered here.

### loom.md — §7 (Go/no-go test)

Product strategy. The four gates are coherent and tighter than I expected. "Is the full suite better because of it, without making the others incomplete in its absence?" — this gate would have caught some of the integration-dependency creep I have seen in other suites. I note it approvingly and move on.

---

## Key Finding

The Loom doctrine is sound and the federation principle is the right model for keeping audit authority clear. What the docs collectively reveal, however, is a specific gap between doctrine and v0.1 implementation that is compliance-relevant: the SARIF translation path from Wardline through Clarion into Filigree is currently load-bearing in the sense the doctrine says integration must not be. In v0.1, Clarion's translator is the only conduit for Wardline findings reaching Filigree's triage store, and the briefing acknowledges this is temporary pending a native Wardline emitter. What neither document addresses is finding deduplication identity across translator versions — specifically, whether Filigree will recognize a finding emitted by Clarion v0.2's translator as the same finding as one emitted by Clarion v0.1's translator, or whether a version bump silently creates duplicate finding records and orphans existing triage state. This is not an architectural flaw in the doctrine; it is an unspecified contract in the implementation. In a SOC 2 environment, an unspecified deduplication key is a finding in its own right, because audit trail continuity depends on it.

---

## Unanswered Questions

**1. What is the stable deduplication key for a Wardline-sourced finding in Filigree, and is it guaranteed invariant across Clarion translator versions?**

The briefing says Filigree preserves `metadata` verbatim, including `metadata.wardline_properties.*`. But finding deduplication — the mechanism Filigree uses to recognize that a new scan result is the same finding as an existing one, not a new finding — depends on some fingerprint or key that must be stable across Clarion releases. If that key is Clarion-derived (based on EntityId, normalized rule ID, or translated location), a Clarion version bump can cause Filigree to create a new finding record rather than matching the old one. Old triage state is then orphaned with no signal to operators. What is that key, how is it defined, and who owns its stability contract?

**2. Does the metadata verbatim-preservation guarantee cover top-level SARIF fields (ruleId, level, physicalLocation) or only the SARIF property-bag extensions?**

The briefing states Wardline's "SARIF property-bag extensions survive ingest under namespaced keys." Property-bag extensions are non-standard fields. Standard SARIF fields like `ruleId`, `level`, and `locations[].physicalLocation.uri` are the primary audit anchors in a SARIF-consuming workflow. If Clarion's translator normalizes these into Clarion-native fields and does not also preserve them verbatim in `metadata.wardline_properties.*`, then reconciling a Filigree finding against raw Wardline SARIF output requires Clarion as an intermediary. That reintroduces a load-bearing dependency the doctrine explicitly prohibits.

**3. Are Filigree finding records immutable facts with mutable lifecycle state, or mutable records where state transitions overwrite prior state?**

The briefing asserts "findings are facts, not just errors," which implies an immutable-fact model: a finding is created once, and subsequent triage actions (acknowledge, suppress, fix) are recorded as state transitions on top of the immutable base record. Under SOC 2, this matters because audit trail completeness requires that a suppression decision not erase the original finding — it must be queryable as "this finding existed, was seen by this person, and was suppressed for this stated reason." If Filigree implements findings as mutable records where `status=suppressed` overwrites `status=open`, the audit trail is incomplete. The briefing does not confirm which model Filigree implements.

**4. What is the operational classification of Clarion's `--no-filigree` degraded mode from a compliance standpoint, and is it documented as out-of-scope for audit?**

When Clarion runs without Filigree and writes findings to local JSONL, those findings exist outside Filigree's tracked store and outside any audit trail that depends on Filigree's triage history. In a regulated environment, a scan that runs in degraded mode and emits findings to local JSONL is a compliance gap — the findings occurred, the triage record does not. The briefing mentions degraded mode as a graceful fallback. It does not address whether this mode is appropriate in SOC 2 scope, what controls prevent it from being used where Filigree-backed triage is required, or whether local JSONL output is considered an alternative audit artifact. This needs to be a documented operational constraint.

**5. When Clarion's EntityId minting algorithm changes between versions, what is the migration story for Filigree records that carry stale EntityId cross-references?**

The briefing establishes that Filigree issues reference entities by Clarion EntityId, and Wardline findings are reconciled to EntityIds at ingest. If Clarion v0.2 revises its EntityId scheme — changed namespace format, different entity boundary heuristics — Filigree records from v0.1 carry EntityIds that no longer resolve in the new catalog. The briefing notes that Filigree needs "a published schema-compatibility contract" as a prerequisite for Clarion shipping. But schema compatibility between Filigree's API and Clarion's POST format is a different thing from EntityId stability across Clarion versions. Who is responsible for the migration of existing cross-references when the entity catalog is re-keyed? The doctrine says Clarion maintains the translation layer, but it does not say Clarion is obligated to maintain backward compatibility on previously-emitted EntityIds.

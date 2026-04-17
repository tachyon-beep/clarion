# Journal Entry: briefing.md
## Step B — In-reading and post-reading reactions

**Date:** 2026-04-17

---

### The one-paragraph version

"Three independent tools that enrich one another through narrow additive protocols. Each is fully authoritative in its domain."

Good. "Fully authoritative in its domain" is exactly the framing I need. If Filigree is authoritative for finding lifecycle and triage history, that is a claim I can test against the data-flow section. I am flagging it now to hold them to it later.

Also: "without any shared runtime, store, or orchestrator." That is a strong statement. In my experience, tools that claim no shared state end up with a sync mechanism that *is* a shared store — it just lives in a flat file or a git repo and nobody called it that. I will keep watching.

---

### The product descriptions

**Clarion:** "Authoritative for: the entity catalog, the code graph, guidance sheets, and structural / factual findings." Fine. The word "findings" appearing in Clarion's domain already raises a flag — Filigree is also supposed to be authoritative for findings. I suspect these are different things (Clarion owns the *emission* of structural findings, Filigree owns the *lifecycle* of findings post-ingest). If that is the distinction, it should be stated explicitly. It is not.

**Filigree:** "Authoritative for: issue state, workflow transitions, observation and finding lifecycle, triage history." This is what I care about. Triage history is listed. Good. But "finding lifecycle" is a claim that will need to survive scrutiny when I read about Clarion's SARIF translator. If Clarion produces a finding, translates it, posts it to Filigree — and then Clarion updates its translator — does Filigree's lifecycle record for that finding still point at something stable? The domain claim is here; the durability guarantee is not.

**Wardline:** "Maintains a per-function fingerprint baseline so drift is visible." Fingerprint baseline is important. That means there is a stable external identifier per function — the qualname plus fingerprint. This is the thing that has to survive Clarion's translation layer. I am making a note.

**Shuttle:** Not my concern. Proposed product, no design. I am moving on.

---

### The interaction diagram

The ASCII diagram is fine as a visual summary. What I actually need is the data-flow table below it, so I am going there directly.

---

### Data-flow table — close read

This is where I slow down.

**Row 1: Declared topology — Wardline manifest/fingerprint files → Clarion catalog — File read at `clarion analyze`.**
File read. Not an API call. That means at the moment Clarion analyzes, it reads whatever version of the manifest and fingerprint files are present on disk. There is no version handshake, no schema contract declared here. The briefing later mentions Wardline needs to provide "a stable `REGISTRY_VERSION` that Clarion's plugin pins against" — so this gap is acknowledged elsewhere. But in the data-flow table, it looks clean when it is not yet clean.

**Row 2: Annotation vocabulary — `wardline.core.registry.REGISTRY` → Clarion's Python plugin — Direct import at plugin startup.**
Direct import. That means Clarion's Python plugin takes a runtime dependency on Wardline's Python package. That is a tight coupling that the "each tool is independently usable" principle is supposed to prevent. The briefing acknowledges this implicitly ("degraded-mode fallbacks") but it is not reflected in this table. The table shows the happy path only.

**Row 3: Findings — Clarion → Filigree — `POST /api/v1/scan-results` (Clarion-native schema).**
Clarion-native schema. What is the Clarion-native schema? Where is it defined? Is it versioned? The table does not say. This is the point where I would normally go look for a schema spec and find a JIRA ticket referencing a Google Doc that was last updated in 2022. I am noting the absence.

**Row 4: Findings (Wardline-sourced) — Wardline SARIF → Clarion translator → Filigree — `POST /api/v1/scan-results` via `clarion sarif import`.**

This is the row I came here for. Let me work through it.

Wardline emits SARIF. Clarion has a translator. The translator posts to Filigree. Three steps, two transformations, one place where triage state lives at the end (Filigree). My question: what is the unit of identity that links the Filigree finding record back to the original Wardline SARIF finding? After the translation, if I want to know "who triaged this Filigree finding and why," can I trace it back to the specific Wardline rule and location that produced it?

The briefing says: "Filigree preserves the `metadata` dict verbatim, so Clarion's richer fields and Wardline's SARIF property-bag extensions survive ingest under namespaced keys (`metadata.clarion.*`, `metadata.wardline_properties.*`)."

That is a meaningful statement. Verbatim preservation of metadata is the right design. If Wardline's SARIF property bag carries a stable rule ID and location, and Clarion's translator passes it through into `metadata.wardline_properties.*`, then the lineage is traceable through the metadata even if Filigree has assigned its own primary key. That is acceptable for audit purposes, as long as the metadata is queryable and indexed.

But "Clarion's richer fields" are listed as surviving ingest. What about the Wardline SARIF `ruleId`? Is that passed through? The sentence says "Wardline's SARIF property-bag extensions" survive — not necessarily the top-level SARIF fields like `ruleId`, `level`, `locations[].physicalLocation`. If the translator normalizes those into Clarion-native fields and drops the originals from the metadata, the lineage breaks.

**Row 5: Observations — Clarion consult mode → Filigree — MCP tool call (or HTTP once the endpoint ships).**
"Once the endpoint ships." This is a live integration gap. Noted.

**Row 6: Entity state — Clarion → Wardline (v0.2+).**
Future work. Not relevant to current audit posture.

**Row 7: Issue cross-references — Filigree → Clarion consult surface.**
Filigree references Clarion entity IDs. Fine. This is a read path, no lifecycle implications.

---

### Identity and the shared vocabulary

"The suite has three concurrent identity schemes (Clarion EntityId, Wardline qualname, Wardline exception-register location string) — Clarion maintains the translation layer; neither sibling tool takes on that responsibility."

Three identity schemes. Clarion owns the translation layer. This is honest and it is also a significant single point of responsibility. The briefing is explicit: Clarion maintains the translation. If Clarion's translation layer changes — say, the EntityId minting algorithm is revised in v0.2 — what happens to Filigree records that were created under the v0.1 translation? The stable IDs in Filigree are Filigree-assigned. The Clarion EntityIds stored in cross-references may become stale. The briefing does not address this.

What it *does* say: "Wardline findings carry qualnames that Clarion reconciles to entity IDs at ingest." So the flow is: Wardline uses qualnames, Clarion translates qualnames to EntityIds, EntityIds end up in Filigree issue cross-references. The Filigree finding record itself — what identifier does it carry? Filigree's own primary key, presumably. The Wardline qualname lives in `metadata.wardline_properties.*` if the translator passes it through. That is the chain. It is traceable but it is fragile at the Clarion translation step.

"Filigree preserves the `metadata` dict verbatim." This is the most important sentence in this document for my purposes. Verbatim preservation means the audit trail does not depend on Clarion's translator getting everything right — the raw source data survives in the metadata. That is a good design choice. But "verbatim" only holds if Clarion actually passes the full property bag rather than selectively extracting fields. The briefing claims it does; I have no way to verify from this document.

---

### Principles section

"Findings are facts, not just errors." Yes. This is the framing I want. A finding should be an immutable fact about a point-in-time scan, with mutable lifecycle state sitting on top. The briefing implies this but does not say it in those terms. The audit implication: if a finding is a fact, then suppressing it should not erase it — it should add a triage record that says "suppressed, by whom, when, why." Whether Filigree implements it that way, I do not know from this document.

"Each tool is independently useful." The degraded-mode fallbacks (`--no-filigree`, `--no-wardline`) are good engineering practice and good compliance practice. If Filigree is unavailable, Clarion writes findings to local JSONL. That means there is a findings-out-of-Filigree state that could exist without anyone noticing. Local JSONL is not auditable in the same way as Filigree's tracked store. This is a compliance gap when Clarion is run without Filigree in a regulated environment.

---

### Post-reading summary

The briefing is honest about gaps (missing Filigree endpoint, Wardline REGISTRY_VERSION not yet stable, SARIF translator as a temporary bridge). That honesty is points in its favor.

The critical finding for me: the SARIF translation path (Row 4) is where finding identity is most at risk. The metadata verbatim-preservation claim is the load-bearing safety net. If that claim holds in implementation, the audit trail is traceable. If Clarion's translator selectively extracts rather than passes through, or if the SARIF top-level fields are dropped rather than namespaced into metadata, the lineage breaks silently.

The secondary concern: three identity schemes with Clarion as sole translator is a governance risk. It is not a problem today (Clarion is not yet built). It will become a problem when Clarion ships v0.2 and the EntityId scheme evolves.

**Mood after reading:** More engaged than I expected. The suite has clearly thought about findings as a first-class concept. The concerns I have are real but they are the right kind of concerns — they are about implementation fidelity to stated design principles, not about fundamental design flaws. I want to read loom.md now, specifically to understand the enrichment model and whether it has anything to say about translation fidelity guarantees.

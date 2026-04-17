# Step A — Pre-reading Expectations: briefing.md

**Document:** docs/suite/briefing.md
**Persona:** Priya (p1-infra-skeptic)

## What I expect to find

A briefing doc for a suite of tools that claims to be federated/modular. Likely:

- High-level product overview with names that sound like a metaphor family
  ("Loom", "Shuttle", etc. — already know that from context)
- Claims that tools work independently, integrate optionally
- A diagram showing how tools "connect" that quietly reveals shared dependencies
- Language about "enrichment" or "composition" that I'll need to interrogate
- Probably a table of products with status/maturity markers
- Possibly a vision/mission paragraph I'll skip

## What would make me update my skepticism downward

- A concrete NOT-list: things the suite explicitly will not do, with structural reasons
- Explicit statement of what happens when one tool is absent
- Boundaries described in terms of data formats and protocols, not just intent

## What would confirm my worst fears

- Any mention of a "fabric", "bus", "shared config", or "capability negotiation"
- Products described only in terms of each other, never standalone
- "Enrichment" used as if it explains the coupling rather than names it

---

# Step B — In-reading and Post-reading Reactions: briefing.md

## One-paragraph version

Skipped the audience/purpose header. Went straight to the one-paragraph version
because that's usually where the load-bearing claims live before the author has
had a chance to hedge them properly.

"three independent tools that enrich one another through narrow additive protocols"

Two reads of that. "Enrich" is doing work here. "Narrow additive protocols" —
fine, I'm willing to believe that until the diagram says otherwise. "Independent"
is a claim I'll test against the data-flow table.

"without any shared runtime, store, or orchestrator"

OK. That's a direct claim. Three specific things excluded. I'm noting that. If
the diagram or the data-flow table contradicts any of these three I'm flagging it.

## Product descriptions — skimming for coupling signals

Clarion: "extracts entities... produces structured briefings." Solo story makes
sense. "Consult-mode LLM agents query Clarion through MCP tools" — so it's a
server. That's fine.

Filigree: Already built, in active use. This is the one I know. Issues, findings,
observations. Has an MCP server and a dashboard. Fine.

Wardline: Scanner, pre-commit hook, SARIF output. That's a tight, well-bounded
tool. I trust this description more than I trust the others because SARIF output
is a concrete, industry-standard artifact. No coupling mystery there.

Shuttle: Proposed, no design. I'm not evaluating something that doesn't exist.
Skipping.

## The fabric diagram

First thing I did was read the diagram. Then I read it again.

Clarion posts findings to Filigree via `POST /api/v1/scan-results`. Fine — that's
a push over HTTP. If Filigree is down, Clarion either fails or queues or writes
locally. The briefing says "writes findings to local JSONL" in degraded mode —
that answers part of my question. Marks up in my book.

Wardline feeds files into Clarion at `clarion analyze` time. File-level dependency,
not runtime. Fine.

Clarion feeds observations to Filigree via MCP tool call. That's a runtime
dependency. If Filigree is down during a consult session, the MCP call fails.
I don't see what happens then. Does Clarion buffer? Drop? Surface an error to
the calling agent? "Or HTTP once the endpoint ships" — so the MCP path is the
current path and it's not yet documented whether it's fire-and-forget or
blocking. That's a gap.

Filigree issue cross-references go back into Clarion's consult surface. So
Clarion reads from Filigree at serve time. Two-way HTTP dependency in production
use. The diagram shows it; the text doesn't call it out as a coupling risk. I'm
noting that.

## Data-flow table — line by line

Row 1: Wardline -> Clarion via file read at analyze time. Fine. Offline-composable.

Row 2: `wardline.core.registry.REGISTRY` -> Clarion's Python plugin via "direct
import at plugin startup." 

Stop. Read that again.

Direct import. That means Clarion's Python plugin has Wardline as a Python
dependency. Not a file read. Not an HTTP call. A Python import. That is a
compile-time (or at minimum, startup-time) code-level dependency. If Wardline
changes its registry API, Clarion's plugin breaks at startup. This is not the
same class of coupling as the other rows. The doc is listing it in the same
table as file reads and HTTP calls, but it's structurally different. This is the
row that most concerns me in the entire briefing.

Row 3: Clarion -> Filigree via POST. Already noted. Degraded mode (local JSONL)
exists. OK.

Row 4: Wardline SARIF -> Clarion translator -> Filigree via POST. Clarion owns a
SARIF translator for Wardline's output. That's a format dependency — Clarion
needs to know Wardline's SARIF schema. As long as SARIF is versioned and stable,
manageable. But Clarion is "the translator" here, meaning if Wardline changes its
SARIF property-bag extensions, Clarion has to absorb the change. That's a
maintenance surface that lives inside Clarion.

Row 5: Clarion observations -> Filigree via MCP. Already flagged.

Row 6: Clarion -> Wardline (v0.2+) via Clarion HTTP read API. Deferred. Fine for
now — good that it's not current scope.

Row 7: Filigree -> Clarion consult via Filigree read API. Two-way HTTP at serve
time. Already flagged.

## Identity and shared vocabulary

"Clarion maintains the translation layer; neither sibling tool takes on that
responsibility."

Three concurrent identity schemes. Clarion owns the reconciliation. That makes
Clarion a de-facto shared registry. Not a shared *store* — the store is local
files — but a shared *authority*. If Clarion is unavailable, cross-tool
cross-references break. Whether that matters depends on whether you're in an
offline workflow or a live consult session.

The `metadata` verbatim-preservation trick is smart. Namespaced keys under
`metadata.clarion.*` and `metadata.wardline_properties.*` means Filigree doesn't
need to understand those fields. Good decoupling on Filigree's side.

## Four principles

"Each tool is independently useful" — tested against what I've read:

- Clarion without Filigree: writes findings to local JSONL. Yes, that works.
- Wardline without Clarion: has since day one. Believable given SARIF output.
- Filigree without either: it's a tracker, of course it works alone.

That's actually a clean story. I'm willing to accept principle 3 as structurally
supported, not just aspirational — *except* for the Python import dependency on
Wardline's registry. That's the crack.

"Local-first, single-binary, git-committable state." — this is the principle I
care most about. State files committed to git. No hosted service required. This
is the operational property that would actually make me recommend the suite.
The degraded-mode fallbacks support this. OK, this is a real commitment, not
marketing.

## What the suite needs from Filigree and Wardline for Clarion to ship

This section is more honest than most briefing docs I've read. It names specific
inter-tool asks:

- Filigree needs a pluggable `registry_backend` — so currently Filigree's file
  registry is not pluggable. Clarion can't own it yet. That's a real gap.
- Filigree needs an HTTP endpoint for observation creation — so the current MCP
  path is the only path. That's a runtime dependency on MCP being up.
- Wardline needs a `REGISTRY_VERSION` stable pin and legacy aliases.

Good. This is the kind of honest gap-listing I wanted. It means the "independent"
claim has asterisks on it *in v0.1*, and the doc is saying so. I still think the
direct-import dependency is understated, but the general posture here is honest.

## Current state table

Filigree: built. Wardline: built. Clarion: designed only. Shuttle: proposed.

"Clarion v0.1 delivers the cross-tool protocols that Filigree and Wardline don't
yet speak."

This sentence is the one the author probably wrote without noticing what it says.
Clarion — the not-yet-built tool — is also the tool that bootstraps the
integration fabric. Which means until Clarion ships, the "suite" is two tools
(Filigree and Wardline) that don't talk to each other, plus a bunch of design
docs. That's fine — that's honest — but it also means the enrichment
architecture has never been exercised end-to-end. Every coupling claim in this
doc is theoretical. Good theoretical design, but untested.

## Post-reading summary

Better than I expected. The data-flow table is specific. The degraded-mode
story is real. The principle that most concerns me (direct import of Wardline's
REGISTRY) is present in the table but not flagged as a different class of
coupling than the others. The two-way Clarion/Filigree HTTP dependency at serve
time is present in the diagram but not called out as a coupling risk anywhere in
the text. The honest gap list at the end redeems some of the earlier smoothness.

The word "fabric" appears. I predicted that. The word "enrichment" appears in the
opening paragraph. I re-read it twice. It's used to name a relationship category,
not to explain it — there's a table that does the explaining. That's better than
I expected.

Net position: cautious interest, not dismissal. But I want to read loom.md to
see if the doctrine provides structural enforcement for these claims or just more
vocabulary.

# Reader Journal — Priya (p1-infra-skeptic)

**Persona:** Senior Platform / Infrastructure Engineer
**Reading order:** briefing.md, then loom.md
**Date:** 2026-04-17

---

## Mood Journal

### briefing.md — One-paragraph version

Skipped the audience/purpose header. Went straight to the opening paragraph
because that's where load-bearing claims live before the author has had a chance
to hedge them properly.

"three independent tools that enrich one another through narrow additive protocols"

Two reads of that. "Enrich" is doing work here. "Narrow additive protocols" —
fine, I'm willing to believe that until the diagram contradicts it. "Independent"
is a claim I'll test against the data-flow table.

"without any shared runtime, store, or orchestrator"

That's a direct exclusion list. Three specific things. I'm noting them. If the
diagram or table contradicts any of the three, I'm flagging it immediately.

### briefing.md — Product descriptions

Clarion: entities, clusters, briefings, MCP server. Solo story makes sense.

Filigree: already built, in active use. I know this one. Fine.

Wardline: scanner, pre-commit hook, SARIF output. SARIF is a concrete
industry-standard artifact. This description is more trustworthy than the
others precisely because it names a real format with a known schema. I trust it.

Shuttle: proposed, no design. Not evaluating. Skipping.

### briefing.md — The fabric diagram

Read it. Then read it again.

Clarion posts findings to Filigree via `POST /api/v1/scan-results`. HTTP push.
If Filigree is down, Clarion writes to local JSONL in degraded mode — the briefing
says so. That answers the failure question. Marks up.

Wardline feeds files to Clarion at `clarion analyze` time. File-level dependency,
not runtime. Fine. Offline-composable.

Clarion feeds observations to Filigree via MCP tool call. Runtime dependency.
If Filigree is down during a consult session, the MCP call fails. Does Clarion
buffer, drop, or surface an error? "Or HTTP once the endpoint ships" — so the MCP
path is the current path and whether it's fire-and-forget or blocking is not
documented. Gap.

Filigree issue cross-references feed back into Clarion's consult surface via
Filigree read API. Two-way HTTP dependency at serve time. The diagram shows it.
The text never flags it as a coupling risk. I'm noting it.

### briefing.md — Data-flow table, line by line

Row 1: Wardline files -> Clarion at analyze time. Offline-composable. Fine.

Row 2: `wardline.core.registry.REGISTRY` -> Clarion's Python plugin via direct
import at plugin startup.

Stop. Read that again.

Direct import. Not a file read. Not an HTTP call. A Python import. That is a
code-level, startup-time dependency. If Wardline changes its registry API,
Clarion's plugin breaks at startup — not at some API call, at *startup*. This is
a categorically different class of coupling from every other row in this table,
and the table presents it in the same visual register as "file read at analyze
time." That is not an honest presentation of the coupling surface. This is the
row that concerns me most in the entire document.

Row 3: Clarion -> Filigree via POST. Degraded mode exists. Already noted. OK.

Row 4: Wardline SARIF -> Clarion translator -> Filigree. Clarion owns a SARIF
translator for Wardline's output. Format dependency: if Wardline changes its SARIF
property-bag extensions, Clarion absorbs the change. That's a maintenance surface
living inside Clarion. Manageable if SARIF versioning is rigorous. Not currently
described as such.

Row 5: Clarion observations -> Filigree via MCP. Already flagged.

Row 6: Clarion -> Wardline v0.2+. Deferred. Fine.

Row 7: Filigree -> Clarion consult via Filigree read API. Two-way HTTP at serve
time. Already flagged.

### briefing.md — Identity and shared vocabulary

Three concurrent identity schemes. Clarion owns the reconciliation. That makes
Clarion a de-facto shared identity authority. Not a shared store — the store is
local files — but a shared authority. Cross-tool cross-references require Clarion
to be present and consistent. Whether that matters operationally depends on whether
you're offline or in a live consult session, but the dependency is real.

The `metadata` verbatim-preservation trick is smart. Namespaced keys under
`metadata.clarion.*` and `metadata.wardline_properties.*` means Filigree doesn't
need to parse those fields. Good decoupling on Filigree's side.

### briefing.md — Four principles

"Each tool is independently useful" — I tested this against the table:

- Clarion without Filigree: local JSONL. Yes, that works.
- Wardline without Clarion: has since day one. Believable.
- Filigree without either: it's a tracker. Of course it does.

Clean story — except for the Python import dependency on Wardline's REGISTRY.
That's the crack in the independence claim. Clarion's Python plugin can't start
without Wardline installed.

"Local-first, single-binary, git-committable state" — this is the principle I
care most about. State files in git. No hosted service required. The degraded-mode
fallbacks support it. This is a real operational commitment.

### briefing.md — Prerequisites for Clarion to ship

More honest than most briefing docs I've read. Specific asks named:

- Filigree needs a pluggable `registry_backend` — so currently it isn't pluggable.
  Clarion can't own the file registry yet. Real gap.
- Filigree needs an HTTP endpoint for observation creation — so MCP is the only
  current path. Runtime dependency on MCP.
- Wardline needs a stable `REGISTRY_VERSION` pin and legacy aliases.

This kind of gap-listing is rare. The independence claim has acknowledged asterisks
in v0.1. The direct-import dependency is still understated, but the posture is honest.

### briefing.md — Current state table and the fabric sentence

"Clarion v0.1 delivers the cross-tool protocols that Filigree and Wardline don't
yet speak."

The author probably wrote that sentence without noticing what it says. Clarion —
the not-yet-built tool — is also the tool that bootstraps the integration fabric.
Which means until Clarion ships, the "suite" is two tools that don't talk to each
other, plus design docs. Fine — that's honest — but it also means the enrichment
architecture has never been exercised end-to-end. Every coupling claim in the
briefing is theoretical. Good theory, untested reality.

Net position after briefing.md: cautious interest. Better than I expected on
specificity. Worse than I'd like on the direct-import row. I want loom.md to
tell me whether "enrichment not load-bearing" is enforced structurally or is
just vocabulary.

---

### loom.md — §1 What Loom is

"each fully authoritative in its domain and fully usable on its own. When composed,
they enrich one another through narrow, additive protocols"

Second read of "enrich." Same sentence structure as the briefing. Still a claim,
not a mechanism. But at least §5 promises a concrete test — I'll get there.

"There is nothing called 'Loom' to install, deploy, or keep running."

OK. That's the strongest possible statement of the non-platform position. I'm
reading it as a constraint the author is trying to commit to, not just a current
state description. I'll hold them to it.

### loom.md — §2 Authoritative domains

Each product gets a one-sentence authority statement and a one-sentence
scope description. Fine. What I'm watching for is any authority that overlaps.

Wardline: "its 'configuration' is the source code itself plus the adjacent
declarations — it does not have a separate authoritative config store."

That's a useful clarification. Wardline doesn't have a config file that Clarion
or Filigree might need to read. Good.

Clarion owns the entity catalog. Filigree owns issues and finding triage state.
Wardline owns trust declarations and baselines. These don't overlap on paper.
The identity reconciliation (Wardline qualname -> Clarion entity ID) is where
the authority edges blur — but §6 addresses that directly. Moving on.

### loom.md — §3 Federation, not monolith

"No Loom product may require the full suite to justify its existence."

Good. Hard rule.

"The rule protects against the stealth-monolith failure mode: a 'lightweight glue
layer' that quietly becomes the real system of record, reducing sibling products
to thin clients and making solo mode dishonest."

They named the pattern I've lived through. That earns some credibility. The
question is whether naming it protects against it. In my experience, it doesn't —
not automatically. What protects against it is structural enforcement: separate
release trains, separate deploys, tests that exercise each product in isolation.
Does Loom have those? The doc doesn't say.

### loom.md — §4 Composition law

Three modes: solo, pair, suite. Suite mode must never be mandatory for basic
usefulness. "Pairwise composability is a hard rule, not an aspiration."

I actually like this framing. Pairwise is a more tractable test than "fully
independent." You can verify it product-by-product. Let me run it mentally against
the briefing's data-flow table:

- Clarion + Filigree: Clarion pushes findings, Filigree stores them. Sensible pair.
- Clarion + Wardline: Wardline files enrich Clarion's catalog. Sensible pair.
- Filigree + Wardline: Wardline SARIF -> (Clarion translator) -> Filigree.

Wait. The Filigree + Wardline pair currently requires Clarion as a translator.
The briefing says Wardline findings go through "Clarion sarif import" before
reaching Filigree. So in v0.1, the Wardline + Filigree pair isn't actually a
pair — it's a triangle with Clarion as required middleware. The briefing says
Wardline will "eventually" get a native emitter to Filigree so Clarion's SARIF
translator can retire. "Eventually" is not v0.1.

This is a real violation of §4's pairwise composability rule, and neither doc
calls it out explicitly. The briefing buries it in a subordinate clause under
"What the suite needs." loom.md §4 states the rule without testing it against
current reality.

### loom.md — §5 Enrichment, not load-bearing

"A sibling product may enrich another product's view, but it must never be required
for that product's semantics to make sense."

Two reads. This is the sentence the author calls the load-bearing sentence of the
whole doc. Let me test it against the data-flow table from the briefing.

The failure test: "If removing a sibling product changes the *meaning* of another
product's own data, Loom has centralised too far."

Applying to the direct-import row: `wardline.core.registry.REGISTRY` imported
directly by Clarion's Python plugin at startup. If Wardline is absent, Clarion's
plugin doesn't start. Does that change the *meaning* of Clarion's data? No —
but it means Clarion has no data, because it couldn't initialize. The failure
test as written is about data semantics, not about availability. A strict reading
says the test is not violated because Clarion's data is coherent in Wardline's
absence — there just isn't any Clarion data. A practical reading says a plugin
that can't start without its sibling's package installed is a load-bearing
dependency by any operational definition.

The doc's failure test is too narrow. It catches semantic drift but not
initialization coupling. The direct-import row passes the stated test while
still being a form of coupling that would break in a deployment where Wardline
isn't on the same Python path.

The concrete examples in §5 are good. They're specific. The Clarion example:
"Wardline's annotations enrich Clarion's entity metadata with trust-tier and
policy-semantic information, but Clarion's structural truth is independent of
Wardline's policy truth." True at the data level. Not true at the startup level
for the Python plugin.

### loom.md — §6 What Loom is NOT

This is the NOT-list I wanted. Going through it:

"No shared runtime or daemon." — consistent with what I've seen. No `loomd`.

"Each product configures its own integrations in its own config. Clarion's config
names Filigree's endpoint directly; there is no central registry that everyone
consults." — fine. Bilateral configuration.

"No central store or database." — consistent.

"No system of record for cross-product state." — authority is distributed per
product. Consistent.

"No identity reconciliation service." — "the product that *cares* does the
translation." This is the right principle. Clarion translates Wardline qualnames
because Clarion owns the catalog. Fine in theory. In practice it means Clarion
is doing translation work that Wardline could do if it emitted Clarion-format IDs
natively. The briefing describes this as a v0.2+ goal. For now Clarion carries
the translation tax.

"No capability negotiation bus." — "Products probe each other directly via their
own surfaces. Version skew is handled bilaterally." Fine. But this means version
skew is handled N*(N-1)/2 times, once per pair. At three products that's three
bilateral contracts. Manageable. At six products it gets ugly. Noted for future
reference, not a v0.1 problem.

### loom.md — §7 Go/no-go test

Four questions. They're good questions. The right questions. I'd add a fifth:
"Does it introduce a new class-level import dependency on an existing member
product's internals?" — but the doc doesn't have that one.

Question 2: "Is it useful by itself?" — this is the test that should catch
features masquerading as products. Good.

Question 4: "Is the full suite better because of it, without making the others
incomplete in its absence?" — this is the enrichment-not-load-bearing principle
restated as a gate. Consistent.

The go/no-go test is a useful governance artifact. I'd want to see it applied
retrospectively to Clarion's role as SARIF translator for Wardline-to-Filigree.
Under question 3 ("does it form a sensible story with each existing product
one-to-one?"), Wardline + Filigree currently require Clarion as middleware.
That's a question-3 failure.

### loom.md — §9 Status table

Matches the briefing. No surprises.

"This charter is expected to outlive v0.1 and shape all subsequent product gates.
Its load-bearing sentence is in §5: enrichment, not load-bearing. If that
principle is ever compromised, the rest collapses."

I agree with the author that §5 is the load-bearing sentence. I disagree that
the current design fully satisfies it. The Python import dependency and the
Clarion-as-SARIF-translator role are two places where load-bearing coupling
already exists in v0.1 scope. The principle is stated cleanly. Whether it's
enforced is a different question.

---

## Key Finding

The author has correctly identified the failure mode they're trying to prevent —
they named the stealth-monolith pattern explicitly in §3 — and has written a
doctrine that would, if followed, prevent it. The problem is that the doctrine's
failure test (§5) is scoped to data semantics and misses initialization-time and
middleware-position coupling. Specifically: the direct Python import of
`wardline.core.registry.REGISTRY` into Clarion's plugin is a startup-time
code-level dependency that would not survive a deployment where Wardline is
absent from the Python path, yet it passes the stated failure test because
Clarion's data remains semantically coherent when Wardline is absent — there's
just no data. Separately, the Wardline-to-Filigree data flow currently requires
Clarion as a translator, making the Wardline + Filigree pair a de-facto triangle
in v0.1 scope — a violation of the pairwise composability rule that neither doc
flags. Neither of these problems is fatal to the architecture, but the author
appears unaware that the doctrine they wrote does not cover them. Someone who
reads only the doctrine and not the data-flow table would believe the design
fully satisfies the principle; it doesn't yet.

---

## Unanswered Questions

1. **What happens when Clarion's Python plugin starts without Wardline on the
   Python path?** The briefing says Wardline's `REGISTRY` is a direct import at
   plugin startup. Does the plugin fail hard, emit a warning and degrade, or
   silently omit trust-tier metadata? The degraded-mode story exists for HTTP
   dependencies but is not described for this code-level dependency.

2. **How does the Wardline + Filigree pair work in v0.1 without Clarion as
   middleware?** The data-flow table shows Wardline SARIF going through
   `clarion sarif import` before reaching Filigree. Can a team running Filigree
   and Wardline today, before Clarion ships, get Wardline findings into Filigree
   without standing up a partial Clarion installation? If not, the pair is
   actually a triangle and the pairwise composability claim fails for this
   combination.

3. **What is the versioning contract on `wardline.core.registry.REGISTRY`?**
   The briefing asks Wardline to provide a `REGISTRY_VERSION` and legacy aliases,
   but doesn't say whether this is a pre-ship gate or a best-effort ask. If
   Wardline ships a REGISTRY change before Clarion pins to it, Clarion's plugin
   breaks silently at the import line. Is this tracked as a hard dependency
   between release trains?

4. **Is the enrichment-not-load-bearing principle tested in CI, or only in
   design review?** The doctrine says the principle is the load-bearing sentence
   of the whole charter. But I see no mention of isolation tests — tests that
   exercise each product with siblings explicitly absent. Without those, the
   principle is enforced by human review at design time, which is the weakest
   possible enforcement. By v0.2, when pressure to add features accelerates, will
   a reviewer catch a new load-bearing integration the same way they'd catch a
   failing test?

5. **What is Clarion's behavior when Filigree is unreachable during a consult
   session and an observation needs to be emitted?** The briefing mentions MCP
   as the current path and "HTTP once the endpoint ships" as the future path, but
   neither path has a documented failure mode for observation emission. Does the
   observation get dropped? Buffered locally? Does it surface as an error to the
   calling agent? For a tool that claims local-first operation, a silent drop
   during a live consult session would be a meaningful gap in the enrichment
   story.

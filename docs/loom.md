# Loom

**Audience**: anyone designing, extending, or evaluating whether a new product belongs in the Loom family
**Purpose**: establishes the strategic direction, composition law, and go/no-go test that govern Loom as a suite
**Companion**: `suite-briefing.md` for an introductory 5-minute read

---

## 1. What Loom is

Loom is a three-product suite for code governance and assurance, with a proposed fourth product. Its first tools are **Clarion**, **Filigree**, and **Wardline**, each of which can operate independently, but together form a single operating fabric. The fourth product, **Shuttle**, is proposed and not yet in flight.

The metaphor is deliberate: distinct threads stay distinct but gain value by being woven together. Loom is not a platform that subsumes its constituent tools — it is the coordinated federation they compose into.

## 2. The products and their authoritative domains

Each Loom product is authoritative for exactly one bounded concern:

- **Clarion** — structural truth about the codebase. Answers "what is this codebase and where should I touch?"
- **Filigree** — work state and workflow lifecycle. Answers "what work exists, what state is it in, and what happened?"
- **Wardline** — trust policy and rule enforcement. Answers "what is allowed, and does this still satisfy the declared constraints?"
- **Shuttle** *(proposed)* — transactional scoped change execution. Answers "carry this approved change through the weave, under guard rails."

Shuttle's scope is deliberately narrow: it receives a scoped change intent, binds it to files or entities, orders the edits, applies them incrementally with pre- and post-change checks, rolls back on failure, and lints / commits / emits telemetry on success. It does not plan, triage, or reason about the code it is editing.

## 3. Federation, not monolith

**Loom is a federation, not a monolith. Each member product is authoritative in one bounded domain. Integration must be additive, not compulsory. No Loom product may require the full suite to justify its existence.**

This is the founding architectural law. It protects against the failure mode where tools are nominally separate but only deliver value when deployed together — hidden lock-in disguised as modularity.

## 4. The composition law

Any Loom product must satisfy all three modes:

- **Solo mode** — the product has a complete, respectable use-case by itself
- **Pair mode** — combined with any one sibling, it creates a meaningful capability, not a broken fragment
- **Suite mode** — all together form something richer, but suite mode must never be mandatory for basic usefulness

Pairwise composability is a hard rule, not an aspiration. A product that only works when all siblings are present is a feature of a monolith wearing modular clothing.

## 5. The go/no-go test for future products

Before adopting any new product into Loom, it must pass all four:

1. **Is it authoritative for one narrowly bounded thing?** — if the scope is two or more things, it is two or more products.
2. **Is it useful by itself?** — if siblings are required for minimum utility, it is a feature or adapter, not a product.
3. **Does it form a sensible story with each existing product one-to-one?** — every pairing must yield a coherent workflow; no "this only matters when you also have X and Y" patterns.
4. **Is the full suite better because of it, without making the others incomplete in its absence?** — addition, not patching.

If the answer to any question is no, the candidate is a feature, a protocol, or an adapter — not a product. It may still belong in Loom's surface area, but not as a named member.

## 6. What Loom config owns

Loom config is the shared discovery and identity layer. It owns:

- **Discovery** — how products find one another on a given host
- **Identity mapping** — canonical IDs and translation across product-specific naming schemes
- **Endpoint wiring** — where to reach sibling HTTP / MCP / protocol surfaces
- **Capability advertisement** — what each installed product reports it can do, so siblings can degrade gracefully when a capability is absent

Loom config explicitly does **not** own:

- Shared business logic
- Central orchestration
- Cross-product workflow state
- Any form of "Loom daemon" or mandatory runtime

The moment Loom becomes a mandatory brain, the composition law collapses. Federation survives only by refusing to centralise.

## 7. Naming

Member products are named from weaving mechanics — Clarion, Filigree, Wardline, Shuttle — as distinct proper names rather than subdivisions. There is no "Loom Guard," "Loom Workflow," or "Loom Execute"; each product earns its own identity. The family name sits above the products without dominating them.

## 8. Status

| Product | Status |
|---|---|
| Clarion | Designed; implementation not yet started; first-customer target is elspeth (~425k LOC Python) |
| Filigree | Built; in active use |
| Wardline | Built; in active commit-cadence use |
| Shuttle | Proposed; not in flight; separate design effort when prioritised |

The v0.1 Loom suite is Clarion + Filigree + Wardline. Shuttle enters the suite only when it passes the go/no-go test above and has its own spec, design, and validating customer.

This charter is expected to outlive v0.1 and shape all subsequent product gates.

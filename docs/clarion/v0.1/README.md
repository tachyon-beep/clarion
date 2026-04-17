# Clarion v0.1 Docset

This folder is the canonical Clarion v0.1 document set.

## Canonical design docs

1. [requirements.md](./requirements.md) — the *what*: requirements, constraints, and non-goals.
2. [system-design.md](./system-design.md) — the *how*: architecture, mechanisms, and integration posture.
3. [detailed-design.md](./detailed-design.md) — implementation detail, exact schemas, rule catalogs, and appendices.

## Supporting docs

- [reviews/README.md](./reviews/README.md) — retained historical reviews and panel outputs that shaped the current docset.
- [plans/v0.1-scope-commitments.md](./plans/v0.1-scope-commitments.md) — dated scope and decision memo for the v0.1 design freeze.
- [../adr/README.md](../adr/README.md) — authored ADRs and remaining decision backlog.

## Reading order

- New reader: [../../suite/briefing.md](../../suite/briefing.md) -> [../../suite/loom.md](../../suite/loom.md) -> [requirements.md](./requirements.md) -> [system-design.md](./system-design.md)
- Design reviewer (evaluating completeness, not yet implementing): new-reader path, then [detailed-design.md](./detailed-design.md) and [../adr/README.md](../adr/README.md).
- Implementation work: [requirements.md](./requirements.md) -> [system-design.md](./system-design.md) -> [detailed-design.md](./detailed-design.md) -> [../adr/README.md](../adr/README.md)

## Document roles

- `requirements.md`, `system-design.md`, and `detailed-design.md` are the authoritative layered design set.
- `reviews/` and `plans/` are supporting context, not normative sources.

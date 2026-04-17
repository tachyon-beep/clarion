# Clarion v0.1 Docset

This folder is the canonical Clarion v0.1 document set.

## Canonical design docs

1. [requirements.md](./requirements.md) — the *what*: requirements, constraints, and non-goals.
2. [system-design.md](./system-design.md) — the *how*: architecture, mechanisms, and integration posture.
3. [detailed-design.md](./detailed-design.md) — implementation detail, exact schemas, rule catalogs, and appendices.

## Supporting docs

- [reviews/design-review.md](./reviews/design-review.md) — design review of the pre-restructure single-file design.
- [reviews/integration-recon.md](./reviews/integration-recon.md) — integration reality check against Filigree and Wardline.
- [plans/v0.1-scope-commitments.md](./plans/v0.1-scope-commitments.md) — pre-implementation scope decisions (Q1/Q2/Q3) and the ADR writing sprint that follows.
- [plans/post-commitment-work-brief.md](./plans/post-commitment-work-brief.md) — self-contained handoff brief for the post-commitment rework (ADR sprint, auth flip, cost-model spike).
- [plans/docs-restructure-plan.md](./plans/docs-restructure-plan.md) — historical plan that produced this folder structure.
- [../adr/README.md](../adr/README.md) — authored ADRs and remaining decision backlog.

## Reading order

- New reader: [../../suite/briefing.md](../../suite/briefing.md) -> [../../suite/loom.md](../../suite/loom.md) -> [requirements.md](./requirements.md) -> [system-design.md](./system-design.md)
- Design reviewer (evaluating completeness, not yet implementing): new-reader path, then [detailed-design.md](./detailed-design.md) and [../adr/README.md](../adr/README.md).
- Implementation work: [requirements.md](./requirements.md) -> [system-design.md](./system-design.md) -> [detailed-design.md](./detailed-design.md) -> [../adr/README.md](../adr/README.md)

## Document roles

- `requirements.md`, `system-design.md`, and `detailed-design.md` are the authoritative layered design set.
- `reviews/` and `plans/` are supporting context, not normative sources.

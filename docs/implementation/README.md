# Implementation Plans

This folder is the canonical home for Clarion implementation planning at the work-package
level. It sits one level above any product-version docset because some work packages
(WP9, WP10) span Clarion **and** sibling Loom products (Filigree, Wardline) and therefore
do not belong inside `docs/clarion/v0.1/`.

## Contents

- [v0.1-plan.md](./v0.1-plan.md) — high-level implementation plan for Clarion v0.1: 11
  work packages, dependency order, anchoring docs/ADRs per package, exit criteria, and
  the post-implementation cost-model validation phase.

## Relationship to other docs

- **Scope and commitments**: [`../clarion/v0.1/plans/v0.1-scope-commitments.md`](../clarion/v0.1/plans/v0.1-scope-commitments.md).
  That memo locks *what* v0.1 ships; this plan describes *how* the build proceeds.
- **Authoritative design**: [`../clarion/v0.1/system-design.md`](../clarion/v0.1/system-design.md)
  and [`../clarion/v0.1/detailed-design.md`](../clarion/v0.1/detailed-design.md).
  Each work package below names the sections it implements.
- **Decisions**: [`../clarion/adr/README.md`](../clarion/adr/README.md). Each work package
  names the accepted ADRs it depends on and any backlog ADRs it is expected to surface.

## Out of scope for this folder

- Step-by-step task breakdowns (TDD-grain). Those belong in per-WP execution plans
  written when each package is picked up — not bundled into the high-level plan.
- Filigree issue records. The work-package list here is the source for seeding Filigree
  issues, but the issue tracker (not this doc) is the authoritative state-of-work.

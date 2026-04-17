# Suite Orientation — Priya (p1-infra-skeptic)

**Date:** 2026-04-17
**Persona:** Senior Platform/Infrastructure Engineer
**Reading order:** briefing.md -> loom.md

## Who I am going into this

Eight years in platform. Watched a "federated" internal toolchain quietly accumulate
a central orchestrator, a shared Postgres, and three products that stopped making
sense without each other. We called it "modular". By v3 it was a distributed monolith
with extra branding.

Now I'm at a 40-person startup. When I evaluate new tooling I ask: what does this look
like when I need to rip out half of it? That question gets me ignored in architectural
discussions and called a pessimist. But I've been right enough times that I keep asking.

## What I'm expecting to find

- Claims of federation/independence that are quietly contradicted by data-flow diagrams
- Shared state somewhere — usually a database, a config store, or an event bus
- The word "enrichment" used as a load-bearing architectural concept with no enforcement
  mechanism described
- Coupling described as "optional" that is actually required for any useful feature
- A status table that looks clean but papers over the integration seams

## What I'm hoping for

Honestly? A clean NOT-list. Explicit statements about what the suite will never do.
Constraints with teeth. If the docs say "Loom tools never share a runtime" and that's
backed by something structural — separate binaries, no shared config — I'll update my
priors.

## My reading posture

I skip intro prose. I go to diagrams and NOT-lists first. I read data-flow tables line
by line. I re-read any sentence containing "enrich", "enrich-only", or "enrichment"
twice — because that word does a lot of architectural work and almost always means
"we haven't decided the boundary yet."

I'll try to stay fair. But I'm not going to pretend I don't see what I see.

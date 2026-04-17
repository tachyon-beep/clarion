# Step A — Expectations Before Reading loom.md §3-§5

**Persona:** Marcus (p2-cto-evaluator)
**Document:** docs/suite/loom.md (§3-§5 only; skipping §7 and §8)
**Date:** 2026-04-17

## What I expect to find

The founding doctrine. §3-§5 is where the architectural philosophy lives. I expect:

- §3: some kind of federation principle — probably articulating why the tools don't share a runtime. The "federation axiom" was mentioned in briefing.md, so I expect a formal statement there.
- §4: composition law — how the tools are allowed to interact. Probably tight constraints on what can flow between them and what can't.
- §5: the "failure test" or what the briefing called the enterprise drift problem. My memory says the CLAUDE.md mentions "apply loom.md §5 failure test when reviewing architecture" — so §5 is the anti-drift watchdog.

My concern going in: these kinds of doctrine docs either (a) articulate a real constraint that shapes design decisions, or (b) are post-hoc rationalizations of decisions already made. I want to see whether §3-§5 actually constrains something — is there a decision that was *rejected* because of these principles?

I'm also watching for whether the composition rules are tight enough to guide future contributors or so loose that they're basically platitudes. "Each tool is independently useful" sounds great but if the real integration is load-bearing rather than enriching, the axiom is already violated.

# Journal Entry: loom.md
## Step A — Expectations (written before reading)

**Date:** 2026-04-17

Coming off briefing.md with a specific set of open questions I want loom.md to answer.

**What I expect this document to be:** The founding doctrine. Probably more abstract than briefing.md. The briefing directed me here for the federation axiom, the composition law, and the enrichment-not-load-bearing principle. The persona brief also specifically called out §5 (enrichment, Wardline example) as where I should slow down and work through carefully.

**What I want from §5 (enrichment):** The enrichment model is where my compliance antennae are up. In my experience "enrichment" means one of two things: (a) an additive annotation layer that never touches authoritative data, which is clean; or (b) a pattern where the "enricher" quietly starts owning the thing it was supposed to annotate, because the boundary was never enforced. I want loom.md §5 to tell me clearly which one this is. Specifically: does enrichment have write access to the record it is enriching, or only the ability to add to it?

**The Wardline example:** The briefing already told me Wardline findings go through Clarion's SARIF translator before landing in Filigree. If §5 uses this as the example of enrichment in action, I want to see: (a) who is the enricher and who is the original emitter, (b) what fields can the enricher touch, and (c) what happens to triage state in Filigree when the enricher (Clarion's translator) changes its output.

**The go/no-go test (§7, mentioned in briefing):** Probably product strategy. I will read it but I am not expecting it to have direct data-governance implications. If it does, I will note the surprise.

**Mood going in:** Alert. The enrichment section is where the doctrine either holds up or exposes a hidden mutation problem. I have seen too many "enrichment pipelines" that become the de facto authority over records nobody thought they owned. If loom.md treats enrichment as strictly additive with no back-write path, I will feel better. If it is ambiguous, that ambiguity is a finding.

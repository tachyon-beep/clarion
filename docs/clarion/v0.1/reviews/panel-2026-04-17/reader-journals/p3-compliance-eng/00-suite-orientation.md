# Suite Orientation — Yasmin (p3-compliance-eng)

**Date:** 2026-04-17
**Persona:** Security / Compliance Engineer, SOC 2 Type II regulated SaaS

## Who I am coming in as

I run findings lifecycle for a team that ingests from five scanners: SAST, SCA, secrets, IaC, and one internal custom scanner. Eighteen months of that work. The pain is not detecting findings — the pain is *triage state durability*. Who acknowledged which finding, when, under what rationale, and does that triage decision survive a version bump in the translator that produced the finding ID?

That is the lens I am bringing to Loom. I am not here to evaluate developer experience or naming aesthetics. I am here to answer: does this suite give me auditable, durable findings lifecycle, or does it just shuffle scan output between tools while the audit trail lives in nobody's domain?

## What I expect to find

**briefing.md:** Probably a product overview. I want to find a data-flow table that tells me where findings live and who owns them. I also want to see how the identity scheme works — if Clarion translates Wardline SARIF into Filigree records, is there a stable external identifier that survives that translation? Or does Filigree assign its own ID and the Wardline lineage is just metadata?

**loom.md:** The doctrine doc. I will read it looking for the enrichment model (who can mutate what, and under whose authority) and whether there is a trust-tier model that separates scan emitters from triage owners. Section 5 (enrichment, Wardline example) sounds directly relevant. My concern is that "enrichment" is a pattern that can silently overwrite authoritative data if the enricher is not carefully scoped.

## What I am not reading for

I will not evaluate the code-archaeology angle — that is someone else's concern. I will not engage with the weaving metaphor or naming decisions. The composition law reads like product strategy to me; I may under-read it unless it has direct data-governance implications.

## Key question I want answered

When Wardline emits a SARIF finding that Clarion translates and posts to Filigree, who is authoritative for the finding's lifecycle after that handoff? And if Clarion's translator changes between versions — different rule ID mapping, different severity normalization — what happens to existing triage state in Filigree? Does it survive, break, or silently diverge?

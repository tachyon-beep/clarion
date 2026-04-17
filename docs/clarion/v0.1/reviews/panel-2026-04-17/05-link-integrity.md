# Link & Cross-Reference Integrity Audit — 2026-04-17

**Scope**: all `.md` files under `/home/john/clarion/docs/` following commit dfb9d95 (docs restructure).

**Method**: extracted every `[text](path)` link and every "see X" / "defined in X" phrase naming a doc or section. Verified targets exist; verified anchors resolve; verified the ADR index and reading orders cross-consistent. Links verified: **67 relative Markdown links**, all targets present.

## Summary

No broken links. A small number of stale filename survivals and one internal inconsistency. Severity-grouped below.

---

## Broken links

*(none)*

Every relative `[text](path)` link resolves to an existing file:

- `docs/README.md` → `suite/README.md`, `clarion/README.md`, `suite/briefing.md`, `suite/loom.md`, `clarion/v0.1/README.md`, `clarion/adr/README.md` — all present.
- `docs/suite/README.md` → `loom.md`, `briefing.md`, `../clarion/README.md` — all present.
- `docs/suite/briefing.md` → `loom.md`, `../clarion/v0.1/{README,requirements,system-design,detailed-design}.md`, `../clarion/v0.1/reviews/{design-review,integration-recon}.md` — all present.
- `docs/suite/loom.md` → `briefing.md` — present.
- `docs/clarion/README.md` → `v0.1/README.md`, `adr/README.md` — present.
- `docs/clarion/adr/README.md` → four ADR files and `../v0.1/{detailed-design,system-design}.md` — all present.
- Each authored ADR (001-004) → siblings + `../v0.1/{system-design,detailed-design}.md` + review files — all present.
- `docs/clarion/v0.1/README.md` → canonical set, reviews, plans, `../adr/README.md`, and up-refs to suite briefing/loom — all present.
- `docs/clarion/v0.1/{requirements,system-design,detailed-design}.md` → intra-v0.1 links, `../adr/…`, `../../suite/loom.md` — all present.
- `docs/clarion/v0.1/plans/docs-restructure-plan.md` → `../../README.md`, `../README.md` — present.
- `docs/clarion/v0.1/reviews/{design-review,integration-recon}.md` → `../README.md`, `../detailed-design.md`, `design-review.md` — all present.

## Anchor links

Two anchor links exist; both resolve:

- `system-design.md:1175` → `./detailed-design.md#11-architecture-decisions` — target has `## 11. Architecture Decisions` at line 1483 (GitHub slug = `11-architecture-decisions`). OK.
- `detailed-design.md:1487` → `./system-design.md#12-architecture-decisions` — target has `## 12. Architecture Decisions` at line 1171. OK.

No other `#anchor` links are used in the docset.

---

## Stale references (low severity — historical context retained intentionally)

These are unbroken but reference the pre-restructure filename `2026-04-17-clarion-v0.1-design.md`, which no longer exists. All five sit inside documents explicitly marked as "historical note" or describing the rename operation, so they are intentional history — flagging for awareness only:

- `docs/clarion/v0.1/plans/docs-restructure-plan.md:14,123,187` — plan narrative and `git mv` command.
- `docs/clarion/v0.1/reviews/design-review.md:3` — "Original reviewed document (pre-restructure)".
- `docs/clarion/v0.1/reviews/integration-recon.md:3` — same pattern.
- `docs/clarion/v0.1/detailed-design.md:1689` — revision-history table entry describing the rename.

None of these are unqualified references requiring a fix. They survive in "historical note" scope and are correct in that frame.

## Inconsistent references

### MEDIUM — ADR backlog count mismatch between system-design §12 and detailed-design §11

- `system-design.md:1204` says "ADR-001 through ADR-004 are authored and Accepted as standalone files… ADR-005 through **ADR-013** are written alongside early implementation (P1)."
- `detailed-design.md:1527` says "ADR-005 through **ADR-013** are to be written alongside early implementation."
- But both tables list **ADR-005 through ADR-020** (sixteen backlog items). The "ADR-005 through ADR-013" phrasing is a leftover from before ADR-014-020 were added and now undercounts the backlog by seven entries. Both documents carry the identical stale phrase — likely a copy at the time the parallel tables were established.
- Recommendation: change both to "ADR-005 through ADR-020".

### LOW — `detailed-design.md:1526` writing-cadence line

Detailed-design §11's writing cadence also says "ADR-002 through ADR-004 must exist as markdown files before the implementation plan is authored." This is now true (they exist as of the ADR backlog scan), so the sentence is accurate but oriented to a past planning moment. Not a defect — flagged only because it sits adjacent to the ADR-013/020 miscount.

### LOW — `requirements.md:759, 917, 959` etc. use prose "System Design §N" instead of linked anchors

Throughout `requirements.md`, the **See** cross-references use the form `System Design §3 (Data Model)` as plain text rather than as anchor links into `system-design.md#...`. These all match real §-numbers in `system-design.md` (verified §1-§12 headings), so they are accurate; but they are not clickable. This is a consistency/UX issue rather than a broken link. Not in scope to change, flagged for awareness.

---

## ADR index consistency

`docs/clarion/adr/README.md` lists all four authored ADR files (001-004) — each exists and is correctly titled. The "backlog still tracked in the detailed design" section lists ADR-005 through ADR-020 with statuses that match both `system-design.md §12` and `detailed-design.md §11` (including ADR-008 as "Superseded by ADR-014"). The three tables agree on membership, titles, and supersession.

## Reading-order consistency

- `docs/README.md` entry points: briefing → loom → clarion v0.1 README. Consistent.
- `docs/suite/README.md`: briefing → loom. Consistent with above.
- `docs/clarion/README.md`: points to `v0.1/README.md` and `adr/README.md`. Consistent.
- `docs/clarion/v0.1/README.md` "New reader" path: `suite/briefing.md` → `suite/loom.md` → `requirements.md` → `system-design.md`. Consistent with the suite-level entry point.
- `docs/clarion/v0.1/detailed-design.md:29` "Starting fresh" path: briefing + loom first, then requirements + system-design. Consistent.
- `docs/clarion/v0.1/requirements.md` and `system-design.md` preambles list the same sibling set. Consistent.

No reading-order contradictions across the tree.

---

## One-line verdict

Link graph is clean post-restructure. One real defect: the phrase "ADR-005 through ADR-013" appears in both `system-design.md:1204` and `detailed-design.md:1527` and should read "ADR-005 through ADR-020" to match the tables above it.

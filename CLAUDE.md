# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository state

This repo is **documentation-only** today — there is no Rust source, no Python plugin, no `Cargo.toml`. The v0.1 design freeze landed on 2026-04-18; first implementation commits (Sprint 1, WP1) are about to begin. Build/test/lint commands do not yet exist; do not invent them. When source code lands, update this file.

The eventual shape (per ADR-001 + the Sprint-1 plan) is a Cargo workspace with a Rust core plus an editable Python plugin under `plugins/python/`. The Sprint-1 demo script in `docs/implementation/sprint-1/README.md` §3 is the canonical first-build recipe.

## What Clarion is, in one paragraph

Clarion is a code-archaeology tool: it ingests a codebase, extracts entities (functions, classes, modules), clusters them into subsystems, and serves structured briefings to consult-mode LLM agents over MCP so those agents do not have to re-explore the tree on every question. Single-binary Rust core + language plugins (Python first); SQLite-backed local state under `.clarion/`; designed for "enterprise rigor at lack of scale." Target first customer is `elspeth` (~425k LOC Python).

Clarion is one of three (soon four) products in the **Loom** suite. The other repos — `filigree` and `wardline` — are not vendored here but are owned by the same author and are referenced extensively. Cross-product work in WP9/WP10/Sprint-2+ is within-scope, not external.

## Doctrine you must read before changing design docs

The Loom federation axiom in `docs/suite/loom.md` (especially §3–§5) is **load-bearing for every architectural decision in this repo**. The three rules:

1. Each product is solo-useful.
2. Each pair composes meaningfully on its own.
3. Integration is enrich-only — a sibling may add information to another product's view but must never be required for that product's semantics to make sense.

Before proposing or accepting any change that adds a new dependency, "lightweight glue layer," shared registry, or cross-product mediator, run it against the §5 failure test (semantic / initialization / pipeline coupling). Centralisation creeps back in naturally; treat any "wouldn't it be easier if we just..." proposal as suspicious.

Two named v0.1 asterisks (Wardline→Filigree pipeline coupling via Clarion; Python plugin's `wardline.core.registry.REGISTRY` import) have written retirement conditions in `loom.md` §5. Do not add new asterisks without the same.

## Documentation map

```
docs/
├── suite/                         Loom-wide doctrine (read-first for new contributors)
│   ├── briefing.md                5-minute introduction
│   └── loom.md                    Founding doctrine, federation axiom, go/no-go test
├── clarion/
│   ├── v0.1/                      Canonical product docset for the current version
│   │   ├── requirements.md        The WHAT — REQ-/NFR-/CON-/NG- IDs, baselined
│   │   ├── system-design.md       The HOW — architecture, mechanisms, §2–§11 with `Addresses:` headers
│   │   ├── detailed-design.md     Implementation reference — schemas, rule catalogs, appendices
│   │   ├── plans/                 Scope memo (v0.1-scope-commitments.md)
│   │   └── reviews/               Retained historical reviews (supporting context, not normative)
│   └── adr/                       Authored architecture decision records (ADR-001 … ADR-022)
└── implementation/                Work-package sequencing (lives ABOVE v0.1/ because WPs span siblings)
    ├── v0.1-plan.md               11 WPs in dependency order, with anchoring docs/ADRs per WP
    └── sprint-1/                  Per-sprint execution plan (walking-skeleton: WP1+WP2+WP3)
```

### Reading order by intent

- **New to the project**: `docs/suite/briefing.md` → `docs/suite/loom.md` → `docs/clarion/v0.1/README.md`.
- **Implementing**: `requirements.md` → `system-design.md` → `detailed-design.md` → relevant ADRs → the WP doc under `docs/implementation/`.
- **Reviewing a design proposal**: read the requirement IDs it cites, then the system-design section listed in those requirements' `See` lines, then check whether any Accepted ADR already constrains the answer.

## Where canonical truth lives

When the same fact appears in multiple files, this is the precedence:

1. **Accepted ADRs** in `docs/clarion/adr/` — the locked decisions. 16 are Accepted at v0.1; six remain Backlog and are tracked inside `system-design.md` §12 / `detailed-design.md` §11 until promoted.
2. **`requirements.md`** — REQ-/NFR-/CON-/NG- IDs are stable and load-bearing (filigree issues and commit messages cite them by ID; never reuse a retired ID).
3. **`system-design.md`** — `Addresses:` headers on each §2–§11 section define the requirement acceptance surface for that subsystem.
4. **`detailed-design.md`** — exact schemas, rule catalogues, appendices.
5. Reviews under `docs/clarion/v0.1/reviews/` are supporting context only, not normative. Do not cite a review as the source of a current decision; cite the ADR or design doc that absorbed it.

If `requirements.md` and `system-design.md` disagree, the requirement wins and the design doc is the bug. If an ADR exists, it overrides both.

## Implementation work-package vocabulary

Work is organised as numbered Work Packages (WP1–WP11) and grouped into sprints. Each WP doc has the same skeleton: scope, deliverables, exit criteria, anchoring system-design sections, ADRs satisfied, ADRs surfaced, unresolved questions.

Sprint 1 commits a numbered set of "lock-ins" (L1–L9) — design surfaces that are cheap to change before the sprint closes and expensive after. When touching anything in `wp1-scaffold.md`, `wp2-plugin-host.md`, or `wp3-python-plugin.md`, check the lock-in table in `docs/implementation/sprint-1/README.md` §4 first; later sprints will read and write against those exact shapes.

## Key terminology to use consistently

- **Entity ID** (per ADR-003 + ADR-022): three colon-separated segments — `{plugin_id}:{kind}:{canonical_qualified_name}`, e.g. `python:function:auth.tokens.refresh`. The plugin owns segments 1 and 3; the core never invents kinds.
- **Finding**: a unified record type for defects, structural observations, classifications, metrics, and suggestions — emitted by Clarion (and other Loom tools) into Filigree via `POST /api/v1/scan-results`. See ADR-004.
- **Observation**: fire-and-forget agent note (see Filigree workflow). Distinct from a Finding.
- **Guidance sheet**: institutional knowledge attached to an entity (Clarion-authored).
- **Briefing**: structured per-entity summary that Clarion serves to consult-mode agents.
- **Loom suite**: the federation. Refer to it as "the Loom suite" in docs (per project memory). Member products are Clarion, Filigree, Wardline, and the proposed Shuttle.

Avoid: "Loom platform," "Loom runtime," "Loom broker," "Loom store" — Loom is a family name and a doctrine, not anything that runs (per `loom.md` §6).

## Editorial conventions for design docs

- ADR files are immutable once Accepted, except for status changes and "Superseded by ADR-NNN" links. To revise an Accepted ADR, write a new ADR that supersedes it.
- Each requirement statement has: stable ID, plain-English statement, rationale, verification method, and a `See:` link to the addressing system-design section. Match the existing pattern when adding requirements.
- When renaming or moving design files, prefer `git mv` over leaving redirect stubs behind. The user has explicitly rejected legacy-filename "history preservation" tech debt.

## Task tracking

`filigree` is the issue tracker for this project (config in `.filigree/`, MCP server registered in `.mcp.json`). The global `~/CLAUDE.md` file describes the workflow and CLI/MCP commands; do not duplicate that here. Project-specific notes:

- Sprint 1 issues should be seeded as `WP1`, `WP2`, `WP3`, plus a `Sprint 1 close` issue blocked-by all three. Labels follow the `release:v0.1`, `sprint:1`, `wp:N`, `adr:NNN` scheme described in `docs/implementation/sprint-1/README.md` §8.
- Filigree issue bodies should cite `REQ-*` / `NFR-*` / ADR IDs verbatim — those IDs are how design docs and tracker stay linked.

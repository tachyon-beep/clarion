# The Suite — A Briefing

**Audience**: engineers, reviewers, or stakeholders new to the three-tool suite
**Purpose**: explain what each tool does, how they fit together, and what state the suite is in today
**Reading time**: ~5 minutes

---

## The one-paragraph version

**Clarion**, **Filigree**, and **Wardline** are three independent tools that compose into an enterprise-grade code governance suite for small teams. Clarion builds a trustworthy catalog of a codebase and answers structural questions. Filigree tracks the issues, findings, and observations that arise from examining that codebase. Wardline declares and enforces the trust topology that constrains how code is allowed to behave. Each tool is useful on its own; together, they deliver rigor that normally requires enterprise-scale platform teams — without the operational weight.

---

## The three tools

### Clarion — the code-archaeology catalog

**Role**: indexes the source tree and answers structural questions.

Clarion ingests a codebase, extracts entities (functions, classes, modules, packages), clusters them into subsystems, and produces structured briefings that summarise each entity's purpose, maturity, and relationships. Consult-mode LLM agents query Clarion through MCP tools so they never need to spawn an explore-agent to answer "what are the entry points?" or "what calls this function?" — Clarion answered that during its batch analysis and caches the result.

**Authoritative for**: the entity catalog, the code graph, guidance sheets (institutional knowledge attached to entities), and structural / factual findings.

**Typical invocation**: `clarion analyze <project>` for batch indexing; `clarion serve` for MCP + HTTP consult.

**Status**: designed, not yet built. Target first customer is `elspeth` (~425k LOC Python).

### Filigree — the workflow and findings tracker

**Role**: tracks issues, observations, findings lifecycle, and their triage.

Filigree is where work lives. It holds the project's issues, the observations (fire-and-forget notes) that agents emit during work, the findings that scanners produce, and the lifecycle state of each (open, acknowledged, fixed, suppressed). It exposes an MCP server so agents can query and mutate work items directly, and a dashboard for human operators.

**Authoritative for**: issue state, workflow transitions, observation and finding lifecycle, triage history.

**Typical invocation**: `filigree list`, `filigree create`, `filigree claim-next` from CLI; MCP tools from agents; HTTP dashboard for humans.

**Status**: already built and in active use.

### Wardline — the trust-topology enforcer

**Role**: declares and enforces trust topology at commit cadence.

Wardline understands "which code is allowed to do what." Modules declare their trust tier (`INTEGRAL`, `ASSURED`, `GUARDED`, `EXTERNAL_RAW`) and annotate functions with decorators that assert behavioural constraints (`@validates_shape`, `@integral_writer`, `@fail_closed`, `@handles_secrets`, and 38 others across 17 annotation groups). Wardline's scanner verifies that code satisfies what it claims, emits findings when it doesn't, and maintains a per-function fingerprint baseline so drift is visible.

**Authoritative for**: tier declarations, annotation semantics, trust-topology invariants, dataflow enforcement.

**Typical invocation**: `wardline scan` at commit cadence (pre-commit hook or CI); SARIF output uploaded to GitHub Security.

**Status**: already built and in active use.

---

## How they interact

The suite is composed via two narrow protocols and a shared identity scheme.

### The fabric at a glance

```
                          ┌─────────────────┐
                          │   Filigree      │
                          │ issues,         │
                          │ findings,       │
                          │ observations    │
                          └────┬─────────▲──┘
                               │         │
                      findings │         │ read (triage state,
           (POST /api/v1/      │         │  cross-refs)
              scan-results)    │         │
                               ▼         │
   ┌──────────────┐     ┌──────────────┐
   │   Clarion    ├────►│  scan import │
   │  catalog +   │     │  + observations│
   │  briefings   │◄────┤              │
   └──────▲───────┘     └──────────────┘
          │
          │ ingest (wardline.yaml,
          │  fingerprint.json,
          │  exceptions.json,
          │  REGISTRY)
          │
   ┌──────┴───────┐
   │   Wardline   │
   │  scanner +   │
   │  SARIF       │
   └──────────────┘
```

### Data flows

| Flow | From | To | Mechanism |
|---|---|---|---|
| Declared topology | Wardline manifest / fingerprint files | Clarion catalog | File read at `clarion analyze` |
| Annotation vocabulary | `wardline.core.registry.REGISTRY` | Clarion's Python plugin | Direct import at plugin startup |
| Findings | Clarion | Filigree | `POST /api/v1/scan-results` (Clarion-native schema) |
| Findings (Wardline-sourced) | Wardline SARIF → Clarion translator | Filigree | `POST /api/v1/scan-results` via `clarion sarif import` |
| Observations | Clarion consult mode | Filigree | MCP tool call (or HTTP once the endpoint ships) |
| Entity state | Clarion | Wardline (v0.2+) | Clarion HTTP read API; Wardline currently re-scans |
| Issue cross-references | Filigree | Clarion consult surface | Filigree read API |

### Identity and the shared vocabulary

The glue between tools is the **entity ID**. Clarion owns the entity catalog and mints stable symbolic identifiers (`python:class:auth.tokens::TokenManager`). Filigree issues reference entities by Clarion ID. Wardline findings carry qualnames that Clarion reconciles to entity IDs at ingest. The suite has three concurrent identity schemes (Clarion EntityId, Wardline qualname, Wardline exception-register location string) — Clarion maintains the translation layer; neither sibling tool takes on that responsibility.

Findings are the other glue: every tool emits findings into Filigree's `POST /api/v1/scan-results` with a distinct `scan_source` (`clarion`, `wardline`, and so on). Filigree preserves the `metadata` dict verbatim, so Clarion's richer fields (`kind`, `confidence`, `related_entities`) and Wardline's SARIF property-bag extensions survive ingest under namespaced keys (`metadata.clarion.*`, `metadata.wardline_properties.*`).

---

## Principles that shape the suite

Four commitments keep the three tools from drifting into overlap:

1. **Clarion observes, Wardline enforces.** Clarion detects that an annotation is present; Wardline determines whether the annotated code satisfies the semantic it declares. Clarion never re-implements Wardline analyses; Wardline never re-implements Clarion's graph.
2. **Findings are facts, not just errors.** A unified `Finding` record type carries defects, structural observations, classifications, metrics, and suggestions across all three tools.
3. **Each tool is independently useful.** Clarion works without Filigree (writes findings to local JSONL). Wardline works without Clarion (has since day one). Filigree works without either.
4. **Local-first, single-binary, git-committable state.** No hosted service is required; `.clarion/`, `.filigree/`, and Wardline's JSON state files are all meant to be committed and shared.

---

## Current state

| Tool | Built? | In use? | First customer |
|---|---|---|---|
| Filigree | Yes | Yes — active development | `filigree` itself; this project |
| Wardline | Yes | Yes — commit-cadence scanner | Wardline's own codebase |
| Clarion | No — designed only | Not yet | `elspeth` (~425k LOC Python) targeted for v0.1 validation |

### What Clarion v0.1 ships

A single-binary Rust core plus a Python language plugin. The core handles storage, LLM orchestration, clustering, and MCP serving; the plugin handles Python parsing, import resolution, and entity extraction. v0.1's scope is **bootstrapping the suite fabric**, not joining it: Clarion v0.1 delivers the cross-tool protocols that Filigree and Wardline don't yet speak.

### What the suite needs from Filigree and Wardline for Clarion to ship

Because Clarion is the work that weaves the fabric, several changes land in the sibling tools as prerequisites:

- **Filigree**: a pluggable `registry_backend` so Clarion can own the file registry; an HTTP endpoint for observation creation; a published schema-compatibility contract.
- **Wardline**: a stable `REGISTRY_VERSION` that Clarion's plugin pins against; a commitment to maintain legacy-decorator aliases; eventually, a native emitter to Filigree so Clarion's SARIF translator can retire.

Clarion's design (`docs/superpowers/specs/2026-04-17-clarion-v0.1-design.md`, §11 Suite Bootstrap) enumerates these asks with owner and sequence. Clarion ships with degraded-mode fallbacks (`--no-filigree`, `--no-wardline`) so it doesn't block on the slowest of three release trains.

---

## Where to read next

| If you want to… | Read |
|---|---|
| Understand Clarion's full design | `docs/superpowers/specs/2026-04-17-clarion-v0.1-design.md` |
| See what the design reviewer flagged | `docs/superpowers/specs/2026-04-17-clarion-v0.1-design-review.md` |
| See the integration reality check | `docs/superpowers/specs/2026-04-17-clarion-integration-recon.md` |
| Work with Filigree today | `/home/john/filigree` — `CLAUDE.md` and `filigree --help` |
| Work with Wardline today | `/home/john/wardline` — `docs/spec/` |

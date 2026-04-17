# Clarion ADR Index

This folder is the canonical home for authored Clarion architecture decision records.

## Authored ADRs

| ADR | Title | Status |
|---|---|---|
| [ADR-001](./ADR-001-rust-for-core.md) | Rust for the core | Accepted |
| [ADR-002](./ADR-002-plugin-transport-json-rpc.md) | Plugin transport: Content-Length framed JSON-RPC subprocess | Accepted |
| [ADR-003](./ADR-003-entity-id-scheme.md) | Entity ID scheme: symbolic canonical names | Accepted |
| [ADR-004](./ADR-004-finding-exchange-format.md) | Finding-exchange format: Filigree-native intake | Accepted |

## Backlog still tracked in the detailed design

The following decisions are still backlog items rather than authored ADR files. Their current summaries live in [../v0.1/detailed-design.md](../v0.1/detailed-design.md) §11 and [../v0.1/system-design.md](../v0.1/system-design.md) §12.

| ADR | Title | Current state |
|---|---|---|
| ADR-005 | `.clarion/` git-committable by default; DB included, run logs excluded | Backlog |
| ADR-006 | Clustering algorithm: Leiden with Louvain fallback | Backlog |
| ADR-007 | Summary cache key design and invalidation | Backlog |
| ADR-008 | Filigree file-registry displacement as breaking change | Superseded by ADR-014 |
| ADR-009 | Structured briefings vs free-form prose | Backlog |
| ADR-010 | MCP as first-class surface | Backlog |
| ADR-011 | Writer-actor concurrency model | Backlog |
| ADR-012 | Token auth in v0.1 | Backlog |
| ADR-013 | Pre-ingest secret scanner with LLM-dispatch block | Backlog |
| ADR-014 | Filigree `registry_backend` flag and pluggable `RegistryProtocol` | Backlog |
| ADR-015 | Wardline-to-Filigree emission ownership | Backlog |
| ADR-016 | Observation transport via Filigree HTTP endpoint | Backlog |
| ADR-017 | Severity mapping, rule-ID round-trip, and dedup policy | Backlog |
| ADR-018 | Identity reconciliation and Wardline REGISTRY pinning | Backlog |
| ADR-019 | SARIF property-bag preservation | Backlog |
| ADR-020 | Degraded-mode policy and explicit suite fallbacks | Backlog |

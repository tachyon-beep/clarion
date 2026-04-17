# ADR-001: Rust for the Core

**Status**: Accepted
**Date**: 2026-04-17
**Deciders**: qacona@gmail.com
**Context**: Clarion v0.1 core implementation language

## Summary

Clarion's core will be implemented in Rust. This is a directive decision that optimizes for single-binary distribution, predictable local operation, and robust plugin/process supervision.

## Context

Clarion's core is responsible for:

- single-binary distribution across developer machines
- SQLite-backed storage and local HTTP/MCP serving
- subprocess supervision for language plugins
- bounded-cost LLM orchestration

The product posture is local-first and operationally lightweight. The implementation language therefore affects packaging, concurrency, storage ergonomics, and deployment complexity.

## Decision

We will implement the Clarion core in Rust.

## Alternatives Considered

### Alternative 1: Go

**Description**: use Go for the core binary and service surfaces.

**Pros**:

- simple single-binary distribution
- approachable contributor pool
- strong HTTP tooling

**Cons**:

- weaker fit for the desired SQLite and data-structure ergonomics already assumed in the design
- less alignment with the author's chosen implementation direction

**Why rejected**: the primary author directed Rust for v0.1, and the rest of the design already assumes Rust-native ecosystem choices.

### Alternative 2: Python or TypeScript

**Description**: implement the core in a higher-level runtime language.

**Pros**:

- faster initial prototyping
- lower barrier for contributors

**Cons**:

- runtime dependency burden conflicts with single-binary local-first posture
- weaker fit for plugin supervision and packaged distribution goals

**Why rejected**: these options reintroduce runtime-management overhead that the product is explicitly trying to avoid.

## Consequences

### Positive

- straightforward single-binary distribution
- mature ecosystem for HTTP, async orchestration, and SQLite
- clean subprocess boundary with Python plugins

### Negative

- higher contribution and recruiting bar than a higher-level language
- more implementation ceremony for some product surfaces

### Neutral

- plugin authors remain decoupled from the core language because plugin transport is subprocess-based

## Related Decisions

- Related to: [ADR-002](./ADR-002-plugin-transport-json-rpc.md), [ADR-003](./ADR-003-entity-id-scheme.md), [ADR-004](./ADR-004-finding-exchange-format.md)

## References

- [Clarion v0.1 system design](../v0.1/system-design.md)
- [Clarion v0.1 detailed design](../v0.1/detailed-design.md)

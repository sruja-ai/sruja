# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the Sruja project.

## What are ADRs?

ADRs are documents that capture important architectural decisions made along with their context and consequences. They help:

- **Document decisions**: Why certain approaches were chosen
- **Preserve context**: What alternatives were considered
- **Track evolution**: How decisions change over time
- **Onboard new team members**: Understand the reasoning behind the architecture

## ADR Format

Each ADR follows this structure:

1. **Status**: Proposed, Accepted, Deprecated, Superseded
2. **Context**: The issue motivating this decision
3. **Decision**: The change that we're proposing or have agreed to implement
4. **Consequences**: What becomes easier or more difficult because of this change

## ADR Index

| Number | Title                                                                     | Status    | Date |
| ------ | ------------------------------------------------------------------------- | --------- | ---- |
| 001    | [Use Result Type for Error Handling](./001-result-type-error-handling.md) | Accepted  | 2024 |
| 002    | [Monorepo Structure with Turbo](./002-monorepo-structure.md)              | Superseded| 2024 |
| 003    | [WASM for Browser Integration](./003-wasm-browser-integration.md)         | Accepted  | 2024 |
| 004    | [Graphviz WASM for C4 Layouts](./004-graphviz-wasm-layout.md)              | Accepted  | 2024 |

## Summary

- **001**: Result types for error handling (Rust uses `Result` natively; applies to TS/extension where relevant).
- **002**: Superseded—repo is no longer a Turbo monorepo; see ADR body for current layout (`crates/`, `extension/`).
- **003**: WASM for browser/extension; implementation is Rust → WASM via `crates/sruja-wasm`.
- **004**: Graphviz WASM as the layout engine for C4-style diagrams where needed.

## Creating a New ADR

1. Copy the template: `cp TEMPLATE.md 00X-decision-title.md`
2. Fill in the template with your decision
3. Update this README with the new ADR
4. Submit as part of your PR

## References

- [ADR GitHub](https://github.com/joelparkerhenderson/architecture-decision-record)
- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)

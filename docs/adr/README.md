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

| Number | Title | Status | Date |
|--------|-------|--------|------|
| 001 | [Use Result Type for Error Handling](./001-result-type-error-handling.md) | Accepted | 2024 |
| 002 | [Monorepo Structure with Turbo](./002-monorepo-structure.md) | Accepted | 2024 |
| 003 | [WASM for Browser Integration](./003-wasm-browser-integration.md) | Accepted | 2024 |
| 004 | [LikeC4 Model Format](./004-likec4-model-format.md) | Accepted | 2024 |

## Recent ADRs

- **001**: Documents the decision to use Result types for functional error handling
- **002**: Explains the monorepo structure and tooling choices
- **003**: Describes WASM integration strategy for browser usage
- **004**: Documents the choice of LikeC4 as the model format foundation

## Creating a New ADR

1. Copy the template: `cp TEMPLATE.md 00X-decision-title.md`
2. Fill in the template with your decision
3. Update this README with the new ADR
4. Submit as part of your PR

## References

- [ADR GitHub](https://github.com/joelparkerhenderson/architecture-decision-record)
- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)


# Sruja Architecture Platform — Documentation Index

This index summarizes the major subsystems implemented:

- Entities DSL: Domain-first modeling of business entities, relations, invariants, lifecycle, versioning, and constraints.
- Event DSL: Domain-driven event modeling with schema, lifecycle effects, guarantees, versioning, publishers/consumers.
- Rule DSL: Atomic validation rules with a shared expression engine.
- Policy DSL: Governance policies composed of rules/controls with compliance evaluation.
- Approval Policy DSL: Declarative approval requirements with targets, conditions, exceptions, and auto-approve.
- Query Language (SQL-lite): Unified expressions and operators used across rules and policies.
- MCP Server: Endpoints for load/explain/query/diagram/diff; review/evaluate; domain model (entities/events); approval policy validation.
- LSP: Context-aware completions and hover docs for Policies (keywords, targets, attributes, operators), IR-driven hints.
- Policy Evaluation Engine: Diff-aware evaluator producing approvals and diagnostics for CI and IDE.
- Diff Engine (Semantic): High-level, semantic diff powering approval conditions and explanations.
 - Brownfield Adoption: Strategy for zero-friction onboarding, auto-discovery, partial models.
- AI‑First Importer: Multi-agent LLM pipeline with deterministic normalizer for brownfield systems.
 - Human‑in‑the‑Loop Review UI: Notebook, PR-style panel, and visual diagram review with confidence, trace, and staged changes.

## Quick Links
- Entities DSL: see `docs/entities.md`
- Event DSL: see `docs/events.md`
- Rule & Policy DSL: see `docs/rules-policies.md`
- Approval Policy DSL: see `docs/approval-policy.md`
- MCP Server: see `docs/mcp.md`
- LSP Completions & Hover: see `docs/lsp.md`
- Policy Evaluation Engine: see `docs/policy-eval.md`
- Semantic Diff Engine: see `docs/diff-engine.md`
 - Brownfield Adoption: see `docs/brownfield-adoption.md`
- AI‑First Importer: see `docs/ai-first-importer.md`
 - Human‑in‑the‑Loop Review UI: see `docs/hitl-review-ui.md`

## Build & Test
- Build all: `go build ./...`
- Run rules/policies tests: `go test ./pkg/review ./pkg/policy`
- Run new DSL parser tests: `go test -tags newdsl ./pkg/language`
- Run legacy suites (if applicable): `go test -tags legacy ./...`

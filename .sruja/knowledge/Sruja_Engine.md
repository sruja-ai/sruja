# Validation Engine

> Validates architecture models against rules.

## Purpose

The engine runs rule-based validation on the AST produced by the language crate: unique IDs, valid references, no cycles, no orphans, SLO/causal/feedback-loop integrity, and other governance rules. It emits diagnostics (errors/warnings) used by CLI, LSP, and WASM.

## Responsibilities

- Execute validation rules (orphan, cycle, valid_ref, etc.)
- Compute architecture health scores and drift
- Provide a sync/async execution surface for CLI and LSP

## Dependencies

- **Internal**: Sruja_Language (AST), Sruja_Export (validates before export)
- **External**: sruja-diagnostics for severity and reporting

## Known Risks

- Rule set must stay aligned with language spec and user expectations
- Performance on large graphs (thousands of elements)

## Suggested Improvements

- [ ] Keep rules documented in engine and in skill/rules (sruja-architecture)
- [ ] Avoid circular dependency: engine must not depend on LSP or CLI

## Related Decisions

- ADR001: Rust for Core and CLI

## Code Locations

- `crates/sruja-engine/` — Validator, rules (orphan, cycle, valid_ref, slo, etc.)
- `skills/sruja-architecture/rules/` — Human-readable rule guidance

---
*Last updated: 2025-03-16 by Sruja*

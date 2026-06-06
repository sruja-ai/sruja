# Sruja Intent

> Intent vs. reality comparison and architecture critique.

## Purpose

Compares declared architectural intent (from ADRs, `.sruja` files, design docs) against actual implementation to detect boundary drift, intent violations, and undocumented architectural changes. Includes a critique engine for evaluating architecture quality.

## Responsibilities

- Parse ADRs and extract declared intent
- Normalize intent into a structured `IntentModel`
- Detect drift between declared intent and actual implementation
- Generate intent violation reports
- Run adversarial architecture critique (quality, risk, complexity)
- Evaluate architecture against best practices

## Dependencies

- **Internal**: Sruja_Scan, Sruja_Language
- **External**: thiserror, serde, serde_json, chrono, regex, walkdir, colored

## Key Types

- `IntentContext` — Context for intent comparison
- `IntentModel` — Normalized declared intent
- `DeclaredComponent`, `DeclaredRelationship` — Declared architecture elements
- `DriftDetector`, `Drift`, `DriftKind` — Drift detection
- `CritiqueEngine`, `CritiqueReport`, `CritiqueFinding` — Architecture critique
- `AdrParser`, `ParsedAdr` — ADR parsing
- `IntentReport`, `IntentViolation` — Violation reports

## Code Locations

- `crates/sruja-intent/` — Intent crate
- `src/model/` — Intent model types
- `src/compare.rs` — Drift detection
- `src/critique.rs` — Critique engine
- `src/parser.rs` — ADR parsing

---
*Last updated: 2026-06-06*

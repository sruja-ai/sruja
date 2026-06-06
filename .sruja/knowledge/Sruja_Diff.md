# Sruja Diff

> Architecture drift detection and graph comparison.

## Purpose

Compares architecture graphs to detect drift, violations, and differences. Maps git diffs to architecture elements, calculates health scores, validates proposals against reality, and detects circular dependencies and orphan modules.

## Responsibilities

- Compare two graphs (proposal vs. actual) and produce diff results
- Detect architectural drift between declared and actual architecture
- Find circular dependencies and orphan modules
- Map git diffs to architecture elements
- Calculate architecture health scores
- Validate proposals against current reality
- Track architectural velocity over time

## Dependencies

- **Internal**: Sruja_Language, Sruja_Scan, Sruja_Diagnostics, Sruja_Intent
- **External**: serde, serde_json, thiserror, git2, chrono

## Key Types

- `compare_graphs()` — Compare two graphs
- `detect_architectural_drift()` — Detect drift
- `Violation`, `ViolationKind`, `Severity` — Violation types
- `DriftReport`, `DriftConfig` — Drift detection results
- `HealthScoreBreakdown` — Architecture health metrics
- `Proposal`, `ProposalValidation` — Proposal validation
- `map_git_diff()` — Map git changes to architecture elements

## Code Locations

- `crates/sruja-diff/` — Diff crate
- `src/compare.rs` — Graph comparison
- `src/drift.rs` — Drift detection
- `src/health.rs` — Health scoring
- `src/proposal.rs` — Proposal validation
- `src/git_mapper.rs` — Git diff mapping

---
*Last updated: 2026-06-06*

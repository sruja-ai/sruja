# Implementation Plan: Representation, Drift, Policies, Compliance

**Reference:** [REPRESENTATION_DRIFT_POLICY_COMPLIANCE.md](REPRESENTATION_DRIFT_POLICY_COMPLIANCE.md)

This document is the concrete implementation plan with phases, files, and acceptance criteria.

---

## Research Summary

| Area | Current State | Gap |
|------|---------------|-----|
| **Intent from .sruja** | `IntentModel::from_sruja_file` extracts components + relationships via `collect_elements`. | Does not extract **Policy** (DSL has `TopLevelItem::Policy`). Boundaries not in .sruja (only ADRs). |
| **Policy evaluation** | `sruja-graph`: `Policy` + `PolicyRule` + `Constraint` (source_kind, target_kind, allowed); `find_policy_violations()` works. | `sruja-intent::detect_policy_violations` is a **stub** (returns empty). Policies from intent never converted to graph or evaluated. |
| **Intent check CLI** | Loads intent dir, merges models, scan, `DriftDetector.detect()`, prints. | Policy violations always 0; no wiring from DeclaredPolicy to graph evaluation. |
| **Compliance** | No single report or command. | Need `ComplianceReport` and `sruja compliance` that combine structural drift + intent drift + policy violations. |

---

## Phase 1: Policy Extraction and Evaluation

**Goal:** Policies declared in .sruja or ADRs are evaluated against the scan graph; intent check reports real policy violations.

### 1.1 Extract Policy from DSL in IntentModel (sruja-intent)

**File:** `crates/sruja-intent/src/model/mod.rs`

- In `from_sruja_file`, after building `model` from elements/relations, iterate `program.items` for `TopLevelItem::Policy(p)`.
- For each `p` (sruja_language::Policy): push `DeclaredPolicy { name: p.id, description: p.description.unwrap_or_else(|| p.category.clone()), scope: vec![], rules: vec![PolicyRule { description: p.title.clone(), constraint: p.description.unwrap_or_else(|| format!("{} must not violate {}", p.category, p.enforcement)) }], source_ref: ... }`.
- Use one rule per policy; constraint text from description or category/enforcement for now.

**AC:** Parsing a .sruja file that contains `X = policy "Y" { category "security" }` produces an IntentModel with one DeclaredPolicy and one PolicyRule.

### 1.2 Constraint Parser (CLI)

**File:** `crates/sruja-cli/src/compliance.rs` (new)

- Add `parse_constraint_to_rule(s: &str) -> Option<(sruja_types::NodeKind, sruja_types::NodeKind, bool)>`.
- Patterns: `"X must not call Y"`, `"X cannot call Y"` (case-insensitive, X/Y normalized to snake_case) → `Some((source_kind, target_kind, false))`.
- Map string to NodeKind via `sruja_types::NodeKind::from_str` (already supports "external_api", "database", "service", "module", etc.).
- Return None if pattern doesn’t match (rule skipped).

**AC:** `parse_constraint_to_rule("external_api must not call database")` → `Some((ExternalApi, Database, false))`.

### 1.3 Evaluate Intent Policies via Graph (CLI)

**File:** `crates/sruja-cli/src/compliance.rs`

- `evaluate_policy_violations(intent: &IntentModel, scan_graph: &sruja_scan::Graph) -> Vec<sruja_intent::Drift>`:
  - Create `KnowledgeGraph::new()`.
  - `merge_scan_into_graph(&mut kg, scan_graph, "")`.
  - For each `intent.policies`, for each `rule`: parse `rule.constraint` with `parse_constraint_to_rule`; if `Some((src_kind, tgt_kind, false))`, build `graph::Policy` with one `PolicyRule` and `Constraint { source_kind: Some(src_kind), target_kind: Some(tgt_kind), allowed: false, message: rule.description }`, `kg.add_policy(...)`.
  - `kg.find_policy_violations()` → for each `PolicyViolation` build `sruja_intent::Drift { kind: PolicyViolation, severity: from policy.severity, description: message, evidence: [...], suggestion: Some(...) }`.
  - Return vec of Drift.

**AC:** Given intent with policy "external_api must not call database" and scan graph with an edge from an ExternalApi node to a Database node, `evaluate_policy_violations` returns at least one Drift with kind PolicyViolation.

### 1.4 Recompute Summary and Score (sruja-intent)

**File:** `crates/sruja-intent/src/compare/mod.rs`

- Add `DriftReport::recompute_summary_and_score(&mut self)` that: recomputes `summary` from `drifts` (counts by DriftKind), then recomputes `drift_score` and `health` using existing formula (extract into a helper or reuse inside this method).
- Expose the score formula in a way that can be called with drifts + summary (e.g. `compute_drift_score(summary, drifts)` and `classify_health(score)` as public or crate-public).

**File:** `crates/sruja-intent/src/compare/mod.rs` (DriftDetector)

- Make `compute_drift_score` and `classify_health` callable (pub(crate) or pub) so `DriftReport` can use them, or move the logic into `DriftReport::recompute_summary_and_score`.

**AC:** After appending drifts to a DriftReport and calling `recompute_summary_and_score()`, `summary.policy_violations` and `drift_score` reflect the new drifts.

### 1.5 Wire Policy Evaluation into intent_check (CLI)

**File:** `crates/sruja-cli/src/commands/intent.rs`

- After `report = detector.detect(&merged_model, &graph)`:
  - Call `evaluate_policy_violations(&merged_model, &graph)` (from compliance module).
  - Append returned drifts to `report.drifts`.
  - Call `report.recompute_summary_and_score()`.
- Ensure JSON and markdown output include the new policy drifts.

**AC:** `sruja intent check -r . -i <dir>` with a .sruja that defines a policy and code that violates it reports policy_violations > 0 and lists the violations.

---

## Phase 2: Compliance Report and Command

**Goal:** Single compliance report type and `sruja compliance` command that combine structural drift, intent drift, and policy violations; exit 1 when non-compliant.

### 2.1 ComplianceReport Type (sruja-report)

**File:** `crates/sruja-report/src/compliance.rs` (new)

- Define:
  - `ComplianceStatus { Compliant, NonCompliant }`
  - `ComplianceReport { status, structural_violations: Vec<Violation>, intent_drifts: Vec<IntentDriftEntry>, policy_violations: Vec<PolicyViolationEntry>, boundary_violations_count: u32, remediation_checklist: Vec<String>, health_score: u8 }`
  - Use existing types where possible: e.g. `sruja_diff::Violation` for structural, or minimal DTOs that serialize to the same shape.
- Builder or constructor that takes:
  - structural report (sruja_diff::DriftReport),
  - intent drift report (sruja_intent::DriftReport),
  - and builds status (Compliant iff no errors / no critical drifts), flattened lists, and a short remediation_checklist (e.g. "Fix N structural violations", "Resolve M policy violations").

**File:** `crates/sruja-report/src/lib.rs`

- `pub mod compliance; pub use compliance::{ComplianceReport, ComplianceStatus, ...};`
- sruja-report may need to depend on sruja-intent for DriftReport type, or we use a minimal DTO in report and CLI converts. Prefer minimal DTO in report (no intent dep if possible): e.g. `IntentDriftEntry { kind: String, severity: String, description: String }`, `PolicyViolationEntry { policy_name: String, message: String, source: String, target: String }`.

**AC:** Can build ComplianceReport from structural DriftReport + intent DriftReport; status is NonCompliant when there are errors or policy violations.

### 2.2 sruja compliance Command (CLI)

**File:** `crates/sruja-cli/src/main.rs`

- Add subcommand: `Compliance { repo, architecture: Option<String>, intent: Option<String>, format: String }`.
- `architecture`: path to baseline .sruja (optional). If present, run drift with baseline.
- `intent`: path to intent dir (optional); default to repo/docs/architecture or similar.

**File:** `crates/sruja-cli/src/commands/compliance.rs` (new)

- `compliance(repo, architecture, intent, format)`:
  1. Scan repo.
  2. Structural: run `detect_architectural_drift(&graph)`; if `architecture` given, parse .sruja, `program_to_graph`, `compare_graphs`, get structural violations from diff result.
  3. Intent: load intent dir, merge models, `DriftDetector::detect()`, then `evaluate_policy_violations`, append and recompute.
  4. Build ComplianceReport from (1) and (2).
  5. Print (text or JSON per format); if status == NonCompliant, `std::process::exit(1)`.

**File:** `crates/sruja-cli/src/commands/mod.rs`

- `pub mod compliance; pub use compliance::compliance;`

**AC:** `sruja compliance -r .` runs and prints a compliance report; with `-a file.sruja` and violations, status is non-compliant and exit code 1.

---

## Phase 3 (Optional / Follow-up)

- **Boundary extraction from .sruja:** If DSL gains boundary declarations, extend `from_sruja_file` to populate `intent.boundaries`.
- **Remediation checklist:** From violation types, add concrete hints or doc links.
- **Compliance report JSON schema:** Document for CI consumers.

---

## File Touch List

| Crate | File | Change |
|-------|------|--------|
| sruja-intent | model/mod.rs | Extract Policy from program.items in from_sruja_file. |
| sruja-intent | compare/mod.rs | Add DriftReport::recompute_summary_and_score; expose compute_drift_score/classify_health if needed. |
| sruja-cli | compliance.rs (new) | parse_constraint_to_rule, evaluate_policy_violations. |
| sruja-cli | commands/intent.rs | Call evaluate_policy_violations, append drifts, recompute. |
| sruja-report | compliance.rs (new) | ComplianceReport, ComplianceStatus, builder. |
| sruja-report | lib.rs | Export compliance module. |
| sruja-cli | commands/compliance.rs (new) | compliance() implementation. |
| sruja-cli | commands/mod.rs | Add compliance module. |
| sruja-cli | main.rs | Add Compliance subcommand and dispatch. |

---

## Dependencies

- sruja-report: Add sruja-diff (already?) and optionally avoid sruja-intent by using DTOs; CLI will convert.
- sruja-cli: Already has sruja-intent, sruja-graph, sruja-diff, sruja-report.

---

## Acceptance Criteria (E2E)

1. **Policy from .sruja:** A .sruja file with a policy whose description is "external_api must not call database" is loaded by intent; intent check shows policy violations when the scan graph has such an edge.
2. **Compliance command:** `sruja compliance -r . -a architecture.sruja` produces a single report (text or JSON) with status, structural violations, intent/policy violations, and exits with 1 when there are violations.
3. **CI use case:** A pipeline can run `sruja compliance -r . -a arch.sruja --format json` and fail the build when status is non-compliant.

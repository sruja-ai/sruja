# TIMELINE_PLAN.md — Code Review

**Reviewed against:** current codebase (sruja-cli, sruja-diff, sruja-intent, sruja-scan, evaluation/real-world-test).  
**Purpose:** Verify the plan’s description of existing behavior and proposed changes matches the code; flag inaccuracies and implementation notes.

---

## 1. Current state (Section 2) — **Matches code**

| Claim in plan | Code location | Verdict |
|---------------|----------------|---------|
| `sruja scan . --output out.json` produces one graph JSON per run | `sruja_scan::scan_repo` → `Graph` (serialized as JSON) | ✓ |
| `sruja drift-diff -b base.json -h head.json` diffs two graph JSONs | `commands::drift_diff` in `crates/sruja-cli/src/commands/scan.rs`; `sruja_diff::compare_graphs(&head_graph, &base_graph)` | ✓ |
| `drift_by_commit.sh REPO [BASE] [HEAD]`: checkout base → scan → checkout head → scan → drift-diff; restores branch; does not persist graphs | `drift_by_commit.sh`: temp files `.sruja_graph_base_$$.json` / `.sruja_graph_head_$$.json` in `SCRIPT_DIR`, trap EXIT rm | ✓ |
| `sruja intent check -r . -i DIR`: loads from DIR, default `repo/docs/architecture`; ADRs from `DIR/adr/decisions/*.md` and `*.sruja` under DIR | `intent_check` in `commands/intent.rs`: `intent_dir = intent_path.unwrap_or_else(|| repo_path.join("docs").join("architecture"))`; `sruja_intent::IntentIntelligence::load_from_directory` uses `dir.join("adr").join("decisions")` for .md and walkdir for .sruja | ✓ |
| ADR parser in `crates/sruja-intent/src/parser/adr.rs`; path `dir.join("adr").join("decisions")` only | `sruja-intent/src/lib.rs` line 34: `let adr_dir = dir.join("adr").join("decisions");` | ✓ |

---

## 2. drift-diff JSON and report mapping (Sections 3.5, 4.3, 11) — **Matches**

- **DiffResult / DiffSummary:**  
  `crates/sruja-diff/src/types.rs`: `DiffResult { summary: DiffSummary, ... }`, `DiffSummary { proposed_components, existing_components, new_components, missing_components, new_dependencies, removed_dependencies, health_score }`.
- **Semantics:**  
  `scan.rs` comment (lines 34–35, 66–68): actual = head, proposed = base → `missing_components` = new at head, `new_components` = removed since base. Plan’s mapping is correct:
  - report `new_components` := `summary.missing_components`
  - report `removed_components` := `summary.new_components`
  - `new_edges` := `summary.new_dependencies`, `removed_edges` := `summary.removed_dependencies`
- **drift-diff -f json:**  
  Supported: `Commands::DriftDiff { format }` and `drift_diff(..., format)`; `format == "json"` → `serde_json::to_string_pretty(&diff_result)`.
- **Violations:**  
  `DiffResult.violations` use `Severity::Error | Warning | Info`; plan’s `violations_summary: { errors, warnings }` is implementable by counting by severity.

---

## 3. Gaps and scope — **Accurate**

- No multi-ref persisted capture, no ref selection from tags/branches, no chained timeline report: correct.
- ADR discovery only under one dir and only `.../adr/decisions` for .md: matches `sruja-intent`; plan’s “many OSS use docs/adr/ or doc/adr/” is a valid gap.

---

## 4. Intent and ADR index (Phase 3) — **Needs one clarification**

- **Multiple `-i` / SRUJA_ADR_DIRS:**  
  Today `sruja intent check` takes a single `-i` (Option<String>). Plan’s “repeatable -i or SRUJA_ADR_DIRS” is a correct description of the desired change.
- **ADR index export:**  
  Current `sruja intent check -f json` prints `IntentReport` (drift-focused: intent_source, reality_source, drift_score, health, violations, summary counts, suggestions). It does **not** include a list of parsed ADRs (path, number, title, status, date, tags). So Phase 3’s “extend intent check -f json to include adrs: [...] or add sruja adr-index” is accurate — either extend the JSON or add a dedicated command; the plan does not assume existing ADR list output.

---

## 5. Scripts and layout — **Not yet implemented**

- **capture_timeline.sh** and **timeline_report.sh** are not present under `evaluation/real-world-test/`. The plan describes deliverables, not current state; no code conflict.
- **lib.sh:**  
  `find_sruja` exists and is used by `drift_by_commit.sh`; plan’s “Dependencies: find_sruja (lib.sh)” is correct.  
  Note: `find_sruja` resolves relative to `$(dirname "${BASH_SOURCE[0]}")/../..` (repo root), so when sourced from `evaluation/real-world-test/`, it points to repo root and then to `target/debug/sruja` or `target/release/sruja` — correct for capture/report scripts in the same dir.

---

## 6. Test repos and examples — **Minor doc note**

- Plan and TEST_ON_REAL_PROJECTS.md mention gitea, etcd, caddy, react-admin, saleor, express.  
- **test-repos/MANIFEST.md** currently lists only **express** and **fastapi** (and mislabels express as “Language: Python”; express is Node.js). So “gitea” and others are either added by another setup (e.g. `setup_repos.sh --complex`) or are placeholders. Implementers should confirm which repos exist when running examples like `./capture_timeline.sh gitea`.

---

## 7. Suggested plan edits (optional)

1. **Section 4.3 (timeline report):**  
   Explicitly state that when invoking `sruja drift-diff -f json`, the script must parse **stdout** (the single JSON object), not a file, unless the CLI is later extended to support `-o file`.

2. **Section 3.4 (ADR index), status field:**  
   Plan shows `"status": "Accepted"`. Code uses `AdrStatus` enum (Proposed, Accepted, Deprecated, Superseded { by }, Rejected, Draft). Serialization will be string (e.g. "Accepted", "Superseded"); if “Superseded” is serialized with the `by` value, document the exact format (e.g. `"Superseded"` with optional `"superseded_by": 20`).

3. **Appendix A (default branch):**  
   Plan’s “fall back to master” and “git symbolic-ref refs/remotes/origin/HEAD” matches real-world usage; no code change needed, but implementers can reuse the same pattern as in other scripts (e.g. avoid assuming `main` exists).

---

## 8. Summary

| Area | Verdict |
|------|--------|
| Current state table (scan, drift-diff, drift_by_commit.sh, intent, ADR path) | Matches code |
| DiffResult/DiffSummary and drift-diff JSON mapping | Correct |
| drift-diff -f json support | Present |
| Intent: single -i, no ADR list in JSON | Correct; Phase 3 correctly describes new work |
| capture_timeline.sh / timeline_report.sh | Not in repo; plan is forward-looking, no conflict |
| lib.sh find_sruja | Exists and suitable for new scripts |
| Test repos (gitea etc.) | MANIFEST lists express/fastapi; clarify or add repos for examples |

**Conclusion:** TIMELINE_PLAN.md is consistent with the current codebase. No code changes are required for the plan to be accurate; the only recommendations are the small documentation clarifications above and confirming test-repo setup for the example commands.

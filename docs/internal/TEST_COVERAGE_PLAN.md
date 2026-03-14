# Test Coverage Plan

Structured plan for adding missing unit and integration tests across the Sruja workspace. Tasks are ordered by priority (P0 = critical path, P1 = important, P2 = improve coverage).

---

## Summary: Modules Without Tests

| Crate | Module(s) | Priority | Notes |
|-------|-----------|----------|--------|
| sruja-diff | convert.rs, source_ref.rs | P0 | Core comparison pipeline; convert used by tests.rs indirectly |
| sruja-diff | drift.rs | P0 | Drift detection; covered partially by drift_e2e, no unit tests |
| sruja-report | compliance.rs | P1 | Covered by report_test.rs; optional inline unit tests |
| sruja-diagnostics | format.rs | P0 | format_diagnostic has doc-test; add unit tests |
| sruja-diagnostics | types.rs, codes.rs, reporter.rs | P1 | DTOs/codes; add serde or invariant tests if needed |
| sruja-engine | rules/simplicity.rs | P1 | Rule returns empty; test name + empty program |
| sruja-engine | validator/config.rs, core.rs, builder.rs | P1 | Used by validator; add builder/config tests if public API |
| sruja-export | json/exporter.rs | P1 | Covered by json_export tests; add unit tests for edge cases |
| sruja-export | context/exporter.rs, dsl/printer/* | P2 | Context covered by context_export; printer submodules |
| sruja-scan | npm.rs, cargo.rs | P1 | npm_smoke exists; add unit tests for resolve_workspace_globs etc. |
| sruja-scan | tree_sitter/detector.rs | P2 | Language detection logic |
| sruja-intent | parser/mod.rs, model/mod.rs | P2 | Parser has adr tests; module wiring |
| sruja-semantic | analysis/*, vocabulary/graph.rs | P2 | Analysis and vocabulary |
| sruja-language | parser/* (elements, primitives, etc.) | P2 | Parser tests.rs + example_files cover many paths |
| sruja-cli | config.rs, views.rs, commands/* | P2 | E2E covers commands; config/views unit tests |
| sruja-lsp | server.rs | P2 | server_integration tests; optional unit |
| sruja-wasm | lib.rs | P2 | WASM target; browser or wasm-bindgen-test |

---

## Task List (Structured)

### Phase 1: Core diff and diagnostics (P0)

1. **sruja-diff convert.rs**
   - Add `#[cfg(test)] mod tests` in convert.rs.
   - Test `program_to_graph` with empty program (empty graph).
   - Test with one system + one container (nodes and optional edge).
   - Test `relation_label_to_edge_kind` via program_to_graph (reads -> ReadsFrom, writes -> WritesTo, calls -> Calls).

2. **sruja-diff source_ref.rs**
   - Functions are `pub(super)`; test via compare/drift or add a small public test helper.
   - Option: test `collect_cycle_sources`, `collect_edge_sources`, `collect_node_path_source` with minimal Graph in a tests/ or in lib.rs test module that uses diff internals.

3. **sruja-diagnostics format.rs**
   - Add unit tests for `format_diagnostic`: with/without context, with/without suggestions, severity and code in output.

### Phase 2: Report, engine, export (P1)

4. **sruja-report compliance.rs**
   - Already covered by report_test.rs (ComplianceReport::from_parts, status, serde). Optional: add inline `#[cfg(test)]` tests for remediation_checklist content and boundary-only case.

5. **sruja-engine rules/simplicity.rs**
   - Test rule name, and that empty program returns empty diagnostics; non-empty program returns empty (current behavior).

6. **sruja-export json exporter**
   - Existing json_export tests; add unit tests in json/exporter.rs for empty program -> "{}", export_compact, and one non-empty program roundtrip.

7. **sruja-scan npm.rs**
   - Test `resolve_workspace_globs` (if made pub(crate) or exposed for testing) or add integration test with temp package.json. Alternatively add unit tests for scan_npm_repo with minimal package.json in tests/.

### Phase 3: Drift and integration (P1)

8. **sruja-diff drift.rs**
   - Unit tests for `find_circular_dependencies`, `find_orphan_modules`, `find_layer_violations_advanced`, `find_god_modules` with minimal Graph instances. These are the core helpers used by detect_architectural_drift.

### Phase 4: Remaining crates (P2)

9. **sruja-diagnostics** codes – **Done** (codes.rs: E1xx/E2xx/E3xx/W001/E4xx format tests).
10. **sruja-export** context/exporter – **Done** (empty program, unknown template → general header).
11. **sruja-cli** config – **Done** (get_builtin_views, empty yaml, get_view, AnalysisDepth, ThresholdConfig).
12. **sruja-scan** cargo.rs – **Done** (missing Cargo.toml, invalid Cargo.toml → error).
13. **sruja-semantic** analysis/coupling – **Done** (empty analyze, length mismatch, default threshold).
14. **sruja-language** parser submodules – optional (parser/tests.rs, example_files).
15. **sruja-lsp** server – optional unit tests for request handling.

---

## Execution Order (This Session)

1. Create this plan (done).
2. sruja-diff: convert.rs unit tests — **Done** (empty program, writes→WritesTo, single node).
3. sruja-diff: source_ref.rs — **Done** (test_drift_cycle_violation_includes_source_refs_when_edges_have_evidence in tests.rs).
4. sruja-diagnostics: format.rs unit tests — **Done** (code/severity/message, location, context, suggestions).
5. sruja-engine: simplicity.rs unit tests — **Done** (rule name, empty program, non-empty program).
6. sruja-report: compliance — **Skipped** (already covered by report_test.rs).
7. sruja-export: json exporter unit tests — **Done** (empty export/compact, compact valid JSON, with_extended).
8. sruja-scan: npm unit tests — **Done** (invalid JSON and missing file error paths in npm.rs).
9. sruja-diff: drift.rs unit tests — **Done** (find_circular_dependencies, find_orphan_modules).

### Phase 4 (continued)

10. sruja-diagnostics: codes.rs — **Done** (code constant format tests).
11. sruja-export: context/exporter.rs — **Done** (empty program, unknown template).
12. sruja-cli: config.rs — **Done** (builtin views, deserialize, get_view, AnalysisDepth, ThresholdConfig).
13. sruja-scan: cargo.rs — **Done** (missing/invalid Cargo.toml error paths).
14. sruja-semantic: analysis/coupling.rs — **Done** (empty analyze, mismatch, default).

---

## Test Conventions

- Prefer `#[cfg(test)] mod tests` in the same file for unit tests.
- Use existing test helpers (e.g. sruja_diff::tests, sruja_scan::Graph::new).
- Integration tests stay in `crates/*/tests/*.rs`.
- Run after each batch: `cargo test -p <crate> --no-fail-fast`.

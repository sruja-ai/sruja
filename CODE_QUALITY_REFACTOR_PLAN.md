# Code Quality Refactor Plan

This document tracks the code quality review (large files, test gaps, Clippy, TODOs) and the plan to address it.

## Completed

### Clippy (34 → 0 warnings)

- **sruja-diff**: Removed redundant `score.max(0)` (u8 already ≥ 0 after `saturating_sub`).
- **sruja-graph**: Collapsed nested `if` in `merge_edge`; `split(['.', '/', '_'])` instead of manual char comparison; introduced `BrandesBfsResult` type alias in centrality; `#[allow(clippy::too_many_arguments)]` on Tarjan `strongconnect` (stateful algorithm).
- **sruja-intent**: `is_some_and` instead of `map_or(false, ...)`; regex moved out of loop in ADR title extraction; `match` replaced with `if` for single variant; collapsed nested `if`; removed unused `mut`/`let`; `#[allow(dead_code)]` on `DriftDetector::config` (reserved for future use).
- **sruja-semantic**: `clamp(0.0, 1.0)` / `clamp(0.0, 100.0)` instead of `.max().min()`; removed needless borrow in `components_using_term(term)`.
- **sruja-cli**: Removed redundant `use sruja_diff`; replaced closures with `CliError::Json` and `SourceRef::display_string`; `is_none_or` instead of `map_or`; `CliError::Io` instead of redundant closure.
- **sruja-export**: `#[allow(clippy::only_used_in_recursion)]` on `print_element` (recursive by design); `#[allow(clippy::too_many_arguments)]` on `write_toc`.

Run: `cargo clippy --all-targets --all-features -- -W clippy::all` to confirm clean.

---

## Large Files (>500 lines)

| File | Lines | Recommendation |
|------|-------|----------------|
| `crates/sruja-language/src/parser.rs` | ~2307 | Split into `lexer.rs`, `parser.rs`, `combinators.rs` (or expression/statement/decl submodules). |
| `crates/sruja-engine/src/validator.rs` | ~1290 | Group rules into `rules/` submodules (e.g. `rules/cycle.rs`, `rules/governance.rs`) and keep validator as orchestrator. |
| `crates/sruja-mcp/src/tools.rs` | ~1167 | Split by domain: e.g. `tools/scan.rs`, `tools/analyze.rs`, `tools/drift.rs`, `tools/lint.rs`. |
| `crates/sruja-export/src/dsl/printer.rs` | ~1164 | Consider `printer/` with blocks for components, relationships, scenarios. |
| `crates/sruja-diff/src/lib.rs` | ~1084 | Extract submodules: e.g. `diff/`, `health/`, `drift_config.rs`. |
| `crates/sruja-diagnostics/src/lib.rs` | ~960 | Split by concern: severity, source location, formatting, collection. |
| `crates/sruja-language/src/ast.rs` | ~921 | Acceptable for AST definitions; optional split by AST node kind if it grows. |

**Suggested order**: parser split (highest impact), then validator rule extraction, then MCP tools and diff submodules.

---

## Test Coverage Gaps

| Crate | Action |
|-------|--------|
| sruja-engine | Added `tests/validator_test.rs`: parse + `validate_sync` with default rules. |
| sruja-graph | Added `tests/scc_test.rs`: `SccAnalyzer` on small graphs. |
| sruja-intent | Added `tests/drift_test.rs`: `DriftDetector` and `IntentModel` basics. |
| sruja-diagnostics | Add unit tests for `Diagnostic`, severity, and source location. |
| sruja-diff | Add tests for diff logic and health score. |
| sruja-runtime | Add tests for trace loading and analysis. |
| sruja-semantic | Add tests for clustering and similarity. |

Priority: expand engine, graph, and intent tests (invariants, boundary cases, regression).

---

## TODOs / FIXMEs

Files with TODOs or FIXMEs to triage:

- `parser.rs`, `cycle.rs`, `scenario_validation.rs`, `features.rs`, `server.rs`, `intent.rs`

Recommendation: open a tracking issue or add a short comment with ticket/context for each; resolve or document deferral.

---

## Next Steps

1. **Parser split**: Introduce `lexer`/combinator modules and move parsing helpers from `parser.rs` in small steps; keep `Parser` API unchanged.
2. **Validator**: Extract one or two rules into `rules/` (e.g. `cycle`, `governance`) and wire from `validator.rs`; repeat for remaining rules.
3. **Tests**: Add invariant and regression tests for engine rules, graph SCC/centrality, and intent drift; run `cargo test --workspace` regularly.
4. **TODOs**: Audit and either fix or document each TODO/FIXME with owner or ticket.

# CLI & MCP Consolidation Summary

## CLI Consolidation (Complete)

### Phase 1: Remove Deprecated Commands
- Removed `Commands::Check`, `Commands::Evaluate`, `Commands::Evolution` enum variants
- Removed `EvolutionCommand` subcommand
- Removed `alias = "context"` from `AiContext`
- Removed `was_invoked_as()` helper
- Removed deprecated dispatch arms in `run.rs`

### Phase 2: Update Documentation
- Updated `init`/`start` doc comments to point to `sruja inspect quickstart` and `sruja inspect onboard`
- Updated CI workflow (`.github/workflows/sruja-check.yml`) from `sruja check` to `sruja drift --ci`
- Updated all test files to use new command names
- Updated handler references across 20+ files

### Phase 3: Update Help Text
- Updated `status.rs`, `health.rs`, `context_score.rs` help text to say "Use `sruja status` for a unified view"
- Updated `Status` doc comment to "Unified repo status: truth freshness, structural health, AI readiness, density, agent memory"

### Phase 4: Add Format Flags to AI Command
- Added `--format markdown|json|for-ai` and `--cache-friendly` flags to `ai` command
- `json`/`for-ai` delegate to `context_export` with `Vec<String>` repo wrapping

### Phase 5: Un-hide Grouped Commands
- Un-hid `propose`, `intent`, `event`, `memory`, `decision` in `commands.rs`

## MCP Tool Consolidation (Complete)

### Tool Profile Alignment
Aligned tool profiles with `docs/FEATURE_CONSOLIDATION.md`:

**Minimal (10 tools):**
- `sruja_list_architecture_index`
- `sruja_get_topology`
- `sruja_get_elements`
- `sruja_get_task_context`
- `sruja_get_focus_briefing`
- `sruja_get_drift_state`
- `sruja_verify_task`
- `sruja_search_memory`
- `sruja_check_drift`
- `sruja_get_repomap`

**Coding (15 tools) - Default:**
All minimal tools plus:
- `sruja_get_author_evidence`
- `sruja_hybrid_query`
- `sruja_explain_discovery`
- `sruja_critique`
- `sruja_suggest_context_prune`

**Arch (17 tools):**
All coding tools plus:
- `sruja_explain_element` (read paths)
- `sruja_evaluate_proposal` (read paths)

**Full:**
All tools (backward compatible) - ~63 tools

### Documentation Updates
Updated tool counts in:
- `docs/HOST_AGENT_INTEGRATION.md`
- `docs/mcp_setup.md`
- `docs/MESSAGING.md`
- `docs/FEATURE_TIERS.md`
- `.architecture-skill.md`
- `docs/FEATURE_CONSOLIDATION.md`
- `book/src/reference/cli.md`

## Key Decisions

1. **`ai` absorbs `ai-context` formats**: `--format json|for-ai` delegates to `context_export`; `ai-context` stays hidden for IDE formats
2. **`status` is the unified health command**: Already showed health + context scores; fixed misleading text
3. **`quickstart`/`onboard` not merged into `start`**: Too different (read-only analysis vs file-creating setup)
4. **Handler code preserved**: `check.rs` (shared with `drift --ci`), `evolution.rs` (shared with `intent history`)
5. **`drift --ci -f text` overrides to `github-actions`**: Existing behavior in `run.rs`

## Verification

- All tests pass (12 CLI parse, 190 unit, 6 E2E)
- Build clean
- Formatting clean
- No remaining references to deprecated commands

## Status

- **VS Code extension commands**: Already well-organized with context menu groups (`1_harness`, `2_export`, `3_diagram`, `9_advanced`) and Command Center as unified entry point
- **Init command flags**: Already clean - no duplicate flags exist. Documentation points to `sruja inspect quickstart` and `sruja inspect onboard`
- **CLI tests**: 190 tests passing
- **Pre-existing issues**: `sruja-graph` and `sruja-intent` have missing `auto_context` field errors (unrelated to consolidation)

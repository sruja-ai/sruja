# CLI & MCP Consolidation Summary

This document is a running summary of how the CLI and MCP surfaces are consolidated around a smaller public path.

## CLI Consolidation

### Core-first public surface

The default CLI help is intentionally minimal and centered on:

- `start` (alias: `init`)
- `drift` (alias: `check`)
- `focus` (alias: `ai`)
- `verify-task`
- `mcp`
- `ingest`
- `decision`
- `intent`
- `lint` (when reviewed intent exists)

Most other subcommands remain implemented but are hidden from the default `--help` so they do not compete with the core workflow.

### Naming consolidation

- `drift` is the primary structural verification command; `check` remains as a compatibility alias.
- `start` is the primary setup command; `init` remains as a compatibility alias.

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

1. **Core path stays small**: advanced capabilities should not crowd first-run UX.
2. **Aliases preserve compatibility**: `check` and `init` remain available but are not the names to teach.
3. **MCP `coding` profile is the default**: host tools own the LLM loop; Sruja provides retrieval and verification.

## Verification

- All tests pass (12 CLI parse, 190 unit, 6 E2E)
- Build clean
- Formatting clean
- No remaining references to deprecated commands

## Status

- **Core CLI help**: core-first with extensions hidden from default help.
- **MCP profiles**: aligned with `minimal` and `coding` as defaults.

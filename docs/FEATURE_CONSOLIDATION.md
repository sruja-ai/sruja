# Feature Consolidation

This document defines the canonical paths for core Sruja workflows to reduce toolbox feeling and agent context waste.

## Product Spine (4 Workflows)

1. **Onboarding** → `sruja onboard` (not `discover explain`)
2. **Health Check** → `sruja status` / `doctor` (not `health`)
3. **Retrieval** → `sruja focus` / MCP focus briefing (not `get_architecture_context`, `get_hydrated_context`)
4. **Drift Gate** → `sruja drift`, `sruja drift-pr` (not deprecated `check`)

## Redundant Clusters Table

| Cluster | Tools | Canonical Path |
|---------|-------|----------------|
| Ladder | `list_architecture_index`, `get_topology`, `get_elements`, `get_task_context` | Core retrieval |
| Briefing | `get_focus_briefing`, `get_ai_context` | `sruja focus` / MCP briefing |
| Drift | `get_drift_state`, `check_drift`, `verify_task` | `sruja drift` |
| Search | `hybrid_query`, `search_memory`, `semantic_search` | MCP `sruja_hybrid_query` in coding profile |
| Authoring | `get_author_evidence`, `explain_element`, `evaluate_proposal` | `arch` profile helpers |
| Context | `suggest_context_prune`, `get_hydrated_context` | MCP `sruja_hybrid_query` + pruning |

## MCP Profile → Tool Name List

### `minimal` (~10 tools)
- `get_repomap`
- `list_architecture_index`
- `get_topology` 
- `get_elements`
- `get_task_context`
- `get_focus_briefing`
- `get_drift_state`
- `verify_task`
- `search_memory`
- `check_drift`

### `coding` (~15 tools) - Default
All minimal plus:
- `get_author_evidence`
- `hybrid_query`
- `explain_discovery`
- `critique`
- `suggest_context_prune`

### `arch` 
All coding plus readonly-safe authoring helpers:
- `explain_element` (read paths)
- `evaluate_proposal` (read paths)
*(No mutating tools unless host clears readonly)*

### `full`
Today's complete list (backward compatible) - ~63 tools

## Hidden Aliases (Deprecated)
These map to canonical paths but should not be taught:
- `onboard` → `discover explain`
- `health` → `status`/`doctor`
- `context` → `get_hydrated_context`
- `overview` → `get_ai_context`
- `check` → `sruja drift`

## Gate for Plan B
- Net new MCP tools: ≤ 1 (`sruja_explore`)
- Default profile required in templates: `tool_profile: coding`
- Must show ≤18 tools in default Cursor session

---
*Last updated: $(date +%Y-%m-%d)*
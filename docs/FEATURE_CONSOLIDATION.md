# Feature Consolidation

Canonical paths for OSS traction. Public tiers: [FEATURE_TIERS.md](./FEATURE_TIERS.md).

## Product spine (OSS — three workflows)

1. **Scan + drift** → `sruja start`, `sruja drift --structural-only --advisory` (no `.sruja` required)
2. **Brief agent** → `sruja focus`, `sruja ai`; MCP `coding` profile (15 tools, no `sruja_agent_run`)
3. **Verify** → `sruja verify-task`; `lint` / `drift -a repo.sruja` when reviewed intent exists

Team-only: `onboard`, `workflow`, `propose`, `drift-pr`, federation (Tier 2).

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

### `minimal` (10 tools)
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

### `coding` (15 tools) - Default
All minimal plus:
- `sruja_get_author_evidence`
- `sruja_hybrid_query`
- `sruja_explain_discovery`
- `sruja_critique`
- `sruja_suggest_context_prune`

### `arch` (17 tools)
All coding plus readonly-safe authoring helpers:
- `sruja_explain_element` (read paths)
- `sruja_evaluate_proposal` (read paths)
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
- Must show 15 tools in default Cursor session

---
*Last updated: $(date +%Y-%m-%d)*
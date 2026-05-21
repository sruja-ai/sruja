# Feature Consolidation

Canonical paths for OSS traction. Public tiers: [FEATURE_TIERS.md](./FEATURE_TIERS.md).

## Product spine (OSS — three workflows)

1. **Scan + drift** → `sruja start`, `sruja drift --structural-only --advisory` (no `.sruja` required)
2. **Brief agent** → `sruja focus`, `sruja ai`; MCP `coding` profile (≤18 tools, no `sruja_agent_run`)
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
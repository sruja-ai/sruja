# Feature tiers (OSS traction)

Public story references **Tier 1 only**. Tier 2 is documented under Advanced; Tier 3 is merged away or removed after deprecation (see [CHANGELOG](../CHANGELOG.md)).

## Tier 1 — OSS public (three workflows)

| Workflow | Commands | MCP (`coding` profile) |
|----------|----------|------------------------|
| **Scan + drift** | `start` (`init`), `drift --structural-only`, `sync` | `sruja_check_drift`, `sruja_get_drift_state` |
| **Brief agent** | `focus`, `ai` | Ladder + `sruja_get_focus_briefing`, `sruja_get_task_context`, `sruja_hybrid_query` |
| **Verify** | `verify-task`, `lint` (when `repo.sruja` exists) | `sruja_verify_task` |

**Also Tier 1:** `mcp` (stdio server; default `SRUJA_MCP_TOOL_PROFILE=coding`, ≤18 tools).

**Tier 1b (optional):** `sruja-architecture` skill — promotes scan evidence to reviewed `repo.sruja`. Not required day one.

### Not in default MCP coding profile

- `sruja_agent_run` — use host editor loop + `verify-task`
- `get_hydrated_context` / `get_architecture_context` — use ladder + `hybrid_query`

## Tier 2 — Power user / team

`propose`, `intent`, `impact`, `critique`, `ingest`, `author`, `publish`/`compose`, `review`/`daily`, `workflow`, `agent plan`, `agent record`, `export`, `fmt`, `tree`, `drift-pr`, federation.

Diagram preview (extension) is Tier 2 — derived views, not the product center.

## Tier 3 — Removal backlog (merge → deprecate → delete)

| Fold into | Sources |
|-----------|---------|
| `drift --ci` | `check` |
| `ai` | `ai-context`, `context` |
| `start` | `quickstart`, `overview`, `onboard` |
| `status` | `doctor`, `health` (metrics) |
| `review` | `daily` |
| `intent evaluate` | top-level `evaluate` |
| Host verify | `agent run` (hidden; plan-only via `agent plan`) |

**Deferred (after core three are excellent):**

- **discovery-primary** — document scan IR as default overlay path (not OSS hero yet).
- **community-rehab** — one honest technical post after dogfood output is repeatable ([OSS_METRICS.md](./OSS_METRICS.md)).
- Registry dashboards, new MCP category tools.

See [FEATURE_CONSOLIDATION.md](./FEATURE_CONSOLIDATION.md) for the full merge matrix.

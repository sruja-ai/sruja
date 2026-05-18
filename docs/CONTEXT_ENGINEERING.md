# Sruja Context Engineering Platform

Sruja is more than an architecture-as-code tool; it is a **Context Engineering** platform designed to provide high-fidelity, task-scoped context for AI coding agents.

By quantifying and surfacing architectural evidence, Sruja reduces the "context gap" that leads to AI hallucinations and architectural drift.

## Context graphs: Sruja vs industry usage

Industry articles often use **“context graph”** to mean a **decision lineage** layer: time-stamped decision events, exceptions, approvals, and precedent—so agents can answer “how did we handle this before?” Sruja uses the same phrase for something adjacent but distinct: a **governed structural + intent graph** for a repository (see [domain-schema.md](architecture/domain-schema.md)). That graph encodes what *should* be true, validates relationships, and links evidence—not full enterprise decision desks.

Sruja still approximates parts of the decision-trace idea for **software architecture**:

| Layer (common framing) | In Sruja |
|------------------------|----------|
| 1. Execution / observability | `sruja sync`, repository scan, manifests, SCIP—signals from the codebase and tooling |
| 2. Semantic aggregation | Boundary inference, file→element binding, focus hydration, hybrid query routing |
| 3. Graph + time | `.sruja` declarations, drift vs baseline, `sruja context` with `--base-ref` / `--head-ref`, optional **temporal block** on `sruja focus` |
| 4. Agent memory | `.sruja/agent_memory.json`, run snapshots under `.sruja/runs/`, **facts bundles** under `.sruja/agent/runs/<run_id>/`, append-only **context events** in `.sruja/context_events.jsonl` (intent check, drift, merged proposals) |

**Context events** (`context_events.jsonl`) and **MCP** tools `sruja_get_context_events` / `sruja_get_agent_learnings` expose that lineage to agents without conflating it with the declared architecture graph itself. Decision and workflow traces use **`context_event/v2`** rows (optional `trace_id`, `decision_id`, `actor`, `source`, and related fields); use `sruja event append` / `sruja decision trace` to record and inspect them.

For the portable **governed context + decision lineage** model—Decision Records (generalized ADRs), hypothesis vs reviewed truth, reference *code-to-production* workflow, HITL taxonomy (`precedent`, `exception`, `correction`, `guardrail`), and federation notes—see **[context-graph-for-agents.md](context-graph-for-agents.md)**.

## Core Pillars

### 0. AI Coding Brief
Before starting an AI-assisted coding session, run `sruja ai` to generate a paste-ready brief that combines the task, current worktree, architecture health, context score, changed files, guardrails, verification commands, and task-scoped JSON context.

**Command:** `sruja ai -r . --task "Fix parser diagnostics"` or `sruja ai -r . --file crates/sruja-cli/src/main.rs`

### 1. Context Score (AI-Readiness)
The **Context Score (0–100)** is the "Lighthouse score" for your repository's context. It measures five critical dimensions:
- **Architecture Coverage**: % of code modules mapped in your `.sruja` files.
- **Decision Completeness**: How many ADRs and decisions are linked to architecture elements.
- **Evidence Freshness**: How recently the architectural evidence was refreshed via `sruja sync`.
- **Relationship Density**: The connectivity of your architecture graph.
- **External Context**: The availability of non-code context (ADRs, design docs, etc.).

**Command:** `sruja context-score` or `sruja status`

### 2. Task-Scoped Briefing (Focus)
Before an AI agent starts a task, it needs to know the specific architectural constraints and impact area. `sruja focus` generates a comprehensive briefing including:
- **Blast Radius**: Upstream and downstream impact analysis.
- **Linked Decisions**: Active ADRs affecting the target.
- **Boundary Constraints**: Inferred policy violations.
- **AI Instructions**: Specific guidance for the LLM.

**Command:** `sruja focus --file <path>` or `sruja focus --element-id <id>`. Optional git range: `sruja focus --element-id MySystem.Api --base-ref main --head-ref HEAD` adds diff-mapped components and architecture fingerprints for that range.

### 3. Documentation Ingestion
Import external context (Design Docs, RFCs, ADRs) into the `.sruja/context/` directory. Sruja automatically indexes these files and links them to architectural components via YAML front-matter.

**Command:** `sruja ingest <path-to-doc>`

**Front-matter Example:**
```yaml
---
elements: [Payment.Service, Database.Transactions]
category: adr
---
# ADR-005: Idempotency keys for payments
...
```

## Integrating with AI Agents

### MCP (Model Context Protocol)
Sruja provides an MCP server that exposes these context engineering capabilities directly to AI editors (Cursor, Trae, Windsurf, etc.).

MCP is the **structured tooling** interface: it answers what grounded architecture data the session can retrieve, while the host application handles multi-step reasoning and any multi-agent routing. For a concise map from common agentic-orchestration literature (sequential pipelines, hierarchical coordinators, A2A, and so on) to Sruja’s role, see [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md).

**Tools:** dozens of read/query tools plus a smaller set of mutating tools (proposals, scratchpad, sandbox, agent run). See the full table in **[mcp_tools_reference.md](mcp_tools_reference.md)**. For locked-down hosts, set `SRUJA_MCP_READONLY=1` so only read/query tools are listed and callable; set `SRUJA_MCP_LOG=1` for one JSON line per invocation on stderr.

**Highlights:**
- **Progressive disclosure ladder:** `sruja_list_architecture_index` → `sruja_get_topology` → `sruja_get_elements` (each response includes token estimates and `next_suggested_tool`).
- `sruja_get_context_score`: Repository-level AI-readiness.
- `sruja_get_focus_briefing`: Task-scoped briefing (optional `base_ref` / `head_ref` for temporal context).
- `sruja_get_task_context`: Task hydration; set `cache_friendly: true` (MCP) or `sruja ai-context -f for-ai --cache-friendly` for invariant/tools/volatile JSON suited to prompt caching.
- `sruja_get_architecture_context`: Component-level hydration (prefer the ladder for large repos).
- `sruja_get_context_events`: Recent intent/drift/proposal-merge events from `.sruja/context_events.jsonl`.
- `sruja_get_agent_learnings`: Agentic Memory entries for an element ID.
- `sruja_get_diagnostic_full`: Fetch full linter/diagnostic text when MCP output was head/tail truncated.
- **MCP resources:** `sruja://context/invariant.md`, `sruja://architecture/main`, and related URIs (see [mcp_tools_reference.md](mcp_tools_reference.md#mcp-resources-resourceslist-resourcesread)).
- **MCP prompts:** `sruja_mcp_guide`, `sruja_review_change`, and other templates via `prompts/get`.
- **Editor sync:** `sruja sync-ide-rules -r .` regenerates `.cursorrules`, Copilot/Claude/Gemini rules, and `llms-architecture.txt`; `--check` for CI drift on generated files.
- **Session pruning:** `sruja_suggest_context_prune` — graph-aware `keep_ids` / `compress_ids` for long agent sessions.
- **Drift injector:** `sruja_get_drift_state` — structured `drift_state/v1` block for host middleware. MCP hosts can set `initializationOptions.watch_drift: true` to receive `notifications/drift_state` after initialize; VS Code: **Sruja: Refresh architecture state**.

### PR & CI Integration
You can use the context score as a gate in CI. If a PR significantly drops the context score (e.g., by adding many unmapped modules), the build can fail, ensuring context stays fresh as the codebase grows.

**Command:** `sruja daily` (alias for `sruja review`) shows the score in the daily dashboard.

## Best Practices
1. **Sync Daily**: Run `sruja daily` every morning to refresh evidence.
2. **Link Everything**: Use the `elements:` field in your markdown docs to link them to the architecture.
3. **Fix Quick Wins**: `sruja context-score` provides a list of "Quick Wins"—tasks that provide the highest ROI for your context health.

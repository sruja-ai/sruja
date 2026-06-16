# Sruja Context Engineering Platform

Sruja is an AI coding agent that provides high-fidelity, task-scoped context for software development. By quantifying and surfacing architectural evidence, Sruja reduces the "context gap" that leads to AI hallucinations and architectural drift.

## Context graphs: Sruja vs industry usage

Industry articles often use **“context graph”** to mean a **decision lineage** layer: time-stamped decision events, exceptions, approvals, and precedent—so agents can answer “how did we handle this before?” Sruja uses the same phrase for something adjacent but distinct: a **governed structural + intent graph** for a repository (see [domain-schema.md](architecture/domain-schema.md)). That graph encodes what *should* be true, validates relationships, and links evidence—not full enterprise decision desks.

Sruja still approximates parts of the decision-trace idea for **software architecture**:

| Layer (common framing) | In Sruja |
|------------------------|----------|
| 1. Execution / observability | `sruja sync`, repository scan, manifests, SCIP—signals from the codebase and tooling |
| 2. Semantic aggregation | Boundary inference, file→element binding, focus hydration, hybrid query routing |
| 3. Graph + time | `.sruja` declarations, drift vs baseline, `sruja ai-context` with `--base-ref` / `--head-ref`, optional **temporal block** on `sruja focus` |
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

---

## Context Graph for Agents

Sruja is the **portable governed context layer** for any agent host (Cursor, Claude Code, CI, Slack bots, internal runners). The host orchestrates steps; Sruja holds **reviewed structure**, **decision lineage**, **append-only traces**, and **hypotheses** that are not promoted to truth without human review.

### Product boundary

**Core promise**

- Reviewed truth in `repo.sruja` (and optional domain schemas).
- **Decision Records** (generalized ADRs): reviewed "why" with stable IDs, status, and links to graph elements.
- **Evidence and lineage** via scans, `.sruja/context_events.jsonl`, run snapshots, and agent memory.
- Task-scoped retrieval through `sruja focus`, `sruja ai`, and MCP tools.
- Validation before risky action: drift, intent, compliance, and explicit trace events.

**Non-goals**

- Sruja is **not** the full agent runtime or a general business workflow engine (no Slack/Jira/Salesforce automation product).
- **Model-extracted or ingested text is never reviewed truth** until it passes human review and proposal flows.

### Decision Record (DR) — generalized ADR

An **ADR** is a **profile** of a Decision Record: typically `type: architecture`, linked elements, and `category: adr` when ingested through `sruja ingest`.

Decision Records also cover product, operational, governance, and agent decisions. Store human-reviewed records as Markdown with YAML front matter under **`.sruja/decisions/`** (gitignored by default in some setups—copy to `docs/decisions/` if you want them in version control).

#### Front matter template

```yaml
---
id: DR-2026-001
type: architecture   # architecture | product | operational | security | agent | exception
status: proposed     # proposed | accepted | superseded | rejected | expired
scope: repo          # repo | workflow | system | organization
elements:
  - Sruja.Context
actors:
  - human
sources:
  - docs/adr/001-example.md
  - .sruja/runs/run-123/facts_bundle.json
trace_id: trace-abc
supersedes: []
---
# Decision title

## Context

## Decision

## Alternatives Considered

## Evidence

## Consequences

## Follow-up Checks
```

### Append-only traces (context events)

All lineage events live in **`.sruja/context_events.jsonl`** (single append-only log). Built-in kinds include `intent_check`, `drift`, and `proposal_merge`. **Decision and agent workflow** events use `schema_version: context_event/v2` and optional fields (`trace_id`, `decision_id`, `run_id`, `actor`, `source`, `tool`, `elements`, `evidence_refs`, `summary`, …) so agents can answer "how did we get here?"

#### Suggested `kind` values for decision workflow

| `kind` | Meaning |
|--------|---------|
| `decision_opened` | Work on a decision or hypothesis started |
| `context_retrieved` | Governed context pulled (focus, graph, bundle) |
| `evidence_cited` | Explicit evidence attachment |
| `alternative_considered` | Option recorded |
| `human_handoff` | Escalation to human |
| `override_recorded` | Human overrode agent/tool |
| `decision_accepted` | DR moved to accepted |
| `decision_superseded` | DR replaced by another |
| `decision_applied` | Outcome linked to graph or code |
| `validation_passed` | Gate succeeded |
| `validation_failed` | Gate failed |

### Hypothesis pipeline (learned ≠ reviewed)

```text
unstructured source (Slack, PR, email, call notes)
  → sruja ingest  →  .sruja/context/
  → extraction / sruja learn  →  .sruja/learned_facts.jsonl  (hypotheses)
  → human review  →  proposal / accept
  →  accepted Decision Record and/or repo.sruja update
  →  graph + context_events linkage
```

**Rule:** anything in `learned_facts.jsonl` or model-labeled output is a **decision hypothesis** until a human accepts it into a DR or the architecture graph.

### HITL taxonomy (human-in-the-loop)

When a human intervenes, classify outcomes so future agents retrieve the right **trust level**:

| Tag | Meaning |
|-----|---------|
| `precedent` | Reusable guidance for similar tasks |
| `exception` | Allowed once or narrowly—do not generalize |
| `correction` | Prior agent/model/tool behavior was wrong |
| `guardrail` | Standing instruction for future agents |

### Lifecycle rule (all workflows)

Every supported workflow must declare:

1. **Retrieve** governed context (focus, decisions, precedents, exceptions).
2. **Record** lineage (context events / decision traces).
3. **Validate** against architecture and policy where applicable.
4. **Link** outcomes back to decisions or graph elements.

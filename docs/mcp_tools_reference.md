# Sruja MCP tools reference

This document lists MCP tools exposed by `sruja mcp` (see [mcp_setup.md](mcp_setup.md)). Tool names and behavior are defined in `crates/sruja-cli/src/commands/mcp.rs`; this page is maintained for operators and prompt authors.

## Retrieval ladder

Use MCP tools inside your AI editor (Cursor, Copilot, Claude) for the best experience. The ladder guides you to the right tool:

| Step | When | Tool |
|------|------|------|
| 1. Before starting a task | Blast radius, decisions, AI instructions | **`sruja_get_focus_briefing`** |
| 2. Paste-ready AI brief | Share context with AI assistant | **`sruja_get_task_context`** |
| 3. Inside AI editor | Editor integration (Cursor, Copilot, Claude) | All MCP tools |
| 4. Investigation | "Why is this like this?" | **`sruja_query_graph`**, **`sruja_explain_element`**, **`sruja_bm25_search`** |

For CLI-based retrieval: use `sruja focus` before a task, `sruja ai` for a paste-ready brief, and `sruja why`/`sruja query` for investigation.

**Automation:** For CI and dashboards that call the CLI with `-f json`, several commands emit `metric_type` / `metric_description` (and `learn` emits `artifact_kind`) so scores are not confused—see [CLI reference — JSON metric hints](../book/src/reference/cli.md#json-output-metric-hints).

## Environment variables

| Variable | Values (truthy) | Effect |
|----------|-----------------|--------|
| `SRUJA_MCP_READONLY` | `1`, `true`, `yes`, `on` (case-insensitive) | `tools/list` omits mutating tools; `tools/call` returns a validation error if a mutating tool is invoked. |
| `SRUJA_MCP_LOG` | same as above | After each `tools/call`, emit one JSON line on **stderr** with `tool`, `repo`, `ms`, `ok`, and optional flattened `error` (for log aggregation / tracing). |

## Mutating tools (hidden when `SRUJA_MCP_READONLY` is set)

These may write under `.sruja`, change git worktrees, run a user-supplied gate command, or drive the agent apply loop:

- `sruja_propose_topology_change`
- `sruja_commit_evolution`
- `sruja_add_element`
- `sruja_add_relationship`
- `sruja_propose_change`
- `sruja_ai_scratchpad`
- `sruja_sandbox`
- `sruja_evaluate_proposal`
- `sruja_record_learning`
- `sruja_record_learn_feedback`
- `sruja_agent_run`

## Tools by category

### Code map and discovery

| Tool | Summary |
|------|---------|
| `sruja_get_repomap` | Token-oriented repository map (tree-sitter). |
| `sruja_explain_discovery` | Why Sruja inferred the repo shape; what to review next. |

### Architecture context (markdown / task-scoped)

| Tool | Summary |
|------|---------|
| `sruja_get_architecture_context` | High-level context and rules; optional `file`, `element_id`, `intent`. |
| `sruja_get_architecture_summary` | Compact overview (layers, boundaries, flows). |
| `sruja_get_task_context` | Rich task context (element, file, git diff, or query) with optional enrichment. |
| `sruja_get_hydrated_context` | One element plus source and neighbors; optional enrichment. |
| `sruja_get_operational_context` | Runbooks, constraints, incidents for repo or element. |

### Graph navigation (deterministic)

| Tool | Summary |
|------|---------|
| `sruja_get_neighbors` | Upstream/downstream neighbors for an id. |
| `sruja_find_path` | Path between two ids. |
| `sruja_get_entrypoints` | External APIs, systems, or nodes with no incoming edges. |
| `sruja_get_data_stores` | Databases and queues. |
| `sruja_explain_element` | Deep dive: centrality, neighbors, comments; optional enrich narrative. |

### Federation (multi-repo)

| Tool | Summary |
|------|---------|
| `sruja_get_system_context` | Composed system from `system.index.json`. |
| `sruja_list_elements` | Elements across federated repos, optional `kind` filter. |

### Validation, drift, and quality

| Tool | Summary |
|------|---------|
| `sruja_check_drift` | Drift JSON vs architecture truth. |
| `sruja_validate_change` | Lint/drift for a set of changed files. |
| `sruja_evaluate_mutation` | Fitness / mutation evaluation on a `.sruja` file. |
| `sruja_preflight_check` | Policies and risks before editing given files. |
| `sruja_critique` | Adversarial review of a change (files / proposal / git range). |

### Natural-language retrieval

| Tool | Summary |
|------|---------|
| `sruja_hybrid_query` | **Default** NL path: auto-routes graph vs semantic vs hybrid. |
| `sruja_query_graph` | NL over the graph with scan-backed facts; optional enrich. |
| `sruja_semantic_search` | Embedding similarity over elements (needs vector index). |
| `sruja_bm25_search` | Keyword search over `.sruja/context/` documents. |

### Context engineering scores and briefings

| Tool | Summary |
|------|---------|
| `sruja_get_context_score` | AI-readiness score and breakdown; optional `format` (`text` / `json`). |
| `sruja_get_focus_briefing` | Task-scoped briefing for `file` or `element_id`; optional `run_id`, `format`. Optional **`base_ref`** / **`head_ref`** add **temporal** context (git-range diff → scan components, architecture fingerprints at base vs working tree)—same semantics as `sruja focus --base-ref` / `--head-ref`. |
| `sruja_get_context_events` | Read recent **append-only lineage** events from `.sruja/context_events.jsonl` (written on `intent check`, `drift`, merged proposals, and `sruja learn`). Args: optional `limit` (default 50), optional `kind` filter (`intent_check`, `drift`, `proposal_merge`, or `learn_run`), optional `details_substring` filter. |
| `sruja_get_learned_facts` | Read **`.sruja/learned_facts.jsonl`** (deterministic scan + drift vs reviewed architecture). Hypotheses, not `repo.sruja` truth. Args: optional `limit` (default 200), optional `status` (`observed`, `proposed`, etc.). Requires `sruja learn` to have been run. |
| `sruja_get_evidence_graph` | Return **`.sruja/evidence_graph.json`** (scan snapshot written by `sruja learn`). |
| `sruja_get_evidence_for_claim` | Resolve a **`claim_id`** against learned facts and attach matching scan nodes from the evidence graph when ids align. |
| `sruja_record_learn_feedback` | Append **approve/reject** for a learned **`fact_id`** to **`.sruja/learn_feedback.jsonl`** (**mutating**); rejects suppress that fact in future learn-generated proposals. |
| `sruja_get_agent_learnings` | Return **Agentic Memory** entries relevant to an **`element_id`** (guardrails / playbooks recorded via `sruja agent record` or MCP `sruja_record_learning`). |

For how this relates to “context graph” terminology in industry writing versus Sruja’s governed graph, see [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md#context-graphs-sruja-vs-industry-usage).

### Contracts and behavior

| Tool | Summary |
|------|---------|
| `sruja_get_state_machine` | State machine for an element. |
| `sruja_get_contract` | API contract for an element. |

### Agent memory and scratchpad

| Tool | Summary |
|------|---------|
| `sruja_memory_clusters` | Thematic clusters/tags from agentic memory (read). Complements **`sruja_get_agent_learnings`**, which filters by **element id**. |
| `sruja_ai_scratchpad` | Read/append legacy scratchpad markdown (**mutating** on append). |
| `sruja_sandbox` | Git worktree sandbox lifecycle (**mutating**). |
| `sruja_record_learning` | Structured agentic memory entry (**mutating**). |

### Evaluation and agent loop (**mutating**)

| Tool | Summary |
|------|---------|
| `sruja_evaluate_proposal` | Context score plus optional gate command. |
| `sruja_agent_run` | Observe→plan→optional apply agent loop. On **apply**, also writes **`facts_bundle.json`** under `.sruja/agent/runs/<run_id>/` (verification bundle + recorded learnings) for replay. |

### Proposals and model edits (**mutating**)

| Tool | Summary |
|------|---------|
| `sruja_propose_topology_change` | Topology proposal plus impact. |
| `sruja_propose_change` | Structured proposal (elements/relationships/removals). |
| `sruja_commit_evolution` | Append evolution log entry. |
| `sruja_add_element` | Add a model element. |
| `sruja_add_relationship` | Add a relationship. |

## Choosing NL retrieval tools

1. Start with **`sruja_hybrid_query`** unless you have a reason not to.
2. Use **`sruja_query_graph`** when you need the explicit graph-query pipeline or specific `enrich` tuning.
3. Use **`sruja_semantic_search`** when you only need embedding-ranked element hits (and an index exists).

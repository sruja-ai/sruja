# Sruja MCP tools reference

This document lists MCP tools exposed by `sruja mcp` (see [mcp_setup.md](mcp_setup.md)). Tool names and behavior are defined in `crates/sruja-cli/src/commands/mcp.rs`; this page is maintained for operators and prompt authors.

## Retrieval ladder

Use MCP tools inside your AI editor (Cursor, Copilot, Claude) for the best experience.

### Progressive disclosure (token-efficient)

For architecture orientation, prefer this **Index → Topology → Detail** chain. Each response includes `estimated_tokens`, `truncated`, `next_suggested_tool`, and `element_ids` for chaining.

| Layer | Tool | When |
|-------|------|------|
| **Index** | **`sruja_list_architecture_index`** | Discover element ids, kinds, cycles, and policy samples without loading full DSL |
| **Topology** | **`sruja_get_topology`** | Upstream/downstream for one `id` (`depth` default 1, max 4) |
| **Detail** | **`sruja_get_elements`** | Batch detail for ids from index/topology |
| **Task** | **`sruja_get_task_context`** | Hydrated task context after you know the focus area (`cache_friendly: true` for prompt-cache-ordered JSON) |

Full diagnostic logs truncated in tool output are stored under `.sruja/vfs/diagnostics/`; fetch with **`sruja_get_diagnostic_full`** and the `sruja-vfs://diagnostics/...` URI.

### Task briefing and investigation

| Step | When | Tool |
|------|------|------|
| Architecture authoring (synthesis input) | Capped communities, entrypoints, manifest edges — not full graph | **`sruja_get_author_evidence`** |
| Before starting a task | Blast radius, decisions, AI instructions | **`sruja_get_focus_briefing`** |
| Paste-ready AI brief (CLI) | Share context outside MCP | `sruja ai` / `sruja ai-context -f for-ai` |
| Investigation | "Why is this like this?" | **`sruja_query_graph`**, **`sruja_explain_element`**, **`sruja_bm25_search`** |

For CLI-based retrieval: use `sruja focus` before a task, `sruja ai` for a paste-ready brief, and `sruja why`/`sruja query` for investigation. Use `sruja ai-context -f for-ai --cache-friendly` when the host supports prompt caching (stable invariant prefix, volatile task tail).

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
- `sruja_record_context_event`
- `sruja_record_decision_event`
- `sruja_create_decision_record`
- `sruja_link_decision_to_element`

## Tools by category

### Code map and discovery

| Tool | Summary |
|------|---------|
| `sruja_get_author_evidence` | Load or build `.sruja/author_evidence.json` (`author_evidence/v1`) for grounded authoring; includes `estimated_tokens` and `next_suggested_tool`. |
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

### Progressive disclosure (architecture ladder)

| Tool | Summary |
|------|---------|
| `sruja_list_architecture_index` | Index layer: compact element list + validation signals (`max_tokens`, optional `kinds` filter). |
| `sruja_get_topology` | Topology layer: upstream/downstream for `id` (`depth`, `max_tokens`). |
| `sruja_get_elements` | Detail layer: element payloads for `ids[]` (`max_tokens`). |
| `sruja_get_diagnostic_full` | Read full text for a truncated diagnostic (`uri` from VFS). |

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
| `sruja_get_context_events` | Read **append-only lineage** from `.sruja/context_events.jsonl` (`context_event/v1` and **`context_event/v2`** traces). Args: `limit`, optional `kind`, `details_substring`, optional **`decision_id`**, **`trace_id`**, **`element_id`**, **`decision_lineage_only`**. |
| `sruja_get_decisions` | List **Decision Record** files under `.sruja/decisions/` (YAML front matter + markdown). |
| `sruja_get_decision_trace` | Return events for a **`decision_id`** from the same log. |
| `sruja_get_learned_facts` | Read **`.sruja/learned_facts.jsonl`** (deterministic scan + drift vs reviewed architecture). Hypotheses, not `repo.sruja` truth. Args: optional `limit` (default 200), optional `status` (`observed`, `proposed`, etc.). Requires `sruja learn` to have been run. |
| `sruja_get_evidence_graph` | Return **`.sruja/evidence_graph.json`** (scan snapshot written by `sruja learn`). |
| `sruja_get_evidence_for_claim` | Resolve a **`claim_id`** against learned facts and attach matching scan nodes from the evidence graph when ids align. |
| `sruja_record_learn_feedback` | Append **approve/reject** for a learned **`fact_id`** to **`.sruja/learn_feedback.jsonl`** (**mutating**); rejects suppress that fact in future learn-generated proposals. |
| `sruja_record_context_event` | Append a full **`event`** object to **`context_events.jsonl`** (**mutating**); same contract as CLI `sruja event append`. |
| `sruja_record_decision_event` | Append **`context_event/v2`** lineage (**mutating**): `kind`, `summary`, optional `decision_id`, `trace_id`, `actor`, `source`, `tool`, `elements`, `evidence_refs`, `outcome`. |
| `sruja_create_decision_record` | Create **`.sruja/decisions/<id>.md`** and emit **`decision_opened`** (**mutating**). |
| `sruja_link_decision_to_element` | Add **`element_id`** to a Decision Record’s YAML (**mutating**). |
| `sruja_get_agent_learnings` | Return **surfaced** Agentic Memory hits for an **`element_id`** (same token cap as focus; increments `retrieval_count`). Not the full `find_relevant` set. |

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
| `sruja_record_learning` | Structured agentic memory entry (**mutating**). Optional **`hitl_kind`**: `precedent` \| `exception` \| `correction` \| `guardrail` (see [context-graph-for-agents.md](context-graph-for-agents.md)). |

CLI lifecycle (not MCP yet): `sruja agent curate`, `agent update`, `agent merge`, `agent delete` — see **Agentic memory utility** in [AGENTS.md](../AGENTS.md).

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

## MCP resources (`resources/list`, `resources/read`)

Stable `sruja://` URIs for prompt-cache-friendly prefixes (optional `repo` / `path` in params, defaulting to the MCP server root):

| URI | Content |
|-----|---------|
| `sruja://architecture/main` | Declared architecture DSL (`repo.sruja`) |
| `sruja://context/invariant.md` | Layers, boundaries, forbidden patterns, MCP ladder |
| `sruja://context/llms-architecture.txt` | Compact brief (same shape as repo-root `llms-architecture.txt`) |
| `sruja://diagrams/current.mmd` | Mermaid export (when architecture file exists) |
| `sruja://decisions/index` | JSON index of `.sruja/decisions/` (when present) |

## MCP prompts (`prompts/list`, `prompts/get`)

Deterministic task templates (return MCP `messages` with text content):

| Prompt | Arguments |
|--------|-----------|
| `sruja_new_service_scaffold` | `service_name` (required) |
| `sruja_review_change` | `files` (required), `description` (optional) |
| `sruja_focus_task` | `element_id` and/or `file` |
| `sruja_mcp_guide` | none — canonical ladder workflow |

## IDE rule sync (CLI)

`sruja sync-ide-rules -r .` writes `.cursorrules`, `.github/copilot-instructions.md`, `CLAUDE.md`, `.gemini/AGENTS.md`, and **`llms-architecture.txt`** in one pass. `just context-sync` / `make context-sync` call this command.

`sruja sync-ide-rules -r . --check` exits non-zero if any of those files drift from the current architecture (CI-friendly).

## Active context management (Phase 3)

| Tool | Summary |
|------|---------|
| `sruja_suggest_context_prune` | Returns `keep_ids` / `compress_ids` from graph topology vs `active_element_ids` (host applies compression). |
| `sruja_get_drift_state` | Compact `drift_state/v1` JSON for structured injection (use instead of pasting full drift). |
| CLI | `sruja drift -r . -f drift-state` or `sruja drift-state -r .` — same payload for scripts and the VS Code command. |

**MCP drift injector:** pass `initializationOptions: { "watch_drift": true }` on `initialize`. After `notifications/initialized`, the server emits `notifications/drift_state` with the same `drift_state/v1` payload (structured block for host middleware). Server advertises support via `capabilities.experimental.watchDrift`.

After host compresses chat history, append a lineage row with `sruja_record_context_event` kind `context_compressed` and `details.suppress_recompress_turns` (3–5). CLI helper: `record_context_compressed` in context events.

**Token reduction fixture:** `crates/sruja-cli/tests/fixtures/context_prune_long_trace.json` and unit test `long_trace_prune_meets_twenty_percent_token_reduction` (≥20% trace tokens when host drops `compress_ids`).

## Persistent memory (Phase 4)

| Tool | Summary |
|------|---------|
| `sruja_search_memory` | FTS over `.sruja/memory.sqlite` (learnings, context events, decision records). Hits labeled `hypothesis` or `reviewed_truth`. |
| `sruja_get_memory_timeline` | Chronological slice around `anchor_id` or `anchor_timestamp`. |
| `sruja_reindex_memory` | Rebuild index from `agent_memory.json`, `context_events.jsonl`, `.sruja/decisions/*.md`. |
| CLI | `sruja memory search|timeline|reindex` (hidden subcommands). |

Auto-reindex when source files change; index never writes to `repo.sruja`.

## Choosing NL retrieval tools

1. Start with **`sruja_hybrid_query`** unless you have a reason not to.
2. Use **`sruja_query_graph`** when you need the explicit graph-query pipeline or specific `enrich` tuning.
3. Use **`sruja_semantic_search`** when you only need embedding-ranked element hits (and an index exists).

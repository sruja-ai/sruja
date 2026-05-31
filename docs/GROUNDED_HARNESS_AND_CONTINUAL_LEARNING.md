# Grounded harness and continual learning (host-owned)

Sruja is a **deterministic harness** for architecture-as-code: it validates `.sruja`, compares models to the codebase, and exposes grounded context to AI editors. It is **not** a full agent runtime. The editor or CI host (Cursor, Claude Code, your runner) owns the LLM loop—planning, reflection, and open-ended reasoning—while Sruja supplies evidence, gates, memory, and MCP tools.

This guide explains how to combine both layers for **continual learning in token space**: the system improves through recorded outcomes and curated memory without retraining model weights.

---

## Why passive codegen decays

When a language model refactors or designs architecture from an isolated prompt, there is no built-in check against the real repository. The result is often structurally invalid DSL, relationships that contradict imports, or boundaries that ignore existing coupling. Over time, teams accumulate drift between `repo.sruja` and code.

A **grounded harness** shifts correctness from neural weights to a compiler: every proposal is linted, compared to scan evidence, and optionally blocked in CI before merge.

---

## Product boundary

| Layer | Owner | Responsibility |
|-------|--------|----------------|
| **Harness** | Sruja CLI + MCP | `sync`, `lint`, `drift`, `intent check`, proposals, agent memory, focus/ai briefings |
| **Agent host** | Your editor / CI / script | Act (generate), optional Reflect/Learn (narrative), tool orchestration beyond Sruja |
| **Reviewed truth** | Humans + promotion flow | `repo.sruja`, Decision Records, approved proposals |

See also [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md) and [context-graph-for-agents.md](context-graph-for-agents.md).

---

## Report vs repository (correction table)

Some articles describe a monolithic `ContinualLearningAgent` with `facts`, `user_preferences`, and `learned_patterns` in memory, plus `reflect_and_learn()` and automatic skill generation. **That is a host pattern, not a single shipped binary in this repo.**

| Often described elsewhere | What Sruja ships today |
|---------------------------|-------------------------|
| `ContinualLearningAgent` / `TrajectoryTracker` / `reflect_and_learn()` | **Host workflow** (e.g. Cursor agent + `sruja-architecture` skill) consuming Sruja JSON |
| Memory buckets: facts, preferences, patterns | **`.sruja/agent_memory.json`** with `LearningEntry` objects (`context`, `hypothesis`, `outcome`, `guardrail_advice`, optional `hitl_kind`) |
| LLM supervisor rewrites system prompt | **Not shipped** — optional host step via `--enrich-cmd` on run artifacts |
| Auto-created skill `.md` files | **Not shipped** — install or author skills manually ([INSTALL_AS_SKILL.md](INSTALL_AS_SKILL.md)) |
| `sruja agent run --autonomous` | **Does not exist** — use `agent plan` → review → `agent apply` (or `agent run --mode apply`) |
| `sruja onboard` only | Prefer **`sruja inspect onboard`** for markdown briefings; see [README.md](../README.md) |

Team preferences and stable domain facts belong in **Decision Records** (`.sruja/decisions/`), **editor rules**, or **skills**—not in a separate memory schema inside `agent_memory.json`.

---

## Grounded harness: CLI commands

| Command | Role in the agentic loop |
|---------|---------------------------|
| `sruja lint <file> [--format json]` | Primary rejection gate for invalid or skill-violating DSL |
| `sruja fmt <file>` | Deterministic formatting for reviewable diffs |
| `sruja tree <file>` | Low-token structural map of elements |
| `sruja export json <file>` | Machine-readable graph for tools |
| `sruja sync -r .` | Refresh scan evidence (`.sruja/author_evidence.json`, `.sruja/context.json`) |
| `sruja drift -r . -a repo.sruja` | Fitness function: architecture vs implementation |
| `sruja intent check` | Code changes vs declared intent |
| `sruja discover --explain -r .` | Community detection (LPA) when no baseline exists |

**Lint loop (works well with local OSS models):**

1. Host proposes `.sruja` edits.
2. Run `sruja lint repo.sruja --format json`.
3. If errors exist, feed the JSON array back to the model: “Fix these lint errors.”
4. Repeat until clean, then run `sruja drift`.

Validation uses the WASM-first pipeline (editor extension and CLI share the same rules).

---

## Three lanes of truth

Authoring is split so models do not confuse evidence with reviewed architecture:

| Lane | Tool | Output | Trust |
|------|------|--------|-------|
| **Facts** | `sruja sync`, `sruja author evidence` | `.sruja/author_evidence.json`, `.sruja/context.json` | Citeable evidence |
| **Synthesis** | Skill / MCP; optional `sruja author propose --enrich-cmd` | `.sruja/proposals/<id>.json`, `repo.sruja.working` | Hypothesis until promoted |
| **Enforcement** | `sruja lint`, `sruja drift`, `sruja propose approve` | Pass/fail, violations | Machine-checkable |

Full detail: [plans/GROUNDED_ARCHITECTURE_AUTHORING_PLAN.md](plans/GROUNDED_ARCHITECTURE_AUTHORING_PLAN.md).

---

## Continual learning in token space (Act → Record → Reflect → Learn)

Weights are fixed; **experience** is stored in files and injected into the next prompt.

| Phase | Host (editor/CI) | Sruja harness |
|-------|------------------|---------------|
| **Act** | LLM proposes DSL or code scoped to a goal | `sruja focus`, MCP ladder, `sruja ai` / `ai-context` |
| **Record** | Save chat/run notes if needed | `agent plan` / `agent run`; `.sruja/agent/runs/<id>/facts_bundle.json` on apply; `.sruja/context_events.jsonl` on drift/intent/propose |
| **Reflect** | Optional: summarize `facts_bundle.json` via `--enrich-cmd` | `sruja agent curate` (suggest-only report) |
| **Learn** | Merge advice into team skill or rules | `sruja agent record` or MCP `sruja_record_learning`; `agent merge` / `agent update` |

### Agent memory schema (`LearningEntry`)

Example entry (fields may include optional `id`, `run_id`, `hitl_kind`, `evidence_refs`, retrieval counters):

```json
{
  "id": "learn-abc123",
  "timestamp": "2026-05-19T12:00:00Z",
  "context": "Promote payment boundary proposal",
  "hypothesis": "Split PaymentService container reduces drift violations",
  "outcome": "success",
  "guardrail_advice": "Always run drift after promoting proposals that touch PaymentService",
  "affected_elements": ["Shop.PaymentService"],
  "hitl_kind": "precedent",
  "evidence_refs": [".sruja/proposals/pay-001.json"]
}
```

`hitl_kind` values: `precedent`, `exception`, `correction`, `guardrail`.

CLI:

```bash
sruja agent record -r . \
  -c "Drift fix for Auth" \
  -H "Extract Auth as separate container" \
  -o success \
  -g "Run intent check when moving auth modules" \
  --hitl-kind precedent

sruja agent history -r . -f json
sruja agent curate -r .
```

Utility signals (`retrieval_count`, `task_success_after`) are documented in [AGENTS.md](../AGENTS.md#agentic-memory-utility-srujaagent_memoryjson).

### Bounded agent loop (Headless/CI Convenience Wrappers)

> [!IMPORTANT]
> The primary integration path for active development is the host coding agent (Cursor, Claude Code, Cline, Windsurf) querying Sruja's passive MCP tools and executing Sruja check gates. The CLI commands below (`agent run`, `agent plan`, `agent apply`) are **optional headless/CI convenience wrappers** to run verification steps in non-interactive environments — they are NOT replacements for the host's orchestrator.

```bash
# 1. Plan (reviewable JSON)
sruja agent plan -r . --goal "Reduce drift on Shop.API" --file src/shop/api.rs --print

# 2. Human or CI reviews the plan file

# 3. Apply with verification
sruja agent apply -r . --plan docs/plans/run-xxx.json

# Or combined observe/plan/apply (Headless/CI only):
sruja agent run -r . --goal "..." --file path/to/file.rs --mode plan
sruja agent run -r . --goal "..." --file path/to/file.rs --mode apply
```

**MaTTS (parallel trajectories):** `sruja agent run ... --trajectories 3` runs sandboxed attempts (git worktrees when available) and distills guardrails by contrasting success vs failure—deterministic contrast, not an LLM judge.

Configure bounds in `.sruja/config.toml`:

```toml
[agent]
max_steps = 5
max_runtime_ms_per_step = 120000
auto_record_learnings = true
default_trajectories = 3

[integrations]
default_provider = "cmd"
cmd = "ollama run llama3"
timeout_ms = 15000
max_bytes = 20000
```

---

## Artifacts reference

| Path | Purpose |
|------|---------|
| `.sruja/agent_memory.json` | Curated learnings (`LearningEntry`) |
| `.sruja/context_events.jsonl` | Append-only lineage (drift, intent, proposal merge) |
| `.sruja/agent/runs/<run_id>/facts_bundle.json` | Replay bundle after agent apply verification |
| `.sruja/proposals/<id>.json` | Hypothesis architecture changes |
| `.sruja/author_evidence.json` | Scan-backed authoring evidence (prefer over raw `graph.json` for agents) |
| `.sruja/learned_facts.jsonl` | Hypotheses from `sruja learn` — **not** reviewed truth |

---

## Local and open-source inference (`--enrich-cmd`)

Sruja does not require proprietary APIs. Enrichment reads **grounded JSON from stdin** and expects **markdown on stdout**.

```bash
# One-off
sruja inspect onboard -r . -f markdown --enrich-cmd 'ollama run llama3'

sruja author propose -r . --enrich-cmd 'ollama run llama3'

sruja agent plan -r . --goal "..." --file src/lib.rs --enrich --enrich-cmd 'ollama run llama3'
```

Environment overrides (team or CI):

- `SRUJA_ENRICH_PROVIDER` — `cmd` (default) or `openai`
- `SRUJA_ENRICH_CMD` — command for `cmd` provider
- `SRUJA_ENRICH_MODEL`, `SRUJA_ENRICH_BASE_URL`, `SRUJA_ENRICH_API_KEY` — OpenAI-compatible local servers (vLLM, TGI)

Repo defaults: `.sruja/config.toml` `[integrations]` block (see [README.md](../README.md)).

---

## Daily operational loop

```bash
# 1. Refresh evidence
sruja sync -r .

# 2. Task briefing
sruja focus --file src/api/main.rs -r .

# 3. Synthesize (host or headless)
#    Skill/MCP default; optional:
sruja author propose -r . --enrich-cmd 'ollama run llama3'

# 4. Gates before promotion
sruja lint repo.sruja --format json
sruja drift -r . -a repo.sruja
sruja propose approve <id>   # when using proposals

# 5. CI envelope
sruja drift --ci -r .
```

Scaffold GitHub Actions: `sruja init --ci` (alias `sruja start --ci`).

---

## Systems thinking: flows, externals, blast radius

- Model **external systems** and scenarios in `.sruja` so agents see dependencies you do not control.
- Use **scenario / flow** syntax for data lineage (origin → transforms → destinations).
- Before large changes: `sruja focus --element-id <id>` or MCP impact/neighbor tools.
- Discovery without baseline: `sruja discover --explain -r .` (community detection + confidence hints).

---

## IDE and CI integration

- **VS Code / Cursor extension:** validation on save, diagram preview, “Copy Agent Guide”, architecture review commands.
- **MCP:** progressive disclosure (`sruja_list_architecture_index` → topology → elements → task context). Setup: [mcp_setup.md](mcp_setup.md). Catalog: [mcp_tools_reference.md](mcp_tools_reference.md).
- **CI:** fail PRs on `sruja drift --ci`; optional baseline with `sruja baseline`.

Host-side reflect playbook: [.cursor/commands/sruja-reflect-on-run.md](../.cursor/commands/sruja-reflect-on-run.md).

---

## What Sruja does not do

- Run a 24/7 autonomous coding agent (`--autonomous` is not a CLI flag).
- Automatically rewrite its own system prompts or mint new skills from trajectories.
- Replace your editor’s agent for general refactors, web search, or unbounded file edits.
- Treat `learned_facts.jsonl` or LLM synthesis as reviewed architecture without promotion + lint + drift.

**Kill rule:** If a workflow cannot name define intent → understand context → detect drift → review change, keep it out of primary automation ([feature_tightening_spec.md](feature_tightening_spec.md)).

---

## Advanced: extending the CLI (contributors only)

The CLI is a Rust workspace (`sruja-cli`). Teams may add validation subcommands via `clap` subcommands for org-specific compliance. This is **not** the default integration path for agent users—prefer MCP tools, config, and hooks.

---

## Further reading

- [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md) — MCP vs in-product runtime
- [context-graph-for-agents.md](context-graph-for-agents.md) — Decision Records and trace events
- [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) — Context pipeline principles
- [plans/GROUNDED_ARCHITECTURE_AUTHORING_PLAN.md](plans/GROUNDED_ARCHITECTURE_AUTHORING_PLAN.md) — Authoring lanes
- [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md) — Install `sruja-architecture` skill
- Book: [Agentic AI course](../book/src/courses/agentic-ai/course-overview.md) — Module 4, Lesson 4 (grounded harness)

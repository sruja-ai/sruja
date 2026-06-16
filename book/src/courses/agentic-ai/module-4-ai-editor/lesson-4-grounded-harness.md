---
title: "Lesson 4: Grounded harness and continual learning"
weight: 4
summary: "Sruja as a CLI-first autonomous agent with an independent deterministic grader."
---

# Lesson 4: Grounded harness and continual learning

## Agent loop vs passive harness

Sruja operates in two modes:

| Mode | How | Who owns the loop |
|------|-----|-------------------|
| **Autonomous** (`sruja agent loop`) | CLI drives comprehend → plan → execute → critique → replan | Sruja |
| **Passive harness** (MCP) | Editor host calls Sruja gates on demand | Editor host |

| | Sruja (grader) | Editor / CI (host) |
|---|-----------------|---------------------|
| Validates `.sruja` | `sruja lint`, `sruja fmt` | Proposes DSL edits |
| Compares model to code | `sruja drift`, `sruja intent check` | Interprets violations |
| Stores learnings | `.sruja/agent_memory.json` | Reflects on runs (optional) |
| Open-source LLM | `--enrich-cmd`, `config.toml` | Runs Ollama, vLLM, etc. |

The **deterministic layer** (verify-task, drift, lint, intent) is always the independent grader — the actor never grades itself, whether the actor is `agent loop` or your editor.

Full reference: [Grounded harness and continual learning](../../../../docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md).

## Checklist: grounded architectural task

1. **Sync evidence** — `sruja sync -r .`
2. **Brief the task** — `sruja focus --file <path> -r .` or MCP task context
3. **Propose** — skill/MCP synthesis or `sruja author propose --enrich-cmd '…'`
4. **Lint loop** — `sruja lint repo.sruja --format json` until clean; feed errors back to the model
5. **Drift gate** — `sruja drift -r . -a repo.sruja`
6. **Promote** — `sruja propose approve <id>` or human merge to `repo.sruja`
7. **Record** — `sruja agent record` or MCP `sruja_record_learning` on success/failure

## Bounded agent loop

```bash
sruja agent plan -r . --goal "Fix drift on Checkout" --element-id Shop.Checkout --print
# Review plan JSON, then
sruja agent apply -r . --plan docs/plans/<run-id>.json
```

Optional MaTTS parallel attempts:

```bash
sruja agent run -r . --goal "..." --file src/checkout.rs --mode apply --trajectories 3
```

Replay bundle after apply: `.sruja/agent/runs/<run_id>/facts_bundle.json`.

## Local inference (Ollama example)

```toml
# .sruja/config.toml
[integrations]
default_provider = "cmd"
cmd = "ollama run llama3"
timeout_ms = 15000
max_bytes = 20000
```

```bash
sruja inspect onboard -r . -f markdown --enrich-cmd 'ollama run llama3'
```

## Memory and curation

Learnings are **`LearningEntry`** records—not separate buckets for “facts” vs “preferences”:

```bash
sruja agent record -r . \
  -c "Checkout boundary" \
  -H "Split cart from payment container" \
  -o success \
  -g "Run drift after any Checkout container change" \
  --hitl-kind precedent

sruja agent curate -r .
# After reviewing curate output, merge duplicates into one entry
sruja agent merge -r . --ids id1,id2 -c "Merged context" -H "..." -g "Combined guardrail"
```

## MCP ladder (editors)

1. `sruja_list_architecture_index`
2. `sruja_get_topology`
3. `sruja_get_elements`
4. `sruja_get_task_context` (`cache_friendly: true` when caching)

Setup: [mcp_setup.md](../../../../docs/mcp_setup.md).

## Reflect / learn

After `agent loop` or `agent apply`, optionally:

1. Read `facts_bundle.json` and recent `agent_memory.json` entries.
2. Use your editor agent or `--enrich-cmd` to draft narrative lessons (markdown only).
3. Persist with `sruja agent record` or `sruja_record_learning`.
4. Run `sruja agent curate` before merging duplicate learnings.

Cursor users: run the project command **Sruja: Reflect on agent run** (see `.cursor/commands/sruja-reflect-on-run.md`).

## What you learned

- Sruja constrains probabilistic output with deterministic gates.
- Continual learning lives in **artifacts and memory**, not weight updates.
- `sruja agent loop` owns the full closed loop; in editor mode, Sruja owns Record/Learn storage and enforcement while the host does Act.

**Next:** [Grounded harness and continual learning](../../../../docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) for the full reference.

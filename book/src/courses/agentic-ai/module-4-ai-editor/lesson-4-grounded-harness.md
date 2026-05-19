---
title: "Lesson 4: Grounded harness and continual learning (host-owned)"
weight: 4
summary: "Use Sruja as a deterministic harness while your editor owns the LLM loop."
---

# Lesson 4: Grounded harness and continual learning (host-owned)

## Harness vs runtime

| | Sruja (harness) | Editor / CI (host) |
|---|-----------------|---------------------|
| Validates `.sruja` | `sruja lint`, `sruja fmt` | Proposes DSL edits |
| Compares model to code | `sruja drift`, `sruja intent check` | Interprets violations |
| Stores learnings | `.sruja/agent_memory.json` | Reflects on runs (optional) |
| Open-source LLM | `--enrich-cmd`, `config.toml` | Runs Ollama, vLLM, etc. |

Sruja does **not** ship `ContinualLearningAgent`, `--autonomous`, or automatic skill generation. Those are **patterns you implement in the host**, using Sruja JSON as ground truth.

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
# Review plan JSON, then:
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
# After reviewing curate output, merge duplicates into one entry:
sruja agent merge -r . --ids id1,id2 -c "Merged context" -H "..." -g "Combined guardrail"
```

## MCP ladder (editors)

1. `sruja_list_architecture_index`
2. `sruja_get_topology`
3. `sruja_get_elements`
4. `sruja_get_task_context` (`cache_friendly: true` when caching)

Setup: [mcp_setup.md](../../../../docs/mcp_setup.md).

## Reflect / learn (host-owned)

After `agent apply`, optionally:

1. Read `facts_bundle.json` and recent `agent_memory.json` entries.
2. Use your editor agent or `--enrich-cmd` to draft narrative lessons (markdown only).
3. Persist with `sruja agent record` or `sruja_record_learning`.
4. Run `sruja agent curate` before merging duplicate learnings.

Cursor users: run the project command **Sruja: Reflect on agent run** (see `.cursor/commands/sruja-reflect-on-run.md`).

## What you learned

- Sruja constrains probabilistic output with deterministic gates.
- Continual learning lives in **artifacts and memory**, not weight updates.
- The host owns Act/Reflect; Sruja owns Record/Learn storage and enforcement.

**Next:** [Agentic orchestration patterns and Sruja](../../../../docs/AGENTIC_ORCHESTRATION_AND_SRUJA.md) for industry pattern mapping.

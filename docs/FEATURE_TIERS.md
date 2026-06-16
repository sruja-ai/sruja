# Core Foundations And Extensions

Sruja now treats a small set of workflows as the product center. Everything else should either strengthen those workflows or live as an extension.

## Core Foundations

These are the workflows Sruja should lead with in the README, docs, demos, and editor templates.

| Foundation | Job | Commands | MCP / integration |
|------------|-----|----------|-------------------|
| **Capture** | Bring durable knowledge and decisions into the repo | `ingest`, `decision`, optional `lint repo.sruja` | `sruja_get_author_evidence` when authoring reviewed intent |
| **Retrieve** | Brief a human or AI before a change | `focus`, `ai`, `why`, `mcp` | `sruja_get_focus_briefing`, `sruja_get_task_context`, `sruja_hybrid_query` |
| **Verify** | Check that code still matches reality and intent | `drift`, `intent check`, `verify-task`, `sync` | `sruja_check_drift`, `sruja_get_drift_state`, `sruja_verify_task` |

### Core Story

Public messaging should teach this loop:

1. capture knowledge and decisions
2. retrieve task-scoped context before editing
3. verify the result after editing

### Default MCP Position

- Public templates use `SRUJA_MCP_TOOL_PROFILE=coding`
- `sruja agent loop` owns the full closed loop for autonomous coding
- Sruja's deterministic layer provides context, evidence, and verification in either mode

### Optional Reviewed Intent

`repo.sruja` stays optional. It is reviewed truth for teams that want durable intent in Git, not a day-one requirement.

## Extensions

These features remain valuable, but they should be documented as extensions built on the core foundations above.

### Architecture Authoring Extension

- `author`
- `propose`
- advanced `.sruja` authoring and proposal workflows
- `sruja-architecture` skill

### Visualization Extension

- `export`
- diagram preview
- context graph and dashboards
- Mermaid, D2, GraphML, Neo4j, Obsidian outputs

### Team Review Extension

- `critique`
- `review` / `daily`
- `drift-pr`
- `compliance`
- CI reporting and policy-oriented workflows

### Federation Extension

- `publish`
- `compose`
- multi-repo packaging and retrieval

### Agent Ops Extension

- `agent loop` (autonomous closed-loop coding agent)
- `agent plan`
- `agent apply`
- `agent record`
- learnings, memory curation, run snapshots

### Workflow Extension

- `workflow`
- `aidlc`
- lifecycle scaffolding and gated process support

### Analytics And Inspection Extension

- `context-score`
- registry/index tooling
- graph history
- deeper health and intelligence surfaces

## Move Out Of The Core Story

These should not be taught as first-run product surfaces:

- diagram-first positioning
- generic “AI platform” or “agent orchestration” framing
- large DSL feature checklists
- overlapping onboarding paths that compete with `focus`, `ai`, and `verify-task`

## Fold Or Hide

These should be folded into clearer canonical paths or kept out of primary docs:

| Canonical path | Fold or hide |
|----------------|--------------|
| `focus` / `ai` | `ai-context`, old `context` naming |
| `start` | `quickstart`, `overview`, `onboard` as primary entry points |
| `verify-task` / `agent loop` | `agent run` as a public surface |
| `status` or a future single status page | `doctor`, `health` overlap |
| `review` | `daily` as a separate headline feature |

## Removal Backlog

Features that do not clearly strengthen capture, retrieve, or verify should be candidates for deprecation or migration into advanced docs only.

Examples:

- hidden legacy aliases
- duplicate health and onboarding concepts
- inspection surfaces without a strong user moment

---

## Product Messaging

**Public one-liner:** Sruja scans your repo and reports structural problems AI coding tends to introduce; optional `.sruja` and MCP help your editor stay inside boundaries.

**Do not lead with:** architecture DSL, diagram designer, context engineering platform, or "architecture intelligence."

---

## Tier 1a — OSS hero (no `.sruja` required)

Deterministic structural scan. Catches what AI changes in repo topology.

```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja start -r .
sruja drift -r . --structural-only --advisory
```

Always show: **what we found**, **what we could not infer**, **what changed** (on repeat runs). Use `drift -f json` for machines.

**Messaging principles**

- Lead with scan + anti-drift + MCP gates — not skill install or DSL syntax.
- Avoid hero copy: "instant", "60 seconds", "zero-config wow", "intelligence".
- `.sruja` is a **structured snapshot for viz + optional CI intent**, not day-one requirement.

## Tier 1b — Skill (optional promotion)

Install `sruja-architecture` when you want reviewed `repo.sruja` in version control:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

The skill runs Tier-1 commands (sync, drift, lint) — it is an accelerator, not the product definition.

**Draft vs reviewed:** `repo.sruja.draft` = evidence sketch; `repo.sruja` = reviewed truth. `sruja sync` writes `.sruja/author_evidence.json` for synthesis.

---

## Retrieval ladder (CLI + MCP)

| Step | When | CLI | MCP |
|------|------|-----|-----|
| 1 | Before editing | `sruja focus -r . --file <path>` | `sruja_get_focus_briefing` |
| 2 | Paste-ready brief | `sruja ai -r . --task "…"` | `sruja_get_task_context` |
| 3 | Deep graph | — | Ladder + `sruja_hybrid_query` |
| 4 | Investigation | `sruja why` / `query` | `sruja_query_graph`, etc. |

## MCP tool profiles

- **`coding`** (default, 15 tools): ladder, focus, drift, verify, memory search, hybrid query, critique, prune.
- **`minimal`**, **`arch`**, **`full`** (compat only — not in public templates).

Set `SRUJA_MCP_TOOL_PROFILE=coding`. **Not** in coding profile: `sruja_agent_run` (legacy) — use `sruja agent loop` for autonomous coding, or MCP tools for editor-host integration.

---

## Agent loop vs editor loop

### Autonomous loop (`sruja agent loop`)

```text
1. Sruja: focus / drift state / boundary context
2. Sruja LLM edits code (via built-in file/shell tools)
3. Sruja: critique + verify (deterministic grader)
4. Sruja: replan or approve
```

### Editor loop (MCP)

```text
1. MCP: focus / drift state / boundary context
2. Host LLM edits code
3. sruja confidence -r . -f md          (or verify-task for CI gates)
4. sruja agent record -c "…" on failure (optional)
```

The **confidence report** composes verify-task, intent check, drift, and review into a single human-readable artifact: what changed, what evidence was checked, what risks remain, and what to inspect at 3AM.

---

## Three pillars (technical docs)

- **Grounding** — scan/graph evidence in prompts
- **Validation** — lint, drift, verify-task
- **Persistence** — optional reviewed `repo.sruja`

**Pitch line:** Sruja gives the model your real topology and validates what it produces.

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
- Host editors own the LLM loop
- Sruja provides context, evidence, and verification

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
| `verify-task` / host verify | `agent run` as a public surface |
| `status` or a future single status page | `doctor`, `health` overlap |
| `review` | `daily` as a separate headline feature |

## Removal Backlog

Features that do not clearly strengthen capture, retrieve, or verify should be candidates for deprecation or migration into advanced docs only.

Examples:

- hidden legacy aliases
- duplicate health and onboarding concepts
- inspection surfaces without a strong user moment

See [FEATURE_CONSOLIDATION.md](./FEATURE_CONSOLIDATION.md) for the wider consolidation matrix.

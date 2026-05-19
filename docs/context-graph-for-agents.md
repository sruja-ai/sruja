# Context graph for agents

Sruja is the **portable governed context layer** for any agent host (Cursor, Claude Code, CI, Slack bots, internal runners). The host orchestrates steps; Sruja holds **reviewed structure**, **decision lineage**, **append-only traces**, and **hypotheses** that are not promoted to truth without human review.

This document defines the **reference workflow**, **artifact contracts**, and **lifecycle** that other workflows must follow. For how Sruja frames “context graph” vs industry usage, see [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md).

## Product boundary

**Core promise**

- Reviewed truth in `repo.sruja` (and optional domain schemas).
- **Decision Records** (generalized ADRs): reviewed “why” with stable IDs, status, and links to graph elements.
- **Evidence and lineage** via scans, `.sruja/context_events.jsonl`, run snapshots, and agent memory.
- Task-scoped retrieval through `sruja focus`, `sruja ai`, and MCP tools.
- Validation before risky action: drift, intent, compliance, and explicit trace events.

**Non-goals**

- Sruja is **not** the full agent runtime or a general business workflow engine (no Slack/Jira/Salesforce automation product).
- **Model-extracted or ingested text is never reviewed truth** until it passes human review and proposal flows.

Align autonomous loops with [AGENTS.md](../AGENTS.md): `sruja agent run` consumes Sruja evidence and emits reviewable artifacts—it is not a substitute for the editor’s agent or unconstrained refactors.

For the full harness + host-owned learning model (correction table, Ollama/`--enrich-cmd`, daily loop), see [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md).

## Decision Record (DR) — generalized ADR

An **ADR** is a **profile** of a Decision Record: typically `type: architecture`, linked elements, and `category: adr` when ingested through `sruja ingest`.

Decision Records also cover product, operational, governance, and agent decisions. Store human-reviewed records as Markdown with YAML front matter under **`.sruja/decisions/`** (gitignored by default in some setups—copy to `docs/decisions/` if you want them in version control).

### Front matter template

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

**Statuses** map to the in-graph `Decision` model where applicable: `deprecated` in graph may correspond to `expired` or `superseded` in files—prefer explicit `superseded` + `supersedes` in DR files.

## Append-only traces (context events)

All lineage events live in **`.sruja/context_events.jsonl`** (single append-only log). Built-in kinds include `intent_check`, `drift`, and `proposal_merge`. **Decision and agent workflow** events use `schema_version: context_event/v2` and optional fields (`trace_id`, `decision_id`, `run_id`, `actor`, `source`, `tool`, `elements`, `evidence_refs`, `summary`, …) so agents can answer “how did we get here?”

### Example trace event (JSON)

```json
{
  "schema_version": "context_event/v2",
  "timestamp": "2026-05-15T12:00:00Z",
  "kind": "context_retrieved",
  "outcome": "ok",
  "policy_fingerprint": null,
  "strict": null,
  "details": {},
  "trace_id": "trace-abc",
  "decision_id": "DR-2026-001",
  "run_id": "run-123",
  "workflow_id": null,
  "actor": "agent",
  "source": "mcp",
  "tool": "sruja_get_focus_briefing",
  "elements": ["Sruja.Context"],
  "subject_ids": [],
  "evidence_refs": ["repo.sruja", ".sruja/context.json"],
  "summary": "Retrieved blast radius before changing context event storage"
}
```

### Suggested `kind` values for decision workflow

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

Use `sruja event append` and MCP `sruja_record_context_event` / decision tools to append rows; use `sruja event list` and `sruja_get_context_events` to read.

**JSON schema:** [schemas/context_event_record.schema.json](../schemas/context_event_record.schema.json) (draft 2020-12; required fields for each JSONL row).

## Hypothesis pipeline (learned ≠ reviewed)

```text
unstructured source (Slack, PR, email, call notes)
  → sruja ingest  →  .sruja/context/
  → extraction / sruja learn  →  .sruja/learned_facts.jsonl  (hypotheses)
  → human review  →  proposal / accept
  →  accepted Decision Record and/or repo.sruja update
  →  graph + context_events linkage
```

**Rule:** anything in `learned_facts.jsonl` or model-labeled output is a **decision hypothesis** until a human accepts it into a DR or the architecture graph.

## HITL taxonomy (human-in-the-loop)

When a human intervenes, classify outcomes so future agents retrieve the right **trust level**:

| Tag | Meaning |
|-----|---------|
| `precedent` | Reusable guidance for similar tasks |
| `exception` | Allowed once or narrowly—do not generalize |
| `correction` | Prior agent/model/tool behavior was wrong |
| `guardrail` | Standing instruction for future agents |

Use `sruja agent record` / MCP `sruja_record_learning` with optional **`hitl_kind`** (`precedent`, `exception`, `correction`, `guardrail`) to tag entries in agent memory. CLI: `sruja agent record ... --hitl-kind precedent`.

## Reference workflow: code-to-production debugging

Use this as the **first** general-agent workflow; others must map to the same lifecycle stages.

| Stage | Systems of record | Systems of engagement | Sruja usage |
|-------|-------------------|------------------------|-------------|
| Triage | GitHub issue/PR, CI logs, deploy manifests | AI editor, PR comments | `sruja focus`, MCP `sruja_get_focus_briefing`, `sruja_get_decisions`, traces |
| Analyze | Repo, ADRs/DRs, `repo.sruja` | Thread, pair session | `sruja why`, `sruja query`, graph tools |
| Decide | DR files, architecture | Review, incident channel | `sruja decision accept`, events `decision_accepted` |
| Validate | CI, policy | Agent | `sruja drift`, `sruja intent`, `sruja critique`, `validation_*` events |
| Close | Merged PR, rollback, incident resolved | Postmortem | `decision_applied`, learnings with `hitl_kind` |

**Endpoint examples:** PR merged, rollback completed, incident resolved.

## Lifecycle rule (all workflows)

Every supported workflow must declare:

1. **Retrieve** governed context (focus, decisions, precedents, exceptions).
2. **Record** lineage (context events / decision traces).
3. **Validate** against architecture and policy where applicable.
4. **Link** outcomes back to decisions or graph elements.

If a workflow cannot name these stages, keep it out of primary automation until it can ([PRODUCT_FEATURE_ALIGNMENT_REPORT.md](PRODUCT_FEATURE_ALIGNMENT_REPORT.md#canonical-workflows)).

## Federation (multi-repo)

For multi-agent or multi-repo work, use **publish / compose** ([FEDERATION.md](FEDERATION.md)): each repo publishes a bundle; `system.index.json` composes a system view. Agents should request **only the impacted slice**. Cross-repo IDs should be canonical (`repo_id::local_id`). Conflicts surface explicitly—no silent merge.

A future **bundle profile** may include Decision Record metadata and tail references for `context_events.jsonl`; single-repo MVP proves value first.

## Governance (minimum direction)

- **PII:** redact before ingest where required; document hooks at trust boundaries.
- **Attribution:** every event carries `actor` and `source` when known.
- **Read-only MCP:** `SRUJA_MCP_READONLY=1` exposes only retrieval tools ([mcp_tools_reference.md](mcp_tools_reference.md)).
- **Access-aware retrieval:** hosts must not expose MCP results beyond the initiating user’s entitlements (policy outside core CLI for now).

## Success criteria

An agent (via CLI + MCP) can answer:

1. What **architecture truth** applies here? (`repo.sruja`, graph, focus)
2. What **changed recently**? (drift, intent events, git optional)
3. What **similar decisions** existed? (Decision Records, precedents)
4. What **failed last time**? (learnings, `validation_failed`, critique)
5. What **human overrides or exceptions** exist? (traces, `hitl_kind`, DR status)
6. What am I **allowed to see**? (host + org policy; redaction)
7. What **validation must pass** before action? (drift/intent/compliance + explicit events)

## Related artifacts

- [architecture/agent-context-graph.sruja](architecture/agent-context-graph.sruja) — example node/edge kinds for modeling agents and workflows in Sruja DSL.
- [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) — pillars and MCP ladder.
- [mcp_tools_reference.md](mcp_tools_reference.md) — tool list and readonly behavior.

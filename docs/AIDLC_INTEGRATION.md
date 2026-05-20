# AI-DLC integration with Sruja

Sruja integrates [awslabs/aidlc-workflows](https://github.com/awslabs/aidlc-workflows) as an **orchestration harness**: durable workflow artifacts, phase gates, drift/lint on `repo.sruja`, and CI checks. The **editor host** (Cursor, Claude Code, etc.) still runs AIDLC markdown rules and generates application code; Sruja does not ship an in-process LLM executor.

## Vendoring

- Vendor path: `aidlc-workflows/` at the repository root (or set `SRUJA_AIDLC_RULES` to an `aidlc-rules/` directory).
- License: Apache-2.0 (see `aidlc-workflows/LICENSE`).

## Workflow layout

```text
.sruja/workflows/<workflow_id>/
  manifest.json          # workflow/v2, optional aidlc section
  inception/
    scope.md
    impact.json
    aidlc-docs/          # AIDLC inception artifacts
  construction/
    task-plan.md
    linked_proposal_ids.json
    aidlc-docs/
  operations/
    deploy-scope.json
  audit.jsonl
```

Application code stays in the repo root or `workspace/` per AIDLC rules—not inside `aidlc-docs/`.

## CLI (extends `sruja workflow`)

| Command | Purpose |
|---------|---------|
| `workflow init --with-aidlc [--aidlc-profile minimal\|full] [--install-aidlc-rules]` | Create workflow + AIDLC dirs |
| `workflow install-rules` | Copy rules to `.aidlc/aidlc-rules/` and `.aidlc-rule-details/` |
| `workflow status [--check]` | Gate readiness + `aidlc` block in JSON |
| `workflow validate` | Same checks as `status --check`, validation-focused output |
| `workflow audit --event "..."` | Append `audit.jsonl` |
| `workflow trace [--check]` | Run aidlc-traceability (Python) when vendored |
| `workflow run --vision path [--dry-run]` | Optional aidlc-evaluator subprocess |

## MCP

- `sruja_workflow_gate_check` returns `aidlc_missing` and `aidlc_stage` when AIDLC is enabled.
- `sruja_get_task_context` accepts optional `workflow_id` and `phase` to tune context scope.

## CI

Optional label `aidlc-workflow` runs `.github/workflows/sruja-aidlc-gate.yml`:

1. `sruja workflow status --check`
2. `sruja drift -r . -a repo.sruja`

## Cursor

Use `.cursor/commands/sruja-workflow-aidlc.md` with phase `inception`, `construction`, or `operations`.

## Related

- [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md)
- [plans/AI_DLC_ACE_SYNTHESIS_PLAN.md](plans/AI_DLC_ACE_SYNTHESIS_PLAN.md)

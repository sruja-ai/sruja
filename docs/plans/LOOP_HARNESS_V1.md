# Loop Harness Improvements (v1)

**Status:** Proposed  
**Owner:** Product + CLI/MCP (`sruja-cli`, `sruja-agent`)  
**Last updated:** 2026-06-12

This document proposes a practical “loop engineering” increment that strengthens Sruja’s role as a **harness** (evidence, gates, policy, replay, learnings) without turning Sruja into an **orchestrator** (LLM runtime, supervision, autonomous coding).

## Goal

Make host-run agent loops more reliable and reviewable by shipping:

- deterministic verification entry points
- explicit loop policy controls
- richer run artifacts for replay/debug
- safer post-run learning extraction tied to verification outcomes

## Non-goals

- `agent run --autonomous`
- in-process LLM orchestration or multi-agent debate inside Sruja
- skill routing (“pick one of N skills”) inside Sruja
- auto-minting skills from trajectories

## Product shape (one sentence)

Hosts run the loop; Sruja makes the loop **grounded, bounded, and auditable**.

## v1 Deliverables

### 1) Loop policy (`.sruja/config.toml`)

Add an opinionated, small configuration surface for safe defaults in CI and editor-hosted workflows.

Proposed keys (initial):

- `[verify] default_profile = "coding"`
- `[agent.loop_policy]`
  - `max_attempts` (default `1` for CI, `3` for interactive)
  - `require_verify_before_record_learning = true`
  - `escalate_on = ["boundary_violation", "drift_increase", "lint_error"]`
  - `allowed_paths = [...]` (optional; deny-by-default for apply in CI)

### 2) Run artifacts: facts bundle v2 (additive)

Extend `.sruja/agent/runs/<run_id>/facts_bundle.json` with:

- `host` metadata (editor/ci, invocation mode)
- `skills_used[]` (string identifiers from the host)
- `verify_profile` (coding/bugfix/review/arch)
- `files_touched[]` (best-effort, from git status + plan scope)
- `decision_points[]` (stop/continue, retry, escalation)

This is additive and should preserve backwards compatibility with `facts_bundle/v1`.

### 3) Explain / replay run (read-only)

Add a read-only command to summarize the evidence and next action from an existing run directory:

- Inputs: `--run-id <id>` or `--path .sruja/agent/runs/<id>/`
- Outputs:
  - a concise summary of what happened (plan, apply, verify)
  - why it failed (failed step IDs, key stderr excerpts)
  - what evidence is missing (if any)
  - recommended next single action (smallest step to reduce uncertainty)

### 4) Learning extraction guardrails

Tighten the rule: learnings are recorded only when verification passes, unless explicitly marked as a correction.

Minimum enforcement:

- default `auto_record_learnings = false` unless `verify-task.all_passed = true`
- require `--hitl-kind correction` (or similar) to record learnings on failure

## Acceptance criteria

- A host can integrate with one stable contract: `focus → act → verify-task → record learning`.
- CI can fail deterministically on failed verify profile with machine-readable output.
- A reviewer can answer, from artifacts alone:
  - what changed?
  - what checks ran and passed?
  - what failed and why?
  - what should be tried next?
- No new features imply that Sruja is a general-purpose coding agent.

## Risks and mitigations

- **Risk:** “loop engineering” messaging dilutes the product into “another agent”
  - **Mitigation:** Keep user-facing wording as “verification, policy, replay, learnings”.
- **Risk:** Schema churn for artifacts
  - **Mitigation:** additive schema bumps with explicit `schema_version` and stable paths.
- **Risk:** Config complexity
  - **Mitigation:** ship good defaults; keep config optional and small.

## References

- `docs/HOST_AGENT_INTEGRATION.md`
- `docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md`
- `docs/AGENTIC_ORCHESTRATION_AND_SRUJA.md`

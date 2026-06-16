# Enterprise adoption (AI-DLC + host gates)

This guide is for teams adopting Sruja under enterprise constraints: **repeatability**, **low-noise CI**, **offline-ready operation**, and **auditability**.

Sruja is a **CLI-first autonomous coding agent** that owns the full
observe → act → verify → critique → replan loop (`sruja agent loop`). It also
works as a **deterministic grader** (lint/drift/intent/focus/verify-task/workflow
gates) from any editor or CI host. See [HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md).

## One blessed loop (canonical)

The adoption goal is to make this loop the default everywhere (local dev, AI editor, CI):

```text
workflow status --check (if AI-DLC) → focus/ai → host ACT → verify-task → drift-pr/drift
```

## Adoption tiers

### Tier 0 — Evaluation (CLI-only, no CI)

Use this when you want signal fast without committing to enforcement.

- **Scan/drift (advisory):**
  - `sruja start -r . --prompt`
  - `sruja drift -r . --structural-only --advisory`
- **Get task context for an agent:**
  - `sruja focus -r . --file <path>`
  - or `sruja ai -r . --task "<task>"`

Exit criteria:
- You can reproduce results across runs, and findings are mostly actionable.

### Tier 1 — Team adoption (CI gate + host verification)

Use this when you want to **block regressions**, not rewrite the whole repo.

- **Host gate:** use `sruja verify-task` in CI (see templates under `templates/`).
- **PR-only drift:** prefer `sruja drift-pr ...` so you fail only on **new** violations.

Recommended rollout:
- Start **advisory**, capture a baseline, then move to **enforce**.
- Keep policy and baselines machine-readable and reviewable (see `ENTERPRISE_POLICY.md`).

Exit criteria:
- CI fails on new violations and stays enabled after 2–3 weeks of normal development.

### Tier 2 — AI-DLC workflow projects (durable phase artifacts)

Use this when you need **phase gates**, durable artifacts, and traceability.

Core commands (see [AIDLC_INTEGRATION.md](AIDLC_INTEGRATION.md)):

- `sruja workflow init -r . --with-aidlc --install-aidlc-rules`
- `sruja workflow status -r . --check`
- `sruja workflow summary -r . --format json`

Recommended CI posture:
- Gate on `workflow status --check` and on drift (PR-only drift if you are mid-migration).

Exit criteria:
- Your workflow artifacts under `.sruja/workflows/<id>/` are stable, reviewable, and the gates are deterministic.

## What to read next

- [FEATURE_TIERS.md](FEATURE_TIERS.md) (what is Tier 1 vs advanced)
- [AIDLC_INTEGRATION.md](AIDLC_INTEGRATION.md) (AI-DLC workflow usage)
- [HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md) (host vs harness boundary)
- [ENTERPRISE_POLICY.md](ENTERPRISE_POLICY.md) (advisory → baseline → enforce)


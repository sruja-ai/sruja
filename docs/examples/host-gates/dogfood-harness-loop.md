# Dogfood harness loop (Phase 0.4)

Use this checklist on a **small Sruja repo PR** to prove the harness-plugin story end-to-end.

## Prerequisites

- `sruja` on PATH (`make build` or install script)
- MCP registered (`.cursor/mcp.json` or extension **Register MCP**)
- Skills: `sruja-harness` (+ `sruja-architecture` if touching `repo.sruja`)

## Loop

| Step | Action | Artifact |
|------|--------|----------|
| 1 START | `sruja drift -r . -f json` (or MCP drift state) | Note `truth_status` / violations |
| 2 START | `sruja focus -r . --file <path>` | Paste briefing into agent context |
| 3 ACT | Host agent (Cursor) makes the code change | Git diff |
| 4 VERIFY | `sruja verify-task --profile coding -r . -f json` | `verify_task/v1` JSON, exit 0 |
| 5 RECORD | On failure: `sruja agent record -r . -c "…" -H "…" -o failed -g "…"` | `.sruja/context/` learning |
| 6 OPTIONAL | `sruja workflow status --check` if PR uses AIDLC workflow | `audit.jsonl` |

## Profile selection

| Change type | Profile |
|-------------|---------|
| Rust/CLI feature | `coding` |
| Single-file bugfix | `bugfix` + `--file` |
| Pre-merge review | `review` |
| `repo.sruja` / architecture DSL | `arch` |

## PR description template

```markdown
## Harness loop
- [ ] focus briefing used for blast radius
- [ ] `sruja verify-task --profile <profile> -r .` passed locally
- [ ] CI verify workflow green (if enabled)

## Notes
<what verify caught or N/A>
```

## CI

- **This repo:** [.github/workflows/sruja-verify-task.yml](../../../.github/workflows/sruja-verify-task.yml) (builds CLI from PR, runs `verify-task --profile coding`).
- **Other repos:** copy [verify-task-pr.yml](./verify-task-pr.yml) or [templates/github-actions/sruja-verify-task-pr.yml](../../../templates/github-actions/sruja-verify-task-pr.yml).

## Non-goals for this PR

- Do not use `sruja agent run` as the primary execution path
- Do not expand MCP to `full` profile for routine coding

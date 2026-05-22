# Host gate examples

Copy-paste patterns for wiring Sruja as an **AI coding harness** (not a second agent). The host owns Act; Sruja owns START (focus/drift) and VERIFY (`verify-task`).

See [HOST_AGENT_INTEGRATION.md](../../HOST_AGENT_INTEGRATION.md) and [COMMUNITY_SKILLS_STACK.md](../../COMMUNITY_SKILLS_STACK.md).

| File | Use |
|------|-----|
| [verify-task-pr.yml](./verify-task-pr.yml) | GitHub Actions: block PR when verify fails |
| [pre-apply-shell.sh](./pre-apply-shell.sh) | Local hook before applying agent patches |
| [dogfood-harness-loop.md](./dogfood-harness-loop.md) | Checklist for one dogfood PR on this repo |

Install skills:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

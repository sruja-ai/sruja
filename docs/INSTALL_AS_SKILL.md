# Install Sruja as a Skill

Use `sruja-architecture` as the default install. It now covers the old design-rules and architecture-discovery flows in one core skill.

## Install

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Recommended Prompt: Architecture Discovery

`Use sruja-architecture. Run sruja discover --context -r ., gather evidence from the repo, ask targeted questions if scope or externals are unclear, generate architecture.sruja, then run sruja lint and fix until it passes.`

## More

- Full walkthrough: [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md)
- Skill catalog: [../skills/README.md](../skills/README.md)

---
description: After an agent mistake, improve repo AI guidance (Sruja)
---

Something went wrong with an AI-assisted change (wrong behavior, missed convention, bad architecture). **Outer loop:** improve the system so it is less likely to repeat.

Work with the user to:

1. **Characterize the miss** — What should have happened vs what happened? Was it ignorance of a rule, missing validation, or ambiguous scope?
2. **Pick the smallest durable fix** among:
   - `AGENTS.md` — add a gate, clarify a command, document a workflow step
   - `.cursor/rules/sruja-dev.mdc` (or a new `.mdc` rule) — file-type or area-specific constraint
   - `.cursor/commands/*.md` — if the same long prompt was needed repeatedly
   - User or repo **skills** under `.agents/skills/` (or project skill paths) — multi-step procedure
   - Issue / PR **templates** — acceptance criteria or checklists the agent skipped
3. **Draft the concrete edit** (patch-level: what section or file to add or change). Keep wording short and imperative.
4. **Verify** — Explain how the next agent run would behave differently.

Do not rewrite unrelated docs. One focused improvement beats a broad rewrite.

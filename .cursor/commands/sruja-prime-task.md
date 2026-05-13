---
description: Prime context for a Sruja task (scope + codebase lens)
---

You are working on the Sruja repo. Follow `AGENTS.md` (especially **AI agent workflow**).

**Goal:** Load enough context to plan or implement safely—without pasting huge unrelated trees.

Do this in order:

1. If the user gave a file path, run `sruja focus --file <that path>` from the repo root and summarize blast radius and constraints.
2. If they gave an issue URL or ID, restate acceptance criteria and link to any spec they mentioned.
3. Skim **project / scope layer** only: relevant `docs/architecture/*.sruja`, `repo.sruja`, or ADRs that bound this work. Do not pick final edit lists yet unless obvious.
4. Use **narrow reads** (search + open specific files) or `sruja mcp -r .` for architecture questions—avoid dumping entire crates into the transcript.
5. List **open questions** for the user (max 5). Ask them one batch or one at a time depending on what blocks planning.

Do not write implementation code until the user confirms scope or points you at an agreed plan artifact.

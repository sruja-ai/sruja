---
description: Produce a task-level implementation plan (Sruja)
---

You are planning a single task in the Sruja workspace. Obey `AGENTS.md` and **two-layer planning**: this command is **task layer** only (files, order, validation)—not full product scope.

Inputs the user should provide (ask if missing):

- Issue / goal statement (or link)
- Optional: target file or crate

Steps:

1. Confirm **scope layer** is already settled (what ships, what is out of scope). If unclear, stop and ask—do not invent product scope.
2. Inspect only the code paths needed: search, then read relevant modules. Prefer `sruja focus` / MCP over bulk reads.
3. Produce **one plan artifact** with this structure (markdown, save where the user wants—often PR branch `plan.md` or issue comment):

   - **Summary** — One paragraph.
   - **User-visible behavior** — Bullets.
   - **Files to create or change** — Path list with one-line rationale each.
   - **Task order** — Numbered steps (dependencies first).
   - **Validation** — Exact commands: at minimum `just check` (or `make check`) for Rust; add crate-scoped tests if they said so. For `.sruja`, include `sruja lint <file>`.
   - **Risks / unknowns** — What could invalidate the plan.

4. Ask the user to **review and freeze** the plan before implementation.

Do not start large refactors in this step unless the user explicitly asked for code in the same turn.

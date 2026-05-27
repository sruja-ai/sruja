---
description: Implement from a frozen plan in a focused session (Sruja)
---

**Session rule:** Treat this as implementation-only. The user should have a **frozen written plan** (issue section, `plan.md`, or pasted markdown). If they only have a vague goal, stop and tell them to run **sruja-plan-feature** first or paste the plan.

Steps:

1. Read the plan once. List the plan’s **task order** as a todo list and execute in order.
2. Follow `AGENTS.md` and `.cursor/rules/sruja-dev.mdc`. Respect crate boundaries and existing patterns.
3. Implement the smallest coherent diffs. No drive-by refactors outside the plan.
4. Run the **validation** section from the plan. If the plan omitted commands, run `just check` (or `make check`) from repo root. For `.sruja` edits, run `sruja lint` on touched files.
5. Summarize: files changed, tests run, any plan deviation (there should be none; if you had to deviate, say why explicitly).

If the conversation already contains a long exploration before this command, prefer **starting a new chat**, pasting only the plan + paths—per `AGENTS.md` session hygiene.

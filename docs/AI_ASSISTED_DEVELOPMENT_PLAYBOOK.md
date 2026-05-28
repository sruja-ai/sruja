# AI-assisted development playbook (Sruja harness)

This document turns common “AI assisted development” advice into an **enforceable, repeatable workflow** using Sruja’s deterministic harness: architecture evidence, explicit boundaries, and local verification gates.

If you want the high-level mental model first, start with:
- `docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md`
- `docs/HOST_AGENT_INTEGRATION.md`
- `docs/mcp_setup.md`

---

## Goal

Enable fast iteration with AI coding assistants **without** accumulating silent structural debt (layer violations, circular dependencies, “god modules”, diagram drift).

**In scope**
- Bounded code generation and refactors inside known architectural constraints
- Local “verify-task” loop before commit / PR
- Shared skills + editor rules for consistent outcomes across a team
- Architecture diagrams as exported artifacts (not hand-maintained drawings)

**Out of scope**
- Autonomous “AI engineer” workflows (Sruja is a harness; the editor/host owns the agent loop)
- Replacing code review (the harness reduces risk; humans still review intent and product correctness)

---

## Daily workflow (recommended)

### 1) Put a harness on the assistant (grounded context)

**What you want:** The assistant should *pull* bounded, machine-readable evidence instead of you pasting architecture rules into chat.

- **MCP setup**: follow `docs/mcp_setup.md`
- **Tool profile**: keep `SRUJA_MCP_TOOL_PROFILE=coding` for day-to-day tasks
- **Read-only mode (recommended)**: `SRUJA_MCP_READONLY=1` so an assistant can’t mutate proposals or write scratchpads unintentionally

**Editor rules as a stable “floor”**

Run this whenever architecture or dependency rules change:

```bash
sruja sync-ide-rules -r .
```

This keeps files like `.cursorrules`, `CLAUDE.md`, `.gemini/AGENTS.md`, and `llms-architecture.txt` aligned with the repo’s current architecture context.

### 2) Shift validation left (verify locally)

Treat this as the “adult supervision” loop: generate → verify → iterate → only then commit.

```bash
# Features / refactors
sruja verify-task --profile coding -r .

# Bug fixes (tight scope; include a target file)
sruja verify-task --profile bugfix --file <path> -r .

# Pre-merge hardening
sruja verify-task --profile review -r .
```

The goal is to catch architecture drift, broken boundaries, and intent mismatches **before** a reviewer sees the diff.

Cursor command reference: `.cursor/commands/sruja-verify-task.md`

### 3) Prefer architecture-as-code over “prompted rules”

Instead of repeating “don’t import X from Y” in every conversation:

- Maintain a reviewed baseline: `repo.sruja`
- Use deterministic enforcement:
  - `sruja lint repo.sruja`
  - `sruja drift -r . -a repo.sruja`

This makes structural constraints **versioned and reviewable**, and lets tools enforce them consistently across humans and assistants.

### 4) Standardize skills across the team

Use skills to make the assistant behave consistently across developers.

Start here:
- `docs/GETTING_STARTED_SKILL.md`
- `docs/INSTALL_AS_SKILL.md`
- `docs/COMMUNITY_SKILLS_STACK.md`

Recommended baseline for teams:
- A “task prime” skill (how to use `sruja focus`, the MCP ladder, and how to keep diffs small)
- A “verify-task before done” skill (always end with `sruja verify-task`)
- A “no drive-by refactors” skill (explicitly defer incidental cleanup)

### 5) Prevent diagram & documentation drift (exported byproduct)

Treat diagrams as **exported artifacts** from the code + architecture baseline, not manually curated drawings.

Book tutorial:
- `book/src/tutorials/basic/export-diagrams.md`

The enforcement loop is the same:
- Update code or `repo.sruja`
- `sruja lint` + `sruja drift`
- Export diagrams for updated visualizations

---

## CI envelope (minimal)

Add a PR gate that runs the same checks you run locally:
- `sruja verify-task` in CI

Examples/templates:
- `.github/workflows/sruja-verify-task.yml`
- `templates/github-actions/sruja-verify-task-pr.yml`
- `docs/examples/host-gates/verify-task-pr.yml`

---

## Practical guardrails (what breaks first)

- **Large diffs**: AI is most dangerous when it changes too much at once. Keep PRs small; validate after each slice.
- **Boundary erosion**: the harness can block forbidden dependencies, but it won’t invent missing architectural intent—write/maintain `repo.sruja` as reviewed truth.
- **“Looks right” bugs**: structural checks don’t prove product behavior; continue to require tests + review.


---
title: "Getting Started"
weight: 1
summary: "Start with drift + focus + verify-task. Add repo.sruja only when you want reviewed intent in CI."
difficulty: "beginner"
estimatedTime: "5 minutes"
---

# Getting Started

Sruja’s core value is **keeping AI edits aligned with your repo structure**:

- **Before** edits: brief the agent with `focus`
- **After** edits: run deterministic gates with `verify-task`
- **Always**: detect structural drift from real code, not from diagrams

You can get value on day one **without** writing any `.sruja` files.

---

## Two ways to use Sruja

- **Tier 1 (recommended first): harness without `repo.sruja`**
  - Structural scan + drift checks from your actual code
  - Agent briefings (`focus`) and post-edit gates (`verify-task`)
  - Best for evaluation and immediate guardrails
- **Tier 2 (optional): reviewed intent in Git (`repo.sruja`)**
  - Versioned, reviewable architecture intent
  - CI drift gates compare reality vs declared intent
  - Exports (Markdown/Mermaid) become derived artifacts

---

## Tier 1: Harness in 5 minutes (no `repo.sruja`)

### Step 1: Install the CLI

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

If you’re working from source in this repo:

```bash
just build  # or: make build
./target/release/sruja --help
```

### Step 2: Get a first structural read

From your repo root:

```bash
sruja start -r .
sruja drift -r . --structural-only --advisory
```

This reports repo-level structural issues that AI edits tend to introduce (cycles, layer violations, hub modules), plus an “uncertain” section where inference isn’t confident.

### Step 3: Use Sruja with an AI editor (focus → edit → verify)

Run a file-scoped briefing before a risky edit:

```bash
sruja focus -r . --file path/to/file.rs
```

After the agent edits:

```bash
sruja verify-task --profile coding -r .
```

### Step 4 (optional): Wire MCP for tool-based briefings

If your editor supports MCP, run Sruja as an MCP stdio server:

```bash
sruja mcp -r .
```

For host/editor setup details, see [Host agent integration](https://github.com/sruja-ai/sruja/blob/main/docs/HOST_AGENT_INTEGRATION.md).

---

## Tier 2 (optional): Reviewed `repo.sruja` in Git

Tier 2 is for teams that want **reviewed intent** in Git and strict CI enforcement. Keep it minimal at first.

### Step 1: Generate and validate intent

Prerequisites:

- **AI editor** – Cursor, Copilot, Claude, Continue.dev, etc.
- **AI skill** – `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` (see [Install as a Skill](../../docs/INSTALL_AS_SKILL.md))

The skill uses repo evidence (Tree-sitter graph + summaries) to propose `repo.sruja`. When you have a draft:

```bash
sruja lint repo.sruja
```

### Step 2: Refresh evidence and check drift

```bash
sruja sync -r .
sruja drift -r . -a repo.sruja
```

### Step 3: CI drift gate (recommended)

One-time baseline:

```bash
sruja baseline -r . -o .sruja/violations.baseline.json
```

CI check that fails only on new problems:

```bash
sruja drift --ci -r . --baseline .sruja/violations.baseline.json -f github-actions
```

---

## About architecture files (optional)

You can use Sruja purely as a harness (Tier 1) and never commit an architecture file.

If you do want reviewed intent in Git (Tier 2), today that intent is stored as an **architecture file** (`repo.sruja`). The exact authoring format is **not the product center** and may evolve; the stable value is the workflow around it:

- evidence from code → task-scoped context → deterministic gates → drift checks in CI

---

## Quick reference

| Goal | Command |
|------|---------|
| Structural scan (no files required) | `sruja start -r .` |
| Structural drift (advisory) | `sruja drift -r . --structural-only --advisory` |
| Brief an agent before edits | `sruja focus -r . --file <path>` |
| Post-edit gates | `sruja verify-task --profile coding -r .` |
| Refresh evidence | `sruja sync -r .` |
| Lint reviewed intent | `sruja lint repo.sruja` |
| Drift vs reviewed intent | `sruja drift -r . -a repo.sruja` |
| Start MCP server | `sruja mcp -r .` |

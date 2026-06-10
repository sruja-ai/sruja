---
title: "Getting Started"
weight: 1
summary: "Start with drift + focus + verify-task. Add repo.sruja only when you want reviewed intent in CI."
difficulty: "beginner"
estimatedTime: "5 minutes"
---

# Getting Started

Sruja’s core value is **capturing knowledge, retrieving task context, and verifying changes against decisions**:

- **Capture**: bring important docs and decisions into the repo
- **Retrieve**: brief the next change with `focus`, `ai`, or MCP
- **Verify**: run deterministic checks after editing

You can get value on day one **without** writing any `.sruja` files.

---

## The core loop

- **Capture**: ingest docs and record decisions you want future humans and AI to reuse
- **Retrieve**: generate task-scoped context before touching code
- **Verify**: confirm the result still matches structure and intent

---

## Core workflow in 5 minutes

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

This reports repo-level structural issues that AI edits tend to introduce, plus an “uncertain” section where inference is weaker.

### Step 3: Retrieve task context before editing

Run a file-scoped briefing before a risky edit:

```bash
sruja focus -r . --file path/to/file.rs
```

For a paste-ready brief:

```bash
sruja ai -r . --task "Refactor auth boundary"
```

### Step 4: Verify after editing

After the agent edits:

```bash
sruja verify-task --profile coding -r .
```

### Step 5 (optional): Wire MCP for tool-based briefings

If your editor supports MCP, run Sruja as an MCP stdio server:

```bash
sruja mcp -r .
```

For host/editor setup details, see [Host agent integration](https://github.com/sruja-ai/sruja/blob/main/docs/HOST_AGENT_INTEGRATION.md).

---

## Optional: reviewed intent in Git

Add `repo.sruja` only when you want **reviewed intent** in Git and stricter CI enforcement. Keep it minimal at first.

### Step 1: Generate and validate intent

Prerequisites:

- **AI editor** – Cursor, Copilot, Claude, Continue.dev, etc.
- **AI skill** – `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` (see [Install as a Skill](../../docs/INSTALL_AS_SKILL.md))

The skill uses repo evidence to propose `repo.sruja`. When you have a draft:

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

## About reviewed intent

You can use Sruja for capture, retrieve, and verify without ever committing an architecture file.

If you do want reviewed intent in Git, today that intent is stored as an **architecture file** (`repo.sruja`). The exact authoring format is **not the product center** and may evolve; the stable value is the workflow around it:

- evidence from code → task-scoped context → deterministic gates → drift checks in CI

---

## Quick reference

| Goal | Command |
|------|---------|
| Structural scan | `sruja start -r .` |
| Structural drift | `sruja drift -r . --structural-only --advisory` |
| Brief an agent before edits | `sruja focus -r . --file <path>` |
| Generate a paste-ready task brief | `sruja ai -r . --task "..."` |
| Post-edit verification | `sruja verify-task --profile coding -r .` |
| Refresh evidence | `sruja sync -r .` |
| Lint reviewed intent | `sruja lint repo.sruja` |
| Drift vs reviewed intent | `sruja drift -r . -a repo.sruja` |
| Start MCP server | `sruja mcp -r .` |

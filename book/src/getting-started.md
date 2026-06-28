# Quick Start

**Get Sruja, your autonomous coding agent, running in about 5 minutes.** It scans your repo, briefs each task, edits code in a closed loop, and verifies results with an independent grader before you ship. No `repo.sruja` required on day one.

---

## Step 1: Install the CLI (1 minute)

```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja start -r .
sruja drift -r . --structural-only --advisory
```

You get file-level findings (cycles, layer violations, hub modules) and a **could not infer** section where the scan is uncertain.

---

## Step 2: Wire your editor (MCP + harness skill)

**MCP (Cursor template in the repo):** register `sruja mcp` with profile `coding` and `SRUJA_MCP_READONLY=1`. See [Host agent integration](https://github.com/sruja-ai/sruja/blob/main/docs/HOST_AGENT_INTEGRATION.md).

**Skills:**

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

Optional — reviewed architecture in Git:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

**Host loop:**

```text
focus / drift  →  your agent edits code  →  verify-task  →  (optional) agent record on failure
```

---

## Step 3: Run gates locally

```bash
# Before a task (paste briefing into the agent)
sruja focus -r . --file path/to/file.rs

# After the agent edits
sruja verify-task --profile coding -r .
```

In VS Code/Cursor: **Sruja: Brief this file**, **Sruja: Verify task**, **Sruja: Register MCP Server**.

---

## Step 4 (optional) — Reviewed `repo.sruja`

When you want versioned architecture intent in CI, ask your agent:

```text
Use sruja-architecture. Gather evidence, generate or update repo.sruja,
then sruja lint repo.sruja and sruja drift -r . -a repo.sruja.
```

Diagrams are exports, not the product center:

```bash
sruja export mermaid repo.sruja > architecture.mmd
```

---

## Quick Reference

| What you want | Command |
|---------------|---------|
| **First scan** | `sruja start -r .` then `sruja drift -r . --structural-only --advisory` |
| **Harness skill** | `npx skills add … --skill sruja-harness` |
| **Brief agent** | `sruja focus -r . --file <path>` |
| **After AI edit** | `sruja verify-task --profile coding -r .` |
| **Architecture skill** | `npx skills add … --skill sruja-architecture` |
| **Lint reviewed truth** | `sruja lint repo.sruja` |

---

## Common Questions

**"What if `sruja` isn't found?"**

```bash
export PATH="$HOME/.local/bin:$PATH"
# or: just build (or make build) && use target/release/sruja
```

**"Do I need MCP?"**

No for CLI-only workflows. MCP is the best experience inside Cursor/Claude Desktop.

**"Is Sruja a coding agent?"**

Yes. `sruja agent loop` drives the full closed-loop agent (comprehend → plan →
execute → critique → replan). Sruja also works as a passive harness from any
editor (Cursor, Copilot) via MCP.

**"What's the difference between `quickstart` and `discover`?"**

- **`quickstart` / `start`** — human-friendly onboarding
- **`discover`** — detailed JSON for AI generation (architecture skill)

---

## VS Code extension

Install **Sruja Language Support** for `.sruja` syntax, validation, harness commands, and MCP registration.

---

## Learn more

- [Beginner path](docs/beginner-path.md)
- [Install as skill (full)](https://github.com/sruja-ai/sruja/blob/main/skills/README.md)
- [Host gate examples](https://github.com/sruja-ai/sruja/tree/main/docs/examples/host-gates)
- [Getting started (full DSL walkthrough)](docs/getting-started.md)

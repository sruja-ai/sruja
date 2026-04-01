---
title: "Getting Started"
weight: 1
summary: "Generate architecture with AI in 5 minutes."
difficulty: "beginner"
estimatedTime: "5 minutes"
---

# Getting Started

**Architecture from your code—no DSL learning required.**

Your AI writes and maintains `.sruja` files. You just need to know what to ask for.

---

## Prerequisites

- **AI editor** – Cursor, Copilot, Claude, Continue.dev, etc.
- **AI skill** – Install first: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` (see [Install as a Skill](../../docs/INSTALL_AS_SKILL.md))
- **Sruja CLI** – Needed when the skill runs discover/lint/drift; the skill will guide you to install it if missing (`curl -fsSL https://sruja.ai/install.sh | bash`)

---

## Step 1: Gather evidence (or let the skill do it)

When you use the **sruja-architecture skill**, it runs discovery for you. Discovery is **not** just a file list—under the hood the CLI runs **Tree-sitter** on your code to build a **graph of components and their dependency relations** (who imports whom, which modules call which). That graph is the evidence the skill uses.

If you want to run discovery yourself (e.g. to inspect the summary), run:

```bash
cd your-project
sruja discover --context -r . --format json
```

**What this does:** Scans your repo with Tree-sitter, builds the component/dependency graph, then outputs a **summary** of that graph (so the AI can scope and ask targeted questions). The skill prefers `.sruja/context.json` and `.sruja/graph.json` when present (e.g. after `sruja sync` or the extension's **Refresh repo context**); when missing, it runs discover for you—no need to run a command first. For the full graph on demand: `sruja scan -r . -o graph.json`, or use the graph written by sync at `.sruja/graph.json`.

**Summary output includes:**
- Component and edge **counts** (from the Tree-sitter graph)
- Primary language and framework
- Inferred architecture style (monolith / microservices)
- Suggested areas (top-level path segments for scoping)

**Example summary (actual schema):**

```json
{
  "repo": "my-app",
  "scan_scope": { "included": [], "excluded": [] },
  "components": 42,
  "edges": 58,
  "primary_language": "TypeScript",
  "framework": "React",
  "architecture_style": "monolith",
  "domain": null,
  "suggested_areas": ["src", "lib", "apps"]
}
```

The **full graph** is written by `sruja sync` to `.sruja/graph.json`. You can also produce it on demand with `sruja scan -r . -o graph.json`.

---

## Step 2: Generate Architecture with AI

In your AI editor, run:

```
Use sruja-architecture. Analyze the discovery output:
[JSON from step 1],
identify systems, containers, and their relationships,
generate repo.sruja using C4 context and container levels,
then run `sruja lint` and fix until it passes.
```

**What your AI will do:**

1. **Analyze** the JSON output from discovery
2. **Ask questions** if scope is unclear (e.g., "What's this service for?")
3. **Generate** `repo.sruja` with your architecture
4. **Validate** it with `sruja lint`
5. **Fix** any errors automatically

---

## What repo.sruja Looks Like

```sruja
import { * } from 'sruja.ai/stdlib'

// External actors
MobileApp = person "Mobile App" {
  description "Customer-facing mobile application"
}

// Main system
MyApp = system "My Application" {
  description "Handles user requests and processing"

  // Containers (deployable units)
  API = container "API Service" {
    technology "Node.js + Express"
    description "RESTful API for mobile and web clients"
  }

  Worker = container "Background Worker" {
    technology "Node.js + Bull"
    description "Processes async jobs (emails, reports)"
  }

  // Datastores
  Database = database "Primary DB" {
    technology "PostgreSQL"
    description "Stores user data and transactions"
  }

  Cache = database "Redis Cache" {
    technology "Redis"
    description "Caches frequently accessed data"
  }
}

// Relationships (how things connect)
MobileApp -> MyApp.API "HTTPS requests"
MyApp.API -> MyApp.Database "SQL queries"
MyApp.API -> MyApp.Cache "Redis get/set"
MyApp.Worker -> MyApp.Database "SQL queries"
```

**Key concepts:**

- **person** – External actors (users, systems calling you)
- **system** – Major boundary (your entire application)
- **container** – Deployable unit (API, worker, web frontend)
- **database** – Data storage or cache
- **->** – Relationship with protocol description

---

## Step 3: Validate

After the AI generates `repo.sruja`, validate it:

```bash
sruja lint repo.sruja
```

**What this checks:**

- **Syntax errors** – Invalid structure or keywords
- **Circular dependencies** – A depends on B, B depends on A
- **Orphan elements** – Something with no connections
- **Missing fields** – Required information not provided

**Fix errors:** Paste the lint output to your AI and say: "Fix these errors."

---

## Step 4: Export for Documentation

### Export Markdown

```bash
sruja export markdown repo.sruja > ARCHITECTURE.md
```

Creates a readable document you can share with your team.

### Export Mermaid Diagram

```bash
sruja export mermaid repo.sruja > ARCHITECTURE.mmd
```

Creates a diagram you can:
- Open in [Mermaid Live Editor](https://mermaid.live)
- Import into VS Code with the extension
- Add to documentation

### Export JSON

```bash
sruja export json repo.sruja > ARCHITECTURE.json
```

Machine-readable format for tools and automation.

---

## Understanding C4 Levels

Sruja uses the C4 Model, which organizes architecture into levels:

| Level | What It Is | Example |
|--------|--------------|----------|
| **Person** | External actors | Users, external systems, third-party APIs |
| **System** | High-level boundary | "Order System", "User Management System" |
| **Container** | Deployable unit | "API Service", "Web App", "Worker" |
| **Component** | Internal part | "Payment Module", "Auth Controller" |

**Recommended:** Start with Person + System + Container levels. Add components only when you need more detail.

---

## Common Questions

**"When should I use stdlib imports?"**

Always. It saves time by providing standard types (person, system, container, etc.) so you don't define them manually.

**"What if discovery doesn't find my code?"**

1. Check your language is supported (JavaScript, Python, Go, Rust, Java)
2. Make sure you're in the correct directory
3. Try `sruja quickstart -r .` to see what's detected

**"How detailed should repo.sruja be?"**

**Start minimal.** Only model what you actually need:
- External actors calling your system
- Major containers (services, apps)
- Key datastores

Add more detail only when it provides value.

**"Can I edit repo.sruja manually?"**

Yes, but it's easier to let AI do it. If you do edit manually:
- Run `sruja lint` before committing
- Validate syntax with the extension

---

## Next Steps

- **Beginner Path:** [Beginner Path](./beginner-path.md) – 7 steps to go deeper
- **Examples:** [Examples Gallery](../examples/index.md) – Real-world architectures
- **Language Reference:** [Language Specification](../reference/language-spec.md) – Complete DSL guide

---

## Quick Reference

| Want to | Command |
|----------|----------|
| **Analyze code** | `sruja discover --context -r . --format json` |
| **Generate baseline** | `sruja quickstart -r . --generate-baseline` |
| **Validate** | `sruja lint repo.sruja` |
| **Refresh evidence** | `sruja sync -r .` |
| **Repo health** | `sruja status -r .` |
| **Export Markdown** | `sruja export markdown repo.sruja > doc.md` |
| **Export Mermaid** | `sruja export mermaid repo.sruja > diagram.mmd` |
| **Export JSON** | `sruja export json repo.sruja > arch.json` |
| **Check drift** | `sruja drift -r . -a repo.sruja --format json` |

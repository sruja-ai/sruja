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

- **Sruja CLI** – See [Quick Start](../getting-started.md) to install
- **AI editor** – Cursor, Copilot, Claude, Continue.dev, etc.
- **AI skill** – See [Install as a Skill](../../docs/INSTALL_AS_SKILL.md)

---

## Step 1: Analyze Your Codebase

Run this in your project folder:

```bash
cd your-project
sruja discover --context -r . --format json
```

**What this does:** Analyzes your code and returns detailed information.

**Output includes:**
- Repository structure
- Detected technologies (Node.js, Python, Go, etc.)
- Module boundaries
- Entry points
- Dependencies

**Example output:**

```json
{
  "repo": "my-app",
  "technologies": ["Node.js", "PostgreSQL", "Redis"],
  "modules": [
    {"name": "api", "type": "service"},
    {"name": "worker", "type": "service"}
  ],
  "databases": [
    {"name": "postgres", "technology": "PostgreSQL"}
  ]
}
```

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
| **Validate** | `sruja lint repo.sruja` |
| **Export Markdown** | `sruja export markdown repo.sruja > doc.md` |
| **Export Mermaid** | `sruja export mermaid repo.sruja > diagram.mmd` |
| **Export JSON** | `sruja export json repo.sruja > arch.json` |
| **Check drift** | `sruja drift -r . --format json` |

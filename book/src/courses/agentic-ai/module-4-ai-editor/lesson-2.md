---
title: "Lesson 2: Context-Driven AI Assistance"
weight: 2
summary: "Build and optimize architecture context for AI-assisted development."
---

# Lesson 2: Context-Driven AI Assistance

## Why Context Matters

AI editors work best with relevant context. Without architecture context:
- AI makes assumptions about your system
- AI doesn't know your constraints
- AI suggests generic patterns that may not fit

With architecture context:
- AI understands your system's structure
- AI knows which components are critical
- AI provides relevant suggestions

## Building Architecture Context

### Basic Context

```bash
# Build context for current repo (outputs cursor rules format)
sruja ai-context -r .

# Output as JSON
sruja ai-context -r . --format json

# Output as Markdown
sruja ai-context -r . --format markdown
```

### Cross-Repo Context

```bash
# Build context across multiple repos
sruja ai-context -r repoA -r repoB -r repoC

# Useful for monorepos or microservices
```

### Focus Mode

For specific tasks, use focus mode to get a task-scoped briefing:

```bash
# Get context focused on a specific file
sruja focus --file src/api/main.rs -r .

# Get context for a specific component
sruja focus --element-id PaymentService -r .

# Optional: compare a git range to the scan graph (temporal context on the briefing)
sruja focus --element-id MySystem.Api --base-ref main --head-ref HEAD -r .

# Base only: head defaults to HEAD
sruja focus --file src/api/main.rs --base-ref main -r .
```

### Context lineage on disk

Sruja also records **lightweight decision-style traces** for the repo (not a separate graph model—see [CONTEXT_ENGINEERING](../../../../../docs/CONTEXT_ENGINEERING.md#context-graphs-sruja-vs-industry-usage)):

- **`.sruja/context_events.jsonl`** — append-only JSON lines when you run **`sruja intent check`**, **`sruja drift`** (with architecture compare), or **`sruja propose approve …`** (merged proposal). Useful for “what did we last check?” in CI or audits.
- **`.sruja/agent_memory.json`** — curated **`LearningEntry`** records (`context`, `hypothesis`, `outcome`, `guardrail_advice`, optional `hitl_kind`). Written via `sruja agent record` or MCP `sruja_record_learning`. See [Grounded harness and continual learning](../../../../docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md).
- **`.sruja/agent/runs/<run_id>/facts_bundle.json`** — produced when an **agent apply** run finishes verification (replay bundle).

Focus output includes:
- **Blast radius**: What depends on this
- **Decisions**: Why this was designed this way
- **Boundaries**: What this component owns
- **AI instructions**: How to work with this component

## Context Score

Measure how "AI-ready" your architecture is:

```bash
# Get AI-readiness score (0-100)
sruja context-score -r .
```

The score evaluates 5 dimensions:
| Dimension | What It Measures |
|-----------|-----------------|
| **Documentation** | Are components documented? |
| **Relationships** | Are dependencies explicit? |
| **Constraints** | Are policies defined? |
| **Coverage** | How much code is mapped? |
| **Freshness** | Is context up-to-date? |

## Optimizing Context for AI

### 1. Add Descriptions

```sruja
<!-- partial -->
container PaymentService {
  description "Handles payment processing and fraud detection"
}
```

### 2. Link Requirements

```sruja
<!-- partial -->
container PaymentService {
  requirement "Must comply with PCI-DSS"
  requirement "Must process 1000 TPS"
}
```

### 3. Add Intent

```sruja
<!-- partial -->
intent "Payment Goals" {
  outcome "All payments processed within 5 seconds"
}
```

### 4. Document Decisions

```sruja
<!-- partial -->
PaymentService = container "Payment Service" {
  description "Handles payments"

  adr "ADR-0012: Why we use Stripe instead of PayPal"
  adr "ADR-0015: Why payments are synchronous"
}
```

## Daily Context Sync

For AI editors, run daily sync to keep context fresh:

```bash
# Quick sync
sruja daily -r .

# With health check
sruja doctor -r .
```

This updates:
- `.sruja/context.json`
- `.cursorrules`
- `.copilot-instructions.md`
- `CLAUDE.md`

## Hands-On: Build AI Context

1. **Generate context for your repo:**
   ```bash
   sruja ai-context -r . --format markdown
   ```

2. **Check your context score:**
   ```bash
   sruja context-score -r .
   ```

3. **Focus on a specific component:**
   ```bash
   sruja focus --element-id UserAPI -r .
   ```

4. **Optional — temporal focus for a PR-style range:**
   ```bash
   sruja focus --element-id UserAPI --base-ref main --head-ref HEAD -r .
   ```

5. **Run daily sync:**
   ```bash
   sruja daily -r .
   ```

## Learning Outcomes

- ✅ Generate AI-ready context using `sruja ai-context`
- ✅ Measure architecture AI-readiness with `sruja context-score`
- ✅ Use `sruja focus` for task-specific context (including optional **git-range** temporal context)
- ✅ Know where **context lineage** artifacts live (`context_events.jsonl`, agent memory, agent run facts bundles)
- ✅ Optimize architecture for better AI assistance

## Quiz: Test Your Understanding

### Q1: What command generates AI context for your codebase?

A) `sruja context`
B) `sruja ai-context`
C) `sruja generate`
D) `sruja build`

### Q2: What does `sruja context-score` measure?

A) Network speed
B) AI-readiness of your architecture (0-100 score across 5 dimensions)
C) Code test coverage
D) Server response time

### Q3: What does `sruja focus` provide?

A) Task-specific context including blast radius and design decisions
B) Global system configuration
C) Database schema
D) Network topology

### Q4: What does `.sruja/context_events.jsonl` capture?

A) Raw git packfiles
B) Append-only summaries of intent checks, drift runs, and merged proposals (with optional policy fingerprint)
C) Your editor undo history
D) npm lockfile metadata

## Next Steps

Lesson 3 covers patterns for effective AI pair programming with architecture.
# Sruja Context Engineering Platform

Sruja is more than an architecture-as-code tool; it is a **Context Engineering** platform designed to provide high-fidelity, task-scoped context for AI coding agents.

By quantifying and surfacing architectural evidence, Sruja reduces the "context gap" that leads to AI hallucinations and architectural drift.

## Core Pillars

### 1. Context Score (AI-Readiness)
The **Context Score (0–100)** is the "Lighthouse score" for your repository's context. It measures five critical dimensions:
- **Architecture Coverage**: % of code modules mapped in your `.sruja` files.
- **Decision Completeness**: How many ADRs and decisions are linked to architecture elements.
- **Evidence Freshness**: How recently the architectural evidence was refreshed via `sruja sync`.
- **Relationship Density**: The connectivity of your architecture graph.
- **External Context**: The availability of non-code context (ADRs, design docs, etc.).

**Command:** `sruja context-score` or `sruja status`

### 2. Task-Scoped Briefing (Focus)
Before an AI agent starts a task, it needs to know the specific architectural constraints and impact area. `sruja focus` generates a comprehensive briefing including:
- **Blast Radius**: Upstream and downstream impact analysis.
- **Linked Decisions**: Active ADRs affecting the target.
- **Boundary Constraints**: Inferred policy violations.
- **AI Instructions**: Specific guidance for the LLM.

**Command:** `sruja focus --file <path>` or `sruja focus --element-id <id>`

### 3. Documentation Ingestion
Import external context (Design Docs, RFCs, ADRs) into the `.sruja/context/` directory. Sruja automatically indexes these files and links them to architectural components via YAML front-matter.

**Command:** `sruja ingest <path-to-doc>`

**Front-matter Example:**
```yaml
---
elements: [Payment.Service, Database.Transactions]
category: adr
---
# ADR-005: Idempotency keys for payments
...
```

## Integrating with AI Agents

### MCP (Model Context Protocol)
Sruja provides an MCP server that exposes these context engineering capabilities directly to AI editors (Cursor, Trae, Windsurf, etc.).

**Tools available:**
- `sruja_get_context_score`: Repository-level AI-readiness.
- `sruja_get_focus_briefing`: Task-scoped briefing.
- `sruja_get_architecture_context`: Component-level hydration.

### PR & CI Integration
You can use the context score as a gate in CI. If a PR significantly drops the context score (e.g., by adding many unmapped modules), the build can fail, ensuring context stays fresh as the codebase grows.

**Command:** `sruja daily` (alias for `sruja review`) shows the score in the daily dashboard.

## Best Practices
1. **Sync Daily**: Run `sruja daily` every morning to refresh evidence.
2. **Link Everything**: Use the `elements:` field in your markdown docs to link them to the architecture.
3. **Fix Quick Wins**: `sruja context-score` provides a list of "Quick Wins"—tasks that provide the highest ROI for your context health.

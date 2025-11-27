# Cursor AI Integration

[← Back to Notebooks Index](./README.md)

## Overview

Sruja integrates with **Cursor AI** (and VS Code Copilot Chat) as the primary AI engine for architecture assistance - **without requiring an additional AI subscription**.

**Key Insight:** You don't need to build or host your own AI. Cursor provides the AI reasoning; Sruja provides the architecture state and validation.

## Architecture

```
           ┌──────────────────────────────────┐
           │             Cursor AI            │
           │ (Chat + Inline Edits + Commands) │
           └───────▲──────────────────────────┘
                   │
             Reasoning / Rewrite
                   │
                   ▼
       ┌───────────────────────────────┐
       │  VS Code (Notebook + LSP + UI)│
       │  - Renders cells              │
       │  - Provides code actions      │
       └───────▲───────────────────────┘
               │
               │ Notebook Commands / MCP Calls
               ▼
     ┌──────────────────────────────────┐
     │       Sruja Kernel (WASM/Go)     │
     │  - DSL parsing                   │
     │  - IR state                      │
     │  - Validators                    │
     │  - Query engine                  │
     │  - Snapshots, variants           │
     │  - Diagram generator             │
     └──────────────────────────────────┘
```

## Design Philosophy

- **Cursor AI** = Reasoning engine (stateless)
- **Sruja Kernel** = Architecture state + validation (stateful)
- **MCP Tools** = Action layer

Together they form a powerful architecture assistant.

## Integration Components

### 1. Workspace IR Files

Kernel automatically exports architecture state to files:

```
.sruja/
  sruja-ir.json          # Full architecture IR
  sruja-graph.json       # Dependency graph
  sruja-summary.md       # Human-readable summary
```

Cursor indexes these automatically → becomes architecture-aware.

### 2. Sruja LSP

LSP provides:

- Autocompletion of DSL keywords
- Autocompletion of symbol names
- Hovers showing entity definitions
- Diagnostics
- Go-to-definition

Cursor uses this automatically for DSL editing.

### 3. MCP Tools

Sruja exposes MCP tools that Cursor can call:

- `sruja.read.model` - Get architecture IR
- `sruja.list_violations` - List validation errors
- `sruja.apply_patch` - Apply architecture changes
- `sruja.generate_diagram` - Generate diagrams
- ... (see [MCP Tools Reference](./mcp-tools.md))

### 4. Cursor Project Instructions

Create `.sruja/.cursor-instructions`:

```
You are working in a Sruja Architecture repository.
Use the Sruja DSL in .ipynb cells or .sruja files.
Always align suggestions with `.sruja/sruja-ir.json`.
Use MCP tool calls to validate or apply architecture changes.
When modifying DSL, ensure the IR remains consistent and valid.
```

This makes Cursor AI **architecture-aware** without extra training.

## Workflows

### 1. Architecture Chat Mode

Cursor can create "named chats" with context.

Create `.sruja/architecture.context.md`:

```markdown
# Architecture Context

## Current Architecture
[IR summary from .sruja/sruja-ir.json]

## Principles
- Domain-driven design
- Event-driven communication
- API-first approach

## Rules
- No direct database access from services
- All events must have schemas
- ...
```

Cursor loads this automatically during chat.

### 2. Fix Architecture Violations

**User:** "Fix all architecture violations"

**Cursor AI workflow:**
1. Reads diagnostics from notebook
2. Reads IR from `.sruja/sruja-ir.json`
3. Calls MCP: `sruja.list_violations`
4. Generates DSL patches
5. Calls MCP: `sruja.apply_patch`
6. Notebook re-executes cell
7. New IR exported
8. Cursor verifies again
9. Repeats until clean

**Result:** Self-correcting architecture loop.

### 3. Auto-Generate DSL

Cursor can generate architecture DSL because:

- LSP provides syntax knowledge
- IR provides current architecture context
- MCP tools validate changes

**User:** "Add a retry policy to PaymentService"

**Cursor:**
- Reads current architecture
- Generates DSL patch
- Applies via MCP
- Validates result

### 4. Code Alignment

Sruja provides MCP tools for code alignment:

```
sruja.align_code_with_architecture
```

**Cursor workflow:**
1. Inspects codebase
2. Reads IR
3. Detects missing APIs
4. Suggests code generation/patches
5. Auto-applies (or creates PRs)

### 5. Architecture Refactoring

Cursor can perform:

- API contract drift detection
- Breaking-change suggestions
- Boundary violations fixes
- Naming conventions enforcement
- Automatic diagram regeneration
- Variant exploration

All driven by IR + MCP tools.

## AI Cell Execution

When user writes an AI cell:

```python
ai refine system Billing for reliability
```

**Behind the scenes:**
1. Kernel detects AI cell type
2. Extracts AI instructions
3. Calls Cursor AI with context:
   - Current IR
   - Validation diagnostics
   - Architecture rules
4. Cursor returns suggestions
5. Kernel applies patches (with user approval)
6. Re-validates
7. Updates IR

**User doesn't need OpenAI subscription** - uses Cursor's AI.

## IR-to-AI Context Bridge

To help Cursor understand architecture:

### Export IR Summary

Kernel exports readable summary:

```markdown
# Architecture Summary

## Systems
- Billing (3 containers, 5 components)
- Payments (2 containers, 4 components)

## Entities
- Payment (amount, currency, status)
- Invoice (total, due_date)

## Events
- PaymentAuthorized
- PaymentCompleted

## Violations
- 2 errors, 5 warnings
```

### Architecture Graph

Export dependency graph:

```json
{
  "nodes": [...],
  "edges": [...],
  "metadata": {...}
}
```

Cursor can visualize and reason about relationships.

## Benefits

### ✅ Zero Additional AI Cost

- No OpenAI subscription needed
- No self-hosted LLM needed
- Uses Cursor's existing AI

### ✅ Architecture-Aware AI

- Cursor understands current architecture
- Suggestions are context-aware
- Changes respect architectural constraints

### ✅ Self-Healing Architecture

- AI fixes violations automatically
- Continuous refinement
- Architecture improves over time

### ✅ Code-Architecture Alignment

- AI ensures code matches architecture
- Drift detection and fixes
- Automated compliance

## Setup Instructions

### 1. Install Cursor

Get Cursor IDE (has built-in AI).

### 2. Configure MCP

Add Sruja MCP server to Cursor config:

```json
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp", "serve"]
    }
  }
}
```

### 3. Add Project Instructions

Create `.sruja/.cursor-instructions` (see above).

### 4. Export IR

Kernel automatically exports to `.sruja/sruja-ir.json`.

### 5. Start Using

- Open notebook in Cursor
- Edit DSL cells
- Use Cursor chat: "fix violations"
- Watch AI improve architecture

## Next Steps

- [MCP Tools Reference](./mcp-tools.md) - Available MCP tools
- [Architecture Kernel](./kernel.md) - Kernel details


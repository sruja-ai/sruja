# Phase 1: Core Diagramming Engine

This document outlines Phase 1 implementation plan for building a robust, bidirectional architecture diagramming and modeling engine.

[← Back to Documentation Index](../README.md)

## Goal

Build the foundational Go CLI tool that everything else plugs into. The tool must support:

- **DSL → JSON Model → Mermaid** (one-way compilation)
- **Multi-level abstraction (VHLD → HLD → LLD)**
- **Architecture-aware objects** (Services, APIs, Datastores, Events, Domains)
- **Reusable component templates**
- **Model → JSON Export**
- **Model validation**
- **Mermaid diagram generation** (primary output format)

## Implementation Order

### 1. Define the Metamodel

This is the universal schema. All DSL, diagrams, and future tools must map to this.

**Minimal initial metamodel:**

```json
{
  "workspace": {
    "elements": [
      {
        "type": "Person" | "System" | "Container" | "Component" | "DataStore" | "Queue" | "ExternalService",
        "id": "string",
        "name": "string",
        "description": "string",
        "technology": "string",
        "tags": ["string"]
      }
    ],
    "relations": [
      {
        "from": "element-id",
        "to": "element-id",
        "type": "Uses" | "Publishes" | "Subscribes" | "DependsOn",
        "description": "string"
      }
    ]
  }
}
```

**Advanced later:**
- Domains / Bounded Contexts
- Events
- Requirements
- ADRs
- User journeys

But not now — core first.

---

### 2. Pick the DSL Format

Start simple, DSL-like syntax:

```sruja
system AuthService {
  tech = "Node.js"
  description = "Handles login and tokens"

  component SessionManager {
    tech = "Redis"
  }

  uses UserService "verifies user identity"
}
```

**Later additions:**
- imports
- versioned libraries
- ADRs
- constraints

---

### 3. Build the Parser + JSON Model Compiler

Everything compiles to canonical JSON:

```json
{
  "elements": [...],
  "relationships": [...],
  "views": [...]
}
```

This will also be your **MCP data contract** later.

**Implementation in Go:**
- Use a parser generator (e.g., `goyacc`, `participle`, or hand-written recursive descent)
- Build AST from parsed DSL
- Transform AST to JSON model
- Validate JSON model against schema

---

### 4. Build the Renderer (Diagram Engine)

**Initial approach: Compile to Mermaid**

Since Sruja compiles to Mermaid by default:

- Parse DSL → JSON model
- Transform JSON model → Mermaid C4 syntax
- Output Mermaid diagram code

**Future: Custom UI Renderer** (See [UI & Future Features](../ui-future/README.md))
- Use JSON model as source of truth
- Build custom renderer (React/TypeScript)
- Support force-directed layout (ELK.js or Dagre)
- Zoom/pan, grouping, swimlanes
- Clean edges + auto layout

---

### 5. Build the Visual Editor (UI) - Future Phase

**Planned for later:** See [UI & Future Features](../ui-future/README.md)

---

### 6. Build the Compilation Pipeline

### **DSL → JSON → Mermaid** (Phase 1 - Current Focus)

**Rules:**
- Canonical ordering (to avoid diff noise)
- Immutable IDs for elements
- DSL regeneration must be deterministic
- Conflict resolution rules

This feature alone differentiates you from 99% of tools.

---

### 7. Build Model Validation Engine

**Initial validations:**
- Missing technology
- Missing descriptions
- Unconnected components
- Circular dependencies
- Invalid abstraction level usage

**Later, add:**
- Clean architecture rules
- Microservices constraints
- Organization-specific policies

See [Validation Engine Documentation](../guides/validation-engine.md) for details.

---

## Phase 1 Scope

### Must Have:
- ✅ DSL parser + JSON compiler (Go)
- ✅ JSON model → Mermaid diagram
- ✅ Model validation (basic rules)
- ✅ Basic architecture elements
- ✅ 2 abstraction layers (VHLD, HLD)
- ✅ Export (Mermaid, JSON)
- ✅ CLI tool (`sruja compile`, `sruja validate`)

### Can Skip (for now):
- ❌ LLD
- ❌ Requirements
- ❌ ADRs
- ❌ Journeys
- ❌ Libraries
- ❌ AI/MCP (Future)
- ❌ Visual UI editor (Future - see [UI & Future Features](../ui-future/README.md))
- ❌ Real-time collaboration (Future)
- ❌ Team features (Future)

Keep first version focused on CLI tool.

---

## Architecture Overview

```
                   ┌──────────────────────┐
                   │     DSL File         │
                   │   (.sruja)           │
                   └──────────┬───────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ DSL → AST Parser │
                    │    (Go)          │
                    └───────┬──────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │ JSON MODEL   │  <─── source of truth
                     │   (Go)       │
                     └─────┬────────┘
                           │
          ┌────────────────┴──────────────────┐
          ▼                                   ▼
┌────────────────────┐              ┌────────────────────────┐
│ Mermaid Compiler   │              │ Validation Engine      │
│    (Go)            │              │    (Go)                │
└────────────────────┘              └────────────────────────┘
          │
          ▼
┌────────────────────┐
│ Mermaid Output    │
│ (.mmd)            │
└────────────────────┘
```

This clean separation ensures extensibility.

---

## Why This Approach Works

- ✅ Not locked into Mermaid, D2, or Structurizr
- ✅ Full ownership of metamodel → allows expansion into AI, codegen, learning later
- ✅ Easy to add features incrementally
- ✅ JSON → perfect for MCP / AI / exports
- ✅ DSL → perfect for Git workflows
- ✅ Mermaid output → immediate visualization

This becomes **the architecture platform of the future**.

---

## Technology Stack (Go Implementation)

### Core Engine (Go)
- **Parser**: Hand-written recursive descent or `participle`
- **AST**: Go structs
- **JSON Model**: Standard `encoding/json`
- **Validation**: Custom rule engine
- **Mermaid Compiler**: Template-based code generation

### Future: UI Layer (See [UI & Future Features](../ui-future/README.md))
- **Diagram Editor**: React Flow or custom canvas
- **Layout**: ELK.js or Dagre
- **Monaco Editor**: For DSL editing with LSP

---

## Success Criteria

1. ✅ Parse `.sruja` DSL files
2. ✅ Generate JSON model from DSL
3. ✅ Compile JSON model to Mermaid C4
4. ✅ Validate model (basic rules)
5. ✅ Round-trip: DSL → JSON → DSL (canonical format)
6. ✅ Export to Mermaid file
7. ✅ CLI tool (`sruja compile`, `sruja validate`)

---

## Next Steps

1. Define DSL grammar (formal specification)
2. Implement parser
3. Implement AST → JSON transformer
4. Implement JSON → Mermaid compiler
5. Implement validation engine
6. Build CLI tool
7. Test with real-world examples


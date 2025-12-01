# Implementation Timeline & Dependencies

> **💡 Recommendation**: See [Simplified Plan](../SIMPLIFIED_PLAN.md) for a focused implementation prioritizing core value first.

## Dependency Graph

```
Phase 1: Core Data Transformations (Go)
├── Task 1.0: DSL Parser/Printer Changes
│   └── No dependencies (uses existing parser/printer)
│   └── Blocks: All other tasks (parser/printer foundation)
│
├── Task 1.1: JSON Exporter (AST → JSON)
│   └── Depends on: Task 1.0 (needs updated parser/printer)
│
├── Task 1.2: JSON to AST Converter (JSON → AST)
│   └── Depends on: Task 1.0, Task 1.1 (needs updated parser, JSON structure)
│
├── Task 1.3: CLI Commands
│   └── Depends on: Task 1.0, Task 1.1, Task 1.2
│
├── Task 1.4: Modularization Command (Optional)
│   └── Depends on: Task 1.0, Task 1.1, Task 1.2 (needs updated parser, JSON and JSON-to-AST)
│
└── Task 1.5: Change Commands
    └── Depends on: Task 1.0, Task 1.1, Task 1.2 (needs DSL support, JSON, diff calculation)

Phase 2: HTML Export (Go)
├── Task 2.1: HTML Exporter
│   └── Depends on: Task 1.1 (needs JSON structure)
│
└── Task 2.2: CLI Command for HTML
    └── Depends on: Task 2.1

Phase 3: JavaScript Library (TypeScript)
├── Task 3.1: Sruja Viewer Library (Core)
│   └── Depends on: Task 1.1 (needs JSON structure)
│
├── Task 3.2: Layout Configuration
│   └── Depends on: Task 3.1
│
├── Task 3.3: Styling
│   └── Depends on: Task 3.1
│
├── Task 3.4: View Management
│   └── Depends on: Task 3.1
│
├── Task 3.5: Interactivity
│   └── Depends on: Task 3.1
│
├── Task 3.6: Export Functionality (SVG/PNG)
│   └── Depends on: Task 3.1
│
├── Task 3.7: File Visualization
│   └── Depends on: Task 3.1
│
└── Task 3.8: Change Visualization
    └── Depends on: Task 3.1, Task 1.5 (needs migration model)

Phase 4: Web Studio (TypeScript)
├── Task 4.1: Studio Core
│   └── Depends on: Task 3.1 (needs viewer library)
│
├── Task 4.2: Drag-and-Drop Editor
│   └── Depends on: Task 4.1
│
├── Task 4.3: File Operations (CLI Studio - Go API)
│   └── Depends on: Task 4.2, Task 1.2
│
└── Task 4.5: Studio Export (SVG/PNG)
    └── Depends on: Task 4.1, Task 3.6

Phase 5: Change Tracking
├── Task 1.5: Change Commands
│   └── Depends on: Task 1.1, Task 1.2
│
└── Task 3.8: Change Visualization
    └── Depends on: Task 3.1, Task 1.5

Phase 6: Developer Experience
├── Task 1.7: Language Server Protocol (LSP)
│   └── Depends on: Task 1.1, Task 1.2, Task 1.5
│
└── Task 4.4: Studio Polish
    └── Depends on: Task 4.3, Task 4.5

Phase 7: Adoption (Optional)
├── Task 4.7: Public Studio
│   └── Depends on: Task 4.1 (can reuse Studio)
│
└── Task 5.1: VS Code Extension (Optional)
    └── Depends on: Task 1.7 (LSP), Task 4.1 (Studio)

Phase 8: Additional IDE Support (Deferred)
└── Task 5.2: JetBrains Plugin
    └── Depends on: Task 1.7 (LSP), Task 4.1 (Studio)
    └── **Status**: Build only if users request it
```

## Parallelization Opportunities

**Can be done in parallel:**

1. **Task 1.1 (JSON Exporter) + Task 3.1 (Viewer Library)**
   * JSON Exporter defines JSON structure
   * Viewer Library can start implementing with mock JSON
   * Sync when JSON structure is finalized

2. **Task 1.3 (CLI Commands) + Task 1.4 (Modularization) + Task 1.5 (Change Commands)**
   * All depend on Task 1.1 and 1.2
   * Can be done in parallel once JSON round-trip works

3. **Task 3.2 (Layouts) + Task 3.3 (Styling) + Task 3.4 (Views) + Task 3.5 (Interactions) + Task 3.7 (File Visualization)**
   * All depend on Task 3.1 (Core)
   * Can be done in parallel once core is ready

4. **Task 3.8 (Change Visualization)**
   * Depends on Task 3.1 and Task 1.5
   * Can start after migration commands are done

4. **Task 2.1 (HTML Exporter) + Task 3.1 (Viewer Library)**
   * HTML Exporter is simple (just template)
   * Can be done in parallel

**Must be sequential:**

1. **Task 1.1 → Task 1.2 → Task 1.3** (Core data transformations)
2. **Task 3.1 → Task 3.2/3.3/3.4/3.5** (Viewer library features)
3. **Task 4.1 → Task 4.2 → Task 4.3 → Task 4.4** (Studio features)

## Recommended Implementation Order

### Sprint 1: Foundation (Week 1-2)
**Goal**: Enable DSL ↔ JSON round-trip

1. **Task 1.1: JSON Exporter** (2-3 days)
   * Start immediately
   * Blocks everything else

2. **Task 1.2: JSON to AST Converter** (2-3 days)
   * Start after Task 1.1 JSON structure is defined
   * Can start with basic structure, iterate

3. **Task 1.3: CLI Commands** (1 day)
   * Start after Task 1.1 and 1.2 are done
   * Quick win, user-facing

**Deliverable**: `sruja export json` and `sruja json-to-dsl` commands work

> **Note**: Task 1.4 (Modularization) can be added later if needed. Focus on core round-trip first.

### Sprint 2: HTML Export (Week 2-3)
**Goal**: Generate interactive HTML from JSON

1. **Task 2.1: HTML Exporter** (1-2 days)
   * Start in parallel with Task 3.1 (simple template)

2. **Task 2.2: CLI Command** (0.5 days)
   * Quick after Task 2.1

**Deliverable**: `sruja export html` command works (but diagrams not interactive yet)

### Sprint 3: Interactive Viewer (Week 3-5)
**Goal**: Interactive diagram rendering

1. **Task 3.1: Sruja Viewer Library (Core)** (5-7 days)
   * Start as soon as Task 1.1 JSON structure is defined
   * Use mock JSON initially, sync later
   * **Critical path** - blocks everything else

2. **Task 3.2: Basic Layout** (2-3 days) - Parallel with 3.1
3. **Task 3.3: Basic Styling** (2-3 days) - Parallel with 3.1
4. **Task 3.5: Interactivity** (2-3 days) - Zoom, pan, click

**Deliverable**: Interactive diagrams in HTML export

> **Note**: Tasks 3.4 (View Management), 3.6 (Export), 3.7 (File Visualization) can be added incrementally as needed.


### Sprint 4: Visual Studio (Week 5-8)
**Goal**: Visual drag-and-drop editor

1. **Task 4.1: Studio Core** (3-5 days)
2. **Task 4.2: Drag-and-Drop Editor** (5-7 days)
3. **Task 4.3: File Operations** (2-3 days) - Read/write `.sruja` files via Go API
4. **Task 4.5: Studio Export** (1-2 days) - SVG/PNG export

**Deliverable**: Visual Studio for creating/editing architecture diagrams

### Sprint 5: Change Tracking (Week 9-10)
**Goal**: Track architecture evolution

1. **Task 1.5: Change Commands** (2-3 days) - Create, apply, validate
2. **Task 3.8: Change Visualization** (3-5 days) - Diff, timeline

**Deliverable**: Track and visualize architecture changes

### Sprint 6: Developer Experience (Week 10-11)
**Goal**: Catch mistakes early, smoother workflow

1. **Task 1.7: Language Server Protocol** (5-7 days) - Error diagnostics, basic completion
2. **Task 4.4: Studio Polish** (2-3 days) - Undo/redo, shortcuts

**Deliverable**: LSP shows errors in IDE, Studio has basic polish

### Sprint 7: Easy Tryout (Week 12-13) - Optional
**Goal**: Lower barrier to entry

1. **Task 4.7: Public Studio** (5-7 days) - Simplified (use Go API, no WASM initially)
   - **Defer if**: Adoption is fine without it

**Deliverable**: Let people try without installing CLI

### Sprint 8: IDE Integration (Week 13-14) - Optional
**Goal**: IDE support

1. **Task 5.1: VS Code Extension** (5-7 days) - Basic LSP integration, Studio webview
   - **Defer if**: CLI + Studio is sufficient

2. **Task 5.2: JetBrains Plugin** - **DEFERRED**: Build only if users request it

**Deliverable**: VS Code extension (if needed)

## Timeline Summary

### Core Value (Weeks 1-11) - Must Have

| Sprint       | Duration  | Tasks              | Deliverable                            |
| ------------ | --------- | ------------------ | -------------------------------------- |
| **Sprint 1** | Week 1-2  | 1.1, 1.2, 1.3      | DSL ↔ JSON round-trip                  |
| **Sprint 2** | Week 2-3  | 2.1, 2.2           | HTML export (shareable diagrams)        |
| **Sprint 3** | Week 3-5  | 3.1, 3.2, 3.3, 3.5 | Interactive diagrams                   |
| **Sprint 4** | Week 5-8  | 4.1, 4.2, 4.3, 4.5 | Visual Studio (drag-and-drop editor)    |
| **Sprint 5** | Week 9-10 | 1.5, 3.8           | Change tracking + visualization        |
| **Sprint 6** | Week 10-11 | 1.7, 4.4           | LSP + Studio polish                     |

**Core Value Total**: 11 weeks (2.75 months)

### Adoption (Weeks 12-14) - Optional

| Sprint       | Duration  | Tasks              | Deliverable                            |
| ------------ | --------- | ------------------ | -------------------------------------- |
| **Sprint 7** | Week 12-13 | 4.7 (optional)    | Public Studio (if adoption slow)        |
| **Sprint 8** | Week 13-14 | 5.1 (optional)     | VS Code Extension (if users request)    |

**Total Estimated Time**: 11-14 weeks (depending on optional features)

> **Note**: Sprint 7 and 8 are optional. Build only if needed for adoption or user requests.

## Critical Path

**Must be done in order:**

1. Task 1.1 (JSON Exporter) - **BLOCKS EVERYTHING**
2. Task 1.2 (JSON to AST) - Blocks reverse engineering
3. Task 3.1 (Viewer Core) - Blocks HTML export and studio
4. Task 4.1 (Studio Core) - Blocks studio features

**Can be parallelized:**

* Task 1.1 + Task 3.1 (with mock JSON initially)
* Task 2.1 + Task 3.1
* Task 3.2/3.3/3.4/3.5 (all after 3.1)
* Task 4.2/4.3/4.4 (all after 4.1)

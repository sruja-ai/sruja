# Implementation Plan: Complete Sruja Export & Studio

## Technology Separation

This implementation plan is organized by technology and responsibility:

### 🔵 Go (Backend/CLI)
- **DSL ↔ JSON** transformations
- All file-based operations
- CLI commands
- Located in: `docs/implementation/go/`

### 🟡 TypeScript/JavaScript (Frontend)
- **JSON ↔ Diagram** transformations  
- Interactive visualization
- Web Studio (client-side editor)
- Located in: `docs/implementation/typescript/`

## Architecture

```
┌─────────────┐
│  Sruja DSL  │
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐    ┌─────────────┐
│   Parser    │    │   Printer   │
│ (DSL→AST)  │    │ (AST→DSL)   │
└──────┬──────┘    └──────┬──────┘
       │                  │
       │                  │
       ▼                  │
┌─────────────┐           │
│     AST     │◄──────────┘
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐    ┌─────────────┐
│ JSON Exporter│    │JSON→AST Conv│
│ (AST→JSON)  │    │ (JSON→AST)  │
│   [GO]      │    │    [GO]     │
└──────┬──────┘    └──────┬──────┘
       │                  │
       │                  │
       ▼                  │
┌─────────────┐           │
│    JSON     │◄──────────┘
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐    ┌─────────────┐
│ HTML Exporter│   │  Web Studio │
│ (JSON→HTML) │   │ (JSON↔UI)   │
│   [GO]      │   │  [TS/JS]    │
└──────┬──────┘    └──────┬──────┘
       │                  │
       ▼                  ▼
┌─────────────┐    ┌─────────────┐
│  .sruja.html│    │  Diagram UI │
│ (Interactive)│   │  (Cytoscape)│
└─────────────┘    └──────┬──────┘
                          │
                          ▼
                   ┌─────────────┐
                   │ Export DSL  │
                   │ (via WASM)  │
                   └─────────────┘
```

## Quick Links

- [Overview & Architecture](overview.md) - High-level architecture and separation of concerns
- [JSON Design Summary](JSON_DESIGN_SUMMARY.md) - **Key**: Self-contained JSON with file metadata
- [Go Implementation](go/) - DSL ↔ JSON transformations
- [TypeScript Implementation](typescript/) - JSON ↔ Diagram and Web Studio
- [Timeline & Dependencies](timeline.md) - Implementation order and sprint planning
- [Success Criteria](success-criteria.md) - Acceptance criteria for each phase
- [Change Visualization Overview](change-visualization-overview.md) - Change tracking and visualization features
- [Studio Deployment Modes](typescript/studio-deployment.md) - Where Studio runs and how
- [Self-Hosted Studio](go/SELF_HOSTED_STUDIO.md) - Standalone Studio for sharing previews and designing architectures
- [Public Studio](go/PUBLIC_STUDIO.md) - Zero-friction web app for trying Sruja (no auth, no installation)
- [Sruja Cloud Service](cloud/README.md) - Commercial collaboration and PR automation
- [SDLC Process Review](SDLC_REVIEW.md) - Complete SDLC workflow assessment and recommendations
- [CI/CD Workflows](CI_CD_WORKFLOWS.md) - Automated validation and preview generation
- [Testing Strategy](TESTING_STRATEGY.md) - Comprehensive testing approach
- [Error Reporting Strategy](ERROR_REPORTING_STRATEGY.md) - Consistent error handling across all interfaces
- [Repository Organization](REPOSITORY_ORGANIZATION.md) - Repository structure analysis and recommendations
- [Playground Migration](PLAYGROUND_MIGRATION.md) - Migrating playground from learn app to Public Studio
- [Value Assessment](VALUE_ASSESSMENT.md) - **Critical**: What actually helps developers vs over-engineering
- [Simplified Plan](SIMPLIFIED_PLAN.md) - **Recommended**: Focused implementation plan prioritizing core value

## Key Principles

1. **Clear Separation**: Go handles DSL↔JSON, TypeScript handles JSON↔Diagram
2. **No Backend Required**: Web Studio runs entirely client-side
3. **JSON as Contract**: JSON format is the contract between Go and TypeScript
4. **Round-trip Guarantee**: DSL → JSON → DSL should preserve all information

## Phase Summary

> **💡 Recommendation**: See [Simplified Plan](SIMPLIFIED_PLAN.md) for a focused implementation prioritizing core value.

### Phase 1: Core Data Transformations (🔵 Go)

Goal: Enable DSL ↔ JSON round-trip

- Task 1.1: JSON Exporter (AST → JSON)
- Task 1.2: JSON to AST Converter (JSON → AST)
- Task 1.3: CLI Commands
- Task 1.4: Modularization Command
- Task 1.5: Change Commands (change tracking, snapshots, diffs)
- ~~Task 1.6: Proposal Commands~~ - **REMOVED**: Use ADRs + external systems (GitHub/Cloud Studio) for collaboration. See [Simplified Change Workflow](go/SIMPLIFIED_CHANGE_WORKFLOW.md)

See [go/](go/) for details.

### Phase 2: HTML Export (🔵 Go)

Goal: Generate HTML wrapper for diagrams

- Task 2.1: HTML Exporter (JSON → HTML template)
- Task 2.2: CLI Command for HTML Export

See [go/task-2.1-html-exporter.md](go/task-2.1-html-exporter.md) for details.

### Phase 3: JavaScript Library (🟡 TypeScript)

Goal: Interactive diagram rendering

- Task 3.1: Sruja Viewer Library (Core)
- Task 3.2: Layout Configuration
- Task 3.3: Styling (D2-like appearance)
- Task 3.4: View Management
- Task 3.5: Interactivity
- Task 3.6: Export Functionality (SVG/PNG)
- Task 3.7: File Visualization
- Task 3.8: Change Visualization (diff, snapshots, timeline)

See [typescript/](typescript/) for details.

### Phase 4: Web Studio (🟡 React + TypeScript)

Goal: Client-side diagram editor

- Task 4.1: Studio Core (React UI Framework)
- Task 4.2: Drag-and-Drop Editor (React components)
- Task 4.3: Studio File Operations (CLI Studio - Go API)
- Task 4.4: Studio Polish (undo/redo, shortcuts, UX improvements)
- Task 4.5: Studio Export (SVG/PNG)
- Task 4.6: Studio Changes (create changes visually, export to change files)
- Task 4.7: Public Studio (zero-friction web app with WASM)

### Phase 5: IDE Integration (🟡 TypeScript/Kotlin)

Goal: IDE support for Sruja

- Task 1.7: Language Server Protocol (LSP) - Go implementation
- Task 5.1: VS Code Extension - TypeScript
- Task 5.2: JetBrains Plugin - Kotlin/Java

Technology: React 19 + TypeScript + Cytoscape.js

## Timeline Summary

- Sprint 1 (Week 1-2): DSL ↔ JSON round-trip
- Sprint 2 (Week 2-3): HTML export
- Sprint 3 (Week 3-5): Interactive diagrams (basic)
- Sprint 4 (Week 5-7): Interactive diagrams (complete)
- Sprint 4.5 (Week 6-8): Change commands + change visualization
- Sprint 5 (Week 8-11): Web studio + Public Studio
- Sprint 6 (Week 11-14): LSP + IDE extensions

Total Estimated Time: 14 weeks (3.5 months)

See [timeline.md](timeline.md) for complete timeline and sprint breakdown.

## Next Steps

1. Start with Task 1.1 (JSON Exporter) - Critical path
2. Define JSON structure - Document in [JSON Schema](go/json-schema.md)
3. Create JSON schema - For validation
4. Set up test infrastructure - For comprehensive testing
5. Begin Task 3.1 (Viewer Library) - In parallel with mock JSON

# Implementation Overview

## Technology Responsibilities

### 🔵 Go Implementation
**Responsibility**: DSL ↔ JSON transformations

**Tasks**:
- DSL → AST (existing parser)
- AST → JSON (new exporter)
- JSON → AST (new converter)
- AST → DSL (existing printer)
- CLI commands for JSON export/import
- File modularization/splitting

**Location**: `pkg/export/json/`, `pkg/language/`, `cmd/sruja/`

### 🟡 TypeScript/JavaScript Implementation
**Responsibility**: JSON ↔ Diagram transformations

**Technology Stack**:
- React 19 + TypeScript (for Studio)
- Cytoscape.js (for graph rendering)
- Tailwind CSS (for styling)
- WebAssembly (for DSL export from Studio)

**Tasks**:
- JSON → Cytoscape elements (rendering)
- Diagram UI ↔ JSON (editing)
- View management (Level 1, 2, 3, scenarios, etc.)
- Interactive features (zoom, pan, drill-down)
- Web Studio UI (React TypeScript app)

**Location**: `learn/` (viewer + studio sources); embedded build output copied to `cmd/sruja/studio-dist/` during CLI build. See `go/STUDIO_API.md` for embedding details.

## Data Flow

```
User Input (DSL)
    ↓
[GO] Parser → AST
    ↓
[GO] AST → JSON (Exporter)
    ↓
[TS] JSON → Diagram (Viewer)
    ↓
User interacts with Diagram
    ↓
[TS] Diagram → JSON (Studio)
    ↓
[GO] JSON → AST (Converter) → DSL (Printer)
    ↓
User Output (DSL)
```

## Phase Breakdown

### Phase 1: Core Data Transformations (Go)
**Goal**: Enable DSL ↔ JSON round-trip

- Task 1.1: JSON Exporter (AST → JSON)
- Task 1.2: JSON to AST Converter (JSON → AST)
- Task 1.3: CLI Commands
- Task 1.4: Modularization Command

### Phase 2: HTML Export (Go)
**Goal**: Generate HTML wrapper for diagrams

- Task 2.1: HTML Exporter (JSON → HTML template)
- Task 2.2: CLI Command for HTML Export

### Phase 3: JavaScript Library (TypeScript)
**Goal**: Interactive diagram rendering

- Task 3.1: Sruja Viewer Library (Core)
- Task 3.2: Layout Configuration
- Task 3.3: Styling
- Task 3.4: View Management
- Task 3.5: Interactivity

### Phase 4: Web Studio (React + TypeScript)
**Goal**: Client-side visual editor

- Task 4.1: Studio Core (React UI Framework)
- Task 4.2: Drag-and-Drop Editor (React components)
- Task 4.3: DSL Export from Studio (via WASM)
- Task 4.4: Studio Polish

**Technology**: React 19 + TypeScript + Cytoscape.js

## JSON Format as Contract

The JSON format serves as the **contract** between Go and TypeScript:

- Go ensures JSON is valid and complete
- TypeScript consumes JSON and renders diagrams
- Both teams work independently once JSON schema is defined
- Round-trip preservation is guaranteed by Go

### Key Design: Self-Contained JSON

**JSON is standalone** - no imports or external dependencies:
- All elements from imported files are **flattened** into the JSON
- File source information is preserved via **metadata annotations**
- Each element has `metadata.sourceFile` indicating its origin
- File boundaries can be **reconstructed** when converting back to DSL

This design:
- ✅ Makes JSON portable and self-contained
- ✅ Preserves file organization via metadata
- ✅ Allows Studio to visualize file boundaries
- ✅ Simplifies TypeScript rendering (no import resolution needed)

See [JSON Schema](go/json-schema.md) for the complete format specification.

## Development Workflow

1. **Define JSON Schema** (Go team)
   - Document all JSON structures
   - Create validation tests

2. **Implement Go Exporters** (Go team)
   - AST → JSON
   - JSON → AST
   - CLI commands

3. **Implement TypeScript Viewer** (TS team, in parallel)
   - Use mock JSON initially
   - Sync with real JSON once schema is finalized

4. **Integration Testing**
   - Round-trip tests (DSL → JSON → DSL)
   - End-to-end tests (DSL → JSON → Diagram)

## Key Decisions

1. **No Backend**: Web Studio runs entirely client-side
2. **JSON as Contract**: Clear separation between Go and TypeScript
3. **CDN Hosting**: Viewer library hosted on GitHub Pages (sruja.ai)
4. **WASM for DSL Export**: Use compiled Go code in browser for JSON → DSL

# Diagram ↔ JSON ↔ DSL Round-Trip Architecture (Studio)

## Goals
1. Enable two-way editing between diagram and DSL.
2. Keep a single source of truth in-memory as `ArchitectureJSON`.
3. Provide fast, incremental updates with robust validation and error UX.
4. Keep converters isolated and testable.

## Core Data Model
- Canonical model: `ArchitectureJSON` (TypeScript) mirroring Go JSON export.
- Viewer renders `ArchitectureJSON` to Cytoscape elements.
- DSL and AST exist on the Go side; conversion to/from `ArchitectureJSON` via Go exporter.

## Converters
- DSL → AST (Go): `pkg/language/parser.go` and `pkg/language/ast.go`.
- AST → DSL (Go): `pkg/language/printer.go`.
- AST → JSON (Go): `pkg/export/json/json.go`, with types in `pkg/export/json/json_types.go`.
- JSON → Diagram (TS): `packages/viewer/src/viewer.ts` converts `ArchitectureJSON` to Cytoscape elements.

## Studio Single Source of Truth
- Studio keeps an in-memory `ArchitectureJSON` as the single source of truth.
- Both the diagram and the DSL text editor reflect and mutate this model.
- All edits produce normalized operations against the model (add/remove/update nodes/edges).

## Two-Way Editing Flows
### Diagram → JSON → DSL
1. User performs a diagram edit (add node, connect edge, rename).
2. Studio reduces the GUI action into a model operation and applies it to `ArchitectureJSON`.
3. Studio sends updated `ArchitectureJSON` to Go Studio API to generate DSL (server-side AST → DSL).
4. DSL preview/editor updates; file save persists DSL.

### DSL → JSON → Diagram
1. User edits DSL text in the editor.
2. Studio sends the DSL to Go Studio API to parse (DSL → AST → JSON).
3. Studio replaces in-memory `ArchitectureJSON` and triggers viewer `load()` with minimal diff.
4. Diagram updates incrementally.

## API Contract (Studio Backend)
- Refer to `docs/implementation/go/STUDIO_API.md` for endpoints:
  - `GET /api/files/:path` → reads DSL, returns `ArchitectureJSON`.
  - `POST /api/files/:path` → accepts `ArchitectureJSON`, returns DSL and persists.
  - Additional list/create/delete endpoints for project files.
- All client interactions are via HTTP; no client-side DSL parsing initially for stability.

## Incremental Updates
- Apply patch-style operations to `ArchitectureJSON` so the viewer can update nodes/edges selectively.
- Use IDs and stable hierarchical paths (`System.Container`, etc.) to compute diffs.
- Debounce DSL parsing on text input; only parse on pause or explicit save.

## Error Handling & UX
- DSL parse errors: keep last good `ArchitectureJSON`, show error overlay in the editor with diagnostics.
- Diagram edit validation: prevent illegal edges or unknown IDs, show inline warnings.
- Save failures: keep optimistic UI but display retry banner and revert if necessary.

## Testing Strategy
- Unit-test Go parser/printer and JSON exporter (existing tests cover core grammar).
- Unit-test TS viewer conversion from `ArchitectureJSON` to Cytoscape elements.
- Integration tests: Studio calls API, round-trip DSL↔JSON consistency, and viewer updates.

## Implementation Steps (TypeScript)
1. Use `@sruja/viewer` to render `ArchitectureJSON`:
   - `packages/viewer/src/viewer.ts` exposes `createViewer()` and `SrujaViewer`.
2. In Studio, hold `ArchitectureJSON` in React state and pass to viewer `load()`.
3. Wire diagram events to reducer functions producing model operations.
4. Implement API client to call Go Studio API for DSL generation and parsing.
5. Add a text editor pane for DSL with debounced server-side parse.
6. Handle errors from API and show diagnostics.
7. Write tests for conversion and round-trip consistency.

## References
- Viewer code: `packages/viewer/src/viewer.ts`, `packages/viewer/src/types.ts`.
- Studio example usage: `apps/studio/src/App.tsx`.
- Go language implementation: `pkg/language/*`, JSON exporter: `pkg/export/json/*`.
- TypeScript docs: `docs/implementation/typescript/task-4.1-studio-core.md`, `task-4.2-editor.md`, `task-4.3-dsl-export.md`.


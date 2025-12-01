# TypeScript Implementation: JSON ↔ Diagram

## Responsibility

TypeScript/JavaScript handles all JSON ↔ Diagram transformations:

- ✅ JSON → Cytoscape elements (rendering)
- ✅ Diagram UI ↔ JSON (editing in Studio)
- ✅ View management (Level 1, 2, 3, scenarios, etc.)
- ✅ Interactive features (zoom, pan, drill-down)
- ✅ Web Studio UI (React + TypeScript)

## Technology Stack

- **TypeScript** - Type safety
- **React 19** - UI framework (for Studio)
- **Cytoscape.js** - Graph visualization
- **Tailwind CSS** - Styling
- **Radix UI** - Component primitives

## Tasks

See individual task files for detailed implementation:

1. [Viewer Library Core](task-3.1-viewer-core.md) - JSON → Diagram rendering
2. [Layout Configuration](task-3.2-layout.md) - Layout algorithms
3. [Styling](task-3.3-styling.md) - D2-like appearance
4. [View Management](task-3.4-views.md) - Multiple view types
5. [Interactivity](task-3.5-interactivity.md) - Click, hover, drill-down
6. [Export Functionality](task-3.6-export.md) - SVG/PNG export for standalone HTML
7. [File Visualization](task-3.7-file-visualization.md) - Visualize file boundaries using metadata
8. [Change Visualization](task-3.8-change-visualization.md) - Visual diff, snapshots, timeline views
9. [Studio Core](task-4.1-studio-core.md) - Visual editor UI (React + TypeScript)
10. [Drag-and-Drop Editor](task-4.2-editor.md) - Element manipulation (React)
11. [DSL Export](task-4.3-dsl-export.md) - Export to DSL via WASM
12. [Studio Export](task-4.5-export.md) - SVG/PNG export for Studio
13. [Studio Polish](task-4.4-polish.md) - Undo/redo, shortcuts, etc.
14. [Visual Proposals](task-4.6-proposals.md) - Create/review/modify proposals visually

## Key Principles

1. **Pure Client-Side**: No backend required
2. **JSON as Input**: Consumes self-contained JSON format (no imports to resolve)
3. **File Metadata**: Uses metadata.sourceFile for visualization and organization
4. **CDN Hosting**: Viewer library hosted on GitHub Pages (sruja.ai)
5. **WASM for DSL Export**: Use compiled Go code for JSON → DSL
6. **React for Studio**: Studio is a React TypeScript application

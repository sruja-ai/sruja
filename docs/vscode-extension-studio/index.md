# Sruja Studio — Incremental Build Topics and Roadmap

# 📌 Scope
Sruja Studio targets VSCode: Webview + LSP + Kernel with two-way DSL↔diagram editing, IR-centric binding, and ELK-based layouting. All topics below assume the VSCode extension environment.

Use this index to navigate and build Sruja Studio incrementally.

Topics:
- Architecture Overview — `overview.md`
- Two-Way Binding — `two-way-binding.md`
- Diagram Editor Architecture — `diagram-editor.md`
- IR → Graph Mapping — `ir-graph-mapping.md`
- Graph → IR Patch Spec — `graph-ir-patch.md`
- DSL Patch Generator — `dsl-patch-generator.md`
- Layout Engine — `layout.md`
- Styling System — `styling.md`
- Node Inspector — `inspector.md`
- Search & Command Palette — `search-palette.md`
- Toolbar — `toolbar.md`
- Multi-File & Cross-File — `multi-file.md`
- AI Integration — `ai-integration.md`

Suggested build order:
- Phase 1: VSCode Extension basics, LSP parse/validate, read-only Webview diagrams → `overview.md`
- Phase 2: Two-way binding MVP (Diagram → IR → DSL), notebook cells → `two-way-binding.md`
- Phase 3: AI integration (import, refine, queries) → `ai-integration.md`
- Phase 4: Web Studio (optional stakeholder view) → `overview.md`
- Phase 5: Diagram editor features (Inspector, Palette, Toolbar, Search) → `diagram-editor.md`, `inspector.md`, `search-palette.md`, `toolbar.md`
- Phase 6: Layout engine (ELK), styling system → `layout.md`, `styling.md`
- Phase 7: Performance, undo/redo, multi-file handling → `diagram-editor.md`, `multi-file.md`

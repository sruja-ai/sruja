# Diagram Editor Architecture

# 📌 Scope
Designed for the VSCode Webview Studio: IR-centric two-way DSL↔diagram editing, ELK-based layouting, and LSP/Kernel integration for live sync.

# ⭐ High-Level Philosophy
A hierarchical editor engine composed of:
- Reactive data layer (Graph Store)
- Graph rendering (Cytoscape.js + ELK)
- UI panels (inspectors, sidebars, diff overlays)
- Command & patch router (two-way binding)
- Kernel integration
- AI operation panel

# 🧠 1. High-Level Component Map
VSCode Webview ↔ Sruja Diagram Editor ↔ Component Layers
- Graph Store / IR Cache
- Diagram Renderer
- Panels / Inspectors
- Command & Patch Router
- AI Actions / Suggestions
- Notification & History
- View Modes

# 🧱 2. Core Data Layer: Graph Store
Zustand store mirroring Kernel IR:
- nodes, edges, selected, mode, layout, pendingPatches

# 🖼️ 3. Diagram Renderer Layer
Cytoscape.js responsibilities:
- render nodes/edges, zoom/pan, callbacks, animation, overlays
- supports >10k nodes, incremental updates, subgraphs, ELK integration

# 🗂️ 4. Panels & Inspectors Layer
Key panels:
- Inspector Panel: details, metadata, relations, DSL location, actions
- Hierarchy Navigator: System → Containers → Components
- AI Assist Panel: refactor suggestions, explanations, DSL changes
- Diff Viewer Panel: added/removed/changed nodes, API/events diffs
- Search / Command Palette

# 🔄 5. Command & Patch Router
Diagram actions → Patch Router → Kernel → LSP → DSL → Kernel → Diagram sync
All patch-based, deterministic, reversible.

# 🤖 6. AI Action Engine
- Propose refactoring, detect duplicates, clean naming, infer components
- Architecture: Panel → AICommand → Kernel IR or DSL patches → Editor applies

# 🕹️ 7. Interaction Model
- On select: inspector opens, related highlight, show DSL location
- On drag: ghost preview; on drop emits patch
- Right-click: context menu (rename, move, delete, convert, AI)
- Double-click: drill down system→containers→components

# 🌐 8. View Modes
- C4 Level 1: systems
- C4 Level 2: containers
- C4 Level 3: components
- Event Flow View: producer → event → consumer
- Entity View: domain entities
- Contract View: endpoints + dependencies

# 📐 9. Layout Engine (ELK.js)
- Worker-based; stable positioning; hierarchical; incremental
- Pipeline: GraphStore → LayoutWorker → positions → Cytoscape

# 🧱 10. Event Bus
EventBus for NODE_SELECTED, PATCH, IR_UPDATED, DIFF_UPDATED.

# 🟪 11. Undo/Redo & History
- Patch history stacks; reversible, deterministic; triggers IR regen

# 🔀 12. Multi-File & Cross-File Handling
- References across `.sruja` files; IR compaction creates one IR → one diagram
- Two-way binding modifies correct file via source mapping

# 🟥 13. Cross-Cutting Features
- Snapping & Grid
- Metadata-driven Node Styles
- Diagram Bookmarking
- Mini-Map

# 🧩 14. Component Tree (React)
- components: DiagramCanvas, NodeInspector, RelationInspector, AIActionPanel, Toolbar, SearchPanel, HierarchyPanel, DiffOverlay
- graph: graphStore, patchRouter, elkLayoutWorker, cytoscapeAdapter
- views: C4SystemView, C4ContainerView, EventFlowView, EntityView, ContractView
- utils: layoutUtils, irToGraph, graphToIR, styles

# 🎯 Final Summary
- IR-centric two-way binding; visual edits → DSL patching; AI-assisted modeling
- Multiple views; large graphs; live LSP updates; undo/redo, search, positioning; kernel-validated

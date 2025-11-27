# ELK Layout Config Spec

# 📌 Scope
Layouting for the VSCode Extension Studio using ELK.js in the Webview; supports the two-way editing workflow with stable positions and incremental updates.

# ⭐ Goals
- Clean, readable architecture diagrams
- Hierarchical structures (C4), event flows, domain layouts
- Large graphs; layout stability under small changes; incremental movement
- Partial manual refinements; avoid jitter; grouping support; custom view modes

# 🧠 ELK Layout Strategy
- Default: `elk.algorithm = "layered"`
- Perfect for C4 (top→down) and event flows (left→right); orthogonal routing; compound nodes; stable

# 🧱 Global ELK Config
Key params: direction, spacing, padding, crossing minimization, node placement, hierarchy handling, edge routing.

# 🟦 View Mode–Specific Config
- C4 System View: `elk.direction=RIGHT`, wider spacing, systems as compound nodes
- C4 Container View: `elk.direction=DOWN`, neat vertical stacks
- C4 Component View: tighter spacing
- Event Flow View: LEFT→RIGHT pipeline; NODE/EDGE order; ports behavior
- DDD Domain View: lanes for bounded contexts; INCLUDE_CHILDREN; domain-specific strategies
- Contract View: endpoints → DTOs → events; path strategies

# 🟥 Grouping (Compound Nodes)
- INCLUDE_CHILDREN, mergeEdges; group padding rules

# 🟦 Edge Routing Rules
- Default: ORTHOGONAL; Event flows: SPLINES; Contract: POLYLINE

# 🟧 Node Spacing Rules
- Defaults and overrides for crowded vs large views

# 🟨 Incremental Layout Rules
- Favor straight edges; lock manually moved nodes; relayout only changed subgraphs; preserve ordering

# 🟥 Performance Optimizations
- Run ELK in web worker; delta updates; debounce; bounding box caching; freeze layout during mass changes

# 🟩 Layout Mode Switching
`mapIRToGraph(IR, viewMode)` → `elkWorker.run(graphModel, config)`; smooth transitions via incremental layout

# ⭐ Final Summary
- ELK Layered default; custom configs per view; readable and stable diagrams; compound groups; incremental changes; workers; predictable routing; minimized jitter

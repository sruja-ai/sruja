# Diagram Toolbar Spec

# 📌 Scope
Toolbar for the VSCode Extension Studio Webview: modeling commands emit IR patches, update DSL and diagrams via two-way binding, with ELK layouting controls.

# ⭐ Purpose
Primary modeling command surface: quick modeling actions, global operations, view switching, zoom/pan, AI, filtering, multi-select, undo/redo, layout.

# 🧱 Toolbar Layout Structure
Top of canvas:
`A. Navigation | B. Modeling Tools | C. View Controls | D. AI Tools | E. Layout Tools`

# 🟩 A. Navigation Tools
Search/Command Palette; Undo; Redo; Center/Reset Camera; Zoom In/Out; Zoom to Fit.

# 🟦 B. Modeling Tools
Add System, Add Container, Add Component, Add External System, Add Event/Topic, Add Entity (DDD), Connect Tool.

# 🟨 C. View Controls
Switch modes: System, Container, Component, Event Flow, DDD Domain, Contract/API, Infra (future). Each mode adjusts layout, styling, visibility, inspector, AI context.

# 🟥 D. AI Tools
AI Suggest Improve Architecture; AI Explain; AI Fix Violations; AI Generate Components/Events. Returns patch preview, diff UI, Apply/Edit/Cancel.

# 🟧 E. Layout Tools
Run Layout (incremental/full), Lock/Unlock Layout, Toggle Inferred Nodes, Toggle Policy Violations, Toggle Labels.

# 🟪 Toolbar Behavior Rules
- Context awareness per mode
- Multi-select awareness with batch operations
- Two-way binding integration through PatchRouter → Kernel → DSL → IR → Graph
- Undo/Redo integration
- Performance guard rails (throttle, worker, avoid relayout spam)

# 🟫 Extensibility API
`registerToolbarItem({ id, icon, title, viewModes, action })` for custom nodes/refactors/AI/DDD artifacts.

# ⭐ Final Summary
Modeling commands, navigation, AI tools, view modes, layout tools; generates IR patches; integrates kernel→DSL pipeline; supports multi-select, undo/redo, extensions.

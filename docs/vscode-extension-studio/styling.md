# Node/Edge Styling System

# 📌 Scope
Styling for the VSCode Extension Studio diagrams rendered in the Webview; works with two-way DSL↔diagram editing and ELK layouting.

# ⭐ Goals
- Convey meaning visually; metadata-driven appearance; theme-aware; interaction states
- Multi-view consistency; grouping support; scalable rendering

# 🧱 Styling Foundation
Cytoscape stylesheet layers:
1. Base
2. Type-specific (system, container, component, entity, event)
3. Metadata-specific (inferred, policy-violation, deprecated)
4. Interaction states (selected, hover)
5. View-mode overrides

# 🌗 Theme System
- Light, Dark, High-contrast
- Theme colors, shadows, font; config in `/styles/themes.ts`

# 🟦 Node Types & Rules
- System: round-rectangle, thicker border, group behavior
- Container: rounded rect; metadata influences style
- Component: compact round-rectangle
- Entity (DDD): hexagon
- Event: ellipse, bold, event color; edges inherit color
- External System: diamond/octagon, dashed border
- Datastore: cylinder
- Queue: rectangle with dash pattern

# 🟥 Edge Types & Styling
- Default: orthogonal arrows
- uses/depends/calls: triangle arrows
- Event Flow: vee arrows, bezier, event color
- Read/Write: dashed/bold
- Violations: red, thicker

# 🟨 Metadata-Driven Styling
- inferred=true: opacity 0.6, dashed border
- policyViolations>0: red border, badge
- isCritical=true: critical color, bold
- deprecated=true: dotted border, lower opacity, strikethrough
- tags[]: badge overlays

# 🟩 Interaction States
- Selected: highlight border and connected edges
- Hover: accent border, pointer cursor
- Dragging: opacity 0.8, dashed
- Ghost previews: muted colors

# 🟧 Group/Compound Node Styling
- System group frames; optional container/domain groups

# 🟥 View Mode Overrides
- C4 System: bold system frames; wide arrows
- C4 Container: colored containers; subtle components
- C4 Component: small blocks; no externals
- Event Flow: glowing events; curved colored edges
- DDD Domain: domain shapes
- Contract: endpoints, DTOs distinct

# 🟪 Animation & Transitions
- Batch changes; short animations; animate position/opacity/size only

# 🟫 Diagram Overlays
- Tag badges; violation indicators; inferred icon; search highlights; diff overlay (new/removed/updated)

# ⭐ Final Summary
- Type and metadata-driven styles; view-specific overrides; interaction states; overlays; compound nodes

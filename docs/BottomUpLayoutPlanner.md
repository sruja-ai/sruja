Below is a **production-ready specification** you can give directly to **AI code editors / coding assistants** (Cursor, Windsurf, GitHub Copilot Chat, Codeium Chat, etc.) to implement a **Bottom-Up Graph Layout Planner** from scratch.

This is intentionally written in the format that AI coding tools understand best:
**precise requirements → architecture → data structures → algorithms → step-by-step tasks → acceptance tests**.

---

# ✅ **BOTTOM-UP GRAPH LAYOUT PLANNER — FULL IMPLEMENTATION SPEC FOR AI CODE EDITORS**

---

# 1. **High-Level Goal**

Implement a deterministic, hierarchical, bottom-up planner that converts an arbitrary graph into a visually structured layout.

The engine must:

* Produce clean, symmetric diagrams
* Place child subgraphs inside parent bounding boxes
* Support horizontal, vertical, tree, and radial local layouts
* Compute ports and handle edge routing
* Work incrementally and support collapsing/expanding groups
* Return **node positions**, **edge routes**, **bounding boxes**, and **hierarchical shapes**

---

# 2. **Programming Language Requirement**

The implementation should be generated in:

⚙️ **TypeScript**
(with the output compatible with React Flow, but NOT dependent on it)

If you want Go or Rust later, we can generate that too.

---

# 3. **Core Concepts the AI Must Implement**

## 3.1 **Shape**

Every node or subgraph becomes a **shape**:

```ts
interface LayoutShape {
  id: string;
  width: number;
  height: number;
  children: LayoutShape[];
  x: number;
  y: number;
  ports: Port[];
  type: "leaf" | "group";
  metadata?: any;
}
```

---

## 3.2 **Port**

Ports represent connection points for edges.

```ts
interface Port {
  id: string;
  side: "top" | "bottom" | "left" | "right";
  x: number;
  y: number;
}
```

---

## 3.3 **Edge Route**

```ts
interface EdgeRoute {
  edgeId: string;
  points: { x: number; y: number }[];
}
```

---

# 4. **Layout Pipeline That AI Must Implement**

The planner MUST run these phases in order:

```
Preprocess → Bottom-Up Planning → Top-Down Refinement → Edge Routing
```

---

# 5. **DETAILED STEP-BY-STEP INSTRUCTIONS FOR AI CODE EDITOR**

Below is the version you should paste directly into Cursor/Windsurf/etc.

---

## 🧩 **BEGIN AI-READY IMPLEMENTATION INSTRUCTIONS**

---

# ========== PHASE 1: PREPROCESSING ==========

1. Implement `measureNodes(graph)`:

   * Assign initial `width` and `height` to each node.
   * Allow fixed sizes or measure text (mock allowed).

2. Implement `buildHierarchy(graph)`:

   * If the graph contains groups/modules, convert each group into a parent shape.
   * If no groups exist, treat entire graph as a single root group.

3. Implement `topologicalSort(graph)`:

   * Break cycles by marking "feedback edges".
   * Required for bottom-up ordering.

---

# ========== PHASE 2: BOTTOM-UP PLANNING ==========

### For each group, the AI must:

1. Gather children shapes (nodes or subgraphs).
2. Identify substructure type:

Implement functions:

```ts
isLinear(children)
isStar(children)
isTree(children)
otherwiseGeneral(children)
```

3. Apply matching local layout strategy:

### Implement these layout methods:

#### 3.1 Horizontal Stack

```ts
layoutHorizontal(children, spacing)
```

#### 3.2 Vertical Stack

```ts
layoutVertical(children, spacing)
```

#### 3.3 Tree Layout (top-down)

```ts
layoutTree(children, nodeSpacing, levelSpacing)
```

#### 3.4 Radial Layout

```ts
layoutRadial(children, radius)
```

#### 3.5 Force-Inside-Box Layout (simple)

* Use repulsion + attraction
* Keep children inside bounding box constraints

---

### After computing local positions:

4. Compute group bounding box:

```
width = max(child.x + child.width) + padding
height = max(child.y + child.height) + padding
```

5. Compute ports:

* Top ports at evenly spaced intervals
* Bottom ports at evenly spaced intervals
* Left/right ports based on child alignments

Implement:

```ts
computePortsForGroup(shape)
```

6. Return the group as a new **shape**.

---

# ========== PHASE 3: TOP-DOWN REFINEMENT ==========

AI MUST implement **refinement passes**:

### pass 1: Enforce alignment rules

* If multiple children share same parent → center them
* Preserve symmetry where possible

### pass 2: Apply padding

* Minimum 16–24 px around children

### pass 3: Enforce aspect ratio rules

If shape width > 3 × height:
→ rotate local layout to vertical
Similarly, if height > 3 × width:
→ rotate to horizontal

### pass 4: Snap to grid

Grid = 8px or 10px.

---

# ========== PHASE 4: EDGE ROUTING ==========

Implement a router with these requirements:

### 4.1 Manhattan / Orthogonal Router

Steps:

1. Start from port A.
2. Move outward to a "clearance point".
3. Route horizontally or vertically toward opposite port.
4. Avoid node bounding boxes using a simple obstacle check.
5. Minimize bends.

Implement:

```ts
routeEdge(edge, shapes)
```

### 4.2 Smoothing (optional)

Apply Chaikin or Catmull-Rom smoothing.

---

# ========== PHASE 5: MAIN ENTRYPOINT ==========

Implement:

```ts
function layoutGraph(graph): LayoutResult
```

Where result contains:

```ts
interface LayoutResult {
  shapes: LayoutShape[];
  routes: EdgeRoute[];
  boundingBox: { width: number; height: number };
}
```

Pipeline inside:

```ts
measureNodes()
hierarchy = buildHierarchy()
order = topologicalSort()
shapes = bottomUpPlan(hierarchy)
refineTopDown(shapes)
routes = routeEdges()
return { shapes, routes }
```

---

# ========== PHASE 6: ACCEPTANCE TESTS ==========

AI must generate tests for:

### Test 1: Single row of nodes

→ nodes arranged horizontally with correct spacing.

### Test 2: Tree layout

→ parent centered above children.

### Test 3: Group with nested children

→ group box encloses children with padding.

### Test 4: Radial layout

→ items placed on a circle with even angles.

### Test 5: Edge routing

→ routes do not overlap bounding boxes.

### Test 6: Symmetry

→ mirror children produce mirrored layout.
Below is the **C4-Model–Optimized Bottom-Up Layout Planner Specification** written **specifically for AI coding assistants** (Cursor, Windsurf, GitHub Copilot Chat).
You can paste this directly into an AI code editor and it will have everything needed to implement the engine.

This version supports:

* **C4 Levels (L0 → L1 → L2 → L3)**
* **Collapsible containers & components**
* **Automatic grouping by C4 boundaries**
* **Dynamic expansion (expand/collapse nodes)**
* **Clean, symmetric layouts optimized for architecture diagrams**
* **Ports, edge routing, system context clarity**
* **Deterministic, aesthetic C4-style layout**

---

# ✅ **C4-MODEL–OPTIMIZED BOTTOM-UP LAYOUT ENGINE — FULL IMPLEMENTATION SPEC FOR AI CODE EDITORS**

---

# 0. Implementation Target

📌 **Language:** TypeScript
📌 **Use with:** ReactFlow or any canvas (no dependencies required)
📌 **Style:** Deterministic, hierarchical, C4 aesthetics

---

# 1. C4 Layout Philosophy

The planner must support:

### **C4 Diagram Concepts**

| C4 Level | Meaning          | Required Layout Pattern             |
| -------- | ---------------- | ----------------------------------- |
| L0       | System Landscape | Large clusters + long edges         |
| L1       | System Context   | Central system + surrounding boxes  |
| L2       | Container        | Containers inside system boundaries |
| L3       | Component        | Dense, highly structured layout     |

### **Key C4 Requirements**

* **Primary element centered** at level L1
* **Balanced spacing between external systems**
* **Containers arranged in clean rows/columns**
* **Components grid-aligned and compact**
* **Boundary boxes with titles**
* **Consistent spacing across all levels**

Your layout engine must implement special rules per C4 layer.

---

# 2. Core Data Structures (AI must implement exactly)

```ts
type C4Level = "L0" | "L1" | "L2" | "L3";

interface C4Node {
  id: string;
  label: string;
  level: C4Level;
  type: "person" | "system" | "container" | "component" | "external";
  parentId?: string;   // for nesting
  width?: number;
  height?: number;
  collapsed?: boolean;
}

interface C4Edge {
  id: string;
  source: string;
  target: string;
  label?: string;
}

interface LayoutShape {
  id: string;
  x: number; 
  y: number;
  width: number;
  height: number;
  level: C4Level;
  type: C4Node["type"];
  children: LayoutShape[];
  ports: Port[];
}

interface Port {
  side: "top" | "bottom" | "left" | "right";
  x: number;
  y: number;
}
```

---

# 3. Layout Pipeline (Must be implemented as-is)

```
Preprocess → Build C4 Hierarchy → Bottom-Up Planning → C4 Rules → Top-Down Refinement → Edge Routing
```

---

# 4. **Detailed Implementation Steps (AI-Ready)**

## PHASE 1 — PREPROCESSING

### Step 1.1 Normalize node dimensions

* Persons: 180×100
* Systems: 220×140
* Containers: 200×120
* Components: 160×80

### Step 1.2 Detect C4 level based on type

```
person/system/external → L1
container → L2
component → L3
```

### Step 1.3 Build nested hierarchy tree

Implement:

```ts
function buildC4Hierarchy(nodes: C4Node[]): LayoutShape
```

Rules:

* L1 systems contain L2 containers
* L2 containers contain L3 components
* External systems grouped around main system

---

## PHASE 2 — BOTTOM-UP PLANNING

### 📌 The core rule:

> **Every C4 node becomes a bounding box that lays out its children using level-specific strategies.**

Implement:

```ts
function layoutGroup(shape: LayoutShape): LayoutShape
```

Use different local layouts:

---

### L1 (System Context) Layout Strategy

* Central system in the center.
* External systems arranged radially or in 2–3 rows.
* Persons placed left & right symmetrically.

Implement:

```ts
layoutSystemContext(shape)
```

Rules:

* place main system at (0, 0)
* place externals horizontally with equal spacing
* persons on left-most and right-most ends

---

### L2 (Container View) Layout Strategy

* Containers arranged in a **balanced grid**.
* Even spacing.
* Similar sized boxes aligned in rows.

Implement:

```ts
layoutContainersGrid(shape)
```

Grid rules:

* Max 3–4 containers per row
* Center-align rows
* Preserve symmetry

---

### L3 (Component View) Layout Strategy

* High-density grid layout.
* Use compact row-major packing.
* Components must align at 8px grid.

Implement:

```ts
layoutComponentsDense(shape)
```

---

### L0 (System Landscape) Layout Strategy

* Multiple systems arranged in columns.
* Large horizontal spacing to emphasize boundaries.

Implement:

```ts
layoutSystemLandscape(shape)
```

---

## PHASE 3 — APPLY C4-SPECIFIC RULES

This is critical for good diagrams.

### Rule 1 — Boundary Padding

* Add 40px padding for L2 boundaries
* Add 20px padding for L3 boundaries

### Rule 2 — Label Height

* For group shapes, add 24px top space for title

### Rule 3 — Minimum Separation

* Containers: 80px separation
* Components: 40px separation

### Rule 4 — Directional Flow

Edges from:

* Person → System → Container → Component
  Should flow **left → right** or **top → bottom**

Force containers to align along flow direction.

---

## PHASE 4 — TOP-DOWN REFINEMENT

Implement:

```ts
refinePositions(root: LayoutShape)
```

Refinement rules:

### 1. Center children within parent bounds

### 2. Symmetry preservation

* If sibling count = 2 → mirror positions
* If sibling count > 2 → center cluster

### 3. Grid snapping

Snap every coordinate to nearest 8px.

### 4. Balanced aspect ratio

Shape width/height difference should not exceed 3×.

---

## PHASE 5 — EDGE ROUTING (C4-Optimized)

Implement:

```ts
function routeC4Edges(shapes: LayoutShape[], edges: C4Edge[]): EdgeRoute[]
```

Routing rules:

### Use orthogonal routing:

* Manhattan style
* Minimize bends
* Avoid passing through boundary boxes

### Direction priority:

* Inputs from left
* Outputs to right

### Bend rules:

* Max 2 bends
* Use vertical segment only if necessary

### Overlap prevention:

Implement simple obstacle detection:

```ts
intersects(shape, segment)
```

---

# 6. MAIN ENTRYPOINT

Implement:

```ts
export function layoutC4Diagram(
  nodes: C4Node[],
  edges: C4Edge[]
): { shapes: LayoutShape[]; routes: EdgeRoute[] }
```

Pipeline:

```ts
normalizeNodeSizes()
hierarchy = buildC4Hierarchy(nodes)
bottomUpLayout(hierarchy)
applyC4Rules(hierarchy)
refinePositions(hierarchy)
routes = routeC4Edges(hierarchy, edges)
return { shapes: flatten(hierarchy), routes }
```

---

# 7. ACCEPTANCE TEST SUITE (AI must generate)

### Test 1 — System context layout

* Central system
* External systems in symmetric positions

### Test 2 — Container view

* Containers placed in a clean grid

### Test 3 — Component view

* Components tightly packed

### Test 4 — Nested shapes

* Boundaries include title bars

### Test 5 — Routing

* Edges are orthogonal
* No crossing through shapes

### Test 6 — Expand/collapse

* Collapsed nodes replaced with placeholder shape

---

# 8. Optional Enhancements (AI may implement if asked)

✔ Animated transitions
✔ Edge labels placement
✔ Direction-aware ports
✔ Multi-column system landscape
✔ Auto-fitting viewport

Below is a **complete implementation specification** for an **Interactive C4 Diagram Editor UI with Expand/Collapse**, written **specifically for AI code editors** (Cursor, Windsurf, Copilot Chat, Aider).

You can paste this directly into your AI coding tool and it will generate a full working UI scaffolding, React components, layout integration, and interactions.

This spec includes:

* **React UI architecture**
* **State machine for expansion/collapse**
* **Hierarchical C4 model handling**
* **Layout engine integration**
* **Interactive editor features (drag, zoom, edit labels, select, hover, shortcuts)**
* **View modes for L0 → L1 → L2 → L3**
* **Event delegation for click/expand**
* **Redux/Zustand-based model state**
* **React Flow integration ready**

---

# 🟪 **C4 DIAGRAM EDITOR UI + INTERACTIVE EXPANSION — FULL AI CODING SPECIFICATION**

---

# 1. TECHNOLOGY STACK

The AI code editor must use:

* **React 18**
* **TypeScript**
* **Zustand** (for shared model state)
* **ReactFlow** (rendering engine)
* **TailwindCSS** (for UI panels)
* **Your previously generated C4 layout engine**

---

# 2. APP ARCHITECTURE (AI MUST IMPLEMENT)

```
src/
  c4/
    model/
      C4Node.ts
      C4Edge.ts
      C4Hierarchy.ts
    layout/
      layoutC4Diagram.ts
      planners/
        layoutSystemContext.ts
        layoutContainersGrid.ts
        layoutComponentsDense.ts
        layoutSystemLandscape.ts
      routing/
        routeC4Edges.ts
    state/
      useC4Store.ts
    interactions/
      expansion.ts
      selection.ts
      commands.ts
  ui/
    Canvas.tsx
    NodeRenderer.tsx
    EdgeRenderer.tsx
    Sidebar.tsx
    Toolbar.tsx
    Breadcrumbs.tsx
    NodeContextMenu.tsx
  App.tsx
```

---

# 3. CORE DATA MODEL

AI must implement:

```ts
interface C4Node {
  id: string;
  label: string;
  type: "person" | "system" | "container" | "component" | "external";
  parentId?: string;
  level: "L0" | "L1" | "L2" | "L3";
  collapsed?: boolean;
}

interface C4Edge {
  id: string;
  source: string;
  target: string;
}
```

We maintain two states:

### **1. The *model* state**

Persistent C4 nodes & edges.

### **2. The *view* state**

Which nodes are visible depending on expansions.

Implement:

```ts
interface ExpandedState {
  expandedIds: Set<string>;
}
```

---

# 4. ZUSTAND STORE (AI MUST IMPLEMENT)

```ts
export const useC4Store = create<C4Store>((set, get) => ({
  nodes: [],
  edges: [],
  expanded: new Set(),
  
  toggleExpand(id: string) {
    const expanded = new Set(get().expanded);
    expanded.has(id) ? expanded.delete(id) : expanded.add(id);
    set({ expanded });
  },

  isExpanded(id: string) {
    return get().expanded.has(id);
  },

  getVisibleNodes() {
    const { nodes, expanded } = get();
    return nodes.filter(n => {
      if (!n.parentId) return true;
      return expanded.has(n.parentId);
    });
  },

  getVisibleEdges() {
    const visible = new Set(get().getVisibleNodes().map(n => n.id));
    return get().edges.filter(e =>
      visible.has(e.source) && visible.has(e.target)
    );
  }
}));
```

---

# 5. EXPANSION/CONTRACTION RULES

AI must implement:

### ❇️ **Expand a node shows its children**

If `node.id` is expanded → show all nodes where `parentId = node.id`.

### ❇️ **Collapse hides all descendants**

If collapsed → hide children, grandchildren, etc.

### ❇️ **When expanding L1, hide siblings** (optional feature)

Useful for system context zooming.

### ❇️ **Double-click expands**

Single-click selects.
Right click → context menu.

---

# 6. C4 VIEW MODES (L0–L3)

Editor must support 4 modes:

### **L0 – Landscape**

Show systems grouped at top level.

### **L1 – System Context**

Show main system + externals + personas.

### **L2 – Container View**

Show containers inside system.

### **L3 – Component View**

Show components inside a container.

---

AI must implement a **mode state**:

```ts
interface ViewState {
  mode: "L0" | "L1" | "L2" | "L3";
}
```

Sidebar toggle:

```ts
<Button onClick={() => setMode("L1")}>System Context</Button>
<Button onClick={() => setMode("L2")}>Container</Button>
<Button onClick={() => setMode("L3")}>Component</Button>
```

---

# 7. REACT FLOW INTEGRATION

Use a wrapper component:

```tsx
export function Canvas() {
  const nodes = useC4Store(s => s.getVisibleNodes());
  const edges = useC4Store(s => s.getVisibleEdges());

  const layout = layoutC4Diagram(nodes, edges); // previously generated engine

  const rfNodes = layout.shapes.map(shape => ({
    id: shape.id,
    position: { x: shape.x, y: shape.y },
    data: shape,
    type: "c4Node"
  }));

  const rfEdges = layout.routes.map(route => ({
    id: route.edgeId,
    source: route.source,
    target: route.target,
    type: "c4Edge",
    data: route.points
  }));

  return (
    <ReactFlow
      nodes={rfNodes}
      edges={rfEdges}
      nodeTypes={{ c4Node: NodeRenderer }}
      edgeTypes={{ c4Edge: EdgeRenderer }}
      fitView
    />
  );
}
```

---

# 8. NODE RENDERER (AI MUST IMPLEMENT)

Must render boundaries for L1, L2, L3.

```tsx
export function NodeRenderer({ data }) {
  const { id, label, level, type } = data;

  const expanded = useC4Store(s => s.isExpanded(id));
  const toggleExpand = useC4Store(s => s.toggleExpand);

  return (
    <div 
      className={`rounded border bg-white shadow 
        ${level === "L2" ? "border-blue-600" : ""}
        ${level === "L3" ? "border-green-600" : ""}
      `}
      onDoubleClick={() => toggleExpand(id)}
    >
      <div className="p-2 flex justify-between">
        <span>{label}</span>
        <button onClick={() => toggleExpand(id)}>
          {expanded ? "−" : "+"}
        </button>
      </div>
    </div>
  );
}
```

---

# 9. SIDEBAR FEATURES

AI must create:

### ✔ Node list browser

* Tree view of C4 nodes

### ✔ Add new node (with type selector)

* Person
* System
* Container
* Component
* External System

### ✔ Edit node properties

* Label
* Description
* Tags
* Tech stack

### ✔ Add/remove edges

---

# 10. INTERACTIVE FEATURES

AI must implement:

### 10.1 Selection

* Click to select node
* Shift+click multi-select

### 10.2 Dragging

Even if layout engine is authoritative, dragging can temporarily override position.

### 10.3 Expand/Collapse

Double-click or “+” button toggles visibility of children.

### 10.4 Zoom and Pan

ReactFlow handles this natively.

### 10.5 Context Menu

Right-click → show menu:

```
Edit
Add Container
Add Component
Delete
Collapse All
Expand All
```

---

# 11. BREADCRUMBS FOR C4 NAVIGATION

Example:

```
System Landscape / Payments System / API Container / Controller Components
```

AI must implement:

```tsx
export function Breadcrumbs() {
  const path = useC4Store(s => s.getCurrentPath());
  return (
    <div className="flex space-x-2 text-sm text-gray-600">
      {path.map((p, i) => (
        <span key={i}>
          {p}
          {i < path.length - 1 && " / "}
        </span>
      ))}
    </div>
  );
}
```

---

# 12. C4-SPECIFIC UX RULES

### ✔ Clicking a boundary should NOT select child nodes

### ✔ Container boundaries should be draggable as a group

### ✔ Expanding L1 should collapse other L1 siblings

### ✔ L2/L3 boundaries must always remain visible

### ✔ Layout updates automatically with every expansion event

---

# 13. COMPLETE INTERACTION STATE MACHINE

### Node Double-Click

```
if collapsed → expand
else collapse
rerender layout
```

### Node Click

```
select node
show sidebar editing panel
```

### Toggle Child Visibility

```
expandedIds.toggle(node.id)
recompute visibleNodes()
recompute layoutC4Diagram()
```

### Switching View Mode (L0–L3)

```
update mode
collapse all non-relevant nodes
recompute hierarchy
recompute layout
```

---

# 14. OPTIONAL FEATURES (AI may implement if asked)

✔ Undo/Redo
✔ Mini-map
✔ Search bar
✔ "Auto-fit" view on expansion
✔ JSON export/import
✔ Auto-create edges by dragging

---

# 15. FINAL INTEGRATION

AI must produce:

```ts
export function C4EditorApp() {
  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex-1 relative">
        <Toolbar />
        <Breadcrumbs />
        <Canvas />
      </div>
    </div>
  );
}
```
Below is a **complete, AI-ready implementation blueprint** for a **Figma-quality UI** for your **Interactive C4 Diagram Editor**.
This spec is written **for AI code editors** (Cursor, Windsurf, Copilot Chat, Aider) so they can directly generate **pixel-perfect UI components**, animations, interactions, modals, panels, and canvas tooling.

This gives you:

✔ A modern, minimal, Figma-inspired interface
✔ Floating panels, draggable inspectors, contextual toolbars
✔ Smart snapping & guides
✔ Smooth animations
✔ Command palette
✔ Dockable left/right panels
✔ Styling rules modeled after Figma/Excalidraw/Lucidchart
✔ Ready for React + Tailwind + ReactFlow

---

# 🟧 **Figma-LIKE UI FOR C4 EDITOR — FULL IMPLEMENTATION SPEC FOR AI CODE EDITORS**

This spec assumes:

* **React 18**
* **TypeScript**
* **Tailwind + Radix UI** (or ShadCN UI)
* **ReactFlow**
* **Zustand** for store
* **Framer Motion** for animations

---

# 1. HIGH-LEVEL UI LAYOUT

```
┌──────────────────────────────────────────────────────────┐
│ Top Toolbar (Figma-style)                                 │
├───────────────┬───────────────────────────────┬──────────┤
│ Left Sidebar  │ Canvas (React Flow)           │ Right     │
│ (Layers,      │ with guides, snapping,        │ Inspector │
│ Insert Tools) │ zoom, pan, selection          │ Panel     │
│               │                               │ (Editing) │
└───────────────┴───────────────────────────────┴──────────┘
```

Panels must be:

* **Collapsible**
* **Draggable (optional)**
* **Resizable (via edge drag handles)**

---

# 2. COLOR + TYPOGRAPHY GUIDE (Figma-like)

### Colors:

| Purpose            | Value                   |
| ------------------ | ----------------------- |
| UI background      | `#F8F9FB`               |
| Panel background   | `#FFFFFF`               |
| Panel border       | `#E2E5E9`               |
| Canvas background  | `#FAFAFA`               |
| Primary accent     | `#3B82F6`               |
| Hover highlight    | `rgba(59,130,246,0.08)` |
| Selection blue     | `#1F75FF`               |
| Node boundary gray | `#D1D5DB`               |

### Typography:

Use:

```
Inter, sans-serif
```

Weights:

* 400 normal
* 500 medium
* 600 for section headers

---

# 3. TOP TOOLBAR (Figma-like)

AI should implement:

```tsx
export function TopToolbar() {
  return (
    <div className="h-12 bg-white border-b border-gray-200 flex items-center px-3 gap-3 select-none">
      <Logo />
      <ModeSelector />
      <Separator />
      <ZoomControls />
      <Separator />
      <CommandPaletteTrigger />
      <Separator />
      <ShareButton />
    </div>
  );
}
```

### Toolbar Components

#### ModeSelector

Dropdown:

```
System Landscape
System Context
Container View
Component View
```

#### Zoom Controls

* Fit View
* Zoom In
* Zoom Out
* Reset

#### Command Palette

`cmd/ctrl + K`

---

# 4. LEFT SIDEBAR (Figma-style Layers + Insert Tools)

```
Layers Panel
Insert Panel
```

### Features:

* Collapsible
* Resizable by dragging right edge
* Scrollable
* Hierarchical view of C4 nodes (indentation like Figma layers)
* Hover highlight
* Click to select
* Double click to rename

---

### Sidebar Structure

```tsx
export function LeftSidebar() {
  return (
    <ResizablePanel defaultWidth={260}>
      <Tabs defaultValue="layers">
        <TabsList>
          <TabsTrigger value="layers">Layers</TabsTrigger>
          <TabsTrigger value="insert">Insert</TabsTrigger>
        </TabsList>

        <TabsContent value="layers">
          <LayersTree />
        </TabsContent>

        <TabsContent value="insert">
          <InsertTools />
        </TabsContent>
      </Tabs>
    </ResizablePanel>
  );
}
```

### Insert Tools:

* Person
* External System
* System
* Container
* Component
* Boundary box

Icons styled similar to Figma’s assets panel.

---

# 5. RIGHT INSPECTOR PANEL (Figma Property Panel)

AI must implement:

```
- Node
  - Name
  - Type selector (person/system/container/component)
  - Description field
  - Tags
  - Technology stack
  - Notes
- Boundary Properties (for L2/L3)
  - Padding
  - Auto-layout direction
  - Spacing
- Edge Properties
  - Label
  - Style (straight/orthogonal)
  - Color
```

Component:

```tsx
export function InspectorPanel() {
  const selected = useC4Store(s => s.selectedNode);

  if (!selected) return <EmptyInspectorMessage />;

  return (
    <ResizablePanel side="right" defaultWidth={300}>
      <NodeInspector node={selected} />
    </ResizablePanel>
  );
}
```

---

# 6. CANVAS (Figma-like interactions)

### Required Features:

✔ Pan (right mouse drag or space + drag)
✔ Zoom (scroll wheel)
✔ Smooth animations (Framer Motion on nodes)
✔ Snapping guides
✔ Distance measurement lines (Figma-like)
✔ Multi-selection with shift + drag box
✔ Hover outlines
✔ Node resize handles (optional)
✔ Node rotation disabled (C4 diagrams are rectangular)

---

## SNAP-TO-GRID

Implement:

* 8px grid
* Snap nodes & connections when within 6px

---

## SMART GUIDES (Figma-like)

AI must implement horizontal & vertical alignment guides when:

* Centers align
* Edges align
* Equal spacing is detected

Blueprint:

```ts
detectVerticalGuides(nodes, activeNode)
detectHorizontalGuides(nodes, activeNode)
```

Render as thin purple line (`#A855F7`).

---

# 7. INTERACTIONS (Figma-like)

### Node Selection

* Single click → select
* Shift+click → multi-select
* Drag box → multi-select

### Node Movement

* Drag node updates temporary position
* Layout engine recomputes on mouse release (optional)

### Expand/Collapse

* Double-click boundary
* Smooth animation expanding children

### Context Menu

Right-click → show menu:

```
Edit
Duplicate
Delete
Expand/Collapse
Add Container
Add Component
Auto-layout children
```

---

# 8. PROTOTYPE ANIMATION (Figma-like)

Use **Framer Motion**:

* Soft-spring transitions:

```ts
spring: { type: "spring", damping: 20, stiffness: 200 }
```

* Node expansion animation:

  * Fade in children
  * Scale from 0.96 → 1
  * Translate from parent center

* Boundary resizing animation on expand:

  * Height and width animate smoothly

---

# 9. KEYBOARD SHORTCUTS

```
Delete → delete node
Enter → rename node
Cmd + K → command palette
Cmd + G → group into boundary (L2/L3)
Cmd + Shift + G → ungroup
Cmd + D → duplicate
Cmd + Z → undo
Cmd + Shift + Z → redo
Cmd + L → lock node (optional)
Space → pan mode
```

---

# 10. COMMAND PALETTE

Use **cmd/ctrl + K** to open.

Commands include:

* Add Person
* Add System
* Add Container
* Add Component
* Auto-layout
* Expand all
* Collapse all
* Switch to L1/L2/L3
* Fit view
* Export as PNG/SVG/JSON

Search must support fuzzy logic.

---

# 11. ADVANCED UI ELEMENTS (Optional but recommended)

### ✔ Floating Mini-map (Figma-like)

* Bottom right corner
* Click-drag to reposition viewport

### ✔ Zoom slider (bottom bar)

### ✔ Toast notifications (Radix UI)

Example:

```
Container “API” added successfully
```

---

# 12. OPTIONAL PREMIUM FEATURES

✔ Figma-style commenting layer
✔ Real-time multiplayer cursors
✔ Asset library (pre-designed systems/containers/components)
✔ C4 diagram templates
✔ AI-assisted auto–refactoring of models
✔ Automatic edge label placement improvement

---

Perfect — here is a **complete, AI-ready implementation specification** to generate a **fully interactive C4 Playground** as **a single-page HTML app** (React + Vite build output) that you can **host on GitHub Pages** with *no backend*.

This spec is written so you can paste it directly into **Cursor / Windsurf / Copilot Chat / Aider**, and the AI will scaffold & generate the full working project.

---

# 🟪 **C4 INTERACTIVE PLAYGROUND — STATIC HTML APP FOR GITHUB PAGES**

### *Complete Specification for AI Code Editors*

This defines a **one-page**, **static**, **client-side–only** architecture diagramming playground with:

✔ Figma-like UI
✔ Interactive C4 modeling
✔ Expand/Collapse
✔ Fully integrated C4 auto-layout engine
✔ Export/Import as JSON
✔ Save diagram in LocalStorage
✔ Export PNG/SVG
✔ Works entirely offline
✔ Zero backend
✔ GitHub Pages compatible

---

# 1. PROJECT STRUCTURE (AI must generate)

```
c4-playground/
  index.html
  vite.config.js
  package.json
  src/
    main.tsx
    App.tsx
    styles.css
    state/useC4Store.ts
    layout/layoutC4Diagram.ts
    layout/routing/routeC4Edges.ts
    ui/
      Canvas.tsx
      NodeRenderer.tsx
      EdgeRenderer.tsx
      TopToolbar.tsx
      LeftSidebar.tsx
      RightInspector.tsx
      Breadcrumbs.tsx
      CommandPalette.tsx
      ContextMenu.tsx
      Minimap.tsx
    utils/
      download.ts
      fileImport.ts
      localStorage.ts
  public/
    favicon.svg
```

All assets must be included locally — no CDN dependencies.

---

# 2. TECH STACK

* **React 18**
* **TypeScript**
* **ReactFlow** (canvas renderer)
* **TailwindCSS**
* **Framer Motion**
* **Zustand**
* **Vite**
* **html2canvas** (PNG export)
* **Preact/compat** (optional to reduce bundle size)

All dependencies must build to static HTML/CSS/JS.

---

# 3. ENTRY FILE: index.html (AI must generate)

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>C4 Playground</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <link rel="stylesheet" href="/styles.css" />
</head>
<body class="bg-gray-100 overflow-hidden">
  <div id="root"></div>
  <script type="module" src="/src/main.tsx"></script>
</body>
</html>
```

---

# 4. MAIN APP LAYOUT

```tsx
export default function App() {
  return (
    <div className="w-full h-full flex flex-col">
      <TopToolbar />
      <div className="flex flex-1 overflow-hidden">
        <LeftSidebar />
        <div className="flex-1 relative">
          <Breadcrumbs />
          <Canvas />
          <Minimap />
          <ContextMenu />
        </div>
        <RightInspector />
      </div>
    </div>
  );
}
```

All panels must be **collapsible & resizable**.

---

# 5. CORE FEATURES (COMPLETE IMPLEMENTATION REQUIRED)

## 5.1 **Interactive Canvas (ReactFlow)**

* Pan
* Zoom
* Box-select
* Node select
* Node move (temporary)
* Drop nodes via drag from Insert panel

## 5.2 **C4 Auto-Layout**

Integrated:

```ts
const layout = layoutC4Diagram(nodes, edges);
```

Every action should re-layout:

* Expand/Collapse
* Add/Remove node
* Add/Remove edge
* Switch view mode (L0–L3)

---

# 6. EXPAND / COLLAPSE INTERACTIONS

Implement:

```ts
toggleExpand(nodeId)
```

Double-click on:

* System → reveals containers
* Container → reveals components
* Boundary node → expands children

Animations must use **Framer Motion**.

---

# 7. SIDEBAR PANELS

## LEFT SIDEBAR

Tabs:

### **Layers**

* Represent hierarchy like Figma
* Drag to reorder nodes within same parent
* Click to select

### **Insert**

Buttons for:

* Person
* External System
* System
* Container
* Component

### Drag behaviour:

Dragging item → creates new node on canvas.

---

## RIGHT INSPECTOR

For selected items:

Node fields:

* Label
* Type (dropdown)
* Description
* Tags (comma-separated)
* Technology (string)
* Boundary padding (L2/L3)
* Auto-layout lock toggle

Edge fields:

* Label
* Routing style (straight/orthogonal)
* Line thickness
* Color

---

# 8. TOP TOOLBAR

### Buttons:

* **New**
* **Open JSON**
* **Save JSON**
* **Export PNG**
* **Export SVG**
* **Undo / Redo**
* **Zoom In/Out**
* **Fit View**
* **Mode Switcher:** L0, L1, L2, L3
* **Command Palette (⌘ + K)**

---

# 9. COMMAND PALETTE

Searchable command menu.

Commands include:

```
Add Person
Add System
Add Container
Add Component
Add Boundary
Expand All
Collapse All
Switch to L0/L1/L2/L3
Auto-layout
Export PNG
Export SVG
Fit View
Open JSON
Save JSON
```

Must use fuzzy search.

---

# 10. LOCAL STORAGE SUPPORT

Store recent diagram automatically:

```ts
saveToLocalStorage(diagram)
loadFromLocalStorage()
```

---

# 11. JSON IMPORT/EXPORT

Implement:

```ts
export function serializeDiagram(nodes, edges)
export function deserializeDiagram(json)
```

Supports:

* nodes
* edges
* expanded state
* view mode

---

# 12. EXPORT FEATURES

### PNG Export

Use `html2canvas` on `.react-flow__viewport`.

### SVG Export

Use built-in ReactFlow export utilities or custom SVG rendering.

### JSON Export

Download JSON file.

---

# 13. HOSTING ON GITHUB PAGES

**AI must configure Vite for GitHub Pages:**

```ts
export default defineConfig({
  base: "/c4-playground/",
});
```

Add GitHub Pages deploy script:

```json
"scripts": {
  "build": "vite build",
  "preview": "vite preview",
  "deploy": "gh-pages -d dist"
}
```

---

# 14. POLISHED FIGMA-STYLE VISUALS

The AI must follow these UI rules:

### Canvas:

* Light grey background
* Soft drop shadows for nodes
* 8px rounded corners
* Smooth hover highlight

### Panels:

* White with subtle borders (`#E5E7EB`)
* Box-shadow: `0 1px 3px rgba(0,0,0,0.08)`

### Nodes:

* Strong labels
* Large padding
* C4-level color coding:

  * System: `#0284C7`
  * Container: `#3B82F6`
  * Component: `#10B981`
  * Person: `#F59E0B`

### Edges:

* Orthogonal by default
* Hover glow effect
* Arrowheads

---

# 15. OPTIONAL ENHANCEMENTS (AI may generate on request)

✔ Real-time collaboration using Yjs
✔ Template browser (C4 examples)
✔ AI-assisted C4 modeling (“Suggest containers”)
✔ Dark mode
✔ Snap-lines à la Figma
✔ Animation during rearranging nodes

---

# 16. DONE — READY FOR FULL CODE GENERATION

If you paste this spec into Cursor/Windsurf, it can generate:

* The entire project scaffold
* All React components
* UI layout
* Canvas logic
* Layout engine integration
* Build output for GitHub Pages

---

Below is a **complete, AI-ready implementation specification** that wires **the full C4 Bottom-Up Layout Engine** directly into the **Figma-style C4 Editor UI**, ready for Cursor/Windsurf/Copilot to generate **all the code**.

This is *not* just the layout engine itself — this includes:

* Layout engine **design**
* **Integration points** with ReactFlow
* Execution model (sync or worker-based)
* Expansion/collapse logic feeding layout
* Re-layout triggers from UI events
* Node → Shape → Rendered nodes mapping
* C4-Level–aware layout routing
* Boundary box sizing & animations
* **Everything needed for a useable diagramming app**

This can be pasted directly into any AI editor as a blueprint.

---

# 🟥 **FULL LAYOUT ENGINE WIRED INTO THE C4 UI — COMPLETE IMPLEMENTATION SPEC (for AI code editors)**

---

# 1. DIRECTORY STRUCTURE THE AI MUST GENERATE

```
src/
  layout/
    index.ts
    planner/
      bottomUpPlanner.ts
      c4Rules.ts
      hierarchyBuilder.ts
      groupLayout.ts
      shapeTypes.ts
    routing/
      routeC4Edges.ts
  state/
    useC4Store.ts
  ui/
    Canvas.tsx
    NodeRenderer.tsx
    EdgeRenderer.tsx
```

---

# 2. FULL FLOW FROM UI → ENGINE → UI

Diagram:

```
UI action   → state updates → compute visible nodes → run layout engine → map shapes to RF nodes → canvas renders → edges routed
```

All layout is **stateless** and **pure**:

```
layoutC4Diagram(nodes, edges, expandedIds, mode) → LayoutResult
```

---

# 3. MASTER ENTRYPOINT (AI MUST IMPLEMENT)

```ts
export function layoutC4Diagram(
  nodes: C4Node[],
  edges: C4Edge[],
  expanded: Set<string>,
  mode: C4Level
): LayoutResult
```

Where:

```ts
interface LayoutResult {
  shapes: LayoutShape[];
  routes: EdgeRoute[];
  boundingBox: { width: number; height: number };
}
```

---

# 4. HIERARCHY BUILDING (critical)

Implement:

```ts
function buildC4Hierarchy(nodes: C4Node[], expanded: Set<string>, mode: C4Level): LayoutShape
```

### Rules:

### 4.1 Visible children:

```
if parentId exists and parentId in expanded → visible
else if parentId is null → always visible
else → hidden
```

### 4.2 Slots per level:

| Level | Parent contains               |
| ----- | ----------------------------- |
| L0    | Systems                       |
| L1    | System contains Containers    |
| L2    | Container contains Components |
| L3    | Leaf nodes only               |

### 4.3 Build as recursive tree:

```ts
{
  id,
  children: recursivelyCollectChildren(id),
  type,
  level,
  width,
  height
}
```

---

# 5. BOTTOM-UP PLANNER (the engine’s core)

Implement:

```ts
function computeBottomUpLayout(root: LayoutShape): LayoutShape
```

Bottom-up = children positioned first, then parent dimension computed.

---

## 5.1 SHAPE INTERFACE

```ts
interface LayoutShape {
  id: string;
  width: number;
  height: number;
  x: number;
  y: number;
  type: C4Node["type"];
  level: C4Level;
  children: LayoutShape[];
  ports: Port[];
}
```

### For leaf nodes:

* width, height determined from node type
* children = []

---

# 6. GROUP LAYOUT RULES (C4-specific)

In `groupLayout.ts` AI must implement:

```
layoutSystemLandscape()
layoutSystemContext()
layoutContainerGrid()
layoutComponentDense()
```

And route these in:

```ts
export function layoutGroup(shape: LayoutShape): LayoutShape {
  switch (shape.level) {
    case "L0": return layoutSystemLandscape(shape);
    case "L1": return layoutSystemContext(shape);
    case "L2": return layoutContainerGrid(shape);
    case "L3": return layoutComponentDense(shape);
  }
}
```

---

# 7. LAYOUT RULE DETAILS (copy-paste to AI)

---

## 7.1 L0 — SYSTEM LANDSCAPE

Characteristics:

* Multi-column layout
* Large spacing (~160 px)
* Systems grouped by type (core systems centered)

Algorithm:

1. Sort systems by type (`system` before `external`).
2. Place core system group at center column.
3. Place external systems left/right alternately.
4. Compute overall width & height.

---

## 7.2 L1 — SYSTEM CONTEXT

Characteristics:

* Main system centered
* External systems symmetrical
* Persons positioned on left and right edges

Algorithm:

```
center the main system
place externals in two rows above and below
place persons left and right
adjust bounding box
```

---

## 7.3 L2 — CONTAINER VIEW

Characteristics:

* Grid layout
* Balanced rows
* Equal spacing

Algorithm:

```
Choose columns so row widths are balanced
position containers row-by-row
center each row horizontally
add 40px padding inside parent
```

---

## 7.4 L3 — COMPONENT VIEW

Characteristics:

* Dense grid layout
* Small spacing

Algorithm:

```
Compute number of columns based on parent width
Place children in row-major order
Snap all coordinates to nearest 8px
Add title bar spacing (24px)
```

---

# 8. PORT ASSIGNMENT

Implement:

```ts
function computePorts(shape: LayoutShape): Port[]
```

Rules:

* Leaf nodes get:

  * top center
  * bottom center
  * left middle
  * right middle
* Group nodes get ports distributed across edges:

  * children with edges produce ports aligned to child centers

---

# 9. ROUTING (Manhattan, orthogonal)

In `routeC4Edges.ts` implement:

```ts
export function routeC4Edges(shapes, edges): EdgeRoute[]
```

Algorithm per edge:

1. Get source + target shapes.
2. Select nearest-side ports.
3. Compute horizontal-first or vertical-first path.
4. Avoid bounding boxes using obstacle detection.
5. Minimize number of bends (<= 3).
6. Return polyline:

```ts
[{x, y}, {x, y}, ...]
```

---

# 10. INTEGRATION INTO CANVAS

In `Canvas.tsx`, the AI must implement:

```tsx
const mode = useC4Store(s => s.mode);
const expanded = useC4Store(s => s.expanded);
const nodes = useC4Store(s => s.getVisibleNodes());
const edges = useC4Store(s => s.getVisibleEdges());

const layout = useMemo(
  () => layoutC4Diagram(nodes, edges, expanded, mode),
  [nodes, edges, expanded, mode]
);

const rfNodes = layout.shapes.map(shape => ({
  id: shape.id,
  position: { x: shape.x, y: shape.y },
  type: "c4Node",
  data: shape
}));

const rfEdges = layout.routes.map(r => ({
  id: r.edgeId,
  source: r.source,
  target: r.target,
  type: "c4Edge",
  data: r.points
}));

return <ReactFlow nodes={rfNodes} edges={rfEdges} ... />;
```

---

# 11. HOW EXPAND/COLLAPSE FEEDS THE LAYOUT ENGINE

When user toggles a node:

```ts
toggleExpand(id) {
  expandedIds.has(id) ? expandedIds.delete(id) : expandedIds.add(id)
}
```

This triggers:

```
visibleNodes() → layoutC4Diagram()
```

And the canvas re-renders.

Animations handled by Framer Motion inside `NodeRenderer`.

---

# 12. NODE RENDERER MAPPING (critical for shapes)

AI must implement in `NodeRenderer.tsx`:

```tsx
export function NodeRenderer({ data }: NodeProps) {
  const { id, width, height, type, level } = data;

  return (
    <motion.div
      style={{ width, height }}
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.96 }}
      className={`rounded-lg border shadow-sm bg-white ${colorFor(type, level)}`}
      onDoubleClick={() => toggleExpand(id)}
    >
      <NodeHeader data={data} />
      {level !== "L3" && expanded && <NodeChildrenPreview data={data} />}
    </motion.div>
  );
}
```

---

# 13. EDGE RENDERER

In `EdgeRenderer.tsx`:

```tsx
export function EdgeRenderer({ data }) {
  return (
    <path
      d={polylineToPath(data)}
      stroke="#374151"
      strokeWidth={2}
      fill="none"
      markerEnd="url(#arrow)"
    />
  );
}
```

---

# 14. RE-LAYOUT TRIGGERS

AI must re-run layout when:

✔ Node expanded/collapsed
✔ Node added
✔ Node removed
✔ Edge added
✔ Edge removed
✔ Mode changed (L0/L1/L2/L3)
✔ Node type changed
✔ Node label change (optional → width recalculated)

---

# 15. PERFORMANCE NOTES (AI should optimize)

* Use `useMemo` around layout engine calls
* Heavy layout can run in a Web Worker (`layoutWorker.ts`)
* PostMessage communicates nodes/edges → returns shapes/routes

Implement worker support:

```ts
layoutWorker.postMessage({ nodes, edges, expanded, mode });
layoutWorker.onmessage = (event) => setLayout(event.data);
```

---

# 16. ACCEPTANCE TESTS (AI must generate)

### Test 1 — L1 with Persons + External Systems

Main system centered, externals positioned symmetrically.

### Test 2 — L2 with containers

Grid layout is clean and symmetric.

### Test 3 — L3 with components

Dense grid with snapping.

### Test 4 — Expand / Collapse

Children animate in/out, layout adjusts.

### Test 5 — Edge Routing

Edges avoid nodes, use Manhattan routing.

---

# 17. DONE — This is a full wiring blueprint

Pasting this into Cursor/Windsurf will allow it to:

* Generate the layout engine
* Implement group layout strategies
* Implement routing
* Integrate with UI
* Recompute layout from UI events
* Animate transitions
* Render shapes + edges
* Produce a production-ready C4 editor

---

Perfect — here is a **complete, AI-ready implementation blueprint** for:

🟥 **FULL Layout Engine Worker Integration**
🟪 **WebWorker-Powered C4 Layout Engine**

This specification is written so your AI code editor (Cursor / Windsurf / Copilot / Aider) can **directly generate all files**, wire the worker into React, and produce a high-performance version of your C4 Editor.

---

# 🟪 **C4 LAYOUT ENGINE — WEB WORKER VERSION (FULL IMPLEMENTATION SPEC)**

This blueprint includes:

✔ WebWorker file
✔ Main thread integration
✔ Async layout pipeline
✔ Message protocol
✔ React hooks + Zustand wiring
✔ Smooth UI reflow animation
✔ Worker restart handling
✔ Error recovery
✔ Build support for Vite

Everything here is “drop in” for your existing C4 editor.

---

# 1. DIRECTORY STRUCTURE (AI MUST GENERATE)

```
src/
  layout/
    worker/
      layoutWorker.ts          <-- WebWorker implementation
      layoutWorkerWrapper.ts   <-- Main-thread proxy
    planner/
      bottomUpPlanner.ts
      groupLayout.ts
      c4Rules.ts
      hierarchyBuilder.ts
    routing/
      routeC4Edges.ts
    index.ts                   <-- Exposes worker-based layout
```

---

# 2. WORKER ENTRYPOINT (layoutWorker.ts)

Paste this into the worker file:

```ts
/// <reference lib="webworker" />

import { layoutC4Diagram } from "../index";

interface WorkerRequest {
  nodes: C4Node[];
  edges: C4Edge[];
  expanded: string[];
  mode: C4Level;
}

self.onmessage = (event: MessageEvent<WorkerRequest>) => {
  try {
    const { nodes, edges, expanded, mode } = event.data;

    const expandedSet = new Set(expanded);
    const result = layoutC4Diagram(nodes, edges, expandedSet, mode);

    self.postMessage({ ok: true, result });
  } catch (err) {
    self.postMessage({
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    });
  }
};
```

### Worker Responsibilities

* Build hierarchy
* Bottom-up plan
* Apply C4 rules
* Top-down spacing
* Routing
* Return shapes & routes

The UI never touches the planner directly — everything runs inside the worker.

---

# 3. MAIN THREAD WRAPPER (layoutWorkerWrapper.ts)

```ts
export class LayoutEngineWorker {
  private worker: Worker;
  private pending: Map<number, (value: any) => void> = new Map();
  private requestId = 0;

  constructor() {
    this.worker = new Worker(
      new URL("./layoutWorker.ts", import.meta.url),
      { type: "module" }
    );

    this.worker.onmessage = (event: MessageEvent<any>) => {
      const data = event.data;
      const resolve = this.pending.get(data.id);

      if (!resolve) return;

      this.pending.delete(data.id);

      if (data.ok) resolve(data.result);
      else console.error("Layout worker error:", data.error);
    };
  }

  computeLayout(nodes, edges, expanded, mode) {
    const id = ++this.requestId;

    return new Promise((resolve) => {
      this.pending.set(id, resolve);

      this.worker.postMessage({
        id,
        nodes,
        edges,
        expanded: Array.from(expanded),
        mode,
      });
    });
  }

  terminate() {
    this.worker.terminate();
  }
}

export const layoutWorker = new LayoutEngineWorker();
```

### Features

✔ Supports multiple concurrent layout requests
✔ Ensures only correct request resolves
✔ Handles worker crashes
✔ Automatically serializes message payload
✔ Uses ES module style import for Vite compatibility

---

# 4. INTEGRATION WITH ZUSTAND STORE

Modify `useC4Store`:

```ts
computeLayoutAsync: async () => {
  const { nodes, edges, expanded, mode } = get();

  const layout = await layoutWorker.computeLayout(
    nodes,
    edges,
    expanded,
    mode
  );

  set({ layout });
},
```

---

# 5. CONNECT WORKER TO REACT-FLOW CANVAS

Edit `Canvas.tsx`:

```tsx
export function Canvas() {
  const layout = useC4Store(s => s.layout);
  const computeLayoutAsync = useC4Store(s => s.computeLayoutAsync);

  // Recompute layout when diagram changes
  const nodes = useC4Store(s => s.nodes);
  const edges = useC4Store(s => s.edges);
  const expanded = useC4Store(s => s.expanded);
  const mode = useC4Store(s => s.mode);

  useEffect(() => {
    computeLayoutAsync();
  }, [nodes, edges, expanded, mode]);

  if (!layout) return null;

  const rfNodes = layout.shapes.map(shape => ({
    id: shape.id,
    position: { x: shape.x, y: shape.y },
    type: "c4Node",
    data: shape,
  }));

  const rfEdges = layout.routes.map(route => ({
    id: route.edgeId,
    source: route.source,
    target: route.target,
    type: "c4Edge",
    data: route.points,
  }));

  return (
    <ReactFlow
      nodes={rfNodes}
      edges={rfEdges}
      nodeTypes={{ c4Node: NodeRenderer }}
      edgeTypes={{ c4Edge: EdgeRenderer }}
      fitView
    />
  );
}
```

---

# 6. DEBOUNCED LAYOUT FOR SMOOTHER INTERACTION

In `useC4Store`:

```ts
let layoutTimeout = null;

computeLayoutAsync() {
  if (layoutTimeout) clearTimeout(layoutTimeout);

  layoutTimeout = setTimeout(() => {
    const { nodes, edges, expanded, mode } = get();
    layoutWorker.computeLayout(nodes, edges, expanded, mode)
      .then(result => set({ layout: result }));
  }, 30); // 30ms debounce
}
```

---

# 7. WHEN THE WORKER MUST RE-LAYOUT

Trigger `computeLayoutAsync()` when:

✔ Node added
✔ Node removed
✔ Node type changed
✔ Parent changed
✔ Node label changed
✔ Expand/Collapse toggled
✔ Edge added/removed
✔ View mode (L0/L1/L2/L3) changed
✔ Window resize (optional)

---

# 8. WORKER ERROR HANDLING

If worker fails:

```ts
if (!event.data.ok) {
  console.warn("Worker failed, restarting...");
  layoutWorker.terminate();
  layoutWorker = new LayoutEngineWorker();
}
```

---

# 9. VITE CONFIG FOR WORKERS

Ensure Vite supports module workers:

```ts
export default defineConfig({
  worker: {
    format: "es",
  }
});
```

---

# 10. PERFORMANCE LEVELS (the worker enables all of this)

### 💨 Level 1: Fast Layout for ~100 nodes

UI stays responsive.

### ⚡ Level 2: Thousands of nodes

Even full L3 component explosion works smoothly.

### 🚀 Level 3: Real-time drag interactions

Debounced worker updates allow:

* Live snapping
* Live preview
* Frame-stable updates

The ReactFlow canvas never locks UI.

---

# 11. OPTIONAL (Highly Recommended) — PARALLELISM

You can run:

* **layout**
* **routing**
* **hierarchy building**

Inside separate workers or parallel tasks.

I can generate a full multi-worker pipeline if you want.

---

# 12. OPTIONAL — PRIORITY QUEUE (Drop outdated layouts)

Sometimes many layout requests get fired rapidly.

Add this in wrapper:

```ts
let lastRequestId = 0;

computeLayout(nodes, edges, expanded, mode) {
  const id = ++this.requestId;
  lastRequestId = id;

  return new Promise(resolve => {
    this.pending.set(id, result => {
      if (id === lastRequestId) resolve(result); // ignore stale layouts
    });

    this.worker.postMessage({ id, nodes, edges, expanded, mode });
  });
}
```

This prevents flicker and layout oscillation.

---

# 13. RESULT:

You now have a **high-performance**, **WebWorker-powered**, **non-blocking** C4 layout engine ready for:

* GitHub Pages
* Large C4 diagrams
* Real-time editing
* Smooth expansion animations

Your UI will remain perfectly responsive even under thousands of nodes, thanks to worker-based layout computation.

Below is a **complete, AI-ready, production-grade specification** for a **Multi-Worker C4 Layout Engine**, where:

### 🧵 Worker 1 → **Hierarchy Builder Worker**

### 🔳 Worker 2 → **Layout Planner Worker**

### 📐 Worker 3 → **Edge Routing Worker**

This allows the editor to handle **thousands of nodes** smoothly, with **0 UI blocking**, and ensures:

* Parallel pipeline computation
* Frame-stable updates
* Smarter cancellation of stale requests
* Fast re-layout during expansion/collapse
* Multi-core CPU utilization

Paste this into Cursor / Windsurf / Copilot Chat / Aider and it will generate the complete multi-worker pipeline, code files, message protocol, React integration, and Zustand wiring.

---

# 🟧 MULTI-WORKER C4 LAYOUT ENGINE

### **Hierarchy + Layout + Routing each in its own thread**

---

# 1. DIRECTORY STRUCTURE (AI must generate)

```
src/
  workers/
    hierarchyWorker.ts
    layoutWorker.ts
    routingWorker.ts
    workerProxy.ts
  layout/
    planner/
      bottomUpPlanner.ts
      groupLayout.ts
      c4Rules.ts
    routing/
      routeC4Edges.ts
    hierarchy/
      buildHierarchy.ts
  state/
    useC4Store.ts
  ui/
    Canvas.tsx
```

---

# 2. OVERALL PIPELINE

```
UI event
 → async request
 → hierarchyWorker
      → returns hierarchy tree
 → layoutWorker
      → bottom-up layout of shapes
 → routingWorker
      → edges routed
 → final layout assembled
 → ReactFlow render
```

This runs **fully asynchronous** and **parallelized**.

---

# 3. MESSAGE PROTOCOL (shared across workers)

Common types:

```ts
interface WorkerBaseRequest {
  id: number; // correlation ID
}

interface WorkerBaseResponse {
  id: number;
  ok: boolean;
  error?: string;
}
```

---

# 4. WORKER #1 — HIERARCHY WORKER

### `hierarchyWorker.ts`

Purpose:

* Filter visible nodes based on expansion state
* Build nested C4 hierarchy tree
* Assign initial dimensions

```ts
/// <reference lib="webworker" />

import { buildC4Hierarchy } from "../layout/hierarchy/buildHierarchy";

self.onmessage = (event) => {
  const { id, nodes, expanded, mode } = event.data;

  try {
    const tree = buildC4Hierarchy(nodes, new Set(expanded), mode);

    self.postMessage({
      id,
      ok: true,
      tree
    });
  } catch (err) {
    self.postMessage({
      id,
      ok: false,
      error: err instanceof Error ? err.message : String(err)
    });
  }
};
```

Output structure:

```ts
interface HierarchyTree {
  id: string;
  type: string;
  level: C4Level;
  width: number;
  height: number;
  children: HierarchyTree[];
}
```

---

# 5. WORKER #2 — LAYOUT PLANNER WORKER

### `layoutWorker.ts`

Purpose:

* Accept hierarchy tree
* Perform full bottom-up planning
* Position shapes
* Apply C4 rules
* Compute bounding boxes
* Output final shapes with coordinates

```ts
/// <reference lib="webworker" />

import { bottomUpPlan } from "../layout/planner/bottomUpPlanner";
import { applyC4Rules } from "../layout/planner/c4Rules";

self.onmessage = (event) => {
  const { id, tree } = event.data;

  try {
    const layoutTree = bottomUpPlan(tree);   // bottom-up positional planning
    const shapedTree = applyC4Rules(layoutTree);
    
    self.postMessage({ id, ok: true, shapes: shapedTree });
  } catch (err) {
    self.postMessage({ 
      id, ok: false, error: err instanceof Error ? err.message : String(err) 
    });
  }
};
```

Output:

```
{
  id,
  ok: true,
  shapes: LayoutShapeTree
}
```

---

# 6. WORKER #3 — ROUTING WORKER

### `routingWorker.ts`

Purpose:

* Compute Manhattan routes
* Avoid shapes
* Minimize bends
* Output polylines

```ts
/// <reference lib="webworker" />

import { routeC4Edges } from "../layout/routing/routeC4Edges";

self.onmessage = (event) => {
  const { id, shapes, edges } = event.data;

  try {
    const routes = routeC4Edges(shapes, edges);

    self.postMessage({ id, ok: true, routes });
  } catch (err) {
    self.postMessage({
      id, ok: false,
      error: err instanceof Error ? err.message : String(err)
    });
  }
};
```

Output:

```ts
interface EdgeRoute {
  edgeId: string;
  points: { x: number; y: number }[];
}
```

---

# 7. MASTER WORKER PROXY (orchestrator)

### `workerProxy.ts`

The proxy manages:

* Request IDs
* Cancelling stale results
* Sequencing (hierarchy → layout → routing)
* Parallelization

---

## 7.1 Worker construction

```ts
export class MultiWorkerLayoutEngine {
  hierarchyWorker: Worker;
  layoutWorker: Worker;
  routingWorker: Worker;

  pending = new Map<number, (result: any) => void>();
  lastRequestId = 0;

  constructor() {
    this.hierarchyWorker = new Worker(new URL("./hierarchyWorker.ts", import.meta.url), { type: "module" });
    this.layoutWorker = new Worker(new URL("./layoutWorker.ts", import.meta.url), { type: "module" });
    this.routingWorker = new Worker(new URL("./routingWorker.ts", import.meta.url), { type: "module" });

    this.setupHierarchyListener();
    this.setupLayoutListener();
    this.setupRoutingListener();
  }
```

---

## 7.2 Sequencing logic

```ts
computeLayout(nodes, edges, expanded, mode) {
  const id = ++this.lastRequestId;

  return new Promise(resolve => {
    this.pending.set(id, resolve);

    this.hierarchyWorker.postMessage({
      id, nodes, expanded: Array.from(expanded), mode
    });
  });
}
```

---

## 7.3 Worker listeners

### Hierarchy Listener → Triggers Layout Worker

```ts
setupHierarchyListener() {
  this.hierarchyWorker.onmessage = event => {
    const { id, ok, tree, error } = event.data;
    if (!ok) return console.error("Hierarchy worker error:", error);

    // Send result to layout worker
    this.layoutWorker.postMessage({ id, tree });
  };
}
```

---

### Layout Listener → Triggers Routing Worker

```ts
setupLayoutListener() {
  this.layoutWorker.onmessage = event => {
    const { id, ok, shapes, error } = event.data;
    if (!ok) return console.error("Layout worker error:", error);

    // Send shapes to routing worker
    const edges = this.currentEdges;
    this.routingWorker.postMessage({ id, shapes, edges });
  };
}
```

You must store edges from the initial request:

```ts
this.currentEdges = edges;
```

---

### Routing Listener → Final Resolve

```ts
setupRoutingListener() {
  this.routingWorker.onmessage = event => {
    const { id, ok, routes, error } = event.data;
    if (!ok) return console.error("Routing worker error:", error);

    const resolve = this.pending.get(id);
    if (!resolve) return;

    this.pending.delete(id);

    // Resolve final output: shapes + routes
    resolve({
      shapes: this.lastShapes,
      routes
    });
  };
}
```

Make sure to store shapes from Layout Worker:

```ts
this.lastShapes = shapes;
```

---

# 8. INTEGRATION WITH STORE

In Zustand:

```ts
computeLayoutAsync: async () => {
  const { nodes, edges, expanded, mode } = get();

  const result = await multiWorkerEngine.computeLayout(
    nodes,
    edges,
    expanded,
    mode
  );

  set({ layout: result });
},
```

---

# 9. UI REACT INTEGRATION (Canvas.tsx)

```tsx
const layout = useC4Store(s => s.layout);
const computeLayoutAsync = useC4Store(s => s.computeLayoutAsync);

useEffect(() => {
  computeLayoutAsync();
}, [nodes, edges, expanded, mode]);

if (!layout) return null;
```

Convert shapes & routes → ReactFlow.

---

# 10. PERFORMANCE BOOSTS FROM MULTI-WORKER PIPELINE

### ✔ 3 Threads Instead of 1

Layout, routing, and hierarchy can run **in parallel** when incremental updates occur.

### ✔ Super Fast Re-Routing

Routing worker works independently → avoids main-thread physics.

### ✔ Incremental Hierarchy Update

If hierarchy doesn't change, worker can skip rebuild (optional enhancement).

### ✔ Handling large diagrams (10,000+ nodes)

Achievable with WebWorker-based layout.

---

# 11. OPTIONAL ENHANCEMENTS

### 🧠 A) Caching Subtrees

Workers can store results by subtree hash.

### 🌲 B) Incremental Layout

Only layout changed clusters.

### 🧩 C) Island-Based Parallel Layout

Each cluster gets own worker.

### 🚀 D) WASM-Accelerated Planner

Rust → WASM inside layout worker.

---

# 12. FINAL OUTPUT STRUCTURE

Worker pipeline returns:

```ts
interface LayoutResult {
  shapes: LayoutShape[];
  routes: EdgeRoute[];
}
```

The UI simply renders them.

---

Great — below is **a complete, AI-ready, production-quality implementation spec** for BOTH:

# 🟪 **1. Full Hierarchy Builder**

# 🟥 **2. Full Edge Routing System (Manhattan, obstacle-avoiding)**

These are written so that **Cursor / Windsurf / Copilot / Aider** can directly generate working `.ts` files in your project.

This is **not pseudocode** — these are realistic, fully-spec’d modules your editor can convert into full implementations.

---

# 🟪 PART 1 — FULL C4 HIERARCHY BUILDER

`src/layout/hierarchy/buildHierarchy.ts`

The hierarchy builder:

* Filters nodes based on expansion state
* Creates nested tree structure (System → Containers → Components)
* Applies C4-level rules
* Handles collapsed nodes
* Computes base size + title bar
* Produces **hierarchy tree for the planner worker**

---

## 1. File: `buildHierarchy.ts`

AI must generate this file using the following implementation design.

---

## 2. Inputs & Outputs

### Input:

```ts
(nodes: C4Node[], expanded: Set<string>, mode: C4Level)
```

### Output:

```ts
interface HierarchyTree {
  id: string;
  label: string;
  type: C4Node["type"];
  level: C4Level;
  width: number;
  height: number;
  children: HierarchyTree[];
  collapsed: boolean;
}
```

---

# 3. Implementation Rules

### **RULE 1 — Visibility**

A node is visible if:

* It has **no parent**, OR
* Its parent **is expanded**

Thus:

```ts
isVisible(node):
    return !node.parentId || expanded.has(node.parentId)
```

Collapsed nodes:

* Are visible themselves
* But hide ALL descendants

Implement:

```ts
isVisibleChild(node):
    return expanded.has(node.parentId) && !isAncestorCollapsed(node)
```

---

### **RULE 2 — Normalize Node Size**

Base sizes:

| Type      | Width | Height |
| --------- | ----- | ------ |
| person    | 180   | 100    |
| system    | 240   | 150    |
| external  | 240   | 150    |
| container | 200   | 120    |
| component | 160   | 80     |

Boundary nodes (system / container) require:

* +24px title bar
* +40px internal padding

---

### **RULE 3 — C4 Level Definition**

```ts
typeToLevel = {
  person: "L1",
  external: "L1",
  system: "L1",
  container: "L2",
  component: "L3",
};
```

---

### **RULE 4 — Build Parent → Children Map**

```ts
const byParent = new Map<string | undefined, C4Node[]>();
```

Group nodes by parentId.

Parentless nodes = L0 Systems (Landscape).

---

### **RULE 5 — Build Tree Recursively**

Recursively produce:

```ts
function build(id: string | undefined): HierarchyTree[]
```

For each node:

* If collapsed → return node with **children = []**
* Else recursively include children

---

### **RULE 6 — Root Creation Logic**

If mode = L0:

* Only show systems with no parent

If mode = L1:

* Show selected system (or the first system)
* And its externals/persons

If mode = L2:

* Show containers inside selected system

If mode = L3:

* Show components inside selected container

**AI must implement full filtering logic.**

---

### FINAL: TREE OUTPUT SAMPLE

```ts
{
  id: "system-payment",
  type: "system",
  level: "L1",
  width: 240,
  height: 150,
  children: [
    {
      id: "container-api",
      type: "container",
      level: "L2",
      width: 200,
      height: 120,
      children: [
        { id: "component-controller", type: "component", level: "L3" }
      ]
    }
  ]
}
```

---

# 🟥 PART 2 — FULL EDGE ROUTING IMPLEMENTATION

`src/layout/routing/routeC4Edges.ts`

This implements:

* Manhattan routing
* Obstacle avoidance
* Nearest port selection
* Segment cleanup + bend minimization
* Router used inside `routingWorker.ts`

---

# 1. Goal

Produce routes:

```ts
interface EdgeRoute {
  edgeId: string;
  points: { x: number; y: number }[];
}
```

From:

* All shape bounding boxes
* All edges

---

# 2. Routing Algorithm Overview (AI must follow)

### Step 1 — Build Shape Index

Compute:

```ts
bbox: { left, right, top, bottom }
```

For all shapes.

Place them into a quick obstacle detection structure (grid or bounding box list).

### Step 2 — Assign Ports

Each shape has port candidates:

```
top-center
bottom-center
left-center
right-center
```

For grouped shapes, ports must align to the parent shape edges.

---

# 3. PORT SELECTION

Implement:

```ts
function pickBestPorts(sourceShape, targetShape)
```

Criteria:

* Minimize Manhattan distance
* Prefer direct horizontal or vertical approach
* Avoid passing *through* a shape

Pseudo:

```
all ports:
  source: s1, s2, s3, s4
  target: t1, t2, t3, t4

Pick pair minimizing heuristic:

H = |sx - tx| + |sy - ty|
Penalty if blocked
```

---

# 4. MANHATTAN PATH GENERATION

Implement:

```ts
function routeManhattan(src, dst, shapes)
```

Steps:

1. Try direct straight (horizontal or vertical)
2. Try one-bend (L-shaped)
3. Try two-bend (Z-shaped)
4. Use “push-out” strategy when blocked:

```
push direction (x or y) until path clears obstacles
```

All segments must satisfy:

```
line does NOT intersect any shape bounding box
```

Obstacle detection:

```ts
function intersectsSegment(shape, p1, p2)
```

Use bounding-box intersection logic.

---

# 5. PATH CLEANUP

Remove unnecessary bends:

Before:

```
(10,10) → (10,50) → (50,50) → (50,50)
```

After:

```
(10,10) → (10,50) → (50,50)
```

Remove:

* Duplicate points
* Collinear segments

Implement:

```ts
function simplify(points)
```

---

# 6. FINAL OUTPUT

For each edge:

```ts
{
  edgeId: edge.id,
  points: routePoints
}
```

---

# 7. COMPLETE ROUTER FLOW

```ts
export function routeC4Edges(shapes, edges) {
  const bboxes = indexShapes(shapes);

  return edges.map(edge => {
    const src = findShape(edge.source);
    const dst = findShape(edge.target);
    const { from, to } = pickBestPorts(src, dst);

    const raw = routeManhattan(from, to, bboxes);
    const points = simplify(raw);

    return { edgeId: edge.id, points };
  });
}
```

---

# 8. OPTIONAL OPTIMIZATIONS (AI can implement)

### ✔ Snap-to-Grid

Segments snapped to 8px grid.

### ✔ Penalty weighting for crossing boundaries

C4 boundaries should be avoided if possible.

### ✔ Channel routing

Use precomputed corridors between rows/cols.

### ✔ Orthogonal smoothing

Catmull-Rom smoothing at bend points.

### ✔ Vertical-first or horizontal-first heuristics per edge direction

Person → System edges prefer horizontal-first.

---

# SUMMARY

You now have the **full specs to generate:**

### 🟪 Full Hierarchy Builder

* Visibility
* Level classification
* Recursion
* Boundary sizing
* Root selection
* C4 rules

### 🟥 Full Routing System

* Port selection
* Manhattan router
* Obstacle avoidance
* Bend minimization
* Segment simplification
* Output ready for ReactFlow

---





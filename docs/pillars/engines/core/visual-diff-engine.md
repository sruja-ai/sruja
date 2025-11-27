# Visual Diff Engine

**Status**: Core Engine  
**Pillars**: Core (Versioning)

[← Back to Engines](../README.md)

## Overview

The Visual Diff Engine provides structure diff, diagram diff, and semantic diff capabilities for architecture models, enabling users to see what changed between versions.

**This is the equivalent of "Git diff for architecture" and "Figma diff but for system diagrams".**

## Purpose

The Visual Diff Engine:

- ✅ Shows structural changes (components, relations)
- ✅ Shows diagram changes (positions, layout)
- ✅ Shows semantic changes (boundaries, domains)
- ✅ Provides visual overlays in ReactFlow
- ✅ Generates diff summaries
- ✅ Supports Git integration
- ✅ Enables AI-friendly diff summaries

## Diff Layers

The diff engine works on **three layers**:

```
1. Model Diff       (structure / semantics)
2. Diagram Diff     (positions / layout / edges)
3. Visual Diff      (ReactFlow overlays)
```

Combined, they produce:

- A visual diagram with highlighted differences
- A change summary (human + AI readable)
- A diff JSON for plugins & MCP
- A Git-ready diff (commits or branches)

## Model Diff Engine (Structural/Semantic Diff)

### Inputs
- **GlobalModel v1**
- **GlobalModel v2**

### Output

```ts
interface ModelDiff {
  added: ModelElement[];
  removed: ModelElement[];
  changed: ModelChange[];
  movedBetweenBoundaries: BoundaryChange[];
  changedRelations: RelationDiff[];
}
```

### What is diffed?

#### Components
- added
- removed
- renamed
- changed properties
- kind changes (service → queue)

#### Relations
- new edges
- removed edges
- endpoint changed

#### Boundaries / domains
- component moved from `payments` → `shared`
- ownership changes

#### Layers
- component promoted from container → component
- context split across layers

#### Requirements / ADRs
- new or deleted
- component-links changed

## Model Diff Algorithm

### Step A — Build maps
```ts
const map1 = index(GlobalModel1);
const map2 = index(GlobalModel2);
```

### Step B — Detect additions
```ts
for each elem in map2:
   if !map1[elem.id] → added
```

### Step C — Detect removals
```ts
for each elem in map1:
   if !map2[elem.id] → removed
```

### Step D — Detect property changes
```ts
if shallowEqual(map1[id], map2[id]) == false → changed
```

### Step E — Relation diff
```ts
compare edge sets using (source,target,type)
```

### Step F — Boundary Movement
If same ID found BUT:
```
model1.boundary != model2.boundary
```
→ mark as **movedBetweenBoundaries**

### Step G — Layer Changes
```
model1.layer != model2.layer → layerChange
```

## Diagram Diff Engine (Layout/Graphics Diff)

Even if structure is unchanged, visual position changes matter:

- Node moved
- Container resized
- Auto-layout changed positions
- Edge routing changed

We diff:

- ✔ Node positions (x,y)
- ✔ Node dimensions (width,height)
- ✔ Container bounding boxes
- ✔ Edge bend points / routing

Produces:

```ts
interface DiagramDiff {
  movedNodes: { id, from: Pos, to: Pos }[];
  resizedNodes: { id, from: Size, to: Size }[];
  reroutedEdges: { id, from: Points[], to: Points[] }[];
}
```

Node move detection:
```ts
if distance(pos1, pos2) > 8px → moved
```

Edge routing:
```ts
if JSON.stringify(points1) !== JSON.stringify(points2) → rerouted
```

## Semantic Diff (Highest-Level Diff)

Convert structural + diagram differences into **human concepts**:

Examples:

- ⚡ "PaymentService now depends on FraudService"
- ⚡ "Order API has moved from 'checkout' boundary to 'payments' boundary"
- ⚡ "DB was removed and replaced with a queue"
- ⚡ "User journey UJ-10 changed sequence: Step 2 moved after Step 4"

Semantic diff engine groups:

```ts
interface SemanticChange {
  type: "dependency-added" | "dependency-removed" | "component-moved" | 
        "boundary-change" | "layer-change" | "requirement-impact" |
        "adr-impact" | "visual-change";
  details: any;
}
```

## Visual Diff UI (ReactFlow Overlays)

The most important part — users SEE changes directly.

### Overlay types:

- ✔ **Green = added**
- ✔ **Red = removed**
- ✔ **Yellow = changed properties**
- ✔ **Blue pulse = moved**
- ✔ **Purple = edge added/removed**
- ✔ **Grey highlight = re-routed edge**

### Node overlays:

```tsx
function DiffNodeOverlay({node}) {
  if (diff.added(node.id)) return <GreenGlow />;
  if (diff.removed(node.id)) return <RedCross />;
  if (diff.changed(node.id)) return <YellowBadge />;
  if (diff.moved(node.id)) return <BluePulse />;
}
```

### Edge overlays:

```tsx
<path style={{
   stroke: diff.isAddedEdge(id) ? "green"
        : diff.isRemovedEdge(id) ? "red"
        : diff.isChangedEdge(id) ? "purple"
        : "#aaa"
}} />
```

## Side-by-Side + Inline Diff Modes

### Mode A — Side-by-Side

Left: version A  
Right: version B  
Connected with highlight lines for moved nodes.

### Mode B — Inline diff (preferred)

Single diagram view:

- additions glow in
- removals appear as ghost nodes
- changes pulsate
- moved nodes animate from old → new position

**This is identical to Figma's diff animation pattern.**

## Diff Browser (UI Panel)

Contains:

- Summary
- Components added/removed
- Dependencies added/removed
- Boundary changes
- Layer changes
- Visualization differences
- ADR impact summary

## MCP API

```
diff.model(modelA, modelB)
diff.diagram(diagramA, diagramB)
diff.semantic(modelA, modelB)
diff.visual(modelA, modelB)
diff.summary(modelA, modelB)
diff.git(commitA, commitB)
```

## Strategic Value

The Visual Diff Engine provides:

- ✅ Clear change visualization
- ✅ Git integration
- ✅ Review workflow support
- ✅ Change impact analysis
- ✅ AI-friendly summaries

**This is critical for version control and change management.**

## Implementation Status

✅ Architecture designed  
✅ Diff algorithms specified  
✅ Visual overlays defined  
📋 Implementation in progress

---

*The Visual Diff Engine provides comprehensive diff capabilities for architecture models and diagrams.*


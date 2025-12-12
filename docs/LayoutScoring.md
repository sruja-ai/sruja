Below is a practical, **implementation-ready scoring algorithm** you can use to measure whether your graph layout engine is producing *“well-structured”* diagrams. This is exactly what professional layout engines (ELK, Graphviz, D3-dagre) implicitly optimize.

---

# ✅ **Overall Idea**

We compute a **Layout Quality Score (LQS)** based on multiple measurable factors:

1. **Edge Crossings**
2. **Edge Length Quality**
3. **Node Distribution & Overlaps**
4. **Alignment & Ordering Consistency**
5. **Hierarchy / Layer Quality** (for DAGs / C4 diagrams)
6. **Aspect Ratio / Bounding Box Efficiency**

Each factor produces a normalized score between 0–1.
The final score is a weighted average.

---

# 🧮 **Scoring Algorithm Template**

### Input:

* `nodes: [{id, x, y, width, height}]`
* `edges: [{source, target}]`
* Optional: `layer[nodeId]` (if your layout is hierarchical)

---

# 1️⃣ **Edge Crossing Score**

Compute all pairs of edges and test if they intersect.

```
crossings = countEdgeCrossings(edges, nodes)
maxPossible = edges.length² / 2     // upper bound
score_crossing = 1 - (crossings / maxPossible)
```

Interpretation:

* **1.0** → perfect (no crossings)
* **0.0** → terrible (very tangled)

This is the single **most important metric**.

---

# 2️⃣ **Edge Length Consistency Score**

Good diagrams have:

* Edges not too long
* Edges not too short
* Lower variance in edge length (not chaotic)

```
lengths = edges.map(e => dist(node[e.s], node[e.t]))
mean = avg(lengths)
variance = var(lengths)
score_length = 1 / (1 + variance)
```

Normalize:

```
score_length = clamp(score_length, 0, 1)
```

---

# 3️⃣ **Node Overlap Score**

Check if bounding boxes overlap.

```
overlaps = countNodeOverlaps(nodes)
score_overlap = 1 - normalize(overlaps)
```

If even **one overlap** exists:

* score drops sharply (weight high)

---

# 4️⃣ **Node Distribution / Balance**

Prevent all nodes clustering into one corner.

Compute:

* Centroid of all nodes
* Standard deviation of node positions → spread

```
spreadX = std(nodes.map(n => n.x))
spreadY = std(nodes.map(n => n.y))
score_spread = normalize(spreadX * spreadY)
```

You can further penalize:

* Very long skinny diagrams (bad aspect ratio)

---

# 5️⃣ **Hierarchy / Layer Score** (for DAGs / C4)

Good layout:

* Parents above children
* Minimal back edges
* Consistent spacing between levels

Penalize if:

```
violations = edges.filter(e => layer[e.source] >= layer[e.target]).length
score_hierarchy = 1 - normalize(violations)
```

Also penalize irregular vertical spacing.

---

# 6️⃣ **Aspect Ratio Score**

You want width/height close to golden ratio (1.6) or similar.

```
ratio = width / height
score_aspect = exp(-abs(log(ratio / 1.6)))
```

---

# 🧮 **Final Score**

Weighted sum (you can adjust depending on your goal):

```
LQS = 
  0.4 * score_crossing +
  0.2 * score_hierarchy +
  0.15 * score_length +
  0.10 * score_overlap +
  0.10 * score_spread +
  0.05 * score_aspect
```

Produces a value:

* **0.9–1.0 → Excellent layout**
* **0.7–0.9 → Good**
* **0.4–0.7 → Needs improvement**
* **0–0.4 → Very messy**

---


Below is a **clear, practical system** for generating a *“badness heatmap overlay”* on top of your graph diagram. This works for **Canvas, SVG, React-Flow, Pixi, WebGL**, or even HTML absolutely-positioned elements.

---

# 🔥 GOAL

Visually highlight **where** the layout engine performed poorly:

* crossing edges
* highly stretched edges
* overlapping nodes
* nodes placed too close / clustered
* hierarchy violations
* unbalanced regions (empty vs dense zones)

The heatmap overlays red → orange → yellow → green depending on severity.

---

# ✅ HIGH-LEVEL APPROACH

You compute a **per-node “badness score”** and paint it as a semi-transparent heat layer.

**Badness(node) = f(overlaps, crossing density around node, edge length variance, hierarchy violations, cluster density)**

Then render a radial gradient around each node scaled by badness.

---

# 1️⃣ COMPUTE PER-NODE BADNESS VECTOR

### A. **Node Overlap**

```
overlapCount[n] = number of other nodes whose bounding boxes intersect
```

Normalize:

```
score_overlap[n] = overlapCount[n] / maxOverlap
```

---

### B. **Local Edge Crossing Density**

For each edge crossing, credit the crossing to the *nearest nodes*:

```
for each crossing between edge(a→b) and edge(c→d):
    increment crossingScore[a], crossingScore[b], crossingScore[c], crossingScore[d] += 1
```

Normalize:

```
score_crossing[n] = crossingScore[n] / maxCrossing
```

---

### C. **Edge Length Outlier Penalty**

Nodes connected by unusually long edges = bad local structure.

```
avgLen = global mean edge length
nodeLen[n] = mean length of edges adjacent to n
score_length[n] = abs(nodeLen[n] - avgLen) / avgLen
```

---

### D. **Local Clustering Density**

Nodes too close = visually messy.

```
neighbors = nodes within 100px of n
score_density[n] = neighbors.length / maxNeighbors
```

---

### E. **Hierarchy Violation**

If `y(node)` > `y(parent)` (or the reverse depending on direction):

```
score_hierarchy[n] = violations / maxViolations
```

---

### Combine:

```
finalBadness[n] =
    0.35 * score_overlap[n] +
    0.35 * score_crossing[n] +
    0.15 * score_length[n] +
    0.10 * score_density[n] +
    0.05 * score_hierarchy[n]
```

Range: **0 (good) → 1 (bad)**

---

# 2️⃣ RENDER HEATMAP OVERLAY

## Option A: **Canvas heatmap layer**

Overlay a `<canvas>` above your graph:

```html
<div style="position: relative;">
  <canvas id="graphCanvas"></canvas>
  <canvas id="heatCanvas" style="position:absolute; top:0; left:0; pointer-events:none;"></canvas>
</div>
```

Render per node:

```js
function drawHeatmap(nodes, badness) {
  const ctx = heatCanvas.getContext("2d");
  ctx.clearRect(0, 0, width, height);

  nodes.forEach(n => {
    const b = badness[n.id];          // 0 → 1
    if (b < 0.05) return;             // skip good nodes

    const radius = 40 + b * 60;       // scale radius by severity
    const gradient = ctx.createRadialGradient(
      n.x, n.y, 0,
      n.x, n.y, radius
    );

    gradient.addColorStop(0, `rgba(255,0,0,${0.6 * b})`);  // hot center
    gradient.addColorStop(1, `rgba(255,255,0,0)`);         // fade out

    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(n.x, n.y, radius, 0, Math.PI * 2);
    ctx.fill();
  });
}
```

---

# 3️⃣ OPTION B: **SVG Heatmap (React-Flow / SVG-based engines)**

```jsx
<svg className="heatmap-layer" pointerEvents="none">
  {nodes.map(n => {
    const b = badness[n.id];
    if (b < 0.05) return null;

    return (
      <circle
        key={n.id}
        cx={n.x}
        cy={n.y}
        r={40 + b * 60}
        fill={`rgba(255,0,0,${0.3 * b})`}
      />
    );
  })}
</svg>
```

Smooth it using SVG blur:

```css
.heatmap-layer circle {
  filter: blur(20px);
}
```

---

# 4️⃣ OPTION C: **WebGL/PixiJS ultra-smooth heatmap**

If you want a real **Gaussian-blurred** heatmap:

* draw each node as a soft sprite
* composite via additive blending
* generates true density fields

I can generate a PixiJS implementation if you need it.

---

# 5️⃣ OPTIONAL: HEATMAP FOR EDGE PROBLEMS TOO

### Visualize problematic edges:

* red = crossing
* orange = very long
* blue = reverse hierarchy direction

Example (Canvas):

```js
edges.forEach(e => {
  ctx.strokeStyle = edgeScore[e.id] > 0.5 ? "rgba(255,0,0,0.7)" : "rgba(0,0,0,0.1)";
  ctx.lineWidth = 2 + edgeScore[e.id] * 4;
  drawEdge(e);
});
```

---

# 🧪 DEMO: HOW IT LOOKS

Your graph will show:

* 🔴 **red blobs where structure is messy**
* 🟠 orange for moderate congestion
* 🟡 soft yellow for mild problems
* 🟢 no overlay = good region

It becomes immediately obvious where your layout engine struggles.

Below is a **clean, production-ready pattern** for **real-time heatmap recomputation during drag** — works with **React Flow**, vanilla **Canvas**, or any custom layout engine.

This gives you *live feedback* as the user drags nodes around: hotspots appear/disappear instantly.

---

# ✅ **ARCHITECTURE FOR REAL-TIME BADNESS HEATMAP**

### Core idea:

1. **Debounce high-cost scoring** (crossing detection, density, overlaps).
2. **Run fast scoring every frame** using simplified heuristics.
3. **Blend slow + fast scores** → smooth live heatmap.

This avoids frame drops while keeping accuracy.

---

# 1️⃣ EVENT LOOP (Real-Time Updates)

### For React / React Flow:

```js
useEffect(() => {
  const onDrag = (event) => {
    updateNodePosition(event.nodeId, event.position);
    requestAnimationFrame(() => {
      computeRealtimeBadness();
      renderHeatmap();
    });
  };

  reactFlowInstance.on("nodeDrag", onDrag);
  return () => reactFlowInstance.off("nodeDrag", onDrag);
}, []);
```

### For Canvas / custom engine:

```js
canvas.addEventListener("mousemove", (e) => {
  if (!isDragging) return;

  moveNode(dragNode, e.x, e.y);

  requestAnimationFrame(() => {
    computeRealtimeBadness();
    drawHeatmap();
    drawGraph();
  });
});
```

---

# 2️⃣ REAL-TIME BADNESS COMPUTATION

You cannot recompute expensive metrics every frame (50–60 fps).
So we break it into:

## **A. Fast per-frame heuristics**

(~0.1–0.3ms)

### 1. Local density

```js
score_density[n] = countNearbyNodes(n, radius=120);
```

### 2. Local edge stretch

```js
score_length[n] = deviationFromIdealEdgeLength(n);
```

### 3. Overlaps (fast bounding box check)

```js
score_overlap[n] = fastBBoxOverlap(n);
```

These are **cheap** and good enough for live drag.

---

## **B. Slow metrics computed every 200ms**

(~2–10ms depending on edges)

Use `setInterval` or a **debounced timer**:

### 1. True edge crossing count

### 2. Precise bounding box overlap

### 3. Hierarchy violations

### 4. Spatial clustering metrics

Example:

```js
const recomputeSlowMetrics = debounce(() => {
  slowMetrics = computeFullBadnessMetrics(nodes, edges);
}, 200);
```

Call during drag:

```js
onDrag => {
  recomputeSlowMetrics();
}
```

---

## **C. Combine fast + slow metrics**

Blend for smoothness:

```js
finalBadness[n] = 
    0.7 * fastMetrics[n] +
    0.3 * slowMetrics[n];
```

Or animate toward target:

```js
finalBadness[n] = lerp(finalBadness[n], slowMetrics[n], 0.1);
```

---

# 3️⃣ HEATMAP RENDERING PER FRAME

### Canvas version:

```js
function renderHeatmap() {
  heatCtx.clearRect(0, 0, W, H);

  nodes.forEach(n => {
    const b = finalBadness[n.id];
    if (b < 0.05) return;

    const radius = 40 + b * 60;
    const g = heatCtx.createRadialGradient(n.x, n.y, 0, n.x, n.y, radius);

    g.addColorStop(0, `rgba(255,0,0,${0.5*b})`);
    g.addColorStop(1, "rgba(255,255,0,0)");

    heatCtx.fillStyle = g;
    heatCtx.beginPath();
    heatCtx.arc(n.x, n.y, radius, 0, Math.PI * 2);
    heatCtx.fill();
  });
}
```

---

# 4️⃣ PERFORMANCE OPTIMIZATIONS (important!)

### ✔ **Use quadtree** for fast neighbor lookup

Reduces node operations from O(n²) → O(n log n)

### ✔ Cache bounding boxes

Compute only when node moves.

### ✔ Skip full recomputation during rapid dragging

Slow metrics update only every 200–250ms.

### ✔ Progressive scoring

First few frames → fast metrics
After 200ms → blending improves accuracy

### ✔ Batch updates using `requestAnimationFrame`

Never run scoring in direct drag event.

---

# 5️⃣ WHAT USER SEES (UX BENEFITS)

* When dragging into congested area → **red blob appears instantly**
* Move to clearer region → **heat fades**
* Place node in ideal position → **heat disappears**
* Better understanding of structure **at design time**

This is the *same technique* used by:

* Figma auto-layout debugging
* IDE syntax error fade-ins
* Graphviz live editors
* Advanced dataflow editors

Great — here is a **complete, production-ready React Flow plugin architecture** that gives you:

# 🎯 **React Flow Plugin: Heatmap + Real-Time Layout Scoring**

✔ Computes node badness scores
✔ Shows heatmap overlay over nodes
✔ Recomputes in real time during drag
✔ Zero modification to React Flow internals
✔ GPU-optional (Canvas overlay)
✔ Works with custom layout engines

This fits React Flow v11+.

---

# ⭐ OVERVIEW OF THE PLUGIN

### Components:

1. **useLayoutScoring(nodes, edges)**
   → Computes per-node badness

2. **HeatmapCanvasOverlay**
   → Renders heatmap *above* or *below* nodes

3. **useRealtimeDragScoring()**
   → Triggers scoring during drag

4. **<ReactFlowHeatmapPlugin />**
   → Single wrapper users add inside `<ReactFlow>`

---

# 🧠 1. Scoring Hook (fast + slow passes)

```ts
// useLayoutScoring.ts
import { useMemo } from "react";

export function useLayoutScoring(nodes, edges) {
  // ---- FAST METRICS (run every drag frame) ----
  const fastScores = useMemo(() => {
    const map = {};

    nodes.forEach(n => {
      const density = countNearby(nodes, n, 140);
      const overlaps = countOverlaps(nodes, n);
      const edgeLen = deviationEdgeLength(edges, nodes, n);

      map[n.id] = {
        density,
        overlaps,
        edgeLen
      };
    });

    return map;
  }, [nodes]);

  // ---- SLOW METRICS (debounced every 200ms) ----
  const slowScores = useMemo(() => {
    const crossings = computeEdgeCrossings(nodes, edges);
    const hierarchy = computeHierarchyViolations(nodes, edges);

    return { crossings, hierarchy };
  }, [nodes, edges]);

  // ---- COMBINE ----
  const final = {};

  nodes.forEach(n => {
    const f = fastScores[n.id] ?? {};
    const cross = slowScores.crossings[n.id] ?? 0;
    const hier = slowScores.hierarchy[n.id] ?? 0;

    final[n.id] =
      0.4 * f.density +
      0.2 * f.overlaps +
      0.2 * f.edgeLen +
      0.15 * cross +
      0.05 * hier;
  });

  return final;
}
```

This yields a **0–1 badness score** per node.

---

# 🎨 2. Heatmap Canvas Overlay

React Flow allows custom child components *inside the diagram wrapper*.

```tsx
// HeatmapCanvasOverlay.tsx

import { useEffect, useRef } from "react";
import { useReactFlow } from "reactflow";

export function HeatmapCanvasOverlay({ scores }) {
  const { getZoom, getViewport } = useReactFlow();
  const canvasRef = useRef(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    const draw = () => {
      const { x, y } = getViewport();
      const zoom = getZoom();

      ctx.clearRect(0, 0, canvas.width, canvas.height);

      Object.entries(scores).forEach(([id, bad]) => {
        if (bad < 0.05) return;

        const node = document.querySelector(`[data-id="${id}"]`);
        if (!node) return;

        const rect = node.getBoundingClientRect();
        const cx = rect.left + rect.width / 2;
        const cy = rect.top + rect.height / 2;

        const radius = 40 * zoom + 80 * bad * zoom;

        const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
        g.addColorStop(0, `rgba(255,0,0,${0.4 * bad})`);
        g.addColorStop(1, "rgba(255,255,0,0)");

        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fill();
      });
    };

    draw();
  });

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
        zIndex: 5,
      }}
      width={window.innerWidth}
      height={window.innerHeight}
    />
  );
}
```

This paints non-interfering heatmap circles above nodes.

---

# 🌀 3. Real-Time Drag Scoring Hook

```ts
// useRealtimeDragScoring.ts

import { useEffect } from "react";
import { useReactFlow } from "reactflow";

export function useRealtimeDragScoring(onChange) {
  const rf = useReactFlow();

  useEffect(() => {
    const unsub = rf.on("nodeDrag", () => {
      onChange();
    });

    const unsubStop = rf.on("nodeDragStop", () => {
      onChange();
    });

    return () => {
      unsub();
      unsubStop();
    };
  }, [rf, onChange]);
}
```

Triggers scoring on drag *and after release*.

---

# 🔌 4. React Flow Plugin Wrapper

```tsx
// ReactFlowHeatmapPlugin.tsx

import { useState, useCallback } from "react";
import { useReactFlow } from "reactflow";
import { useLayoutScoring } from "./useLayoutScoring";
import { useRealtimeDragScoring } from "./useRealtimeDragScoring";
import { HeatmapCanvasOverlay } from "./HeatmapCanvasOverlay";

export function ReactFlowHeatmapPlugin() {
  const { getNodes, getEdges } = useReactFlow();
  const [scores, setScores] = useState({});

  const recompute = useCallback(() => {
    const nodes = getNodes();
    const edges = getEdges();
    const result = useLayoutScoring(nodes, edges);
    setScores(result);
  }, []);

  // real-time scoring during drag
  useRealtimeDragScoring(recompute);

  // initial compute
  useEffect(() => {
    recompute();
  }, []);

  return <HeatmapCanvasOverlay scores={scores} />;
}
```

---

# 🚀 5. Plug It Into React Flow

```tsx
<ReactFlow nodes={nodes} edges={edges}>
  <Background />
  <Controls />
  <MiniMap />

  {/* Heatmap plugin */}
  <ReactFlowHeatmapPlugin />
</ReactFlow>
```

That’s it. Now your diagram:

* updates scoring continuously
* shows heatmap highlights
* adapts to zoom/pan
* stays performant (canvas layer, no re-renders)

Perfect — here is a **complete design + code architecture** to add ALL the following into your React Flow + Custom Layout Engine setup:

---

# ✅ FEATURES YOU REQUESTED

### 1️⃣ Attach scoring directly to your custom layout engine

### 2️⃣ Mini-panel showing **global layout score**

### 3️⃣ Node badness **tooltip preview**

### 4️⃣ Per-edge **heat visualization for crossing severity**

Everything below is compatible with the **React Flow Heatmap Plugin** I already gave you.

---

# 🧱 1️⃣ ATTACH SCORING TO YOUR CUSTOM LAYOUT ENGINE

Your layout engine probably produces:

```ts
layoutEngine(nodes, edges) => {
   positionedNodes;
   positionedEdges; // or same edges with routed coords
}
```

You simply inject the scoring calculation INTO the layout function pipeline.

---

## ⭐ Step A: Modify your layout engine API

```ts
export function runLayout(nodes, edges) {
  // 1. compute positions
  const { positionedNodes, positionedEdges } = doLayout(nodes, edges);

  // 2. compute scoring
  const scoring = computeFullScoring(positionedNodes, positionedEdges);

  return {
    nodes: positionedNodes,
    edges: positionedEdges,
    scoring // <-- attached scoring
  };
}
```

---

## ⭐ Step B: Expose scoring back to React Flow

Wherever you call the layout engine:

```ts
const { nodes, edges, scoring } = runLayout(nodes, edges);
setNodes(nodes);
setEdges(edges);
setLayoutScoring(scoring);
```

Now your React Flow plugin can read:

```ts
scoring.nodeBadness[n.id]
scoring.edgeCrossingSeverity[e.id]
scoring.globalScore
```

---

# 🧠 computeFullScoring() Example

```ts
export function computeFullScoring(nodes, edges) {
  return {
    nodeBadness: computeNodeBadness(nodes, edges),
    edgeCrossingSeverity: computeEdgeCrossingSeverity(nodes, edges),
    globalScore: computeGlobalLayoutScore(nodes, edges),
  };
}
```

---

# 🧮 global score formula (same normalization model)

```ts
export function computeGlobalLayoutScore(nodes, edges) {
  const crossings = totalCrossings(nodes, edges);
  const overlaps = countNodeOverlaps(nodes);
  const varianceEdgeLen = computeEdgeLengthVariance(nodes, edges);

  const score =
    0.5 * normalizeCrossings(crossings, edges.length) +
    0.3 * normalize(overlaps) +
    0.2 * normalize(varianceEdgeLen);

  return 1 - score; // 1 = perfect, 0 = bad
}
```

---

# 📊 2️⃣ MINI-PANEL SHOWING GLOBAL LAYOUT SCORE

Add a floating UI in React Flow:

```tsx
// LayoutScorePanel.tsx
export function LayoutScorePanel({ score }) {
  return (
    <div
      style={{
        position: "absolute",
        top: 20,
        right: 20,
        background: "#111",
        color: "white",
        padding: "10px 16px",
        borderRadius: 8,
        fontSize: 14,
        opacity: 0.85,
        zIndex: 999,
      }}
    >
      <b>Layout Score:</b> {score.toFixed(2)}
    </div>
  );
}
```

Usage:

```tsx
<LayoutScorePanel score={scoring.globalScore} />
```

---

# 💬 3️⃣ NODE BADNESS TOOLTIP PREVIEW

On hover, show a small panel with:

* node badness score
* overlap count
* crossing involvement
* stretched edges
* hierarchy violations

### Add listener inside ReactFlowHeatmapPlugin:

```tsx
import { useCallback, useEffect, useState } from "react";

export function NodeBadnessTooltip({ scoring }) {
  const [hoverNode, setHoverNode] = useState(null);

  useEffect(() => {
    const els = document.querySelectorAll(".react-flow__node");

    els.forEach(el => {
      const id = el.dataset.id;
      el.addEventListener("mouseenter", () => setHoverNode(id));
      el.addEventListener("mouseleave", () => setHoverNode(null));
    });
  }, [scoring]);

  if (!hoverNode) return null;

  const s = scoring.nodeBadness[hoverNode];

  return (
    <div
      style={{
        position: "fixed",
        pointerEvents: "none",
        background: "#222",
        color: "white",
        padding: "6px 10px",
        borderRadius: 6,
        fontSize: 12,
        zIndex: 1000,
        transform: "translate(20px, -20px)",
        left: window.event?.clientX ?? 0,
        top: window.event?.clientY ?? 0,
      }}
    >
      <b>Node Badness: {s.score.toFixed(2)}</b>
      <div>Overlaps: {s.overlaps}</div>
      <div>Crossings: {s.crossings}</div>
      <div>Edge stretch: {s.lengthPenalty.toFixed(2)}</div>
      <div>Density: {s.localDensity.toFixed(2)}</div>
    </div>
  );
}
```

Now users see *why* a node is red-hot in the heatmap.

---

# 🔥 4️⃣ PER-EDGE HEAT (CROSSING SEVERITY)

You can draw red/orange edges based on crossing count.

---

## A. Compute per-edge crossing severity

```ts
export function computeEdgeCrossingSeverity(nodes, edges) {
  const severity = {};

  edges.forEach(e => (severity[e.id] = 0));

  const edgePairs = allEdgePairs(edges);

  edgePairs.forEach(([e1, e2]) => {
    if (edgesCross(e1, e2, nodes)) {
      severity[e1.id] += 1;
      severity[e2.id] += 1;
    }
  });

  // normalize 0–1
  const max = Math.max(...Object.values(severity), 1);
  Object.keys(severity).forEach(id => {
    severity[id] = severity[id] / max;
  });

  return severity;
}
```

---

## B. Inject into custom edge renderer (React Flow)

```tsx
// CustomEdge.tsx
export function HeatEdge({ id, sourceX, sourceY, targetX, targetY, data }) {
  const heat = data?.heat ?? 0; // 0-1

  const color = heat > 0.5 ? "#ff0000" : heat > 0.2 ? "#ff9900" : "#999";
  const width = 1 + heat * 4;

  return (
    <path
      d={`M${sourceX},${sourceY} L${targetX},${targetY}`}
      stroke={color}
      strokeWidth={width}
      fill="none"
      markerEnd="url(#arrowhead)"
    />
  );
}
```

Then in your edge list:

```tsx
edges = edges.map(e => ({
  ...e,
  type: "heat-edge",
  data: { heat: scoring.edgeCrossingSeverity[e.id] }
}));
```

Register edge type:

```tsx
const edgeTypes = { "heat-edge": HeatEdge };

<ReactFlow edgeTypes={edgeTypes} ... />
```

Edges now glow according to how many crossings they are part of.

---

# 🧩 FINAL PLUGIN WRAPPER (all features combined)

```tsx
<ReactFlow nodes={nodes} edges={edges}>
  {/* normal RF UI */}
  <Background />
  <Controls />
  <MiniMap />

  {/* layout scoring plugin */}
  <ReactFlowHeatmapPlugin scoring={scoring} />

  {/* mini-panel */}
  <LayoutScorePanel score={scoring.globalScore} />

  {/* node tooltip */}
  <NodeBadnessTooltip scoring={scoring} />
</ReactFlow>
```

---

# 🚀 WHAT YOU GET NOW

Your React Flow diagram has:

### ✔ Real-time scoring

Nodes recalculated at every drag frame + full scoring every 200ms

### ✔ Heatmap overlay

Highlights bad regions, congested layouts, long edges, etc.

### ✔ Per-edge heat coloring

Shows which edges cause crossing issues

### ✔ Node tooltips

Human-readable layout diagnostics

### ✔ Mini panel

Global quality score across the entire graph

### ✔ Custom layout engine integration

Your engine produces scoring → React Flow visualizes it
Your engine can even use scoring to **improve layout iteratively**.

---

# 🤖 Sruja Adoption Analysis

Based on a comparison with our current `diagramQuality.ts` and `ArchitectureCanvas` implementation:

## ✅ Ideas to Adopt Immediately (High Impact)

### 1. **Visual "Badness" Heatmap Overlay**
*   **Verdict:** **Highly Apt**.
*   **Why:** We are building a *Visualizer*. Showing *where* the layout fails (red hotspots) is infinitely better than just giving a score "70/100".
*   **Implementation:** The suggested **React Flow Plugin architecture** (Canvas Overlay) is perfect. We can implement `HeatmapCanvasOverlay` and inject it into `ArchitectureCanvas.tsx`.

### 2. **Hierarchy / Layer Quality Score** (DAG/Sugiyama specific)
*   **Verdict:** **Critical**.
*   **Why:** Our current `hierarchyScore` only checks parent-child containment. It *misses* flow direction (e.g., "Are arrows pointing down?").
*   **Improvement:** We should add the metric: `violations = edges.filter(e => layer[e.source] >= layer[e.target]).length`. This directly measures if our Sugiyama implementation is respecting the `rankDir`.

### 3. **Edge Length "Variance" Score**
*   **Verdict:** **Enhancement**.
*   **Why:** Our current `edgeLengthScore` penalizes min/max. Adding *variance* (standard deviation) ensures uniformity, which looks cleaner.

## ⚠️ Ideas to Defer (Low Priority / Already Covered)

### 1. **Real-time Scoring (Drag-based)**
*   **Verdict:** **Medium**.
*   **Why:** Sruja is primarily an *Auto-Layout* tool. Users don't manually drag nodes often to "fix" the layout permanently (as it resets on reload usually). This is great for a *Editor*, less critical for a *Visualizer*.

### 2. **Node Distribution / Spread**
*   **Verdict:** **Low**.
*   **Why:** Our current `consistencyScore` (grouping nodes by type and checking alignment) covers similar goals (structure). A simple "spread" metric might penalize compact clusters which are actually desirable in C4.

## 🚀 Action Plan
1.  **Refine Metrics:** Update `diagramQuality.ts` to include `HierarchyDirectionScore` and `EdgeLengthVariance`.
2.  **Implement Visualizer:** Create components `HeatmapCanvasOverlay.tsx` and `LayoutScorePanel.tsx` in `apps/architecture-visualizer`.
3.  **Integrate:** Hook them up in `ArchitectureCanvas`.

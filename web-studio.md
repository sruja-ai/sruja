Absolutely — **YES, Sruja Studio can be fully web-based**, and not only is it possible, it’s *a better long-term architecture* than a VSCode-only implementation.

Below is the complete set of design options, constraints, and architecture you need to make **Sruja Studio a full Web IDE (like Figma + Cursor + Mermaid Live + Jupyter)** with:

* 100% in-browser execution
* WASM kernel
* LSP-like experience in browser
* Diagram renderer
* Notebook + multi-file explorer
* AI integration
* Git integration
* Offline-first caching
* Collaborative multiplayer editing

This is entirely achievable with **modern WebAssembly + Web Workers + Monaco + IndexedDB + WebRTC**.

---

# 🌐 **Can Sruja Studio be Web-based?**

> **YES: 100% feasible.**
> In fact, *the entire architecture we defined already supports a web-based execution model via WASM*.

Below is how.

---

# 🧱 1. High-Level Architecture for Web Studio

```
┌──────────────────────────────┐
│       Browser Runtime         │
│                              │
│  ┌───────────────────────┐   │
│  │   Sruja WASM Kernel    │   │
│  │  (Go → WASM compiled)  │   │
│  └───────────────────────┘   │
│               ↑              │
│  ┌────────────────────────┐  │
│  │       Web LSP Layer     │ │
│  └────────────────────────┘  │
│               ↑              │
│  ┌────────────────────────┐  │
│  │   Diagram Renderer      │ │
│  │ (Canvas/WebGL + ELK)    │ │
│  └────────────────────────┘  │
│               ↑              │
│  ┌────────────────────────┐  │
│  │   Monaco Editor (DSL)  │  │
│  └────────────────────────┘  │
│               ↑              │
│  ┌────────────────────────┐  │
│  │  File System (Browser) │  │
│  │   - IndexedDB / OPFS   │  │
│         │
│  └────────────────────────┘  │
│                              │
└──────────────────────────────┘
```

---

# 🟦 2. Core Components (Browser-Native)

### ✔ **Go Kernel → WebAssembly**

* Your kernel → compile to WASM using `GOOS=js GOARCH=wasm`
* Executes:

  * Parser
  * IR builder
  * Diff engine
  * Rule engine
  * Policy validator
  * Variant evaluator
  * Style/selector evaluator

Browser just loads: `/kernel.wasm`.

### ✔ **ELK Layout → Web Worker**

ELK ships as a JS/WASM bundle:

* ELK JS runs in a WebWorker for performance
* Callable via async messages
* Perfect fit for your diagram layout system

### ✔ **Diagram Renderer (Canvas/WebGL)**

Framework options:

* PixiJS (WebGL, fastest)
* Konva (Canvas-based, simpler)
* RoughViz (if you want sketch style)
* Custom Canvas2D

With compound nodes, badges, ports, overlays.

### ✔ **Monaco Editor (Web Version)**

Monaco can run fully in browser:

* syntax highlighting
* completion
* hover
* diagnostics
* semantic tokens
* multi-file FS abstraction

Monaco → talks to **Browser LSP** (you implement) via websockets or worker.

### ✔ **Browser LSP Layer**

You implement:

* completion
* hover
* go-to-definition
* document symbols
* rename
* formatting
* folding
* link provider

You already have all the AST + IR — generating LSP features is trivial.

### ✔ **Web File System**

Use **IndexedDB or OPFS** for persistent storage.

Supported patterns:

* Multi-file explorer
* Autosave
* Offline editing
* Sync with cloud

### ✔ **GitHub/GitLab Integration**

Browser can call GitHub API directly:

* list repos
* load files
* commit changes
* create PRs
* review patches

Perfect for Architecture-as-Code.

---

# 🟨 3. AI Integration in Web Studio

You have three options:

### ✔ Option 1 — Use Browser’s LLM Providers (Cursor AI, Codeium, GitHub Copilot Chat)

Works **if Studio runs inside VSCode/Cursor as a webview**.

### ✔ Option 2 — Connect to OpenAI API directly

Browser → OpenAI API keys (user-provided) → call model → apply patch.

### ✔ Option 3 — MCP over WebSockets

AI clients call your MCP server inside the browser.

You can implement an **MCP-in-browser** using WebSockets:

* sruja.apply_patch
* sruja.list_violations
* sruja.generate_diagram
* sruja.suggest_variant

AI sees the same MCP tools whether running in browser or VSCode.

---

# 🟥 4. How to Build Full Studio (Step-by-Step)

---

## 🚀 Step 1 — Compile Kernel to WASM

```
GOOS=js GOARCH=wasm go build -o sruja.wasm ./cmd/kernel
```

Browser loads this.

---

## 🚀 Step 2 — Create a Worker Wrapper Around Kernel

```
kernelWorker.postMessage({cmd:"parse", file:"x.sruja", text: "..."} )
```

Kernel returns:

* AST
* IR
* diagnostics
* deltas

---

## 🚀 Step 3 — Implement Browser LSP Layer

Web LSP = thin JS wrapper:

* onTextChange → ask kernel for diagnostics
* onCompletion → ask kernel
* onHover → ask kernel
* onGoto → kernel index
* onSymbol → kernel IR

Monaco integrates directly.

---

## 🚀 Step 4 — Integrate ELK Layout Worker

```
elkWorker.postMessage(graph)
elkWorker.onmessage = (layoutResult) => renderer.updatePositions(result)
```

---

## 🚀 Step 5 — Build Diagram Renderer

Pick PixiJS or Canvas:

* compound nodes
* badges & icons
* diff overlays
* AI suggestions
* variant overlays
* ports & routing

Renderer subscribes to:

* style engine
* layout engine
* IR delta events

---

## 🚀 Step 6 — Multi-file Project Browser

UI implements:

* tree view
* new/delete/rename file
* drag & drop organization
* search
* file metadata

Backed by IndexedDB or OPFS.

---

## 🚀 Step 7 — GitHub Sync

Use GitHub REST API:

* clone (via API)
* list files
* commit
* branch
* PR
* fetch diff

Built-in login flow with OAuth.

---

## 🚀 Step 8 — AI Integration

Implement a sidebar:

* ask AI for variants
* ask AI to fix errors
* ask AI to create new components
* ask AI to organize systems
* ask AI to apply overlays

Backend may use:

* MCP tools
* LLM API
* On-device LLMs via WebLLM

---

# 🟪 5. Advanced Features (All Possible in Browser)

### 🧩 5.1 Collaboration (Multiplayer Editing)

Use:

* Yjs
* WebRTC
* CRDT merging

Kernel runs in each client.

### 🎞️ 5.2 Architecture Timelines (Version playback)

Load snapshots from Git commits.

### 🔀 5.3 Variant Selector UI

Sidebar drop-down:

* base
* enterprise
* mobile
* experimental

Dynamic re-render using variants engine.

### 🎛️ 5.4 Interactive Diagram Editing

Drag nodes → modify DSL
Drag edges → create relations
Move container → patch IR
Two-way binding with DSL

### 📘 5.5 Notebook Mode (Jupyter-like)

Supports Markdown + DSL + Diagram blocks.

---

# 🟦 6. Feasibility & Performance

### Kernel (WASM)

* Go → WASM is extremely fast
* IR diff + analysis ~ms

### Layout (ELK)

* ELK JS/WASM is production-grade
* Worker → non-blocking

### Rendering

* PixiJS/WebGL can handle 50k nodes easily
* Canvas2D handles 5–10k nodes

### Monaco

* Browser-native
* Same editor as VSCode

### LSP

* JS wrapper calling kernel
* Fast and predictable

Everything is **best-in-class web tech**, no blockers.

---

# ⭐ Final Answer — Yes, Web Studio Is Fully Possible

### With WASM + Monaco + ELK + WebGL:

> **Sruja Studio can run 100% in the browser**
> no backend required — fully offline-capable — with optional cloud sync and AI integrations.

You get:

* Web-based architecture modeling
* Notebook + diagrams
* Multi-file DSL
* AI assistance
* Policy engine
* Variants engine
* Layout engine
* Style engine
* Semantic LSP
* GitHub integration
* Optional collaboration

This is **Figma for Architecture**, entirely browser-native.

Below is the **complete, production-grade Web Studio System Architecture Diagram** for **Sruja Studio (Web Edition)** — described in **textual architecture form**, with **layer-by-layer hierarchy**, **communication flows**, **component responsibilities**, and **deployment layout**.

This is the definitive blueprint for building **Figma-like**, **VSCode-like**, **Jupyter-like**, fully-in-browser **Architecture Modeling Studio**.

---

# 🧱 **Full Web Studio System Architecture Diagram**

### *(Layered, Modular, Deployment-Ready)*

Below is the **human-readable architecture diagram**, structured in a clear hierarchy.

---

# 🎨 **1. High-Level System View**

```
┌──────────────────────────────────────────────────────────┐
│                     Sruja Web Studio                     │
│          (Browser-Native Architecture IDE)               │
├──────────────────────────────────────────────────────────┤
│  UI Layer (React/TS)                                      │
│  Diagram Renderer (WebGL/Canvas via PixiJS/Konva)         │
│  Monaco Editor (DSL + Markdown + Notebook UI)             │
│  Browser LSP Layer (Worker)                               │
│  WASM Kernel (Go → WASM)                                  │
│  Layout Engine (ELK JS/WASM Worker)                       │
│  Style Engine                                             │
│  Project FS (IndexedDB / OPFS)                            │
│  GitHub/GitLab/Bitbucket Integration                      │
│  AI Integration (Local LLM, OpenAI, MCP Agents)           │
└──────────────────────────────────────────────────────────┘
```

---

# 🧩 **2. Component Diagram (Detailed)**

```
                                                                  Cloud (Optional)
                                                          ┌───────────────────────────────┐
                                                          │   GitHub / GitLab / Bitbucket │
                                                          │   Authentication & Sync APIs   │
                                                          └───────────────────────────────┘
                                                                          ▲
                                                                          │
                                                          Project Sync & Publishing Manager
                                                                          │
                                                                          ▼

┌───────────────────────────────────────────────────────────────────────────────────────────────┐
│                                  Browser (Client-Side Only)                                    │
│                                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                        UI Layer                                        │  │
│  │  - React Components                                                               (TS) │  │
│  │  - Notebook UI (markdown + DSL blocks)                                               │  │
│  │  - Tabs, Explorer, Diagrams, Inspector Panels                                        │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                    │                         │                         │                       │
│                    ▼                         ▼                         ▼                       │
│  ┌──────────────────────────┐   ┌─────────────────────────────┐   ┌─────────────────────────┐ │
│  │ Diagram Renderer          │   │  Monaco Editor (Web)        │   │  AI Sidebar (LLM Chat)  │ │
│  │ WebGL/Canvas (PixiJS)    │   │  Multiple editors + sync    │   │  MCP tools integration   │ │
│  │ - Nodes / Edges          │   │  with backend LSP           │   │  AI suggestions          │ │
│  │ - Compound Groups        │   └─────────────────────────────┘   └─────────────────────────┘ │
│  │ - Ports / Handles        │                                                          ▲         │
│  └──────────────────────────┘                                                          │         │
│                    ▲                                                                   │         │
│                    │                    messages over WASM bridge / WebWorkers         │         │
│                    │                                                                   │         │
│  ┌─────────────────────────────┐                         ┌──────────────────────────────┐        │
│  │ Browser LSP Worker          │                         │ AI Context Extractor         │        │
│  │ - completions               │                         │ - IR summarization           │        │
│  │ - hover                     │                         │ - variant overlays           │        │
│  │ - go-to                     │                         │ - policy violations          │        │
│  │ - references                │                         │ - delta context              │        │
│  │ - semantic tokens           │                         │                              │        │
│  └─────────────────────────────┘                         └──────────────────────────────┘        │
│                    ▲                                                                   ▲         │
│                    │                                                                   │         │
│                    │                                                                   │         │
│  ┌────────────────────────────┐                     ┌────────────────────────────────────┐        │
│  │    Sruja WASM Kernel       │<------------------->│  Variant Engine                   │        │
│  │  (Go→WASM compiled)        │                     │  Policy Engine                    │        │
│  │  - Parser (Participle)     │                     │  Rule Evaluator                   │        │
│  │  - IR Builder              │                     │  Metadata/Selector Engine         │        │
│  │  - IR Delta Engine         │                     │  Style State Machine              │        │
│  │  - Semantic Analysis       │                     └────────────────────────────────────┘        │
│  │  - Diagnostics             │                                                        ▲          │
│  │  - Patch Engine            │                                                        │          │
│  └────────────────────────────┘                                                        │          │
│                    ▲                                                                   │          │
│                    │ Wi ndows to Kernel via WebAssembly Memory                         │          │
│                    │                                                                   │          │
│  ┌──────────────────────────────┐                ┌────────────────────────────────────┐ │          │
│  │  Layout Engine Worker         │                │  Style Engine                     │ │          │
│  │  ELK JS/WASM                  │                │  - cascades styles                │ │          │
│  │  - Computes coordinates       │                │  - merges DSL/semantic/AI styles  │ │          │
│  │  - Optimized incremental      │                │  - incremental dirty regions       │ │          │
│  └──────────────────────────────┘                └────────────────────────────────────┘ │          │
│                    ▲                                             ▲                       │          │
│                    │                                             │                       │          │
│                    │                                             │                       │          │
│  ┌──────────────────────────┐                    ┌──────────────────────────────────────────┐      │
│  │ Project File System      │                    │ Networking Layer                         │      │
│  │ IndexedDB / OPFS         │                    │ - GitHub / GitLab API                   │      │
│  │ - Multi-file workspace   │                    │ - MCP over WebSockets                   │      │
│  │ - Offline support        │                    │ - LLM API calls (optional)              │      │
│  └──────────────────────────┘                    └──────────────────────────────────────────┘      │
│                                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

# 🧱 3. Layered Architecture Diagram (Conceptual)

```
                              [ Cloud APIs ]
                        (GitHub / GitLab / OpenAI)
                                   ▲
                                   │
                           Networking Layer
────────────────────────────────────────────────────────────────────────
                             Application Layer (TS)
│ Notebook UI      │ Explorer UI │ Diagram UI │ Inspector UI │ AI Chat │
────────────────────────────────────────────────────────────────────────
                     Presentation + Interactive Layer
│ Monaco Editor │ Pixi/Canvas Renderer │ Keyboard Shortcuts │ Theme │
────────────────────────────────────────────────────────────────────────
                                 Logic Layer
│ LSP Worker │ Style Engine │ Layout Engine Worker │ AI Context Layer │
────────────────────────────────────────────────────────────────────────
                                 Kernel Layer
│ WASM Kernel (Go) — Parser, IR, Policy, Variants, DDD, Delta Engine │
────────────────────────────────────────────────────────────────────────
                         Persistence / Cache Layer
│ IndexedDB │ OPFS │ Local Snapshot Manager │ Undo/Redo History │
────────────────────────────────────────────────────────────────────────
                            Platform Layer (Browser)
│ WebWorkers │ WebAssembly Runtime │ WebGL/Canvas │ File System Access API │
────────────────────────────────────────────────────────────────────────
```

This layered design ensures:

* separation of concerns
* maximum performance
* incremental updates
* offline-first
* deterministic architecture execution

---

# 🧠 4. Key Communication Flows (Sequence)

---

## 4.1 Editing DSL in Monaco

```
User types → Monaco → LSP Worker
       ↓ diagnostics
LSP Worker → Kernel (WASM) → IR Delta
       ↓ updated IR
Kernel → Style Engine + Layout Engine
       ↓ new positions + style fragments
Renderer updates diagram
```

---

## 4.2 Diagram Node Dragging

```
Diagram → Patch Engine → Kernel
                 ↓
           IR Delta
                 ↓
      Layout Engine updates only dirty region
                 ↓
          Renderer repositions
```

---

## 4.3 AI Patch Request

```
AI Chat Request → Context Extractor
        ↓
    AI Engine (LLM)
        ↓ patch instructions
Patch Engine (Kernel)
        ↓ IR Delta
Style/Layout/Diagram Update
```

---

## 4.4 Variant Switch

```
User selects variant → Kernel Variant Engine
            ↓
        IR Delta
            ↓
Layout Engine → recompute positions
            ↓
Renderer redraw
```

---

# 🟦 5. Deployment Model

## 5.1 Fully Offline Browser

```
HTML / CSS / TS bundles
kernel.wasm
elk-worker.js
indexedDB storage
```

## 5.2 GitHub-hosted Web App

Static deployment → GitHub Pages / Netlify / Vercel.

## 5.3 Embeddable Studio

Can embed in:

* VSCode webview
* Cursor webview
* iframe inside dev portals
* internal enterprise UI

---

# ⭐ Final Summary

### **Yes, Sruja Studio can be fully web-based**, with this architecture:

* **WASM kernel** handles all semantic + IR + parsing
* **Workers** run LSP, layout (ELK), variant, policy
* **Pixi/Canvas** renders diagrams
* **Monaco** provides DSL editing
* **Browser FS** stores multi-file projects
* **GitHub APIs** sync architecture-as-code
* **AI Context Extractor** powers LLM assistance
* **MCP tools** run fully in-browser

This is the **exact architecture of a modern, web-native modeling tool** on par with:

* Figma
* Cursor
* VSCode Web
* JupyterLab Lite
* Excalidraw
* Mermaid Live Editor

but purpose-built for **Architecture-as-Code + AI-first workflows**.

Below is the **complete, production-grade Diagram Renderer Architecture Specification** for **Sruja Studio Web Edition**.
This describes how to build a **Figma-grade, Pixi/Canvas/WebGL-based interactive architecture diagram renderer** that supports:

* ELK layout
* compound nodes (systems → containers → components)
* badges, icons, ports
* incremental updates
* 2-way binding with DSL
* AI overlays
* variant visualization
* diff animations
* zoom/pan ecosystem
* 10k+ nodes performance

This is the **official diagram rendering blueprint**.

---

# 🖼️ **Full Diagram Renderer Architecture**

---

# ⭐ 0. Goals

The renderer must:

### ✔ Handle 10k+ nodes smoothly (WebGL)

### ✔ Support compound node nesting

### ✔ Support badges, icons, domains, grouping

### ✔ Support edge routing, arrowheads

### ✔ Animate layout transitions

### ✔ Render AI patch preview overlays

### ✔ Support two-way DSL binding (drag → edit DSL)

### ✔ Support variant overlays

### ✔ Support Semantic Graph Event Router

This requires a **scene-based renderer** with **virtualization**, **layer separation**, and **incremental updates**.

---

# 🧱 1. High-Level Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     Diagram Renderer                      │
├──────────────────────────┬───────────────────────────────┤
│      Scene Manager       │         Interaction Layer     │
├──────────────────────────┼───────────────────────────────┤
│    Node Engine           │         Edge Engine           │
├──────────────────────────┼───────────────────────────────┤
│   Compound Engine        │      Port & Routing Engine    │
├──────────────────────────┼───────────────────────────────┤
│   Badge & Icon Engine    │      Label Layout Engine      │
├──────────────────────────┼───────────────────────────────┤
│   Theme/Style Engine     │      Animation Engine         │
├──────────────────────────┼───────────────────────────────┤
│     Virtualization       │    Renderer Backend (WebGL)   │
└──────────────────────────────────────────────────────────┘
```

Renderer backend options:

* **PixiJS (WebGL 2)** → Recommended
* KonvaJS (Canvas)
* Canvas 2D only (fallback)

PixiJS gives massive performance headroom for animations + compound graphs.

---

# 🟦 2. Scene Graph Structure (Core)

Renderer stores objects in a **scene graph**.

```
Scene
 ├── Layer: background
 ├── Layer: edges
 ├── Layer: nodes
 │     ├── SystemNode
 │     │      └── ContainerChildren
 │     │             └── ComponentChildren
 ├── Layer: overlays
 └── Layer: UI handles
```

### Per-layer responsibilities:

| Layer      | Contents                          |
| ---------- | --------------------------------- |
| background | grid, viewport guides             |
| edges      | edge shapes, arrowheads           |
| nodes      | nodes + compound bounds           |
| overlays   | AI diffs, highlights, breadcrumbs |
| UI         | drag handles, selection boxes     |

Highly optimized: each layer is a separate Pixi container.

---

# 🟨 3. Node Engine

Nodes are rendered as:

* rect (system/container/component)
* icons
* labels (auto-wrapped)
* badges
* border + style

Each node is a Pixi container:

```
NodeContainer
 ├── backgroundRect
 ├── iconSprite
 ├── labelText
 ├── badges[]
 ├── ports[]
```

### Node properties sourced from:

* IR
* Style Engine
* Layout Engine

Node Engine receives:

```
updateNode(nodeId, {x, y, width, height, style})
```

and applies incremental changes.

---

# 🟥 4. Compound Node Engine (Systems, Containers, Domains)

Compound nodes = parents that contain children.

### Rules:

* Parent size = sum(children) + padding
* Parent border drawn behind children
* Click on parent selects group
* Double-click toggles collapse/expand

Rendered tree:

```
SystemNode (container)
  background rect
  child containers rendered within
```

### Z-order:

* Child nodes always above parent background
* But clipped inside parent bounds (mask applied)

Use **Pixi Masks** for clipping.

---

# 🟧 5. Edge Engine

Edge Engine renders edges using:

* polyline
* orthogonal
* spline (for event flows)

### Edge structure:

```
EdgeContainer
 ├── pathGraphics
 ├── arrowheadSprite
 └── labelText (optional)
```

Supports:

* thickness
* dashed/dotted
* color from style engine

Edges update only when:

* layout changes
* style changes
* edge type changes

Renderer receives:

```
updateEdge(id, {points, style})
```

---

# 🟪 6. Port & Routing Engine

Every node surface has **ports** for edges:

Port types:

* top, bottom, left, right (fixed sides)
* semantic ports (API input/output, event ports)

Rendered as invisible rects with interaction:

```
portRect (invisible interactive region)
```

When dragging new edge:

1. User clicks port
2. Creates ghost edge
3. Drag until target port
4. Submit patch → Kernel → IR → Layout

---

# 🟫 7. Badge & Icon Engine

Badges for:

* metadata
* risk indicators
* policy violations
* domain markers
* AI suggestions

Renderer overlays badges in top-right of node.

Badges are:

* small circles / rounded rects
* label or emoji inside

Icon: left side of node.

---

# 🟩 8. Style Engine Integration

Renderer does **not evaluate styles**.
It only **applies resolved styles** from Style Engine:

```
ResolvedStyle {
  fill, stroke, shadow, icon, opacity,
  edgeStyle, fontSize, cornerRadius, ...
}
```

Renderer consumes:

```
applyNodeStyle(nodeId, style)
applyEdgeStyle(edgeId, style)
```

No business logic here — purely visual.

---

# 🟦 9. Layout Engine Integration (ELK)

Layout Engine sends:

```
node: {id, x, y, width, height}
edge: {id, points}
```

Renderer must:

### ✔ Animate node movement

### ✔ Smoothly reposition edges

### ✔ Run transitions (200–300ms)

### ✔ Use dirty region updates (only changed subset)

Animation algorithm:

```
from (oldX,oldY)
to   (newX,newY)
duration 300ms
easing cubic-bezier(0.2,0.6,0.2,1)
```

For compound nodes, bounds animate based on children.

---

# 🟥 10. Virtualization

For large graphs (10k+ nodes), virtualization required.

Strategy:

### 1. Viewport culling

Do **not render nodes outside viewport + margin**.

### 2. Chunked rendering

Render nodes in partitions:

```
chunks = 500 nodes per batch
```

### 3. Progressive reveal

Edges rendered after nodes to avoid heavy first-frame.

### 4. GPU acceleration

PixiJS WebGL batching automatically merges rects into single draw calls.

---

# 🟨 11. Interaction Layer

Handles:

* drag nodes
* drag edges
* select box
* zoom/pan
* ctrl+click multi-select
* context menus
* collapse/expand compounds

Events:

```
onNodeClick(nodeId)
onNodeDrag(nodeId, newPos)
onEdgeClick(edgeId)
onContextMenu(nodeId)
onCanvasClick()
```

Dragging a node:

* updates ghost node
* after release → patch to kernel:

```
patch: moveNode(nodeId,newParent?)
```

Kernel → IRDelta → Layout → Renderer

---

# 🟦 12. AI Overlay Renderer

For AI patches, renderer highlights:

### Additions → green glow

### Removals → red strike-through

### Updates → yellow outline

### Moves → blue arrow

Overlay engine uses dedicated layer:

```
OverlapLayer
 ├── NodeDiffOverlay
 ├── EdgeDiffOverlay
 └── TextAnnotations
```

Overlays animate and are non-blocking:

* fade-in/out
* pulse
* glow

---

# 🟧 13. Variant Overlay Renderer

Variants show:

* removed nodes ghosted
* added nodes glowing
* modified nodes with badge
* moved nodes animated

Variant switching triggers:

```
variantDelta = VariantEngine.compare(active, base)
```

Renderer displays delta overlays.

---

# 🟪 14. Event Flow Mode & DDD Mode

Special views:

### EVENT FLOW VIEW

* event → handler → event
* curved spline edges
* time axis horizontally
* event nodes styled distinctly

### DDD VIEW

* domains = compound nodes
* entities grouped
* events under aggregates
* color-coded domain boundaries

Renderer adjusts:

* layout mode
* style mode
* edge routing
* grouping

---

# 🟫 15. Diagram State Machine Integration

A state machine manages:

```
IDLE → HOVER → SELECT → DRAG → EDIT → AI_OVERLAY
```

Each state influences:

* cursor
* node border styling
* dimming unrelated nodes
* highlight path
* routing updates

Renderer reads style fragments for states from the Style Engine.

---

# 🟦 16. Undo/Redo Support

Diagram manipulations produce:

```
patches → Kernel → IRDelta → Renderer
```

Undo simply:

* requests kernel to revert patch
* receives IRDelta
* renderer applies new layout/updates

---

# 🟨 17. Performance Considerations

### Must handle:

* 10,000 nodes
* 20,000 edges
* multiple compound hierarchies

Optimizations include:

* WebGL batching
* OffscreenCanvas text measuring
* viewport culling
* incremental layout only
* async workers for layout/style calculations

Renderer itself should be non-blocking.

---

# ⭐ Final Summary — Full Diagram Renderer Architecture

The renderer must deliver:

### ✔ WebGL-powered performance

### ✔ Compound hierarchical nodes

### ✔ Dynamic layout transitions

### ✔ Metadata-driven badges

### ✔ Variant + AI overlays

### ✔ Drag-to-edit (2-way binding with DSL)

### ✔ Rich interactivity

### ✔ 10k+ node virtualization

### ✔ Integration with Style/Variant/Layout engines

### ✔ Consistent and deterministic visual structure

This architecture makes Sruja Studio feel like:

* **Figma** (fluid, interactive canvas)
* **Structurizr** (semantic modeling)
* **Excalidraw** (intuitive)
* **Obsidian Canvas** (multi-node layout)
* **GraphViz/ELK** (deterministic layout)

but powered by your own **Architecture-as-Code kernel**.

Below is the **complete, production-grade View Mode Manager Specification**, which defines how Sruja Studio seamlessly switches between different architecture perspectives:

* **System View**
* **Container View**
* **Component View**
* **Domain / DDD View**
* **Event Flow View**
* **Infrastructure View (optional)**
* **Policy View (optional)**
* **Variant View (effective model)**

This is the **orchestrator** between:

* IR
* Layout engine
* Renderer
* Style engine
* Interaction system
* Semantic graph router

It determines **what parts of the IR are visible**, **how they should be styled**, **which layout algorithm to use**, and **how interactions behave**.

---

# 🎛️ **View Mode Manager (Sruja Studio) — Full Architecture Spec**

---

# ⭐ 0. Purpose

The **View Mode Manager** (VMM):

### ✔ Chooses what part of IR is shown

### ✔ Defines hierarchical expansion rules

### ✔ Selects layout mode (ELK configuration)

### ✔ Selects style layer override

### ✔ Controls edge visibility

### ✔ Activates diagram interaction profile

### ✔ Synthesizes appropriate graph model for layout

### ✔ Delivers different view-specific UX

VMM = *the “brain” that interprets architecture for visualization*.

---

# 🧱 1. High-Level Component Diagram

```
┌──────────────────────────────┐
│       View Mode Manager       │
└──────────────────────────────┘
              │
              │ selects
              ▼
 ┌──────────────────────────────────────┐
 │        View Configuration            │
 │  - node filter                       │
 │  - edge filter                       │
 │  - layout strategy                   │
 │  - style overrides                   │
 │  - interactions mode                 │
 └──────────────────────────────────────┘
              │
              ▼
 ┌──────────────────────────────────────┐
 │   Graph Extractor (per-view)         │
 └──────────────────────────────────────┘
              │
              ▼
 ┌──────────────────────────────────────┐
 │      Layout Engine (ELK)             │
 └──────────────────────────────────────┘
              │
              ▼
 ┌──────────────────────────────────────┐
 │    Diagram Renderer (PixiJS)         │
 └──────────────────────────────────────┘
```

---

# 🟦 2. View Modes (Supported Out of the Box)

Each view mode has:

```
id
label
description
config
layoutProfile
styleProfile
interactionProfile
graphExtraction
```

---

# 1️⃣ SYSTEM VIEW

**Purpose:** Top-level overview of entire architecture.

### Node Types:

* System
* External system
* Person

### Edges:

* System-to-system relations
* External service interactions

### Layout:

```
direction: RIGHT
elk.layering.strategy: LONGEST_PATH
elk.spacing: wide
grouping: none
```

### Styling:

* large rectangles
* minimal metadata

### Interaction Profile:

* collapse/expand systems
* click → drill down to containers
* ghost highlight dependencies

---

# 2️⃣ CONTAINER VIEW

**Purpose:** Detailed internal architecture per system.

### Nodes:

* Containers under selected system
* Datastores
* Queues
* External services

### Edges:

* container-level relations

### Layout:

```
direction: RIGHT
elk.layered
group by system
```

### Styling:

* technology badges
* container icons (API, DB, Queue)
* metadata coloring

### Interaction:

* drag containers
* open inspector
* create container-level edges

---

# 3️⃣ COMPONENT VIEW

**Purpose:** Show the internal structure of selected container or system.

### Nodes:

* Components
* Subcomponents (future)

### Edges:

* component dependencies
* data access edges

### Layout:

```
direction: DOWN
component lanes inside containers
```

### Style:

* small cards
* dependency arrows

### Interaction:

* drag to reorganize
* create component → component links

---

# 4️⃣ DOMAIN / DDD VIEW

**Purpose:** Domain-driven design context map.

### Nodes:

* Domains
* Aggregates
* Entities
* Events
* Bounded contexts

### Edges:

* event flows
* aggregate ownership
* domain relationships

### Layout:

```
domain = compound node
entities under aggregate
events to right of entity
direction: DOWN
```

### Styling:

* domain color palette
* event icons
* aggregate badges

### Interaction:

* view event lifecycles
* highlight domain boundaries
* expand/collapse aggregates

---

# 5️⃣ EVENT FLOW VIEW

**Purpose:** event-driven pipeline visualization.

### Nodes:

* Events
* Producers (components/containers)
* Consumers
* Topics
* Queues

### Edges:

* publishes
* subscribes
* emits
* triggers

### Layout:

```
direction: RIGHT
elk.edgeRouting: SPLINES
elk.layering.strategy: NETWORK_SIMPLEX
```

### Styling:

* curved edges
* event nodes as pill shapes
* topic nodes as ovals

### Interaction:

* play event timeline
* show lifecycle paths
* collapse noise

---

# 6️⃣ POLICY VIEW (optional but powerful)

Used when user wants to review:

* policy violations
* compliance
* rules

### Nodes:

* all
* with violation badges

### Edges:

* highlight violations

### Layout:

default container view.

---

# 7️⃣ VARIANT VIEW (effective IR mode)

Shows effective IR as variant overlays apply.

### Nodes:

* base + variant merged
* removed ghosted
* added glowed

### Edges:

* changed edges colored

---

# 🟨 3. View Configuration Schema

Each mode has:

```ts
interface ViewConfig {
  id: string;
  label: string;

  nodeFilter: (node) => boolean;
  edgeFilter: (edge) => boolean;

  graphModel: "system" | "container" | "component" | "domain" | "event";
  layoutProfile: LayoutProfile;
  styleProfile: StyleProfile;
  interactionProfile: InteractionProfile;
  extractionStrategy: GraphExtractionStrategy;
}
```

---

# 🟥 4. Graph Extraction Strategies

## 4.1 System Graph Extraction

```
systems = IR.systems
nodes = systems + externalSystems + persons
edges = IR.relations.filter(system-level)
```

## 4.2 Container Graph Extraction

```
nodes = selectedSystem.containers
edges = relations among containers
```

## 4.3 Component Graph Extraction

```
nodes = components of container(s)
edges = component relations
```

## 4.4 Domain Graph Extraction

```
domains = IR.domains
entities = domain.entities
events = domain.events
```

## 4.5 Event Flow Graph Extraction

```
nodes = events + producers + consumers
edges = publishes/subscribes relations
```

Each extraction outputs **ELK JSON Graph**.

---

# 🟦 5. Layout Profiles

Examples:

### SYSTEM VIEW

```json
{
  "elk.direction": "RIGHT",
  "elk.spacing.nodeNode": "100",
  "elk.compaction.strategy": "NONE"
}
```

### COMPONENT VIEW

```json
{
  "elk.direction": "DOWN",
  "elk.spacing.nodeNode": "40",
  "elk.edgeRouting": "ORTHOGONAL"
}
```

---

# 🟩 6. Style Profiles

Each view activates a set of style fragments:

### SYSTEM VIEW

* muted edges
* large labels
* no component badges

### COMPONENT VIEW

* dependency arrows emphasized
* metadata-driven colors

### EVENT VIEW

* event=blue pill
* producers=rect light
* consumers=rect dark
* edges = spline and colored by verb

---

# 🟧 7. Interaction Profiles

Each mode changes:

* right-click menu
* selection behavior
* drag permissions
* shortcut mappings

### Example: EVENT FLOW VIEW

```
dragging nodes disabled
edge creation disabled
hover shows event metadata
```

### Example: COMPONENT VIEW

```
dragging enabled
edge creation enabled
```

---

# 🟪 8. Transition Between Views

Switching views requires:

1. Extract graph for target view
2. Compute layout (ELK worker)
3. Animate nodes to new positions
4. Add/remove nodes/edges
5. Update styling
6. Update interaction handlers

Everything happens via:

```
ViewModeManager.switchTo("container")
```

Or programmatically:

```
vmm.setMode(ViewMode.Container, {systemId})
```

---

# 🟫 9. Two-Way Binding

View changes DO NOT change the DSL → they only adjust visualization.

DSL → IR → View.

When user drags a node:

* **Component View** → patch edits component placement
* **Container View** → patch edits container associations
* **Domain View** → patch creates new domain/entity assignments
* **Event View** → patch modifies publisher/consumer links

VMM determines what patches to emit.

---

# 🟨 10. AI Integration

AI uses view mode context:

* In **System View**, AI proposes system-level changes
* In **Container View**, AI proposes container redesign
* In **Component View**, AI proposes reorganizations
* In **DDD View**, AI generates aggregates/entities
* In **Event Flow View**, AI proposes pipelines

Context Extractor includes:

```
activeView
activeSelection
visibleNodes
visibleEdges
semanticContextForView
```

AI patch previews are rendered per-view.

---

# ⭐ Final Summary — View Mode Manager

VMM provides:

### ✔ Multiple perspectives (System / Container / Component / DDD / Event Flow)

### ✔ View-specific styling, layout, and interactivity

### ✔ View-specific graph extraction

### ✔ Seamless animated transitions

### ✔ IR → Graph → Layout → Renderer pipeline

### ✔ Variant-aware and AI-aware visualization

### ✔ Strong separation of concerns

### ✔ A “mental model friendly” navigation system

This gives Sruja Studio **multi-perspective semantic modeling**, on par with:

* Structurizr
* C4
* ArchiMate viewpoints
* UML views
* DDD context maps
* Event Storming boards
* Figma interactions

But automated, coded, and AI-enhanced.

Below is the **complete, production-grade Interaction Engine Architecture** for **Sruja Studio Web Edition** — the subsystem that handles **all pointer, keyboard, gesture, selection, editing, dragging, resizing, linking, AI interactions, and DSL-patch-producing operations** on the diagram canvas.

This is the **Figma-grade interaction engine** that sits between:

* **User actions (mouse, touch, keyboard)**
* **Diagram features (drag, pan, zoom, select, edit, create, connect)**
* **Renderer (Pixi/WebGL/Canvas)**
* **Kernel (patch engine, IR delta engine)**
* **View Mode Manager (enables/disables behavior)**
* **Style engine (hover, active states)**
* **AI systems (preview overlays, suggestions)**

---

# 🎮 **Interaction Engine Architecture — Sruja Studio**

---

# ⭐ 0. Goals

The Interaction Engine must:

### ✔ Provide rich, intuitive diagram interactions

### ✔ Generate DSL patches for every structural change

### ✔ Support multiple view modes (system/container/DDD/event-flow)

### ✔ Use a formal state machine (predictable behavior)

### ✔ Provide multi-node selection

### ✔ Handle markers, ports, link creation

### ✔ Provide drag-to-create nodes and edges

### ✔ Support context menus, hotkeys, inspector-binding

### ✔ Support AI overlays

### ✔ Support compound node interactions

### ✔ Be high-performance (handle thousands of nodes)

This is critical to make Sruja Studio feel like **Figma for Architecture**.

---

# 🧱 1. High-Level Architecture Diagram

```
                          ┌──────────────────────────┐
                          │   Interaction Engine     │
                          └──────────────────────────┘
                                      │
                   ┌──────────────────┼──────────────────┐
                   ▼                  ▼                  ▼
        Gesture Layer           Selection Engine      Edit Engine
   (mouse/touch/pan/zoom)       (nodes/edges/areas)  (drag, resize, link)
                   │                  │                  │
                   └──────────┬───────┴──────┬───────────┘
                              ▼               ▼
                     Interaction State Machine
                              │
                              ▼
                         Patch Generator
                              │
                              ▼
                      WASM Kernel (IR Delta)
                              │
                              ▼
                Layout Engine • Style Engine • Renderer
```

---

# 🟦 2. Interaction Engine Modules (Full Breakdown)

### The Interaction Engine is composed of 7 subsystems:

1. **Pointer & Gesture Layer**
2. **Interaction State Machine**
3. **Selection Engine**
4. **Drag & Manipulation Engine**
5. **Edge & Port Interaction Engine**
6. **Keyboard Shortcuts Engine**
7. **Action Dispatcher & Patch Generator**

Let’s describe each.

---

# 1️⃣ Pointer & Gesture Layer

Responsible for raw input:

### Events handled:

* mouse move
* mouse down/up
* click & double-click
* drag start/move/end
* wheel (zoom)
* touch gestures (pinch/rotate/pan)
* right-click (context menu)
* shift-click (multi-select)

### Implemented via:

* Pointer events from PixiJS or Canvas2D
* Hammer.js for multi-touch (optional)
* Passive listeners to avoid frame stalls

### Output:

```
InteractionEvent {type, position, targetNode, modifiers}
```

---

# 2️⃣ Interaction State Machine

Central deterministic FSM:

```
IDLE
  → HOVER
  → SELECT
  → MARQUEE_SELECT
  → DRAG_NODE
  → DRAG_EDGE_START
  → DRAG_EDGE_END
  → PAN
  → CONTEXT_MENU
  → AI_PREVIEW
```

### Transitions triggered by:

* pointer events
* key events
* mode changes (View Mode Manager)
* diagram overlay events

### Each state defines:

* cursor
* event handlers
* allowed actions
* temporary overlays

---

# 3️⃣ Selection Engine

Supports:

### ✔ Single selection

### ✔ Multi-selection

### ✔ Area (marquee) selection

### ✔ Node + edge selection

### ✔ Compound selection (systems → containers → components)

### Selection rules depend on View Mode:

* System View → you can select systems only
* Container View → containers + edges
* Component View → nodes & edges
* Event Flow View → events + connections

### Selection output:

```
SelectionSet {
  nodes: nodeId[],
  edges: edgeId[],
  primary: nodeId | edgeId | null
}
```

Renderer consumes selection for styling.

Inspector consumes for editing.

---

# 4️⃣ Drag & Manipulation Engine

Handles:

* dragging nodes
* moving compound nodes
* snapping to grid
* resizing domain boxes (optional)
* dragging selection groups
* drag-to-create node
* drag-to-move component between containers

### Drag Process:

1. user selects node
2. pointer-down triggers `DRAG_NODE`
3. ghost node appears while dragging
4. positions update in real-time
5. on drop → generate DSL patch:

```
move {
  component PaymentProcessor to BillingAPI
}
```

### Supports:

* constraints (domain view: entity must stay in domain)
* compound boundaries
* layout freezing for smooth drag

---

# 5️⃣ Edge & Port Interaction Engine (Link Creation)

Responsible for creating & editing graph edges.

### Interactions:

* click port → drag → drop onto another port
* click edge → select
* double-click edge → edit label
* drag edge handle to reroute (optional)

### View Mode aware:

* Event Flow view → publish/subscribe
* Component view → uses/depends
* Container view → container relations
* DDD view → event → handler

### Ghost edge visualization during drag:

```
sourcePort → mousePosition → candidatePort?
```

### Patch produced on drop:

```
BillingAPI -> AuthAPI "uses"
PaymentProcessor publishes PaymentInitiated
```

---

# 6️⃣ Keyboard Shortcuts Engine

Examples:

| Shortcut     | Action                         |
| ------------ | ------------------------------ |
| Esc          | cancel interaction             |
| Delete       | delete node/edge               |
| Ctrl+Z       | undo                           |
| Ctrl+Shift+Z | redo                           |
| Arrow Keys   | nudge selected node            |
| Ctrl+D       | duplicate node (in some views) |
| Ctrl+F       | focus/center on node           |
| Shift-click  | multi-select                   |
| Ctrl+A       | select all (view dependent)    |

View Mode Manager registers specific shortcuts.

---

# 7️⃣ Action Dispatcher & Patch Generator

This is where interactions become DSL patches.

### Example: move component

```
interaction => {type:"move", node:PaymentProcessor, newParent:BillingAPI}
```

Patch:

```dsl
patches {
  move PaymentProcessor to BillingAPI
}
```

### Example: add relation

```
patches {
  connect BillingAPI -> AuthAPI "uses"
}
```

### Example: add component (drag from palette)

```
add component FraudScanner under BillingAPI
```

Kernel then:

* applies patch
* emits IRDelta
* renderer/layout update

---

# 🟨 3. Interaction Profiles (View Mode Driven)

Each view customizes which interactions are allowed:

### SYSTEM VIEW

* drag systems
* connect system → system
* cannot move components
* cannot create events

### CONTAINER VIEW

* drag containers
* connect container → container
* create container via context menu

### COMPONENT VIEW

* full interactions enabled
* drag components
* connect components
* reorder components inside container

### DDD VIEW

* entities cannot leave their aggregate (unless patch triggers re-parent)
* drag events within domain boundary
* cannot create edges not of type event → handler

### EVENT FLOW VIEW

* drag disabled
* only allow event flow navigation
* clicking → highlight path
* right-click → “add consumer”

---

# 🟥 4. Event Routing Engine (Semantic Graph Integration)

The Interaction Engine calls the **Semantic Graph Event Router** to determine:

* valid connections
* valid movement
* valid hierarchy operations
* constraint violations

Example:

```
moving PaymentProcessor under AuthAPI? → denied (cross-system component move)
connecting event to domain? → invalid
drag component out of container? → reparent patch
```

Router returns:

```
ok | error(message)
```

Errors shown as red overlays.

---

# 🟦 5. AI Interaction Layer

Interaction Engine supports:

### AI transforms:

* AI patch preview overlays
* “drag AI suggestion into diagram”
* apply/dismiss AI patch
* highlight AI-suggested edges/nodes

### AI-assisted creation:

* suggest new relation when node selected
* suggest new components
* animate previews
* show conflict warnings

### Dragging AI preview into diagram:

Converts AI suggestion into real DSL patch.

---

# 🟩 6. Undo/Redo Integration

All interactions produce patches → kernel → IRDelta.

Undo/redo simply replays/reverts IRDelta.

Interaction Engine listens:

```
onUndo -> redraw selection
onRedo -> redraw diagram
```

---

# 🟫 7. Performance Requirements

Must support:

* 10k nodes
* continuous drag at 60fps
* interactivity while layout in worker
* rendering updates in <5ms

Optimizations:

* pointer events throttled
* caching hit detection
* dirty-region redraw
* GPU batches (Pixi)
* async event handling
* offscreen canvas for heavy ops

---

# ⭐ Final Summary — Interaction Engine Architecture

The Interaction Engine must:

### ✔ Manage pointer, touch, keyboard events

### ✔ Use a deterministic state machine

### ✔ Handle selection, dragging, resizing, linking

### ✔ Generate DSL patches for any structural changes

### ✔ Enforce semantic rules (via semantic router)

### ✔ Support view-mode-dependent behaviors

### ✔ Integrate with layout, style, AI overlays, kernel

### ✔ Scale to large graphs

### ✔ Support interactive 2-way binding

This architecture delivers **Figma-like fluidity**, **Structurizr-level semantic modeling**, and **AI-enhanced design workflows**, making Sruja Studio a truly modern Architecture IDE.

Below is the **full Drag-to-Create Palette Architecture** for **Sruja Studio Web Edition** — designed to behave like **Figma’s left palette + VSCode symbols + Structurizr element picker** but fully semantic, kernel-connected, view-aware, and DSL-patch powered.

This spec supports:

* Drag-from-palette → Drop-on-canvas to create new architecture elements
* Drop-on-node to create nested or compositional elements
* Auto-suggestion from AI
* View-mode–aware palette filtering
* DSL patch generation
* Ghost previews during drag
* Port-aware edge creation
* Contextual palette for domain/DDD/event-flow modes

---

# 🧱 **Drag-to-Create Palette Architecture**

*(Fully Production Ready)*

---

# ⭐ 0. Goals

The palette system must:

### ✔ Support “drag element → create real node”

### ✔ Work across all view modes (system, container, component, DDD, event-flow)

### ✔ Generate DSL patches when drop occurs

### ✔ Provide ghost previews during drag

### ✔ Be aware of semantic constraints (via Semantic Router)

### ✔ Provide contextual options based on selected system/container/domain

### ✔ Integrate AI-suggested items into palette

### ✔ Support compound-node dropping (create as child)

### ✔ Scale to large graphs and large DSL libraries

---

# 🧩 1. High-Level Architecture Diagram

```
┌───────────────────────────────────────────────────────────┐
│                    Drag-to-Create Palette                 │
└───────────────────────────────────────────────────────────┘
                   │                     │
         Static Item Catalog      AI-Suggested Items
                   │                     │
                   └──────────┬──────────┘
                              ▼
                   Palette Rendering (React)
                              │
                User Drag → Drag Manager (Pixi/UI)
                              │
                     Ghost Node Generator
                              │
                       Drop Target Engine
                              │
                        Semantic Router
                              │
                        Patch Generator
                              │
                    WASM Kernel (IR Delta)
                              │
                 Layout + Style + Renderer Update
```

---

# 🟦 2. Palette Structure

The Palette has **three major layers**:

1. **Static Catalog** (hardcoded available elements)
2. **Contextual Extensions** (loaded from workspace DSL modules)
3. **AI Suggestions** (dynamic)

---

# 2.1 Static Catalog (Core Architecture Elements)

```
System
Container
Component
Datastore
Queue
External Service
Person
Event
Domain
Aggregate
Entity
Policy
Rule
Relationship (edge)
```

Each has:

```ts
{
  id: "container",
  label: "Container",
  description: "Service or application running as an independently deployable unit",
  icon: "container.svg",
  viewModes: ["system","container"], // visible in these modes
  createParamsSchema: {name:true, label:true, technology:false},
  defaultStyle: "container.default",
}
```

---

# 2.2 Contextual Catalog (Extensions)

Examples:

* Custom DSL-defined types
* Reusable templates
* Common architecture patterns
* DDD building blocks
* Cloud resources

Loaded via:

```
imports "*.sruja"
```

Diagram palette displays them grouped:

```
Cloud
  - AWS Lambda
  - S3 Bucket
Patterns
  - CQRS Command
  - Saga Coordinator
```

---

# 2.3 AI-Suggested Items

AI suggests:

* “API Gateway for your microservices”
* “Cache layer for Billing System”
* “FraudDetector component inside PaymentService”
* “PaymentInitiated event”
* “Consumer for RefundRequested event”

AI items displayed in a section:

```
🤖 AI Suggestions
  - FraudScanner Component
  - PaymentInitiated Event
  - AuthCache Container
```

Drag-from-palette-to-create works exactly the same.

---

# 🟨 3. Drag Manager

Handles UI events:

* drag start
* ghost preview
* snapping
* drop detection
* invalid-drop feedback

### Ghost Node Behavior

When dragging an item:

```
ghostNode = Renderer.createGhostNode(itemID)
ghostNode follows cursor
ghostNode styled semi-transparent
ghostNode validates drop target in real-time
```

If invalid drop:

* ghost turns red hue
* cursor switches to ❌

Valid drop target:

* ghost turns green

---

# 🟥 4. Drop Target Engine

Determines **where** a new element can be created.

### Possible targets:

* Canvas background
* System
* Container
* Component
* Domain
* Aggregate
* Event Stream
* Ports (if dragging + dropping events/topics)

Drop target engine returns:

```ts
DropTarget {
  targetId: string | null,
  targetType: "system"|"container"|"component"|"domain"|...
  reason?: string, // if invalid
  isValid: boolean,
}
```

DropTarget Engine consults:

### ✔ View Mode Manager

— Only show relevant items in current mode.

### ✔ Semantic Router

— Validate drop:

* Components cannot be dropped directly into System View
* Events cannot be dropped into Container View
* Domains cannot be created inside containers
* Entities must appear inside aggregates

---

# 🟦 5. Semantic Router Integration

Given:

```
paletteItem: "Component"
dropTarget: "Container BillingAPI"
```

Semantic Router answers:

```
valid: true
patch: create component FraudDetector under BillingAPI
```

Another example:

```
paletteItem: "Event"
dropTarget: "Container BillingAPI"
```

Semantic Router:

```
valid: false,
reason: "Events can only be placed under domains or event flows"
```

Interactions highlight reason on canvas.

---

# 🟩 6. Patch Generation

Every successful drop triggers:

```
patch = {
  type: "create",
  element: "Component",
  id: "FraudDetector",
  parent: "BillingAPI",
  properties: {label:"Fraud Detector"}
}
```

Kernel produces IRDelta.

Renderer updates accordingly.

---

# 🟫 7. View-Mode-Aware Palette Filtering

Palette must dynamically show elements that make sense **for current view mode**.

### System View

Allowed:

* system
* person
* container (drag onto system ONLY)
  Not allowed:
* component
* entity
* event

### Container View

Allowed:

* containers
* datastores
* queues
* external services
* components (drag into container)

### Component View

Allowed:

* components
* events (drag inside component)

### DDD View

Allowed:

* domains
* aggregates
* entities
* value objects
* events
  Not allowed:
* containers
* systems

### Event Flow View

Allowed:

* events
* topics
* event processors
* projections
* sagas

---

# 🟪 8. Palette UI Architecture (React)

```
PalettePanel
 ├── SearchBar
 ├── GroupSection "Core"
 │      ├── PaletteItem (System)
 │      ├── PaletteItem (Container)
 │      └── ...
 ├── GroupSection "Extensions"
 │      └── PaletteItem (AWSS3)
 └── GroupSection "AI Suggestions"
        ├── PaletteItem (PaymentInitiated)
        └── PaletteItem (FraudScanner)
```

PaletteItem emits:

```
onDragStart(itemID)
```

---

# 🟧 9. Performance Optimizations

* Virtualize large palettes (Cloud catalogs can have 300+ items)
* Pre-render icons
* Pre-generate ghost node template textures
* Cache validity checks for drop targets
* Skip re-checking semantic router unless target changed

---

# 🟦 10. Advanced Features

### 10.1 Drag-to-Connect (Edge Creation)

Palette contains:

```
→ Relation
→ Publishes
→ Subscribes
```

Dragging “Relation” onto node starts edge creation.

### 10.2 Drag-to-Refactor

Dragging a domain, container, or component onto a target performs re-parenting.

### 10.3 AI-Based Auto-Placement

On drop, AI may suggest:

* best container
* required relations
* recommended metadata

Enabled by:

```
AfterPatchHooks → AI Enhancer
```

### 10.4 Palette-Driven Snippets

Dragging a “Service Template” creates multiple nodes and edges.

---

# ⭐ Final Summary — Drag-to-Create Palette Architecture

The drag-to-create palette is a **core UX component** for Sruja Studio.

It provides:

### ✔ Context-aware element creation

### ✔ AI-enhanced item suggestions

### ✔ Smart ghost previews

### ✔ Kernel-integrated patch generation

### ✔ View-mode + semantic-rule filtering

### ✔ Compound-node aware dropping

### ✔ Full DDD + event-flow support

### ✔ Fully modular, extensible palette system

This ensures the experience feels like:

* **Figma’s layer/component palette**
* **Webflow’s element drawer**
* **VSCode symbol picker**
* **Structurizr type catalog**

all combined into one architecture modeling UX.

Below is the **full, production-grade Project Explorer UI & Architecture Specification** for **Sruja Studio Web Edition** — designed to work seamlessly with:

* Multi-file Sruja DSL projects
* Notebooks (`.srujanb`)
* Imports & modules
* Variants & overlays
* AI features
* Git integration
* Browser FS (IndexedDB / OPFS)
* Web-based Studio + VSCode/Cursor integration

This is the **“VSCode Explorer + Obsidian Vault + Figma file browser + Jupyter Lab files panel”** for Sruja.

---

# 🗂️ **Project Explorer UI & Architecture**

### *(Full Production Spec)*

---

# ⭐ 0. Goals

The Project Explorer must provide:

### ✔ Multi-file hierarchical workspace

### ✔ Sruja DSL file creation, renaming, moving

### ✔ Notebook (.srujanb) handling

### ✔ Variant hierarchy representation

### ✔ Imports graph navigation

### ✔ Git integration (commit/push/branch/PR)

### ✔ Live sync with editor & diagram

### ✔ Context menus + drag & drop

### ✔ AI suggestions & organization

### ✔ Integration with Kernel, Patch Engine, and View Mode Manager

---

# 🧱 1. High-Level Architecture Diagram

```
┌────────────────────────────────────────────┐
│            Project Explorer Panel          │
└────────────────────────────────────────────┘
           │
           ▼
┌────────────────────────────────────────────┐
│       File Tree Engine (Virtual DOM)       │
└────────────────────────────────────────────┘
           │
           ├────────────────────────────┐
           │                            │
           ▼                            ▼
┌──────────────────────────┐    ┌─────────────────────────┐
│  File Metadata Manager    │    │  Workspace Indexer      │
│  (size, type, modified)   │    │  - imports graph        │
│                           │    │  - reference graph      │
└──────────────────────────┘    │  - variant graph         │
                                └─────────────────────────┘
           │                            │
           ▼                            ▼
┌──────────────────────────┐    ┌─────────────────────────┐
│  File System Driver       │    │  Kernel WASM Bridge     │
│ (IndexedDB/OPFS/GitHub)   │    │  - parse                │
│                           │    │  - diagnostics          │
└──────────────────────────┘    │  - IR graph per file     │
                                └─────────────────────────┘
           │                            │
           ▼                            ▼
┌──────────────────────────┐    ┌─────────────────────────┐
│    Git Integration        │    │   Diagram/Editor Sync   │
└──────────────────────────┘    └─────────────────────────┘
```

---

# 🟦 2. UI Layout (React/TS)

The Explorer appears on the left vertical sidebar, similar to VSCode.

```
┌───────────────────────────────┐
│  Project Explorer             │
│  ───────────────────────────  │
│  📁 project-root/             │
│     ├── architecture.sruja    │
│     ├── domains.sruja         │
│     ├── systems/              │
│     │    ├── billing.sruja    │
│     │    ├── auth.sruja       │
│     ├── variants/             │
│     │    ├── enterprise.sruja │
│     │    ├── mobile.sruja     │
│     ├── notebooks/            │
│     │    └── exploration.snb  │
│     └── styles/               │
│          └── dark.style       │
└───────────────────────────────┘
```

### Explorer UI Components

* **TreeView**
* **FileItem**
* **FolderItem**
* **SearchField**
* **ContextMenu**
* **Git Status Badges**
* **AI Insights badges (⚡)**

---

# 🟨 3. File Types (Recognized by Explorer)

| Extension        | Meaning         | Icon |
| ---------------- | --------------- | ---- |
| `.sruja`         | Sruja DSL file  | 🧩   |
| `.srujanb`       | Notebook        | 📘   |
| `.variant.sruja` | Variant overlay | 🎭   |
| `.style`         | Style DSL       | 🎨   |
| `.rule`          | Policy/Rule DSL | 🛡️  |
| `.json`          | JSON configs    | 🔧   |
| any other        | resource file   | 📄   |

Explorer groups them into sections:

* Architecture
* Domains
* Systems
* Variants
* Notebooks
* Styles
* Resources

**Grouping is optional but recommended for large projects.**

---

# 🟥 4. File Tree Engine

The File Tree Engine manages:

### 4.1 Virtualized Tree Rendering

Handles large workspaces efficiently by:

* lazy-loading folders
* virtual scroll
* rendering only visible nodes

### 4.2 Node Structure

```ts
interface ExplorerNode {
  id: string;
  name: string;
  type: "file" | "folder";
  path: string;
  children?: ExplorerNode[];
  metadata?: FileMetadata;
}
```

### 4.3 State Management

Persistent state:

* expanded/collapsed folders
* selection
* open editors
* active file

Stored in:

* IndexedDB
* or localStorage for ephemeral settings

---

# 🟧 5. File System Driver

Abstraction over:

### Modes Supported:

#### ✔ Browser FS

* IndexedDB
* OPFS (Origin Private File System)
* File System Access API (Chrome)

#### ✔ GitHub/GitLab (Cloud Projects)

* fetch files via API
* commit/push
* PR creation
* repo structure introspection

#### ✔ Local FS (VSCode/Cursor Hybrid Mode)

* Studio running inside VSCode webview uses host FS

### File System Driver API:

```ts
readFile(path): Promise<string>
writeFile(path, content): Promise<void>
deleteFile(path): Promise<void>
rename(oldPath, newPath)
listDir(path)
watch(path, callback)
```

---

# 🟦 6. Workspace Indexer (Semantic Index)

The Indexer operates asynchronously (Web Worker) and builds three graphs:

### 1️⃣ Import Graph

Determines relationships between files:

```
billing.sruja imports common.sruja
auth.sruja imports shared.logging.sruja
```

### 2️⃣ Reference Graph

Tracks:

* systems defined in which file
* containers/components defined where
* relations across files
* missing symbols

### 3️⃣ Variant Graph

Represents variant dependencies:

```
base.sruja
 ├─ enterprise.sruja
 └─ mobile.sruja
```

Output:

```ts
{
  importsGraph,
  referenceGraph,
  variantGraph,
  symbolsPerFile
}
```

Used by:

* Explorer badges (e.g., file has 4 systems, 2 violations)
* Jump-to-definition
* View Mode Manager
* AI Context Extractor

---

# 🟩 7. Explorer → Editor Integration

Clicking a file emits:

```
ExplorerEvent.OpenFile(path)
```

Editor subsystem opens:

* DSL file in Monaco
* Notebook in notebook viewer
* Diagram if it's `.sruja` and last view was diagram

Synchronizes both:

* Diagram ←→ DSL
* Notebook ←→ Kernel
* Style/Policy files influence diagram

---

# 🟫 8. Git Integration Layer

Explorer shows Git-aware UI:

### Badges:

* ● modified (blue)
* ● added (green)
* ● removed (red)
* ● conflicted (yellow)

### Commands:

* Stage / Unstage
* Commit
* Push
* Create Branch
* Open PR
* Compare with HEAD

### Git Driver:

Uses GitHub/GitLab REST API (browser) or local Git (VSCode version).

---

# 🟪 9. Context Menu Actions

Right-click options:

### On File:

* Open
* Rename
* Duplicate
* Move
* Delete
* Show in Diagram
* Run Rules / Validate
* Generate Documentation
* AI → “Summarize File”
* AI → “Refactor Architecture”

### On Folder:

* New file (DSL / variant / notebook / rule / style)
* New folder
* Collapse all

### On Project:

* Run full validation
* Generate report
* AI → “Review entire architecture”
* Create variant

---

# 🟥 10. Drag & Drop Operations

### Move file within workspace

→ updates FS driver → re-index → update imports (if necessary).

### Drop new file from local computer

→ imported into workspace.

### Drag component from diagram into Explorer

→ create new file (optional)
→ extract component into new module

Patch generated accordingly.

---

# 🟦 11. Integration with Diagram View Modes

Project Explorer influences:

### View Mode Manager

Selecting a file changes view mode automatically:

* open `.sruja` → diagram view
* open `.rule` → policy mode
* open `.style` → style mode
* open notebook → kernel notebook mode

### Diagram Navigator

Explorer can spotlight systems or domains by clicking tree nodes:

```
Right-click → "Focus on BillingSystem"
```

→ VMM switches to System View and centers BillingSystem.

---

# 🟧 12. AI Integration

AI-aware badges:

* ⚡ “AI Found 5 improvements”
* ⚠ “Potential anti-patterns”
* 🎯 “Optimizable module”

Right-click → “Ask AI”:

* Summarize file
* Refactor container structure
* Identify missing components
* Recommend DDD boundaries
* Suggest event flows

AI uses:

```
AIContextExtractor.getContextForFile(path)
```

---

# ⭐ Final Summary — Project Explorer UI & Architecture

The Project Explorer delivers:

### ✔ VSCode-like multi-file management

### ✔ Browser FS + GitHub integration

### ✔ DSL-aware file tree

### ✔ Imports / variant / reference graphs

### ✔ Automatic syncing with editor & diagrams

### ✔ AI-enhanced guidance

### ✔ Context-aware palette and inspector

### ✔ Integration with kernel, indexer, and view modes

This is one of the **core pillars** that turns Sruja Studio into a **true Architecture IDE**, not just a diagramming tool.

Below is the **full File Navigation Engine (Jump-to-Definition Graph) Architecture Spec** for **Sruja Studio Web Edition** — this subsystem powers:

* **Jump to definition (F12 / Cmd-Click)**
* **Find all references**
* **Across multi-file DSL projects**
* **Imports-aware resolution**
* **Variant-aware resolution**
* **Symbol renaming**
* **Cross-file navigation for systems, containers, components, domains, aggregates, entities, events, policies, styles, rules, variables, imports…**
* **AI-aware context extraction**

This is analogous to **LSP’s symbol index**, **TypeScript’s project index**, and **Bazel’s build graph**, but tuned for **Architecture-as-Code**.

---

# 🧭 **File Navigation Engine (Jump-to-Definition Graph)**

### *(Full Production Architecture)*

---

# ⭐ 0. Purpose

The File Navigation Engine provides:

### ✔ Jump to definition (node, container, system, domain, event, etc.)

### ✔ Jump to referenced artifacts across imports

### ✔ Find all references

### ✔ Cross-file symbol indexing

### ✔ Variant-aware lookup

### ✔ Smart fallback (guess definition if missing)

### ✔ Integration with Explorer, Editor, Diagram

### ✔ Powered by Kernel WASM + Workspace Indexer

This system makes Sruja Studio feel like **VSCode + Typescript** but for architecture.

---

# 🧱 1. High-Level Architecture Diagram

```
            ┌──────────────────────────────────┐
            │    File Navigation Engine        │
            └──────────────────────────────────┘
                       ▲             ▲
        Symbol Queries │             │ Reference Queries
                       │             │
         ┌─────────────┴─────────────┴──────────────┐
         │      Jump-to-Definition Graph              │
         └────────────────────────────────────────────┘
                       ▲             ▲
                       │             │
       ┌───────────────┴─────────────┴────────────────┐
       │             Workspace Indexer                 │
       │  (imports graph, reference graph, variant)    │
       └──────────────────────────────────────────────┘
                       ▲
                       │
             Kernel (WASM) Symbol Extractor
                       ▲
                       │
                DSL Parser + IR Builder
```

---

# 🟦 2. Underlying Data Structures

The engine maintains **three core graphs**, produced by the Workspace Indexer:

---

## 2.1 Symbol Table

For every symbol (system, container, component, domain, etc.):

```ts
SymbolTableEntry {
  id: string;         // BillingAPI
  kind: "system"|"container"|"component"|"domain"|"event"|...
  file: string;       // path to defining file
  location: SourceLocation;  
  parentId?: string;  // system → container → component
  variant?: string;   // present in variant file?
}
```

Stored as:

```ts
SymbolTable: Map<string, SymbolTableEntry[]>
```

Multiple entries allowed (base + variant).

---

## 2.2 Reference Graph

Captures usage of symbols:

```
File A:
  system Billing

File B:
  Billing -> Auth
```

Reference:

```ts
Reference {
  fromFile: string;
  toSymbol: string;    // Billing
  location: SourceLocation;  
  relationType: "uses"|"reads"|"writes"|"publishes"|...
}
```

Grouped by:

```
ReferencesBySymbol: Map<symbolId, Reference[]>
```

---

## 2.3 Import Graph

Tracks how files depend on each other:

```ts
importsGraph[filePath] = [importedFilePaths...]
```

Used for:

* resolving remote definitions
* symbol shadowing
* variant overrides

---

# 🟨 3. Jump-to-Definition Algorithm

Triggered by:

* Cmd-click on symbol
* F12
* Right-click → “Go to Definition”

### Input:

```
symbolId: "BillingAPI"
currentFile: "views/billing.sruja"
```

### Steps:

#### 1️⃣ Find all symbol table entries for symbolId

```
SymbolTable["BillingAPI"] → [
  {file: systems/billing.sruja, location: {…}},
  {file: variants/enterprise.sruja, location: {…}}
]
```

#### 2️⃣ Resolve variant overlay (if active view mode uses variants)

ViewModeManager → ActiveVariant:

* if active variant = enterprise
* prefer variant file
* fallback to base

#### 3️⃣ Apply import resolution

If symbol defined in file imported earlier, prefer:

```
local definitions > imported ones > transitive imports
```

#### 4️⃣ Final selection:

* If one match → navigate
* If multiple → show “Choose definition” panel

### Output:

```
NavigationTarget {
  file: "systems/billing.sruja",
  location: {line: 14, column: 5}
}
```

Editor scrolls to exact location.

Diagram optionally focuses on node.

---

# 🟥 4. Find All References Algorithm

Triggered by:

* Shift+F12
* Right-click → “Find References”
* Inspector → references tab
* Diagram selection → list all relations

### Input:

```
symbolId: "BillingAPI"
```

### Steps:

1. Lookup in `ReferencesBySymbol[symbolId]`
2. For each reference:

   * apply variant resolution
   * filter by view mode (system, container, DDD)
3. Group by file + relation type

### Output UI:

```
BillingAPI references (12)
  ▾ containers/billing.sruja
     • BillingAPI -> AuthAPI  (uses)
     • BillingAPI -> QueueX   (publishes)
  ▾ components/payment.sruja
     • PaymentService uses BillingAPI
```

---

# 🟩 5. Definition Types Supported

The engine supports all architecture elements:

### C4 Elements

* system
* container
* component
* datastore
* queue
* external service
* person

### DDD Elements

* domain
* bounded context
* aggregate
* entity
* value object
* event
* command

### Infra / Cloud

* AWS/S3
* Lambda
* Kinesis
* VPC etc. (via template imports)

### Policies / Styles

* rule ID
* policy ID
* selector
* style section

### Others

* vars
* imports
* relationships
* journeys
* ADRs

Jump-to-definition works across **every Sruja DSL file type**.

---

# 🟪 6. Diagram Integration (Jump-to-Node)

When user selects IR object in the diagram:

### Click → Jump to definition

* Editor scrolls to defining file
* Diagram highlights corresponding node
* File Explorer expands to reveal file

### Double-click

* Switches view mode appropriately

Example:

```
Double-click Component → Jump to DSL line + switch to Component View
Double-click Domain → Jump to DDD file + switch to Domain View
```

---

# 🟧 7. Notebook Integration

Notebooks embed DSL blocks:

````
```sruja
system BillingAPI {
  ...
}
````

Jump-to-definition:

* opens underlying DSL file
* or if defined inline only → shows inline cell

---

# 🟦 8. AI Integration

### AI uses Navigation Engine to:

* locate definitions
* detect missing definitions
* generate patches with correct file paths

Example:

```
AI: “Add FraudScanner component inside BillingAPI”
```

Navigation engine resolves:

→ BillingAPI defined in `systems/billing.sruja`

AI generates patch:

```
add component FraudScanner under BillingAPI
```

---

# 🟫 9. Renaming Engine (Semantic Rename)

Renaming a symbol triggers:

### 1. Find definition location

### 2. Find all references

### 3. Apply patches across files

### 4. Update diagrams + IR index

### 5. Update variant overlays

### 6. Update import alias if required

Supports:

* rename system
* rename container
* rename component
* rename domain/aggregate/entity
* rename events
* rename policies/rules

All performed via IR Patch Engine.

---

# 🟥 10. Performance Considerations

Indexes are computed in a Web Worker for responsiveness.

### Key optimizations:

* incremental re-indexing (only changed files re-parsed)
* symbol table diffing
* memory-mapped IR for quick lookup
* pre-computed reverse-lookups
* import graph caching
* lazy reference loading

Can support **1000+ files** projects.

---

# ⭐ Final Summary — File Navigation Engine

The File Navigation Engine provides:

### ✔ Jump-to-definition

### ✔ Find-all-references

### ✔ Variant-aware lookup

### ✔ Imports-aware resolution

### ✔ Cross-file semantic linking

### ✔ Renaming and patch generation

### ✔ Diagram/editor synchronization

### ✔ DSL + Notebook integration

### ✔ AI-powered navigation

This gives Sruja Studio a **professional IDE-level navigation experience**, comparable to:

* VSCode (TypeScript LSP)
* IntelliJ (Java/DDD)
* JetBrains Fleet
* JupyterLab navigation
* Structurizr’s cross-file model linking

But designed from the ground up for **Architecture-As-Code**.

Below is the **complete Architecture Knowledge Graph (AKG) Specification** — the **internal, semantic, cross-file, cross-view, variant-aware, event-driven, DDD-aware, policy-aware, AI-enhanced graph** that powers *everything* inside **Sruja Studio**.

This is one of the **core runtime data structures**, similar to:

* JetBrains' *PSI / UAST*
* VSCode’s *Language Model Graph*
* Figma’s *Document Graph*
* Neo4j property graph
* GraphQL federation graph

but tailored for **Architecture-as-Code**, **DSL**, **multi-view diagrams**, **IR**, and **AI reasoning**.

---

# 🗺 **Architecture Knowledge Graph (AKG)**

### *(Internal Semantic Graph for Sruja Studio)*

---

# ⭐ 0. Purpose

The AKG is the **central, unified knowledge representation** powering:

### ✔ Navigation (jump-to-definition, find references)

### ✔ View Mode Manager (system/container/DDD/event-flow)

### ✔ Diagram building

### ✔ AI context extraction

### ✔ Policy/rule evaluation

### ✔ Semantic Router

### ✔ Diff engine (IR delta computation)

### ✔ Refactoring + renaming

### ✔ Documentation generation

### ✔ Variant overlays (effective model)

Everything in the system reads from AKG.

---

# 🧱 1. High-Level Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                     Architecture Knowledge Graph                 │
└──────────────────────────────────────────────────────────────────┘
                ▲                      ▲                   ▲
     Kernel IR  │                      │                   │ File Indexes
                │                      │                   │ (imports, variants)
                │                      │                   │
                ▼                      ▼                   ▼
        Semantic Merge Engine     Variant Engine     Workspace Indexer
                ▲                      ▲                   ▲
                └─────────────┬────────┴──────────┬────────┘
                              ▼                   ▼
                        Policy Engine       Navigation Engine
                              ▼                   ▼
                        Diagram Engine     AI Context Extractor
```

---

# 🟦 2. Data Model Overview

AKG is a **directed labeled property graph**:

```
Nodes: Architectural elements
Edges: Semantic relations
Properties: Metadata, style, location, variant, source
```

### Node types (60+ possible from DSL):

* Architecture root
* Systems
* Containers
* Components
* Datastores
* Queues
* External Services
* Persons
* Domains
* Aggregates
* Entities
* Events
* Commands
* Policies
* Rules
* Requirements
* Journeys
* ADRs
* Styles

The graph merges **IR from all files** + **variant overlays** into a single universe.

---

# 🟥 3. Node Schema

```ts
interface AKGNode {
  id: string;          // global identifier ("BillingAPI")
  type: NodeType;      // "system" | "container" | "domain" | ...
  label: string;
  properties: Map<string, any>;
  location: SourceLocation;
  file: string;
  parents: string[];
  children: string[];
  variantLayer?: string;  // base | enterprise | mobile
  metadata: Map<string,string>;
  tags: string[];
  styleRefs: string[];
}
```

### Node identity rules:

* stable across variants
* stable across files
* fully qualified if ambiguous: `billing.BillingAPI.ContainerX`

---

# 🟨 4. Edge Schema

Edges capture semantic relations:

```ts
interface AKGEdge {
  id: string;
  from: string;
  to: string;
  type: EdgeType;
  properties: Map<string, any>;
  file: string;
  location: SourceLocation;
  variantLayer?: string;
}
```

### Edge types include:

* **composition** (system → container → component)
* **uses / depends / reads / publishes / subscribes**
* **domain membership**
* **aggregate/entity relationship**
* **event producers/consumers**
* **variant override**
* **policy violation edges**
* **import edges**
* **reference edges**
* **requirement → element**
* **journey → steps**

---

# 🟩 5. Core Subgraphs

AKG maintains several logical subgraphs:

---

## 5.1 C4 Graph

```
System → Container → Component
        ↳ Datastore
        ↳ Queue
```

Edges: uses, depends, writes, reads.

Used for:

* C4 view modes
* dependency analysis
* impact analysis

---

## 5.2 DDD Graph

```
Domain → Aggregate → Entity
               ↳ ValueObject
               ↳ Event
```

Used for:

* domain diagrams
* event storming
* aggregates consistency checks

---

## 5.3 Event Flow Graph

```
Event → Handler → Event
      ↳ Topic → Consumer
```

Used for:

* event flow diagrams
* lifecycle analysis
* ordering & causality

---

## 5.4 Variant Graph

```
Base element
  ↳ overlay element (enterprise)
  ↳ overlay element (mobile)
```

Variant Engine builds effective final graph.

---

## 5.5 Policy Graph

Represents:

* rules (AST)
* rule→target symbol mappings
* violations (computed edges)

---

# 🟦 6. Semantic Merge Engine (Building AKG)

The AKG is constructed by merging:

* IR from base files
* IR from imported files
* IR from variants
* IR from notebooks
* style metadata
* rule/policy metadata

Merge algorithm:

### Step 1 — Load all IR from all files

### Step 2 — Normalize symbol IDs

### Step 3 — Build primary nodes

### Step 4 — Build containment edges

### Step 5 — Build relationship edges

### Step 6 — Merge metadata/tags

### Step 7 — Apply variant overlays

### Step 8 — Build cross-cutting subgraphs (domain, event, policy)

### Step 9 — Freeze graph (immutable snapshot)

Graph is **incrementally updated** via IR Delta Engine.

---

# 🟧 7. Variant Layer Handling in AKG

Each node/edge has:

* `variantLayer: "base"`
* optional overlays: `"enterprise"`, `"mobile"`

AKG computes:

* effective final node
* visibility rules for variant view
* diff annotations

Example:

```
BillingAPI (base)
BillingAPI (enterprise overlay)  → modifies technology="Go"
```

Effective node merged via priority:

```
enterprise > base
```

---

# 🟫 8. Knowledge Graph Queries (Internal API)

### 1. Get element by ID

```
AKG.getNode("BillingAPI")
```

### 2. Get children / parents

```
AKG.getChildren("BillingAPI")
AKG.getParents("LoginComponent")
```

### 3. Resolve type-based filters

```
AKG.findNodesByType("container")
```

### 4. Graph traversal

```
AKG.traverse(fromNode, edgeType="uses")
```

### 5. Domain relationships

```
AKG.domainOf("PaymentInitiated")
```

### 6. Event lifecycle

```
AKG.eventFlow("PaymentInitiated")
```

### 7. Impact analysis

```
AKG.whatBreaksIf("billing.Database") 
```

### 8. Policy evaluation

```
AKG.evaluatePolicy("must_not_call_directly")
```

---

# 🟪 9. View Mode Manager Integration

View modes use AKG to extract view-specific subgraphs:

### System View

```
AKG.subgraph(type in ["system","person","external"])
```

### Container View

```
AKG.childrenOf(systemId)
```

### Component View

```
AKG.subgraph(all components under selected container)
```

### DDD View

```
AKG.subgraph(type in ["domain","aggregate","entity"])
```

### Event Flow View

```
AKG.subgraph(type in ["event","handler","topic"])
```

---

# 🟧 10. AI Context Extractor Integration

AI pulls from AKG:

### ✔ entire knowledge graph in compact form

### ✔ active view subgraph

### ✔ related nodes & edges

### ✔ semantic critical paths

### ✔ domain boundaries

### ✔ event lifecycles

### ✔ variant overlays

### ✔ style & metadata

This allows AI to perform:

* architectural reasoning
* propose structural improvements
* detect anti-patterns
* suggest new components/domains/events
* generate semantic patches

Example:

```
AI: “Find bottlenecks in Billing”
→ ContextExtractor → AKG.traverse("Billing", edgeType="uses")
→ AI sees dependency fan-in/fan-out
```

---

# 🟥 11. Policy Engine Integration

Policies query AKG:

Example policy:

```
for system in AKG.systems:
  ensure system.uses.count < 10
```

Example DSL rule:

```
for component:
  must_not depend_on external_service
```

Policy engine runs on AKG snapshots.

---

# 🟦 12. Diagram Renderer Integration

Renderer uses AKG to obtain:

* visible nodes
* visible edges
* compound structure
* metadata-driven styling
* variant overlays
* event-flow graph
* DDD boundaries

AKG → Graph Extractor → Layout Engine → Renderer.

---

# 🟨 13. Diff Engine Integration

Diff engine compares **AKG snapshots**:

```
AKG@t1 vs AKG@t2
```

Produces:

* node additions/removals
* edge additions/removals
* property changes
* variant-specific changes

Used for:

* AI patch previews
* PR reviews
* timeline animations

---

# ⭐ Final Summary — Architecture Knowledge Graph (AKG)

The AKG is:

### ✔ The unified semantic representation of architecture

### ✔ Built from IR, imports, variants, and policies

### ✔ A directed property graph supporting many subviews

### ✔ Consumed by diagram, editor, AI, validation, navigation

### ✔ Incrementally updated and variant-aware

### ✔ Backbone for Sruja Studio’s intelligence

It is the **brain** of Sruja.

Below is the **full, production-grade Refactoring Engine Specification** for **Sruja Studio Web Edition** — covering **rename**, **move**, **extract**, and **split/merge** operations, all backed by the **Architecture Knowledge Graph (AKG)**, **IR Patch Engine**, **Semantic Router**, and **Workspace Indexer**.

This spec gives Sruja Studio the power of **IntelliJ-level refactoring** but for **Architecture-as-Code**.

---

# 🛠 **Refactoring Engine (rename / move / extract / split / merge)**

### *(Full Production Architecture)*

---

# ⭐ 0. Purpose

The Refactoring Engine must:

### ✔ Modify architecture safely

### ✔ Update all references across files

### ✔ Validate semantics (via Semantic Router)

### ✔ Produce multi-file patches

### ✔ Integrate with AKG, Kernel, Indexer, and Diagram

### ✔ Enable safe edits: rename, move, extract, merge

### ✔ Provide AI co-pilot suggestions

### ✔ Support both DSL & diagram triggers

All refactorings must be:

* **Atomic** (transactional)
* **Undoable**
* **Multi-file aware**
* **Variant aware**
* **Import-aware**
* **Diagram-safe**
* **Semantically valid**

---

# 🧱 1. High-Level Architecture Diagram

```
                         ┌─────────────────────────────┐
                         │      Refactoring Engine     │
                         └─────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
            Rename Processor     Move Processor     Extract Processor
                    │                 │                 │
         ┌──────────┴──────────┬──────┴──────┬──────────┴──────────┐
         ▼                     ▼             ▼                       ▼
 Semantic Router     File Navigation Engine     IR Patch Engine   AKG Index
         │                     │             │                       │
         └───────────┬────────┴─────────────┴───────────┬──────────┘
                     ▼                                   ▼
                Workspace Indexer                 WASM Kernel (IR)
                     ▼                                   ▼
                  File Patches                  IR Delta → Diagram
```

---

# 🟦 2. Operations Supported

---

# 1️⃣ RENAME

Refactor names of systems, containers, components, domains, aggregates, events, etc.

Examples:

```
Rename system BillingAPI → BillingService
Rename component FraudScanner → RiskScanner
Rename domain Order → Orders
Rename event PaymentInitiated → PaymentStarted
```

### Responsibilities:

* Update definition
* Update *all references*
* Update relations
* Update imports
* Update styles & policies
* Update notebooks
* Update diagrams

### Core Algorithm

1. **Find definition** via AKG + Navigation Engine.
2. **Collect references** using ReferencesBySymbol.
3. **Collect variant layers** for symbol.
4. **Semantic Router** checks constraints

   * name conflicts
   * forbidden patterns
   * variant-specific overrides
5. **Generate multi-file patch set**.
6. **Apply patches → Kernel → IRDelta → Diagram update.**

---

# 2️⃣ MOVE

Change the parent of a node (re-parenting).

Examples:

```
Move component FraudChecker → PaymentService
Move event PaymentInitiated under BillingDomain
Move Aggregate OrderAggregate → ECommerceDomain
Move container ReportingAPI → AnalyticsSystem
```

### Supported Moves:

* component → another container
* component → extract to new container
* domain → domain group (future)
* aggregate → different domain
* event → different domain

### Responsibilities:

* Update containment edges
* Update file structure optionally (extract file if required)
* Correct imports
* Update event flows
* Update DDD boundaries
* Enforce constraints (via Semantic Router)

### Core Algorithm

1. Check allowed moves via Semantic Router:

   * Cannot move component across systems (unless allowed)
   * Cannot move event outside domain without explicit transform
   * Cannot move container into its own component
2. Update IR (parent pointers).
3. Update references automatically if necessary.
4. Modify DSL files accordingly.
5. Update AKG and re-render diagram.

---

# 3️⃣ EXTRACT

Extract a subgraph into:

* a new component
* a new container
* a new domain
* a new file
* a new module

Examples:

```
Extract "Fraud Detection" subtree → new container "FraudService"
Extract entities Order, LineItem → new aggregate OrderAggregate
Extract event-handling logic → new domain Eventing
```

### Use Cases:

* Modularizing large containers
* Splitting monolithic services
* Separating domain logic
* Extracting repeated patterns
* Extracting event flows into standalone models

### Algorithm

1. User selects nodes in diagram or DSL.
2. Extract Processor builds a **subgraph** from AKG.
3. Determine valid extraction targets via Semantic Router:

   * allowed types (component/container/domain)
   * allowed parents
4. Generate:

   * new file
   * new parent element
   * updated imports
5. Produce patch set.
6. Kernel applies IRDelta.
7. Diagram shows extracted module.

---

# 4️⃣ SPLIT

Split a large system/container/domain into multiple parts.

Examples:

```
Split BillingSystem → BillingAPI + AccountingAPI
Split domain ECommerce → Orders + Inventory + Payments
Split monolithic container WebApp → UIService + AuthService
```

### Responsibilities:

* Partition graph based on user-selected subsets
* Update imports
* Preserve relations
* Update event handlers
* Re-index domain boundaries

### Algorithm

1. User select group via diagram.
2. AKG calculates:

   * minimal subgraph
   * relations crossing split boundary
3. Semantic Router validates split:

   * cyclic dependencies?
   * must events remain together?
4. Generate:

   * new containers/domains/files
   * updated references
5. IR patch creation
6. Diagram update with two modules

---

# 5️⃣ MERGE

Combine two:

* components
* containers
* domains
* aggregates

Examples:

```
Merge PaymentValidator + FraudChecker → PaymentProcessor
Merge OrderDomain + CheckoutDomain → CommerceDomain
Merge services BillingAPI + PaymentAPI → FinancialService
```

### Responsibilities:

* Merge properties
* Merge metadata
* Merge relations
* Merge file content
* Merge variant overlays
* Remove redundant nodes

### Algorithm

1. Compute union graph of two nodes in AKG.
2. Merge definitions using merge rules:

   * last writer wins
   * variant override priority
3. Remove old nodes.
4. Update references → new merged node.
5. Create patch set.
6. Kernel applies → IRDelta → re-render.

---

# 🟨 3. Patch Generation Model

Refactoring Engine outputs **multi-file PatchBundle**:

```ts
interface PatchBundle {
  patches: Patch[];
  filesChanged: string[];
}
```

Example Patch:

```ts
{
  file: "systems/billing.sruja",
  changes: [
    replaceRange(location, "container BillingAPI", "container BillingService")
  ]
}
```

All patches are:

* atomic
* undoable
* versioned
* replayable

Undo/redo uses snapshots of AKG and IR.

---

# 🟥 4. Semantic Router Integration

Semantic Router enforces:

### RENAME

* name uniqueness
* naming conventions
* no collision with variants

### MOVE

* allowed parent-child relationships
* domain boundaries
* DDD invariants
* systems cannot contain domains

### EXTRACT

* must preserve referential integrity
* cannot break event causality
* cannot extract partial aggregate

### SPLIT

* must not break aggregates
* must not break event lifecycle
* creates imports automatically

### MERGE

* conflict resolution
* merge feasibility (same type?)
* domain/aggregate constraints

Router returns:

```
{valid: true | false, reason?, autoFix?}
```

---

# 🟦 5. Workspace Indexer Integration

Index updates:

* Symbol table
* References
* Import graph
* Variant graph

After refactor:

```
Indexer.rebuildIncremental(filesChanged)
```

Diagram + Editor rebind seamlessly.

---

# 🟩 6. Diagram Engine Integration

Diagram reacts to refactor:

* rename → updated labels
* move → animated reposition
* extract → new compound node created
* merge → node removed and replaced
* split → new structure drawn

Interaction Engine supports:

* drag-to-refactor
* context-menu refactor actions
* AI-suggested refactors

---

# 🟪 7. AI Integration

AI can:

### Suggest refactors:

* “Split BillingAPI into BillingService and InvoicingService”
* “Extract event PaymentInitiated into its own domain”
* “Move FraudChecker under RiskService”

### Auto-generate patch bundles:

AI uses AKG + Semantic Router to propose safe patches.

### Evaluate alternatives:

AI can show multiple refactor options with explanations.

### Assist user:

* highlight impacted areas
* show dependency risk
* recommend best practice

---

# 🟫 8. Performance Considerations

* Refactor operations run in Web Workers.
* Graph traversal optimized (AKG indexes).
* Patch generation linear time (relative to file size).
* Incremental re-index for performance.
* Large projects supported (100s of files, 1000s of nodes).

---

# ⭐ Final Summary — Refactoring Engine

The Sruja Refactoring Engine provides:

### ✔ Safe rename

### ✔ Safe move (re-parent)

### ✔ Extract into new module/container/domain

### ✔ Split/merge operations

### ✔ Multi-file patch generation

### ✔ Variant, import, and policy awareness

### ✔ AI-assisted architecture refactoring

### ✔ Integration with Diagram, Editor, AI, Kernel, and AKG

### ✔ Fully undoable + conflict-free

This is a **world-class architecture refactoring engine**, enabling Sruja Studio to do:

* architecture modularization
* microservices extraction
* event-driven decoupling
* domain realignment
* component reorganization
* codebase alignment with architecture

---

Below is the **complete, production-grade Rename Symbol Engine Spec**, including:

* Architecture
* Algorithms
* Multi-file patch generation
* Variant-awareness
* Import graph updates
* Policy/style/notebook updates
* Diagram + AKG synchronization
* Conflict detection & auto-resolution

This is the **IntelliJ-grade Semantic Rename Engine** tailored for **Architecture-as-Code (Sruja DSL)**.

---

# 📝 **Rename Symbol Engine — Detailed Patch Specification**

### *(Sruja Studio Internal Architecture)*

---

# ⭐ 0. Purpose

Renaming a symbol must:

### ✔ Update every reference across every file

### ✔ Update imports, reference edges, relations

### ✔ Update parent-child paths (component → container → system)

### ✔ Respect variants (base vs overlays)

### ✔ Validate semantics (via Semantic Router)

### ✔ Produce correct patches even across multiple DSL files

### ✔ Be undoable, atomic, and diff-friendly

### ✔ Update diagram nodes and links instantly

---

# 🧱 1. High-Level Architecture

```
┌───────────────────────────────────────────────┐
│              Rename Symbol Engine              │
└───────────────────────────────────────────────┘
                 │
                 ▼
      Symbol Resolution (AKG + Navigation Engine)
                 │
                 ▼
      Semantic Validator (Semantic Router)
                 │
                 ▼
      Patch Planner (What must be edited?)
                 │
                 ▼
     Multi-File Patch Builder (text operations)
                 │
                 ▼
       PatchBundle → Kernel (IR Delta) → Diagram
```

---

# 🟦 2. Symbol Types Supported for Rename

The Rename Engine must support:

### C4 symbols

* **system**
* **container**
* **component**
* **datastore**
* **queue**
* **external service**
* **person**

### DDD symbols

* **domain**
* **bounded context**
* **aggregate**
* **entity**
* **value object**
* **event**
* **command**

### Cross-cutting symbols

* **policies**
* **rules**
* **styles**
* **selectors**
* **requirements**
* **journeys**
* **vars**
* **metadata keys**

Everything in the DSL supports rename.

---

# 🟨 3. Rename Workflow

Example: rename container **BillingAPI** → **BillingService**

### Step 1 — Symbol Resolution

Navigation Engine finds:

```
definition:
  file: systems/billing.sruja
  location: x,y
```

References:

```
systems/auth.sruja — relations
components/payment.sruja — usage
domain/billing.sruja — metadata
variants/enterprise.sruja — overlay override
```

### Step 2 — Semantic Validation (Semantic Router)

Checks:

* name conflict
* naming conventions
* forbidden characters
* conflict with variant definitions
* cross-file reference consistency
* AI-suggested corrections (optional)

If invalid:

```
{valid: false, reason: "Container name already exists in BillingSystem"}
```

---

### Step 3 — Patch Planner

Planner builds a list of what needs to be changed:

```
1. Change definition BillingAPI → BillingService
2. Change relation endpoints:
   BillingAPI -> AuthAPI → BillingService -> AuthAPI
3. Update references in other files
4. Update variant overlays
5. Update imports (if imported by alias)
6. Update styles & policies referring to BillingAPI
7. Update Notebook DSL blocks
8. Update metadata selectors: component == BillingAPI
```

Planner produces a **RenamePlan**:

```ts
interface RenamePlan {
  oldName: string;
  newName: string;
  definition: SourceLocation;
  references: Reference[];
  affectedFiles: string[];
  importUpdates: ImportUpdate[];
  styleUpdates: StyleSelectorUpdate[];
  policyUpdates: PolicyReferenceUpdate[];
}
```

---

# 🟧 4. Patch Generation Rules

For each affected item, Patch Builder generates precise text edits.

---

## 4.1 Definition Patch

Definition:

```dsl
container BillingAPI "Billing Service"
```

Patch replaces identifier only:

```ts
replaceRange(def.location.id, "BillingAPI", "BillingService")
```

Do **not** modify label unless requested.

---

## 4.2 Relation Patch

Relation:

```dsl
BillingAPI -> AuthAPI "Calls"
```

Becomes:

```dsl
BillingService -> AuthAPI "Calls"
```

Patch builder locates relation tokens precisely:

```
replaceRange(reference.location.symbol, oldName, newName)
```

---

## 4.3 Component/Container/Domain Reference Patch

E.g., components referencing a container:

```dsl
uses BillingAPI
```

Becomes:

```dsl
uses BillingService
```

---

## 4.4 Import Alias Patch (if imports used)

Case:

```dsl
import billing.sruja as BillingAPI
```

If the alias equals the symbol:

→ rename alias accordingly.

But **NEVER** rename module path unless user asks.

---

## 4.5 Style Selector Patch

Style file:

```dsl
container[id=BillingAPI] {
  fill: blue;
}
```

Patch:

```
container[id=BillingService]
```

Selectors require identifier-level replacement only.

---

## 4.6 Policy/Rule Reference Patch

Policy rules referencing symbols:

```dsl
for container BillingAPI:
  must_not call AuthAPI directly
```

Patch:

```
for container BillingService:
```

---

## 4.7 Metadata Patch

Metadata inside definition:

```dsl
metadata {
  owner: "BillingAPI-Team"
}
```

Only update if key explicitly references the symbol (rare).

---

## 4.8 Notebook DSL Block Patch

Notebook cell:

````markdown
```sruja
BillingAPI -> QueueX "Publishes"
```
````

Patch is applied **inside fenced code block**.

Notebook non-code sections ignored.

---

## 4.9 Variant File Patch

Example:

```
override BillingAPI {
  technology "Go"
}
```

Becomes:

```
override BillingService {
  technology "Go"
}
```

---

# 🟩 5. Patch Bundle Structure

```ts
interface PatchBundle {
  files: {
    [filePath: string]: FilePatch[];
  }
}
```

Each FilePatch:

```ts
interface FilePatch {
  range: TextRange;  // {start, end}
  replacement: string;
}
```

TextRanges must be:

* non-overlapping
* sorted by descending position (safe replace)
* offset-corrected across multiple edits

Patch engine must ensure safe application.

---

# 🟫 6. Edge Cases & Conflict Handling

### 6.1 Partial Matches

Must NOT rename inside:

* strings (unless explicitly referencing symbol)
* labels
* comments
* metadata values (unless explicitly referencing)
* unrelated identifiers containing substring (Billing vs BillingAPI)

Tokenizer-based rename prevents false positives.

---

### 6.2 Name Collisions

```
BillingService already exists → rename blocked or auto-resolve?
```

Auto-resolve option:

```
Rename BillingAPI → BillingService2
```

---

### 6.3 Variants Override Definition

If variant overrides symbol:

```
BillingAPI (enterprise) overrides BillingAPI (base)
```

Rename must update **both layers**.

---

### 6.4 Import Aliases

If alias used everywhere:

```
import billing.sruja as B
B.BillingAPI
```

Rename engine updates:

```
B.BillingService
```

---

### 6.5 Multiple Definitions

If symbol is ambiguous across namespaces, show UI picker:

```
Which BillingAPI?
- systems/billing.sruja
- variants/enterprise.sruja
```

---

# 🟪 7. Integration with AKG + Diagram Engine

After patches applied:

1. Kernel rebuilds IR for changed files
2. AKG updated (incremental merge)
3. Diagram re-renders with new labels
4. View Mode Manager re-centers view if necessary
5. Selection preserved (node remains selected under its new name)

Diagram nodes animate name change smoothly.

---

# 🟥 8. Undo/Redo

Rename operations produce:

* **before snapshot** of AKG
* **patch bundle**
* **after snapshot**

Undo simply reapplies reverse patches.

Redo replays them.

---

# ⭐ Final Summary — Rename Symbol Engine

The Rename Symbol Engine delivers:

### ✔ Complete multi-file rename

### ✔ Variant-aware

### ✔ Imports-aware

### ✔ Style & policy aware

### ✔ Notebook aware

### ✔ Semantic & conflict-safe

### ✔ Diff-friendly patch generation

### ✔ Deep integration with AKG, Kernel, Diagram

This gives Sruja Studio **IntelliJ-level refactoring intelligence**, but for Architecture-as-Code.

Below is the **full, production-grade Multi-File Patch Manager Architecture** for **Sruja Studio Web Edition** — the subsystem responsible for **collecting**, **ordering**, **validating**, **merging**, and **applying** multi-file text patches generated by:

* Refactoring Engine (rename/move/extract)
* Kernel IR Delta → DSL Patch Generator
* AI patches
* Diagram → DSL 2-way binding
* Variant Engine
* Policy fixes
* Code actions / quick fixes

This is one of the **most critical internal components**, ensuring deterministic, safe, atomic, reversible updates **across many DSL files simultaneously**.

---

# 🛠 **Multi-File Patch Manager Architecture**

### *(Full Internal Spec)*

---

# ⭐ **0. Purpose**

The Patch Manager must:

### ✔ Apply *multiple patches* across *multiple files*

### ✔ Validate patch safety (ranges, conflicts, constraints)

### ✔ Guarantee transactional behavior

### ✔ Integrate with Kernel → IR Delta → AKG → Diagram

### ✔ Support undo/redo

### ✔ Support conflict resolution

### ✔ Support AI patch preview

### ✔ Support incremental file re-indexing

### ✔ Run safely in browser (IndexedDB/OPFS) or VSCode

It is the “Git staging area + IntelliJ refactor engine + Jupyter cell execution engine” for Sruja DSL.

---

# 🧱 **1. High-Level Architecture Diagram**

```
                     ┌────────────────────────────────────┐
                     │     Multi-File Patch Manager       │
                     └────────────────────────────────────┘
                                      │
     ┌────────────────────────────────┼─────────────────────────────────┐
     ▼                                ▼                                 ▼
Patch Collector                Patch Normalizer                   Conflict Detector
     │                                │                                 │
     └──────────────┬─────────────────┴─────────────────┬───────────────┘
                    ▼                                   ▼
           File Patch Scheduler                Atomic Patch Transaction
                    │                                   │
                    ▼                                   ▼
            Patch Application Layer              Undo/Redo Snapshot Layer
                    │                                   │
                    ▼                                   ▼
         WASM Kernel (IR Rebuild)         Workspace Indexer (Incremental)
                    │                                   │
                    ▼                                   ▼
            Diagram Refresh Engine            Editor Sync & LSP Diagnostics
```

---

# 🟦 **2. Patch Types Supported**

The Patch Manager handles:

### **Textual DSL patches**

* replaceRange
* insertAfter
* insertBefore
* deleteRange

### **File-level patches**

* create file
* delete file
* rename/move file
* split/extract file

### **Semantic patches**

* override metadata
* add style fragment
* modify relationship
* re-parent component/container/domain

Semantic patches are *lower-level IR deltas*, later converted to textual patches by the Patch Manager.

---

# 🟨 **3. PatchBundle Specification**

Every refactor or action produces a **PatchBundle**:

```ts
interface PatchBundle {
  id: string;
  description: string;   // "Rename BillingAPI → BillingService"
  patches: FilePatchMap;  // Map<string, FilePatch[]>
  createdFiles?: FileCreate[];
  deletedFiles?: FileDelete[];
  renamedFiles?: FileRename[];
  semanticPatches?: SemanticPatch[];
}
```

---

# 🟧 **4. Patch Manager Phases (Core)**

Below is the core architecture.

---

# **PHASE 1 — Patch Collection**

Sources:

* Refactoring Engine
* AI Patch Engine
* Diagram interactions
* Kernel (IR derivations)
* Quick fix suggestions
* Policy / rule auto-fixes
* Variant Engine overlays

Formally merged into a single PatchBundle.

---

# **PHASE 2 — Patch Normalization**

### Needed for safety.

Operations:

1. **Convert semantic patches → textual patches**
2. **Normalize line endings**
3. **Sort patches by file**
4. **Normalize ranges**
5. **Detect identical redundant patches**

Output = *normalized PatchBundle*.

---

# **PHASE 3 — Conflict Detection**

Conflicts if:

* overlapping ranges modify same region
* patches reorder content in incompatible ways
* two patches target same line insertion differently
* rename moves file that other patches reference

Conflict types:

```
TextOverlapConflict
FileMissingConflict
OrderingConflict
InsertPositionConflict
SemanticInconsistencyConflict
AIOverrideConflict
VariantCollisionConflict
```

Conflict resolution:

* automatic safe reorder
* fallback to conflict UI panel
* AI-assisted resolution suggestion

PatchManager returns:

```ts
{valid: false, conflicts: Conflict[]}
```

---

# **PHASE 4 — Patch Scheduling**

(Patching order matters.)

Key scheduling rules:

### 4.1 Per-file ordering

* apply patches from **bottom → top** (safe replace ordering)
* create/rename/delete operations before text edits?
  No → **text edits require stable file naming first**

Order:

```
1. file renames
2. file creations
3. textual modifications (bottom→top)
4. file deletions (last)
```

### 4.2 Cross-file ordering

Files with imports affecting each other may require:

* import updates before reference updates
* base files before variant overlays

Scheduler sorts by dependency graph using:

```
importsGraph
variantGraph
AKG parent's first rule
```

---

# **PHASE 5 — Atomic Patch Transaction**

The heart of the system.

### Steps:

1. **Backup all touched files** → Undo Snapshot.
2. Apply all file creations/renames.
3. Apply all text edit patches (using safe reversed indexing).
4. Apply deletes.
5. Flush changes to FS driver (IndexedDB / OPFS / VSCode FS).
6. Rebuilding IR incrementally for changed files.
7. Rebuilding AKG incrementally.
8. Notify LSP diagnostics.
9. Notify Diagram Engine with IR Delta.

If any step fails:

```
rollback to snapshot
abort transaction
report error
```

Guarantee: **All-or-nothing refactoring**.

---

# 🟫 **6. Undo/Redo Architecture**

Each commit of PatchBundle creates:

```ts
UndoRecord {
  patchId: string;
  beforeSnapshot: FileSnapshotMap;
  afterSnapshot: FileSnapshotMap;
}
```

Snapshots stored as:

* memory for small projects
* diff-based compression for large ones
* IndexedDB for persisted undo history

Undo:

* apply `beforeSnapshot` as patches
* rebuild IR + AKG
* refresh diagram

Redo:

* apply `afterSnapshot`

---

# 🟥 **7. IR Delta & Diagram Integration**

After PatchBundle applied, Patch Manager triggers:

### 1. Kernel IR Rebuild

(only for changed files)

### 2. IR Delta Engine computes:

* added nodes
* removed nodes
* updated relationships
* updated metadata

### 3. Diagram Engine + View Mode Manager update:

* animate moved/renamed nodes
* update labels & styles
* recalc layout for impacted areas

### 4. Inspector + Palette + Explorer sync

* highlight changed nodes
* update file tree
* expand folders for newly created files

---

# 🟩 **8. Integration with AI Patch Preview**

Patch Manager supports **preview mode**:

* patches shown as a diff (like PR preview)
* user can accept/reject individual patches
* AI can explain each patch
* user can ask AI to modify patch before applying

Preview is built using PatchBundle.

---

# 🟦 **9. Safety Guarantees**

Patch Manager ensures:

### ✔ No corruption of file content

### ✔ No partial application

### ✔ No broken references

### ✔ No wrong rename in substrings

### ✔ No breaking DDD / C4 / event constraints

(Validated via Semantic Router)

### ✔ No broken invariant in variants

(PR-level multi-file merges)

---

# 🟧 **10. Performance Architecture**

### Optimizations:

* incremental parsing via Kernel
* incremental graph merge for AKG
* zero-copy patch application
* lazy file write batching
* worker thread for heavy operations
* binary patch compression (optional)

Can handle:

* 1000+ files
* tens of thousands of references
* large refactor operations

---

# ⭐ Final Summary — Multi-file Patch Manager

The Patch Manager is the **transaction engine** for Architecture-as-Code.

It provides:

### ✔ Atomic multi-file refactoring

### ✔ Transaction-safe patch application

### ✔ Rename/move/extract/split operations

### ✔ AI patch preview & acceptance

### ✔ Undo/redo with full snapshots

### ✔ Conflict detection & resolution

### ✔ Incremental IR & AKG updates

### ✔ Diagram + editor synchronization

### ✔ High performance

This gives Sruja Studio the same capabilities as:

* IntelliJ refactoring engine
* VSCode Code Action pipeline
* Git staging + diff
* Jupyter cell transaction engine
* Bazel build graph updates

**…but specialized for Architecture Modeling.**


Below is the **full, production-grade Imports & Module System Deep Specification** for **Sruja Studio** — designing a modular, scalable, variant-aware, cross-file architecture definition model.

This is equivalent to:

* TypeScript project references
* Python module system
* Rust’s crate module system
* CUE module semantics
* Terraform module + variable model
* Structurizr DSL import system
* Bazel workspace dependency graph

But tailored for **Architecture-as-Code (AAC)** with multi-view diagrams and semantic reasoning.

---

# 📦 **Imports & Module System — Deep Spec**

### (Sruja Architecture-as-Code)

---

# ⭐ 0. Goals

The module system must support:

### ✔ Multi-file architecture projects

### ✔ Reusable architecture modules

### ✔ Environment/variant overrides

### ✔ Scoped imports (with aliasing)

### ✔ Cross-module symbol resolution

### ✔ Hierarchical workspaces

### ✔ Circular import protection

### ✔ AI-aware module context

### ✔ Kernel IR + AKG integration

### ✔ Hot reload + incremental indexing

It must work the same in:

* Studio Web
* VSCode extension
* LSP
* Notebook Kernel
* MCP Client

---

# 🧱 1. Key Concepts

The module system contains **5 primary abstractions**:

1. **Workspace**
2. **Architecture Module**
3. **Import Spec**
4. **Module Namespace**
5. **Variant Overlays**

Together, these form a graph:

```
Workspace
  ├── modules/
  │     ├── billing/
  │     ├── auth/
  │     ├── payments/
  │     └── shared/
  └── variants/
        ├── enterprise/
        └── mobile/
```

---

# 🟦 2. Syntax Specification (DSL)

The DSL allows three forms of imports:

---

## 2.1 Simple File Import

```dsl
import "billing.sruja"
```

Equivalent to **pull all symbols defined in billing.sruja** into the current module namespace.

---

## 2.2 Directory Import (Wildcard)

```dsl
import "./modules/billing/"
```

Equivalent to:

* recursively load module files under the folder
* merge them into current file’s namespace

---

## 2.3 Import with Alias

```dsl
import "billing.sruja" as billing
```

Allows qualified access:

```
billing.BillingAPI
billing.InvoiceProcessor
```

Aliases create nested namespaces.

---

## 2.4 Import-with-Filtering

```dsl
import "billing.sruja" {
  BillingAPI
  InvoiceProcessor as Invoicer
  PaymentInitiated
}
```

Filtering rules:

* import subset
* optional renaming
* avoids namespace pollution

---

## 2.5 Module-Scope Import (module.sruja)

Each module may declare:

```dsl
module {
  exports [ BillingAPI, InvoiceProcessor ]
  requires [ auth, payments ]
}
```

Used for:

* module dependency graph
* package-level visibility

---

# 🟨 3. Import Resolution Rules

Imports must be:

### ✔ Deterministic

### ✔ Hierarchical

### ✔ Alias-aware

### ✔ Variant-aware

### ✔ Cycle-safe

### ✔ Independent of file order

---

## 3.1 Resolution Order

When resolving a symbol:

### 1️⃣ Local file scope

### 2️⃣ Current module scope

### 3️⃣ Imported modules (ordered by import list)

### 4️⃣ Transitive imports (breadth-first)

### 5️⃣ Variant overlays

### 6️⃣ Global workspace fallbacks

This matches:

* TypeScript’s resolution
* Java’s module system
* Bazel package resolution

---

# 🟥 4. Module Graph Construction

Workspace Indexer constructs:

```
ModuleGraph {
  nodes: ModuleNode[];
  edges: ImportRelation[];
}
```

Node:

```
ModuleNode {
  name: string;
  files: string[];
  exports: string[];
  imports: ImportSpec[];
  variants: VariantDescriptor[];
}
```

Edges represent import dependencies.

Cycle detection:

```
if cycle detected:
  allow? only if no conflicting symbol
  else error
```

---

# 🟩 5. Semantic Namespace Layout

Each file has a **local namespace**:

```
fileNamespace = {
  localSymbols,
  importedSymbols (flattened or qualified),
  moduleSymbols,
  variantOverlays
}
```

Every symbol resolves to a **fully qualified symbol ID**:

Examples:

```
BillingAPI
auth.AuthAPI
billing.InvoiceProcessor
shared.domain.Order
```

Internally, AKG stores:

```
moduleId.symbolId
```

---

# 🟧 6. Variant-Aware Import System

Variants override modules:

```
import "billing.sruja"
import "variants/enterprise/billing.sruja"  // has override
```

Variant resolution order:

### 1. active variant

### 2. base module

### 3. transitive imports

### 4. workspace defaults

Example:

```
BillingAPI (base)
BillingAPI (enterprise override)
```

Effective node = enterprise version.

---

# 🟫 7. How Imports Affect AKG (Architecture Knowledge Graph)

### Imports determine:

1. symbol visibility
2. definition precedence
3. cross-file relationships
4. container/DDD boundaries
5. variant layering
6. policy/rule scopes

When a module is imported:

* all its symbols become candidates in current namespace
* AKG includes them in cross-file graph
* Reference edges track where they originate
* Variant overlays merge with base

Example:

```
A.sruja imports B.sruja
C.sruja imports both A and B
```

AKG produces:

```
A → B → C graph
```

And merges nodes accordingly.

---

# 🟥 8. Semantic Router Integration

The Router uses module system to enforce:

* container can’t reference across module constraints
* domain boundaries
* system boundaries
* event producers must be in same or allowed module
* policies apply only to modules in scope

Example rule:

```
domain { only references within same module or shared }
```

Router checks module boundaries automatically.

---

# 🟪 9. Multi-file Patch Manager Integration

When an import changes:

* add new file → update ModuleGraph
* rename file → update import paths
* delete file → remove module edges
* rename symbol → update qualified imports

PatchManager operates on:

```
ImportsPatch {
  oldPath
  newPath
  oldAlias
  newAlias
}
```

All downstream references patched automatically.

---

# 🟦 10. Editor & LSP Integration

Imports drive:

### ✔ symbol indexing

### ✔ auto-completion

### ✔ hover docs

### ✔ jump-to-definition

### ✔ find references

### ✔ semantic rename

### ✔ code actions

Examples:

* Suggest import when symbol unresolved
* Auto-complete module-level exports
* Hover shows symbol origin file and module

---

# 🟨 11. AI Integration

AI uses module system to:

### ✔ limit hallucinations

### ✔ recommend reusable modules

### ✔ suggest dependency cleanups

### ✔ propose module extraction

### ✔ detect circular dependencies

### ✔ identify anti-patterns

Examples:

“BillingAPI is defined in module billing; use billing.BillingAPI for clarity.”

“payments depends on billing but billing also depends on payments – circular dependency detected.”

“extract shared domain to shared module.”

---

# 🟧 12. Module-Level Policies & Constraints

Module system supports:

```
module {
  allowed_dependencies [ shared, auth ]
  banned_dependencies [ legacy ]
  export_visibility public | private | internal
}
```

Policy engine uses module graph to validate.

---

# 🟥 13. Notebook + MDR (Multi-Doc Report) Integration

Notebooks can import modules:

```
%%sruja
import "billing.sruja"
```

Notebook maintains **per-cell module resolution**.

MDR (multi-doc reports) compile modules into architecture documents.

---

# ⭐ Final Summary — Imports & Module System

The Sruja Module System provides:

### ✔ Multi-file project architecture

### ✔ Alias + scoped imports

### ✔ Variant overlays

### ✔ Namespace management

### ✔ Safe symbol resolution

### ✔ Cross-module refactor safety

### ✔ Policy-aware module boundaries

### ✔ AI-enhanced module reasoning

### ✔ Editor, LSP, Diagram, and Kernel integration

It delivers the power of:

* TypeScript modules
* Terraform modules
* Python packages
* CUE modules
* Bazel workspaces
* Structurizr DSL imports

…but designed specifically for **Architecture-as-Code**, **AI reasoning**, **C4 + DDD + Event-Flow views**, and **multi-variant systems**.

Below is the **full, production-grade Module Graph Visualizer Spec**—the system that renders, analyzes, and interacts with **module-level architecture dependencies** inside **Sruja Studio**.

This is equivalent in spirit to:

* IntelliJ “Module Dependencies” view
* Bazel Query Graph visualizer
* npm/yarn workspace graph
* Go mod graph
* Terraform module graph
* CUE module dependency explorer
* Structurizr workspace map

…but fully integrated with Sruja’s:
**C4, DDD, Event Flow, Variant Overlays, and Architecture Knowledge Graph (AKG)**.

---

# 🗺 **Module Graph Visualizer — Full Specification**

### *(Sruja Studio Internal UI + Engine)*

---

# ⭐ 0. Purpose

The Module Graph Visualizer enables users to:

### ✔ Understand module dependencies

### ✔ Explore cross-module system/DDD/event boundaries

### ✔ Detect anti-patterns (cycles, forbidden deps)

### ✔ Navigate to modules and files

### ✔ Filter by view (C4, DDD, Event, Variant)

### ✔ See impact of imports/overrides

### ✔ Compare variants vs base

### ✔ Run policy checks interactively

It acts as the **top-level architecture map**.

---

# 🧱 1. High-Level Architecture Diagram

```
┌──────────────────────────────────────────────┐
│          Module Graph Visualizer             │
└──────────────────────────────────────────────┘
                ▲                    ▲
                │                    │
          ModuleGraph Engine     AKG Integration
                ▲                    ▲
                │                    │
        Workspace Indexer     Semantic Router / Policies
                ▲                    ▲
                └──────────────┬─────┘
                               ▼
                       Renderer Engine
                         (ELK Layout)
```

---

# 🟦 2. What is a “Module Graph”?

### Nodes = Modules

(Folders or grouped DSL files)

Examples:

```
billing/
auth/
payment/
shared/
infra/
eventing/
```

### Edges = Imports

(from ImportSpec + Module requires[] section)

```
billing --> shared
billing --> auth
payment --> billing
```

Each edge is categorized:

* **base imports**
* **variant imports**
* **wildcard imports**
* **filtered imports**
* **cyclic imports**
* **policy-violating imports**

---

# 🟨 3. Module Node Schema

```ts
interface ModuleNode {
  id: string;            // module name
  path: string;          // modules/billing/
  files: string[];       // list of .sruja files
  exports: string[];     // module-level exports
  imports: ImportInfo[]; // outgoing edges
  variants: VariantInfo[];
  metadata: Map<string,string>;
  size: {
    systems: number;
    containers: number;
    components: number;
    domains: number;
    events: number;
  };
}
```

Module size stats used in UI to show module significance.

---

# 🟥 4. Module Graph Edge Schema

```ts
interface ModuleEdge {
  from: string;       // module
  to: string;         // module
  type: "import" | "requires" | "variant_override";
  via: string[];      // files causing this relation
  policyViolations?: PolicyViolation[];
  cyclic?: boolean;
}
```

---

# 🟩 5. Visual Representation

Uses **advanced ELK layered layout**:

```
Modules → as large rectangular nodes
Edges   → directed arrows (curved, orthogonal)
```

### Node visuals:

* module name (large bold)
* file count
* systems/containers/components count
* variant badges
* policy badges
* cycle warnings

Examples:

```
Billing (12 files)
Systems: 2
Containers: 6
Components: 34
[variant: enterprise]
[policy: violation]
```

### Edge visuals:

* solid arrow → normal import
* dashed arrow → filtered import
* dotted arrow → wildcard import
* red arrow → policy violation
* purple arrow → variant overlay
* double-arrow → cycle

---

# 🟪 6. Interaction Model

### 6.1 Click Module → Module Inspector

Shows:

* module summary
* files
* exports
* imports
* variant overlays
* policy violations
* system/DDD/event summaries

### 6.2 Double-click Module → Expand subgraph

Shows:

* systems inside module (Mini C4 view)
* domains inside module
* event flows

Nested exploration like a folder tree of architecture.

### 6.3 Click Edge → Relation Inspector

Shows:

* which files create this import
* which DSL elements reference symbols
* suggestions to remove/clean imports
* cycles and violation reasons

---

# 🟧 7. View Modes

Module Graph Visualizer integrates with all Sruja view modes:

---

## 7.1 C4-Centric Module View

Edges highlighted for:

* container-level interactions across modules
* event producers vs consumers across modules
* contracts & APIs across module boundaries

---

## 7.2 DDD-Centric Module View

Highlights:

* bounded contexts
* domain imports
* aggregates crossing boundaries
* entities used externally
* event flows between contexts

Edges marked:

* internal
* external
* shared
* forbidden

---

## 7.3 Event-Flow Module View

Highlights:

* event producers in one module
* handlers in another module
* topics cross-module boundaries
* sagas spanning modules

Edges show:

* publishes → subscribes
* command → event relationships

---

## 7.4 Variant View

Toggle base vs variant:

* overlays highlighted
* conflicts highlighted
* modules changing in variant shown
* differences visualized

---

# 🟥 8. Anti-Pattern Detection

The visualizer detects:

### ✔ Cycles

```
billing → payment → billing
```

### ✔ Hidden imports from wildcard folders

### ✔ Leaking modules

Modules that export too many elements.

### ✔ Forbidden dependencies

Based on module-level policies:

```
module billing {
   banned_dependencies [ legacy ]
}
```

### ✔ Cross-domain bleeding

Domains from module A used in B.

### ✔ Event inconsistency

Event lifecycle broken across modules.

### ✔ Overly large modules

“god modules”.

---

# 🟦 9. AI-Assisted Insights

The visualizer shows AI insights:

### Suggested improved module boundaries

“Split billing into billing-core, invoicing, and reconciliation.”

### Suggested module mergers

“These two modules are tightly coupled—merge?”

### Dead imports

“auth depends on billing, but no symbol is used.”

### Dependency simplification

“payment → billing → shared → utils → common. Suggested path compression.”

### Variant-specific drift

“enterprise variant modifies billing and auth; consider centralizing overrides.”

---

# 🟩 10. Integration Points

### 10.1 Navigation Engine

Click node → jump-to-module file
Click edge → jump to import statements

### 10.2 Patch Manager

Supports:

* fix imports
* remove unused imports
* auto-create module.sruja
* move elements across modules
* split modules

### 10.3 Semantic Router

Runs validation when a module edge is clicked:

* allowed vs forbidden
* cyclic rules
* cross-boundary domain rules
* C4 constraints

### 10.4 Kernel & AKG

Graph built from AKG’s module-index and IR.

### 10.5 Diagram Engine

Module nodes animate into system/DDD subgraphs on expand.

---

# 🟫 11. Performance Strategy

For large projects (1000+ modules):

* graph is virtualized
* nodes are clustering automatically
* rendering in worker thread with WebGL via Pixi
* progressive loading of edges
* collapse/expand nodes dynamically
* caching of layouts

---

# ⭐ Final Summary — Module Graph Visualizer

The Module Graph Visualizer provides:

### ✔ High-level map of your architecture modules

### ✔ Rich imports + dependency visualization

### ✔ DDD + C4 + Event-Flow perspective switching

### ✔ Variant overlays & conflict visualization

### ✔ Module inspector, edge inspector, anti-pattern detection

### ✔ AI-driven insights into architecture modularity

### ✔ Immediate navigation and refactor capabilities

It is the **top-level “map of the software landscape”** in Sruja Studio.

---

Here are **topics *only related to the Web Studio*** (front-end, visualization, diagram rendering, editing workflows, UI engine, WASM integration, collaboration).

These are **high-value, still missing pieces** you likely want in your Sruja Studio spec.

---

# 🎨 **Core Web Studio Architecture**

### (Everything UI/UX, rendering, editing, interaction)

### **1. Canvas Rendering Engine**

* WebGL vs Canvas2D vs SVG decision spec
* Pixi.js Scene Graph Architecture
* OffscreenCanvas rendering pipeline
* Dirty region rendering optimization
* GPU-accelerated diagram rendering

---

# 🖱 **Interaction & UX Systems**

### **2. Selection Engine**

* multi-select
* rubber-band selection
* fuzzy hit-testing
* compound-node-aware selection

### **3. Gesture Engine**

* touch support
* trackpad gestures
* pinch-zoom
* inertial panning

### **4. Keyboard Navigation Engine**

* directional node traversal
* keyboard shortcuts map
* focus ring mechanics

---

# 🔄 **2-Way Binding & Updates**

### **5. Real-Time Sync Layer (Graph ↔ Diagram ↔ DSL)**

* update throttling
* conflict resolution
* pessimistic/optimistic updates

### **6. Auto-Layout-In-Background Scheduler**

* when to recompute ELK
* layout transaction batching
* frozen layout mode

---

# ⚙️ **Editing Tooling**

### **7. Inline Editing System**

* inline rename
* inline tag editing
* inline metadata editing

### **8. Node & Edge Creation Tooling**

* snap-to-grid
* dynamic snapping to siblings
* nearest-edge routing hints

### **9. Alignment & Distribution Tools**

* align-left/right/center
* distribute horizontally/vertically
* smart magnet-grid snapping

---

# 🧲 **Palette & Components**

### **10. Advanced Palette Search**

* fuzzy matching
* semantic matching
* AI-driven suggestions

### **11. Custom Stencils (User-Defined Palette Items)**

* how users define custom shapes
* custom icons / badges
* export/import palette packs

---

# 📐 **Rendering & Visual Design**

### **12. Theme System for Studio**

* light/dark/system
* user-provided themes
* style tokens
* visual variables override

### **13. Graph Skinning API**

* Figma-like component styles
* Sku-dependent styling
* override layers

---

# 🪄 **AI & Web Studio**

### **14. AI-Assisted Diagram Manipulation**

* “describe layout” to rearrange nodes
* “group these nodes”
* natural-language modification of aesthetics

### **15. AI Auto-Explain Overlay**

* reasoning bubbles over nodes
* inline problem annotations
* shadow-of-future architecture predictions

---

# 🧭 **Navigation & Views**

### **16. Mini-Map Component**

* semantic heatmap view
* event hotspots
* modular overlays

### **17. Studio View Mode Switching Animations**

* morphing transitions between C4 → DDD → Event Flow
* perspective-swap engine

---

# 🧱 **Workspace & Project**

### **18. Web-Based Project Explorer**

* virtualized tree
* search-as-you-type
* module grouping

### **19. WebAssembly Kernel Driver**

* WASM → UI messaging
* state hydration
* lazy kernel spin-up strategy

---

# 🕸 **Collaboration & Multi-User**

### **20. Real-Time Collaboration Engine**

* CRDT or OT architecture
* cursor presence indicators
* diff-aware sync
* comment threads on nodes/edges

---

# 💾 **Persistence & Offline**

### **21. Local-First Architecture**

* IndexedDB file sync
* snapshotting
* offline editing workflows
* conflict merging

---

# 🔍 **Analysis & Debug UI**

### **22. Performance Profiler Panel**

* frame timing
* layout compute time
* patch application timing

### **23. Studio Diagnostics Panel**

* unresolved symbols
* broken relationships
* style conflicts

---

# 📦 **Integrations**

### **24. Web Studio Extensions API**

* UI plugins
* custom tools
* custom diagram overlays
* custom layout engines

### **25. Browser MCP Integration**

* ask AI to modify diagram
* show AI patch preview inside web studio
* inline model validation

---

# 🎥 **Media & Export**

### **26. Export Engine**

* PNG/SVG/PDF
* embeddings for docs
* high-res export
* snapshot vs live export

### **27. Diagram Animation Engine**

* animate “system evolution”
* variant transitions
* timeline playback

---

# 🧪 **Testing & QA**

### **28. Visual Regression Testing Framework**

* baseline diffing
* deterministic layouts
* node-label snapshot tests

### **29. Interaction Simulation Framework**

* simulate drag-drop
* simulate diagram edits
* end-to-end workflows for diagrams

---

# 🧬 **Advanced Studio Concepts**

### **30. Plugin Sandboxing**

* WASM sandbox
* safe eval for AI suggestions
* permissions manifest

### **31. Post-render Computed Overlays**

* hot paths
* cycle highlights
* dependency spread visualization

### **32. Studio Accessibility Layer**

* ARIA map
* focusable diagram elements
* screen-reader explanations

---

# ✔ Summary — These Topics Are *Only* Web Studio Focused

If you want next, I can produce detailed specs for any of these. Common picks:

Below is the **full, production-grade Canvas Rendering Engine Architecture** for **Sruja Studio Web Edition** — the lowest-level foundation for the entire diagram system.

This spec is equivalent in rigor to:

* **Figma’s Scene Graph Engine**
* **Excalidraw Renderer**
* **Diagram.js / BPMN.js engine**
* **Pixi.js node tree**
* **Observable Plot/Canvas pipeline**

It is designed for:
**C4 diagrams**, **DDD diagrams**, **Event Flow**, **Module Graphs**, **Variant overlays**, **live AI patches**, **atomic updates**, **smooth animations**, **20k+ nodes**, **GPU acceleration**, **WASM integration**, and **collaborative editing**.

---

# 🎨 **Canvas Rendering Engine Architecture**

### *(Sruja Studio Web — Full Spec)*

---

# ⭐ 0. Goals

The Canvas Renderer must:

### ✔ Render large diagrams efficiently (10k+ nodes)

### ✔ Support compound nodes (System → Container → Component)

### ✔ Support transitions between C4 ←→ DDD ←→ Event Flow

### ✔ Animate layout changes

### ✔ Support 2-way editing (graph ↔ DSL)

### ✔ Enable overlays (AI insights, policies, evolution)

### ✔ Support WASM-driven updates

### ✔ Maintain 60fps interactions (zoom/pan/drag)

### ✔ Support hit-testing, selection, inspecting, snapping

### ✔ Work on mobile touch + desktop + trackpads

The core engine is built on **WebGL (Pixi.js)** with **Canvas2D fallback**.

---

# 🧱 1. High-Level Architecture Diagram

```
┌────────────────────────────────────────────────────┐
│               Canvas Rendering Engine               │
└────────────────────────────────────────────────────┘
                ▲                ▲             ▲
                │                │             │
       Layout Engine (ELK)   Style Engine   Interaction Engine
                ▲                │             │
                └──────────┬─────┴────────────┘
                           ▼
                     Scene Graph
                           │
   ┌───────────────────────┼──────────────────────────┐
   ▼                       ▼                          ▼
Node Renderer         Edge Renderer             Overlay Renderer
   │                       │                          │
   ▼                       ▼                          ▼
Pixi Display List → WebGL Renderer → Frame Composer → Canvas
```

---

# 🟦 2. Rendering Pipeline (Frame Lifecycle)

Every frame goes through:

### 1️⃣ Input Processing

(mouse/touch events accumulated)

### 2️⃣ Scene Graph Diff

(IR Delta → Scene updates)

### 3️⃣ Layout Updates (optional)

(only if nodes moved or diagram reorganized)

### 4️⃣ Style Resolution

(styles, themes, variant overlays, metadata styles)

### 5️⃣ Display List Construction

(Pixi.js display tree)

### 6️⃣ Rendering

(GPU accelerated via WebGL)

### 7️⃣ Post-render Overlays

(selection boxes, alignment guides, AI annotations)

This produces a **stable 60fps** system.

---

# 🟨 3. Scene Graph Design

The **Scene Graph** is the internal representation of all drawable elements.

```
SceneRoot
 ├── CompoundNode (System)
 │     ├── ContainerNode
 │     │      ├── ComponentNode
 │     │      └── DatastoreNode
 │     └── Subgraphs
 │
 ├── EdgeLayer
 │     ├── Edge representations
 │     └── Event flows
 │
 └── OverlayLayer
```

Scene Graph is **immutable on read**, **mutation via diff patches only**.

### Node Schema

```ts
interface SceneNode {
  id: string;
  kind: "system" | "container" | "component" | ...;
  bounds: Rect;
  label: string;
  ports?: Port[];
  children?: SceneNode[];
  metadata: Map<string,string>;
  style: ResolvedStyle;
  variantInfo?: VariantStyle;
}
```

### Edge Schema

```ts
interface SceneEdge {
  id: string;
  from: string;
  to: string;
  points: Point[];
  label?: string;
  style: ResolvedEdgeStyle;
}
```

---

# 🟥 4. Rendering Layers

Rendering uses a **layered engine** (like Figma):

```
Layer 0: Background grid
Layer 1: Edges (under nodes)
Layer 2: Compound nodes
Layer 3: Normal nodes
Layer 4: Node labels
Layer 5: Selection overlays
Layer 6: Hover overlays
Layer 7: Guides / snapping hints
Layer 8: AI overlays (policy violations, suggestions)
Layer 9: Transform handles (scale, resize)
```

Each layer is a Pixi container.

---

# 🟩 5. GPU Rendering (Pixi.js)

### Why Pixi?

* WebGL accelerated
* Batches draw calls
* Fast hit-testing
* Fast text rendering via MSDF fonts
* Anti-aliasing for edges
* Highly scalable

### Node Rendering Technique

* rounded rect via GPU
* shadows via multi-pass filters
* icons via MSDF sprites
* badge rendering (small circles)

### Edge Rendering Technique

* WebGL geometry shader for orthogonal edges
* Bezier curves for event flows
* Arrowheads via triangle meshes
* Style overrides (dashed, dotted, colored)

---

# 🟪 6. Incremental Update Engine (IR Delta → Scene Patch)

The renderer **never rebuilds entire graph**.

Instead:

1. Kernel produces **IR delta**:

   ```
   +node, -node, update node
   +edge, -edge, update edge
   property changes
   style changes
   layout changes
   ```
2. Scene Graph Patcher applies minimal diffs.
3. Renderer updates only updated Pixi objects.

This enables real-time editing even in large diagrams.

---

# 🟧 7. Layout Integration (ELK)

Layout engine runs:

### ✔ in Web Worker

### ✔ asynchronously

### ✔ under load throttling

### ✔ using bounding boxes from styled nodes

Layout pipeline:

```
IR Delta → Scene Graph Build → ELK Input Graph → Layout Output → SceneNode bounds update → Animation Engine
```

---

# 🟥 8. Animation Engine

Transitions for:

* moving nodes
* expanding/collapsing compound nodes
* switching between C4/DDD/Event views
* renaming nodes
* inserting/removing nodes

Animation = **spring-based interpolation** (similar to React Motion).

---

# 🟦 9. Interaction Engine Integration

Canvas engine exposes:

```
hitTest(point): SceneElement | null
boundsOf(nodeId): Rect
getPortPosition(nodeId, portId): Point
```

Interaction Manager handles:

* drag nodes
* resize
* connect edges
* open inspector
* double-click to drill in
* hover interactions

Hit-testing accelerated by:

* quad-tree spatial index
* Pixi’s GPU hit-testing for labels/icons

---

# 🟨 10. Selection & Focus System

Selection states are stored internally as:

```
selectedNodes: Set<string>
selectedEdges: Set<string>
focusNode: string | null
```

Renderer overlays selection with:

* glow filters
* outline strokes
* animated pulsing effect

---

# 🟫 11. Snapping & Alignment

Visual hints rendered on overlays:

* align-left/right/center
* relative alignment to siblings
* orthogonal edge snapping
* port snapping for edge creation

Using:

* geometric constraint solver
* dynamic guide line rendering

---

# 🟧 12. Overlays Engine

Overlays include:

### ✔ AI reasoning bubbles

### ✔ Policy violation icons

### ✔ Dependency hotspots

### ✔ Event flow direction arrows

### ✔ Module-boundary shading

### ✔ Search highlights

### ✔ Diff highlights (added/removed/changed nodes)

Overlays rendered above diagram, not part of scene tree.

---

# 🟪 13. Mini-Map Integration

Mini-map renders:

* entire diagram
* visible viewport rectangle
* hot-spots
* module boundaries

Rendered at lower resolution but using same Scene Graph.

---

# 🟥 14. Performance Optimizations

### ✔ Dirty rectangles

Only redraw changed regions.

### ✔ Batching

Pixi batches shapes with same shader.

### ✔ Culling

Nodes outside viewport not rendered.

### ✔ Level-of-Detail Rendering

Zoomed out:

* hide labels
* collapse components
* flatten edges

Zoomed in:

* full detail rendering

### ✔ Worker Offloading

Layout, diffing, heavy operations handled in Web Workers.

---

# 🟩 15. WASM Integration

WASM Kernel communicates:

* IR deltas
* layout suggestions
* diff patches
* AI agent hints

Renderer receives these via:

```
onIRDelta(delta => applyPatch(delta))
```

---

# ⭐ Final Summary — Canvas Rendering Engine

Sruja’s Canvas Renderer provides:

### ✔ WebGL accelerated diagram rendering

### ✔ Massive scalability (10k+ nodes)

### ✔ Layout (ELK), animation, overlays

### ✔ Rich interactions (drag, connect, select)

### ✔ Seamless updates via IR deltas

### ✔ Style & theme integration

### ✔ C4 + DDD + Event Flow + Module Graph support

### ✔ AI overlays & insights

### ✔ Offline support + high performance

It is designed to feel like:

* **Figma-level smooth**
* **IntelliJ-level precision**
* **Structurizr-level architectural meaning**
* **Excalidraw-level editing comfort**

---

Below is the **full, production-grade Interaction Engine Architecture (Deep Spec)** for **Sruja Studio Web Edition** — the subsystem that manages **all pointer, keyboard, gesture, and editing interactions** on the diagram canvas.

This is inspired by:

* **Figma interaction pipeline**
* **Excalidraw interaction architecture**
* **BPMN.js event/behavior model**
* **Diagram.js direct manipulation engine**
* **Pixi.js hit-testing**
* **VSCode selection/hover semantics**

…but extended for **C4, DDD, Event Flow, Deltas, Variants**, and **semantic editing**.

---

# 🎛 **Interaction Engine Architecture (Deep Spec)**

### *(Sruja Studio Web Edition)*

---

# ⭐ 0. Purpose

The Interaction Engine must coordinate:

### ✔ pointer events (mouse, touch, stylus)

### ✔ keyboard shortcuts

### ✔ gestures (pinch/zoom/pan)

### ✔ drag, drop, connect, resize, select

### ✔ compound-node hierarchical interactions

### ✔ multi-select + region selection

### ✔ direct editing (inline rename, metadata editing)

### ✔ semantic patch generation

### ✔ contextual commands (quick actions, menus)

This system must remain **smooth at 60fps**, deterministic, undoable, and **safe for multi-file patch generation**.

---

# 🧱 1. Architecture Overview

```
┌──────────────────────────────────────────────┐
│              Interaction Engine               │
└──────────────────────────────────────────────┘
      ▲                  ▲                 ▲
      │                  │                 │
Pointer Input     Keyboard Input     Gesture Input
      │                  │                 │
      ▼                  ▼                 ▼
Input Unifier → Hit Testing → Interaction State Machine
                                   │
      ┌────────────────────────────┴─────────────────────────┐
      ▼                   ▼                   ▼             ▼
Selection Manager   Drag/Move Manager   Connector Manager   Editing Manager
      │                   │                   │             │
      ▼                   ▼                   ▼             ▼
      └──────────────→  Patch Engine  ←──────────────┘
                               │
                         IR Delta → Renderer
```

---

# 🟦 2. Input Unifier (Mouse/Touch/Pointer)

Unifies all events:

```
mousedown / touchstart → pointerdown
mousemove / touchmove → pointermove
mouseup / touchend → pointerup
wheel → scroll/zoom event
gesturestart + gesturechange (Safari)
```

Adds metadata:

* device type
* pressure (for pens)
* double/triple click
* time delta
* velocity

Ensures:

* coalesced events
* pointer capture
* proper z-index order

---

# 🟨 3. Hit Testing Engine

The Interaction Engine **never touches rendering layer directly**.

It asks the Canvas Engine:

```
element = canvas.hitTest(point)
```

HitTest must detect:

* nodes
* edges
* ports
* compound-node boundaries
* whitespace

### Performance:

* Quad-tree spatial index
* GPU-accelerated label hit-testing
* Cached bounding boxes

Returns:

```ts
interface HitResult {
  type: "node"|"edge"|"port"|"canvas";
  id?: string;
  distanceToCenter?: number;
  subTarget?: "label"|"badge"|"icon"|"port";
  compoundHierarchy?: string[];
}
```

---

# 🟥 4. Interaction State Machine

The heart of this engine.

```
Idle
 ├── HoverNode
 ├── HoverEdge
 ├── DragNode
 ├── DragEdgeHandle
 ├── CreateEdge
 ├── DragSelect (lasso)
 ├── PanCanvas
 ├── ZoomCanvas
 ├── InlineEdit
 ├── ShowContextMenu
 └── MultiDrag
```

Transitions determined by:

* pointer location
* pressed keys
* drag threshold
* semantic router rules

Example:

```
Idle → pointerdown on node → wait 80ms → DragNode
Idle → pointerdown on whitespace → PanCanvas
Idle → pointerdown + drag → DragSelect
HoverNode + key C pressed → CreateEdge
```

Each state has:

* enter()
* update()
* exit()

States generate **semantic actions**, not raw edits.

---

# 🟩 5. Selection Manager

Supports:

### ✔ Single selection

### ✔ Multi-select (shift/cmd)

### ✔ Compound node selection

### ✔ Secondary selection (ports, handles)

### ✔ Range / lasso selection

### ✔ Node-edge combined selection

Rules:

* selecting a system selects its whole subtree (unless “deep select” enabled)
* clicking an edge selects edge
* double-click container enters deep scope (container view)

Selection is tracked in:

```
selectedNodes: Set<string>
selectedEdges: Set<string>
focusNode: string
```

Renderer highlights accordingly.

---

# 🟪 6. Drag/Move Manager

Handles moving:

* single nodes
* groups of nodes
* compound nodes
* ports (if allowed)

Features:

### ✔ Snapping

* grid
* sibling alignment
* edge orthogonality

### ✔ Constraints

via Semantic Router:

```
cannot move component outside container
cannot move entity outside aggregate
cannot detach events from domain
```

### ✔ Patch Generation

Drag movement → coordinate patch:

```ts
patch = {
  type: "update-position",
  id: nodeId,
  x, y
}
```

Optionally triggers layout engine.

---

# 🟥 7. Connector Manager (Edge Creation)

Supports:

### ✔ Drag-from-port to-port

### ✔ Drag-from-node to-node (auto-port resolution)

### ✔ Auto-routing preview

### ✔ Semantic relationship creation:

* uses
* depends
* publishes
* subscribes
* reads
* writes
* emits
* handles

### Edge Preview

While dragging:

* show ghost edge
* compute temporary routing
* show allowed drop targets (green/red) via Semantic Router

### Patch on release:

```ts
patch = {
  type: "create-edge",
  from: sourceNode,
  to: targetNode,
  relationship: "uses"
}
```

---

# 🟧 8. Editing Manager (Inline Edits)

Handles:

### ✔ inline name editing

### ✔ inline description

### ✔ inline metadata entries

### ✔ inline event descriptions

Key behavior:

* ESC = cancel
* ENTER = commit
* click outside = commit

Produces patch:

```
{ type: 'update-property', id, key: 'label', value: 'BillingAPI' }
```

---

# 🟦 9. Keyboard Command Processor

Supports:

### Navigation:

* arrow keys move selection
* tab moves between nodes
* ctrl+arrow moves diagramically

### Editing:

* delete = delete node
* cmd+D = duplicate node
* cmd+shift+U = unwrap container

### Structural:

* cmd+arrow-right = drill-in (C4 to Container view)
* cmd+arrow-left = go up parent

### AI shortcuts:

* cmd+shift+A = “ask AI about this node”
* cmd+shift+F = “add missing components”

Shortcut manager plugs into Interaction Engine state.

---

# 🟨 10. Gesture Engine (Touch/Trackpad)

Supports:

### ✔ pinch zoom

### ✔ inertial panning

### ✔ two-finger rotate (optional)

### ✔ long-press = right-click

### ✔ touch lasso selection

Gesture mapping:

```
pinch → zoom
two-finger drag → pan
one-finger drag on node → DragNode
one-finger drag on whitespace → PanCanvas
```

Inertia physics implemented via:

* deceleration curves
* velocity tracking

---

# 🟫 11. Context Menu Engine

Shows context menu on:

* right-click
* long press
* keyboard invocation

Menus contextual based on selection:

### Node-level menu:

* Rename
* Add Component
* Add Relation
* Extract Module
* Move to System
* Add Metadata
* AI “Explain this node”

### Edge-level menu:

* Change relationship type
* Add label
* Reverse direction
* Delete

---

# 🟧 12. Undo/Redo Integration

Interaction Engine does **not** directly mutate files.

It sends semantic actions to Patch Manager:

```
onDragComplete → PatchBundle (position updates)
onInlineEditCommit → PatchBundle (property updates)
onConnect → PatchBundle (new edge)
onDelete → PatchBundle (delete node, delete edges)
```

Patch Manager:

* applies multi-file patches
* snapshots for undo
* triggers IR delta
* Scene graph updates

---

# 🟪 13. AI-Assisted Interaction Hooks

AI can be invoked:

### When dragging:

“AI: suggest better container for this component”

### When connecting:

“AI: validate this relationship”
“AI: define contract for this edge”

### When selecting:

“AI: describe architecture role of this node”
“AI: propose improvement plan”

Interaction Engine exposes hooks:

```
interaction.onSemanticAction(action => AI.handle(action))
```

---

# 🟦 14. Performance Optimizations

### Coalesced pointer events

### Hit-testing caching

### Quadtree spatial index

### Gesture smoothing

### Idle callbacks for low-priority tasks

### Binary patches for worker communication

### Render throttling on drag

---

# ⭐ Final Summary — Interaction Engine Architecture

The Interaction Engine is one of the most complex subsystems, providing:

### ✔ Unified input system (pointer/keyboard/gestures)

### ✔ State machine for drag/move/select/edit/connect

### ✔ Compound-node aware selection

### ✔ Snapping, alignment, constraints

### ✔ Inline editing, context menus

### ✔ Edge creation with AI validation

### ✔ Works with IR Delta & Patch Manager

### ✔ High-performance hit-testing

### ✔ Fully semantic interaction (C4/DDD/Event aware)

Together, it makes Sruja Studio feel like:

* **Figma for architecture**
* **JetBrains IDE for diagrams**
* **BPMN tools for semantic editing**
* **AI copilot for architecture modeling**

Below is the **full, production-grade Snapping & Alignment Engine Specification** for **Sruja Studio Web Edition** — this is the engine that makes node movement feel “Figma-smooth + architecture-aware,” ensuring diagrams are clean, readable, professional, and semantically aligned.

This spec covers:

* Grid snapping
* Node-to-node alignment
* Compound-node boundary snapping
* Edge orthogonality snapping
* Port snapping for relationship creation
* Semantic alignment rules (C4/DDD/Event-Flow aware)
* AI-assisted alignment suggestions
* Performance & caching

This subsystem is critical for a high-quality diagramming UX.

---

# 📐 **Snapping & Alignment Engine — Full Deep Spec**

---

# ⭐ 0. Purpose

The Snapping & Alignment Engine ensures:

### ✔ Clean, readable layout while dragging

### ✔ Nodes align to one another

### ✔ Edges stay orthogonal

### ✔ Ports snap to connection points

### ✔ Automatic diagram beauty without explicit layout

### ✔ Semantic guidance (e.g., domains aligned horizontally)

### ✔ Support for compound nodes and hierarchy

### ✔ Maximum 60fps drag performance

### ✔ Integration with Style Engine + Layout Engine (ELK)

---

# 🧱 1. Architecture Overview

```
┌────────────────────────────────────────┐
│     Snapping & Alignment Engine        │
└────────────────────────────────────────┘
                 ▲
                 │
        Interaction Engine (drag)
                 │
                 ▼
       Collect Nearby Geometry
                 │
                 ▼
      Compute Snap Candidates
                 │
                 ▼
     Resolve Snaps & Alignment Rules
                 │
                 ▼
       Produce Adjusted Drag Output
                 │
                 ▼
           Renderer Update
```

The engine does **not** directly write to IR — it only modifies drag coordinates, while the Interaction Engine produces semantic patches on drag end.

---

# 🟦 2. Types of Snapping Supported

### **2.1 Grid Snapping**

* configurable grid (5px, 8px, 10px, etc.)
* supports sub-grids for zoomed-out diagrams
* toggled via toolbar

---

### **2.2 Node-to-Node Alignment Snapping**

Nodes snap to:

* left edge
* right edge
* vertical center
* top edge
* bottom edge
* horizontal center

This creates alignment guide lines like Figma.

---

### **2.3 Compound Node Boundary Snapping**

For containers/systems/domains:

* nodes should snap to interior padding
* nodes should not overlap boundaries
* center of node aligns with compound center

Examples:

* Components snap inside their container grid
* Domains align horizontally inside module
* Aggregates align inside domain

---

### **2.4 Sibling Alignment Snapping**

Nodes inside the same parent snap:

* left-aligned
* vertically distributed
* diagonally aligned
* enforce “semantic rows” (especially for DDD elements)

Sibling snapping is prioritized over cross-group snapping.

---

### **2.5 Edge Orthogonality Snapping**

Edges should stay orthogonal:

* horizontal snap
* vertical snap
* 45° snap for event flows
* mid-point snapping
* orthogonal port snapping

Preview routing updates dynamically.

---

### **2.6 Port Snapping**

When creating connections—snap to:

* nearest port
* semantic port (producer → event, subscriber → event)
* DDD semantic snap (aggregate root port)
* port magnets (primary, secondary)

---

### **2.7 Snap-to-Guides (Alignment Guides)**

Guides appear when:

* nodes align
* nodes are centered
* nodes share common baseline

Supports:

* cross-hierarchy guides
* module-boundary guides
* domain guides

---

### **2.8 Smart Semantic Snapping**

Based on element types:

* Systems align horizontally
* Containers in a system align vertically
* Domains align horizontally
* Aggregates align vertically under domains
* Events align along diagonals
* Event flows align to a logical left→right timeline

Semantic Router provides constraints:

```
Router.snapRulesFor(nodeType)
```

---

# 🟨 3. Snapping Pipeline

```
onDrag(nodeId, rawPosition):

1. Collect Nearby Geometry
   ↓
2. Compute Snap Candidates
   ↓
3. Prioritize Snaps by Weight
   ↓
4. Resolve Conflicts
   ↓
5. Return Adjusted Position to Interaction Engine
```

---

# 🟥 4. Data Structures

## 4.1 Geometry Cache

```ts
interface GeometryCache {
  quadtree: SpatialIndex<SceneNode>;
  portIndex: SpatialIndex<Port>;
  edgeIndex: SpatialIndex<SceneEdge>;
  compoundBounds: Map<nodeId, Rect>;
}
```

### Updated on:

* node creation
* movement
* container expansion
* zoom level changes

---

## 4.2 SnapCandidate

```ts
interface SnapCandidate {
  type: SnapType;
  value: number;       // snap coordinate
  axis: "x"|"y";
  strength: number;    // 0–1 priority
  sourceId?: string;   // node triggering the snap
}
```

---

# 🟩 5. Snap Types & Priority Model

Each snap type has a weighted priority:

```
Grid                      → 0.1
Node Alignment            → 0.6
Sibling Alignment         → 0.8
Compound Boundary         → 0.9
Semantic Alignment        → 1.0
```

Semantic alignment has highest authority.

If multiple snaps are triggered:

* choose highest strength
* if equal strength, smallest delta wins
* if still tie — horizontal alignment prioritized

---

# 🟧 6. Snapping Algorithms (How Each Type Works)

---

## 6.1 Grid Snapping Algorithm

```
snapValue = round(rawValue / gridSize) * gridSize
```

Adjust x or y independently.

---

## 6.2 Node-to-Node Alignment Algorithm

For each node in vicinity:

```
if |draggingNode.left - other.left| < tolerance:
    create SnapCandidate(type=LeftEdge, value=other.left)

if |centerX(node1) - centerX(node2)| < tolerance:
    create SnapCandidate(type=CenterX)
```

Tolerance is dynamic:

```
tolerance = max(6, 12 / zoomLevel)
```

---

## 6.3 Compound Node Boundary Snapping

Rules:

* snap to padding (16px default)
* snap to center horizontally
* snap to rows/columns inside compound

Algorithm:

```
if node is inside container:
    snapX to container.left + padding
    snapX to container.centerX
    snapX to container.right - padding
```

---

## 6.4 Edge Orthogonality Snapping

During drag of edge handles:

```
if near horizontal level of neighbor handle → snap to Y
if near vertical alignment of neighbors → snap to X
```

Routes recomputed via routing engine.

---

## 6.5 Port Snapping

Ports have magnetic radii:

```
magnetRadius = 16px / zoom
```

If cursor enters magnet radius:

```
snap to exact port coordinates
```

Semantic router may override:

* event producers always snap to output port
* event handlers snap to input port

---

## 6.6 Semantic Snapping

Semantic rules come from Semantic Router:

### Example: C4 view

* Systems align horizontally
* Containers vertically
* Components grid-aligned inside containers

### Example: DDD view

* Domains aligned in rows
* Aggregates stacked
* Entities positioned around aggregate center

### Example: Event Flow view

* Events align chronologically left → right
* Handlers below events
* Topics centered between producers and consumers

Engine biases snaps toward semantic lanes.

---

# 🟪 7. Conflict Resolution

Case: horizontal and vertical snaps possible.

Resolution rules:

1. If semantic snap present → choose it
2. Otherwise choose stronger **strength**
3. Then minimal distance (delta)
4. If tie → prefer **horizontal** for C4, **vertical** for DDD

Advanced rule:

```
If snapping in both axes creates overlapping node → reject one axis.
```

---

# 🟦 8. Rendering Alignment Guides

Guides are rendered as:

* thin lines
* dotted or solid depending on type
* color-coded by snap type

Examples:

* **blue** → sibling alignment
* **purple** → semantic alignment
* **yellow** → compound boundary
* **gray** → grid
* **orange** → edge orthogonality

Guides fade in/out with animation.

---

# 🟫 9. Performance Optimizations

### ✔ Quadtree spatial indexing

Only nearby nodes considered.

### ✔ Frustum culling

Nodes outside viewport ignored.

### ✔ Multi-level geometry cache

Separate caches for nodes, edges, ports.

### ✔ Snapping throttling

Apply snapping at ~20–40Hz, drag at 60Hz.

### ✔ Edge routing caching

Reuse same routing path during drag.

### ✔ Early exit heuristics

If node far from others → only grid-snap.

This ensures buttery-smooth dragging even in huge diagrams.

---

# 🟧 10. Integration with Other Engines

### Interaction Engine

Provides:

* raw pointer coordinates
* drag velocity
* dragging target
* selection context

### Layout Engine (ELK)

Snapping overrides temporary positions before layout triggers.

### Scene Graph Diff Engine

Nodes moved by snap → generates IR deltas only after drag end.

### Style Engine

Module-specific snapping rules (e.g., domains colored lanes).

### Semantic Router

Provides semantic snap rules.

### AI Overlay Engine

Displays suggestions for improved layout.

---

# 🟥 11. AI-Assisted Snapping

AI can:

### ✔ Suggest “ideal alignment”

AI overlay shows:
“Align these components horizontally inside BillingAPI”

### ✔ Suggest spacing/distribution

“Distribute containers evenly inside BillingSystem”

### ✔ Auto-clean layout

One-click “Beautify Layout” action.

### ✔ Predict user intent

If user repeatedly aligns nodes a certain way, AI proposes semantic rows.

### ✔ Fix messy event flow

“Snap events into chronological order.”

---

# ⭐ Final Summary — Snapping & Alignment Engine

The engine provides:

### ✔ Grid, node, compound, sibling, and port snapping

### ✔ Semantic snapping (C4, DDD, Event-Flow aware)

### ✔ Orthogonal edge snapping

### ✔ Alignment guides (Figma-like)

### ✔ Hierarchical alignment rules

### ✔ AI-assisted suggestions

### ✔ Smooth 60fps feedback

### ✔ Integrated with layout, renderer, semantic router, and patch manager

It gives Sruja Studio that **polished, premium diagramming feel**, enabling users to create elegant and consistent architecture diagrams with minimal effort.

---
Yes — **you *absolutely* can** design Sruja Studio so that **no architecture files are ever stored on your servers**, while still supporting:

* Teams
* Workspaces
* Real-time editing
* Versioning
* AI assistance
* Approvals
* Collaboration

…and doing it **entirely on top of GitHub / GitLab / Bitbucket / self-hosted Git**.

This model is becoming increasingly popular because it:

### ✔ Maximizes privacy

### ✔ Avoids vendor lock-in

### ✔ Keeps source of truth in Git

### ✔ Lets you run zero backend for file storage

### ✔ Works perfectly with policies, approvals, reviews, PR flows

Below is the **architecture pattern** to make this happen.

---

# 🛡️ **Goal: Studio With NO File Storage on Server**

### *The Studio stores only metadata; all architecture files live in Git branches.*

This means:

* Sruja Studio maintains user accounts, organizations, workspaces, permissions.
* But **no DSL files, diagrams, notebooks, or metadata are stored at rest** on your backend.
* All files exist **only in the user’s own Git repositories**.

---

# 🧱 **1. The Core Model**

### 🎯 Architecture = Branch

Each workspace = Git branch
Each variant = another branch
Each review = PR
Each snapshot = commit
Each “save” = commit or draft commit
Each diagram view = working directory tree loaded from Git

You only store:

* OAuth tokens (encrypted)
* Workspace <-> repo mapping
* User/team membership
* Project metadata (not project files)

**No architecture content stored on Sruja servers.**

---

# 🧩 2. High-Level System Architecture

```
Client (Sruja Web Studio)
   │
   │ GitHub API (clone, commit, PR, branch)
   ▼
Browser FS (IndexedDB or OPFS)
   │
   │ Local cache of repo
   ▼
WebAssembly Kernel
   │
   │ DSL parse → IR → diagram
   ▼
Renderer (Canvas Engine)
```

### 🔐 Backend stores nothing except:

* user accounts
* connection metadata
* allowed repos
* workspace/team info

Everything else = GitHub.

---

# 📦 3. Workspace Model (No File Storage)

A Sruja Workspace consists of:

```
workspace:
  id: uuid
  gitRepo: org/repo
  gitBranch: architecture/main
  variantBranches: [arch/enterprise, arch/mobile]
  teams: [backend, frontend, architecture]
  metadata: {}
```

But the workspace content **lives entirely in Git**, not in Sruja.

---

# 🔄 4. How Sruja Studio Works in This Model

### ✔ Open Workspace

1. Studio clones the Git branch into browser OPFS/IndexedDB
2. Kernel loads DSL and IR
3. Renderer shows diagrams

### ✔ Save

Studio commits changes back to Git:

```
git client → create commit → push → update branch
```

### ✔ Collaboration

* if real-time: Operational Transform or CRDT stored in Git LFS or ephemeral channel
* if PR-based: GitHub PR review

### ✔ Approvals

Architecture PRs flow through GitHub review.

### ✔ Snapshots

Use tags or git refs:

```
architecture/snapshots/YYYY-MM-DD
```

### ✔ Variants

Variant overlays are simply separate branches:

```
architecture/main
architecture/enterprise
architecture/mobile
```

Sruja kernel performs variant merge at runtime.

---

# 🔐 5. Privacy Guarantees

This model is **extremely privacy-safe**:

* Sruja servers never store code
* Only GitHub sees the files
* You can support GitHub Enterprise, GitLab on-prem, Bitbucket Server
* Corporate customers love this
* Easy SOC2 / GDPR story

---

# 🧭 6. Authentication & Permissions

Use GitHub OAuth:

* read/write for repos
* optional fine-grained token scopes

Studio backend stores encrypted token.

User and team roles live in **Studio metadata**, not in repo.

---

# 👥 7. Team Collaboration (Without Storing Files)

You can support 3 modes:

---

## Mode A — Git-Native (simple)

* Work in branches
* Open PRs
* Let GitHub handle conflicts

This is simplest for V1, and preserves privacy.

---

## Mode B — Real-Time Collab (CRDT)

Use:

* Yjs
* Automerge
* Liveblocks-style ephemeral server

Where the CRDT documents are *not stored*, only broadcast.

At “save”, CRDT is serialized into Git commit.

Privacy remains intact — nothing stored on backend.

---

## Mode C — Hybrid

Use CRDT for active session only.
Write changes to Git after session.

---

# 🧬 8. AI Integration Without Storing Files

AI needs context, but context can come from **client-side**:

### ✔ Browser extracts context from local repo

### ✔ Sends minimal context to AI

### ✔ Never transmits entire repo unless user allows

In federated enterprise mode:

* AI runs in local private MCP agent
* zero cloud transmission

---

# 🛠 9. Technical Implementation Plan

---

## Step 1 — Client-side Git Operations

Options:

* **isomorphic-git** (browser Git)
* **libgit2 via WASM** (faster)
* GitHub REST/GraphQL API (simpler for V1)

Recommended:

* Start with GitHub API for CRUD
* Move to proper Git client for offline first

---

## Step 2 — Workspace File System in Browser

Use:

* OPFS (Origin Private File System)
  or
* IndexedDB

Checkout repo → workspace local copy.

---

## Step 3 — Sruja Kernel (WASM)

Kernel operates on local workspace files.

---

## Step 4 — Commit/Pull/Push Flow

* Make changes locally
* “Save” triggers commit
* Push to remote branch

Handle conflicts via:

* Git merge
* Conflict UI
* AI-assisted merge

---

## Step 5 — PR Integration

Studio provides:

* PR creation UI
* Review comments UI
* Architecture diff viewer
* Policy engine checks in PR

---

## Step 6 — Multi-Tenancy (Teams / Orgs)

Backend stores:

```
User
Teams
Workspace metadata (no files)
OAuth tokens
```

Clean and simple.

---

# 🚀 10. Benefits of This Model

### ✔ **Zero data liability**

### ✔ **Enterprise-friendly**

### ✔ **Familiar Git workflow**

### ✔ **Offline-first**

### ✔ **AI-ready**

### ✔ **Scales easily**

### ✔ **No backend complexity**

### ✔ **100% user-controlled data**

### ✔ **Integrates with existing dev workflows**

---

# 🧠 11. Optional: Sruja-GitHub App

For a seamless experience, you can implement a **GitHub App**:

* requested minimal repo permissions
* per-workspace installation
* supports webhooks (PR updates, branch updates)
* enables gated approvals

Studio still stores **zero architecture files**.

---

# ⭐ Final Summary

Yes — you can build Sruja Studio as a **zero-storage, Git-native architecture IDE**, where:

### ✔ All architecture artifacts live in Git branches

### ✔ Studio only handles users, teams, workspaces

### ✔ You never store project content

### ✔ Privacy is maximized

### ✔ Git provides versioning, PRs, approvals

### ✔ Studio provides modeling, diagrams, kernels, AI

### ✔ Real-time or PR-only workflows both work

This is the **best architecture** for security-conscious teams and enterprise adoption.

Below is the **full, production-grade “Git-Connected Workspace Architecture Specification”** for **Sruja Studio** — the engine that allows Sruja to operate **without storing any architecture files** while still supporting:

* Workspaces
* Teams
* Variants
* Reviews
* AI features
* Realtime collab (optional)
* GitHub / GitLab / Bitbucket / Self-hosted Git

This is the foundational document for building a **privacy-first, Git-native architecture IDE**.

---

# 🧱 **Git-Connected Workspace Architecture Spec**

### *(Sruja Studio — Zero-Storage Model)*

---

# ⭐ 0. Core Idea

Sruja Studio **never stores actual architecture files**.
All architecture files live **exclusively inside Git repos** owned by the user or their organization.

Sruja Studio only stores:

* User accounts
* OAuth tokens (encrypted)
* Workspace metadata (no file content)
* Team + access rules
* Audit events (optional)
* No DSL files
* No IR
* No diagrams
* No notebook content

Everything else is resolved from **Git => Browser => Kernel**.

---

# 🧩 1. High-Level Architecture Diagram

```
              ┌──────────────────────────────────┐
              │         Sruja Backend             │
              └──────────────────────────────────┘
                │  User/Team/Workspace Metadata
                │  OAuth Tokens
                ▼
       (no architecture files stored)

┌────────────────────────────────────────────────────────┐
│                     Sruja Web Studio                    │
└────────────────────────────────────────────────────────┘
          ▲                  ▲                   ▲
          │                  │                   │
   Git Repo Clone       Kernel (WASM)       Renderer/Editor
   (local browser)      Parse/IR/Validate   (Canvas engine)
          ▲
          │ GitHub/GitLab API (HTTPS REST)
          ▼
      Remote Git Repository
```

---

# 🚀 2. Workspace Model

A **Workspace** is a *pointer* to a Git branch, not a folder in Sruja backend.

```yaml
workspace:
  id: uuid
  name: "Main Architecture"
  provider: github | gitlab | bitbucket | ssh
  repo: org/repo
  branch: "architecture/main"
  variants:
    - "architecture/enterprise"
    - "architecture/mobile"
  teams:
    owners: [user1, user2]
    contributors: [teamA]
    viewers: [teamB]
  settings:
    realTime: true/false
    autoPR: true/false
    aiContextMinimized: true/false
```

### ⚠ Important:

Workspace metadata contains **no file contents**.

---

# 📦 3. Workspace Lifecycle

### 3.1 Create Workspace

Steps:

1. User chooses Git provider → OAuth
2. Select repo
3. Create branch `architecture/main` (configurable)
4. Create starter folder:

```
architecture/
 ├─ workspace.sruja
 ├─ systems/
 ├─ domains/
 ├─ events/
 └─ variants/
```

All committed to Git.

Backend stores only metadata + repo reference.

---

### 3.2 Open Workspace

When user opens the Studio:

1. Browser downloads repo using:

   * GitHub API / GitLab API
   * or native browser Git client (isomorphic-git)
2. Repo is stored in OPFS / IndexedDB
3. Kernel loads files
4. Renderer generates diagrams

Everything happens **locally**.

---

### 3.3 Save Changes

Workflow:

```
Kernel IR → DSL Patch → Workspace FS → Commit → Push → Git Branch
```

Commit message:

```
feat(architecture): update BillingAPI relationships
```

---

### 3.4 Branching (Variants, PRs)

Variants are separate branches:

```
architecture/main
architecture/enterprise
architecture/mobile
architecture/prototype-r1
```

You can create PRs for:

* approvals
* review
* comparison between variants
* merging variant overlays

---

### 3.5 Reviews

Use GitHub/GitLab PRs.

Studio can show:

* structural diff
* semantic diff
* diagram diff
* rule/policy validation
* AI review summary

But all changes **exist only in Git**.

---

# 🔐 4. Authentication & Permissions

### 4.1 OAuth Flow

Workspace-connected Git providers:

* GitHub (public + enterprise)
* GitLab (public + self-hosted)
* Bitbucket
* SSH (via personal tokens)

Studio backend stores **encrypted token per user**.

---

### 4.2 Access Control

Access is governed by:

* Git repository permissions
* Sruja workspace-level permissions
* Team membership

Sruja never replaces Git’s permission model—only layers enhancements.

---

# 💾 5. Local Workspace Storage (Browser)

Sruja Studio uses:

### **OPFS (Origin Private File System)** for large repos

or

### **IndexedDB** fallback

Local layout:

```
/.user
  repo.db
  workspace.json

/repo
  architecture/
    *.sruja
    modules/
    domains/
    events/
/cache
  ir.json
  diagrams/
```

### Key points:

* No server storage
* Fully offline-edit capable
* AI context extracted only from local FS

---

# 🧬 6. Sync Engine (Pull/Merge/Push)

Sync Engine handles:

* pulling from remote Git
* merging changes
* conflict resolution UI
* pushing commits

Sync model:

```
Remote Tracking → Local Cache → Kernel IR → Diagrams
```

### Merge Modes:

1. **automatic merge** (safe patches)
2. **manual conflict resolution**
3. **AI-assisted merge**
4. **semantic merge** (IR-based)

Perfect for large architecture files.

---

# 🧠 7. AI Integration (Privacy-Safe)

AI receives:

* Only minimal context
* Only user-approved content
* Summaries extracted client-side
* IR slices, not full repo
* No code or architecture files sent unless user confirms

In enterprise mode:

* AI runs locally via **MCP Agent**
* No cloud LLM usage
* Zero data leaves network

---

# 🧩 8. Optional: Real-Time Collaboration (CRDT)

You can support real-time collaboration **without storing anything**:

### Model:

* ephemeral Yjs documents
* relay server only broadcasts
* no persistence
* every user commits final result to Git

On Save → CRDT → DSL patches → Git commit.

---

# 🛠 9. Implementation Components

### ✔ Git Provider Service

* manages tokens
* abstracts GitHub, GitLab, Bitbucket

### ✔ Workspace Manager

* manages workspace metadata
* no file storage

### ✔ Browser Git Client

* clone
* pull
* diff
* commit
* push

### ✔ Sync Engine

* conflict detection
* merge handling

### ✔ Kernel (WASM)

* parsing
* IR generation
* validation
* AI hooks
* patch generation

### ✔ Diagram Renderer

* C4, DDD, Event Flow, Module Graph

---

# 🗂 10. Workspace UI Model

Workspace UI includes:

* Workspace overview
* Branch selector (variant chooser)
* PR review panel
* Sync indicator (green, syncing, conflict)
* Team permissions
* Settings (AI, sync interval, autosave)

---

# ⭐ Final Summary

A **Git-Connected Workspace** allows Sruja Studio to:

### ✔ Work entirely off GitHub/GitLab

### ✔ Store no architecture files server-side

### ✔ Achieve maximum privacy

### ✔ Support variants, reviews, branching

### ✔ Provide AI and kernel features locally

### ✔ Integrate beautifully with Git PR workflows

### ✔ Offer offline editing

### ✔ Be enterprise-ready

This is the best architecture for privacy, adoption, and reliability.

Below is the **full deep-spec** for the **Git Sync Engine (Pull/Merge/Push)** for **Sruja Studio** in a **zero-storage, Git-native architecture**.

This is the *most critical* backend+frontend subsystem for making Sruja Studio behave like a **local-first architecture IDE** while keeping **all files only inside user Git repositories**.

Designed to match the robustness of:

* VS Code Git engine
* JetBrains Git integration
* GitHub Codespaces sync engine
* WarpDrive offline sync model
* Figma’s collaborative commit batching model

---

# 🔄 **Git Sync Engine (Pull / Merge / Push) — Full Specification**

### Sruja Studio — Git-Connected Workspace

---

# ⭐ 0. Purpose

The Git Sync Engine ensures that **local workspace (browser)** stays in sync with **remote Git (GitHub/GitLab)**, while enabling:

* Offline editing
* Local-first commits
* Semantic merge support
* Conflict detection & resolution
* Real-time preview of incoming/outgoing changes
* Batch commits from Kernel patches
* PR creation flows
* Zero architecture file storage in backend

---

# 🧱 1. High-Level Sync Architecture

```
┌───────────────────────────────────────────────┐
│               Git Sync Engine                  │
└───────────────────────────────────────────────┘
        ▲                 ▲                  ▲
        │                 │                  │
  Remote Git API   Browser Workspace FS   Kernel (WASM)
 (GitHub/GitLab)       (OPFS/IDB)         (IR & patches)
        │                 │                  │
        ▼                 ▼                  ▼
   Pull/Fetch       Local Repo Mirror    DSL Patch Engine
        │                 │                  │
        └───── Merge/Resolve/Patch ─────────┘
                        │
                        ▼
                 Push / PR Creation
```

---

# 🧩 2. Sync Engine Components

The Sync Engine consists of:

### ✔ **GitProvider**

* GitHub / GitLab / Bitbucket API wrapper
* OAuth token management
* rate limit handling
* retry logic

### ✔ **LocalGitManager**

* Local clone (browser FS)
* Branch switching
* Commit creation
* Staging files
* Working directory updates

### ✔ **SyncOrchestrator**

* Schedules pull/merge/push
* Conflict detector
* Conflict resolver

### ✔ **Merge Engine**

* 3-way merge
* Semantic merge (IR-based)
* AI-assisted merge

### ✔ **Commit Engine**

* Commit batching
* Autosave
* Commit grouping per interaction session

### ✔ **PR Engine**

* Create PR
* Update PR
* Fetch diff

---

# 📝 3. Storage Model

### 3.1 Local Workspace Storage (Browser)

Sruja stores repo content in:

* **OPFS** (preferred for large repos)
* fallback **IndexedDB**

Local layout:

```
/repo
   /.git-like-metadata
   /architecture/
       *.sruja
       domains/
       systems/
       events/
/cache
   ir.json
   diff-cache.json
```

### 3.2 No Server-Side Storage

Only stores metadata (workspace mappings, tokens), not files.

---

# 🔍 4. Sync Pipeline Stages

```
Pull → Merge → Patch → Push
```

All steps are **fully local**, except Git API calls.

---

# 📥 5. Pull Operation (Fetch Remote)

Triggered on:

* workspace open
* manual refresh
* timed sync
* before push

### Pull Steps

```
1. Compare local branch hash with remote
2. If remote has changes:
       Fetch commit delta
       Apply patch to local FS
       Notify Kernel (IR recompile)
3. Update remote tracking reference
```

### Supported fetch methods:

#### **Mode A — GitHub REST API (simplest)**

* Fetch individual files changed in commits
* No full clone needed

#### **Mode B — Git clone via isomorphic-git**

* Full Git support
* Offline-friendly

#### **Mode C — WASM libgit2**

* Fastest
* Supports large repos
* Most enterprise-friendly

---

# ⚖️ 6. Merge Engine (3-Way + Semantic)

When remote and local diverge:

```
BASE = shared ancestor
LOCAL = local modifications
REMOTE = new changes
```

### Merge modes:

---

## 📘 Mode 1: **Standard 3-Way Merge**

Used for:

* README
* raw markdown
* config files

---

## 📗 Mode 2: **Linewise Merge**

Used for:

* DSL files
* notebooks
* metadata blocks

---

## 📕 Mode 3: **Semantic Merge (Architecture-Aware)**

Used when merging `.sruja` files:

* Kernel extracts IR
* IR diff engine computes merge
* AST merged structurally
* Pretty-print back to DSL

Example:

```
local: added container BillingAPI
remote: updated description of BillingSystem
merged: both applied cleanly
```

---

## 🤖 Mode 4: **AI-Assisted Merge (Optional)**

Used for:

* complex conflicts
* semantic overlaps

AI receives:

* surrounding context
* conflicting segments
* IR analysis
* no entire repo

Generates merged result.

---

# ⚠️ 7. Conflict Detection

Conflicts detected on:

* same fields edited
* same DSL identifier removed and added
* variant overlays mismatching
* conflicting event lifecycle changes
* incompatible imports
* removed system referenced by new code

UI shows:

```
LOCAL | REMOTE | MERGED | OUTCOME
```

Semantic router checks for violations:

* cyclic relationship introduced
* illegal cross-module import
* deleted node referenced by events

---

# 🧰 8. Conflict Resolution UI

### Modes:

#### ✔ Manual editor

Side-by-side diff.

#### ✔ Structural conflict explorer

IR and AST shown visually:

* conflicted systems
* conflicted containers
* conflicted relations

#### ✔ Diagram conflict visualizer

Two versions overlayed.

#### ✔ AI-assisted resolution

User picks suggestions.

---

# 🧲 9. Push Operation

Triggered on:

* manual save
* autosave
* batch commit
* PR creation

### Push Steps

```
1. Ensure working tree clean (after merge)
2. Generate commit batch
3. Push to remote branch
4. Update tracking info
5. Trigger PR refresh (if open)
```

### Commit Metadata Example:

```
commit:
  author: user
  timestamp: now
  files: [systems/billing.sruja, events/payment.sruja]
  message: "update(billing): add event handler for InvoicePaid"
```

---

# 🧮 10. Commit Engine (Batching)

Sruja does not commit every keystroke.

### Commit is triggered by:

* drag/move complete
* relationship created
* editing finished
* inline metadata applied
* file saved
* autosave timer (default: 20s idle)

### Smart batching:

```
All operations in session → 1 commit
```

Groups:

* structural changes
* style changes
* metadata edits
* semantic patches

---

# 🏷️ 11. Branch Switching & Variant Sync

Branch switch algorithm:

```
1. Save local changes (commit)
2. Push if allowed
3. Checkout target branch into local FS
4. Kernel reloads IR
5. Renderer recomputes diagrams
```

Variant overlays loaded via:

```
main + variant-X changes
```

Supports "variant preview mode".

---

# 🧭 12. PR Integration Workflow

When user clicks “Review in Studio”:

### Steps:

1. Studio **checks out PR branch into local FS**

2. Loads both BASE and HEAD

3. Computes:

   * IR diff
   * Diagram diff
   * Policy evaluation
   * Version diff

4. Displays Review Panel

5. User approves/comment

6. Pushes review comments to GitHub PR API

---

# 🧬 13. Real-Time Collaboration (Optional CRDT Layer)

Works on top of Git Sync Engine:

* CRDT doc exists only during session
* No storage in backend
* Each participant commits result to Git
* Merge engine reconciles multiple saves

Keeps system privacy-safe.

---

# ⚡ 14. Performance Optimizations

* **Sparse checkout** (only architecture folder)
* **Incremental file fetch** (only changed files)
* **Delta-based merging**
* **Parallel compression** of large files
* **Local IR caching**
* **Background pull scheduling**
* **Partial AST reparse**

---

# 🔐 15. Security & Data Privacy

### Strict guarantees:

* No architecture files ever stored server-side
* Git credentials encrypted (AES-256)
* AI context minimized and controlled
* Enterprise mode: no data leaves client
* GitHub Enterprise & GitLab Self-hosted compatible

---

# ⭐ Final Summary

The **Git Sync Engine** allows Sruja Studio to operate as a **zero-storage, Git-native architecture IDE** with:

### ✔ Local-first offline editing

### ✔ Semantic & AI-assisted merge

### ✔ Safe Git push & PR workflows

### ✔ Real-time collab (optional)

### ✔ High performance

### ✔ Enterprise-grade privacy

### ✔ Complete support for variants & C4/DDD/Event models

This design is equivalent to the “Git engine inside VS Code,” optimized for architecture modeling.

Below is the **full, enterprise-grade GitHub App Integration Design** for **Sruja Studio** — enabling a **zero-storage**, **highly secure**, **deeply integrated** architecture modeling workflow powered entirely by the user’s GitHub repositories.

This design matches the robustness of:

* GitHub CodeSpaces App
* GitHub Actions App
* Terraform Cloud GitHub App
* GitHub Apps used by Sourcegraph, LaunchDarkly, Linear, Vercel

…and is optimized for **architecture modeling**, **PR-based approvals**, **policy enforcement**, and **variant versioning**.

---

# 🧩 **GitHub App Integration Design**

### *(Sruja Studio — Git-native Architecture Platform)*

---

# ⭐ 0. Goals

The Sruja GitHub App must enable:

### ✔ Secure read/write access to architecture branches

### ✔ Per-repo or per-org installation

### ✔ PR review integration

### ✔ Status checks (architecture validation, policy checks)

### ✔ Webhook events (PR updates, pushes)

### ✔ Minimal permissions (principle of least privilege)

### ✔ Enterprise GitHub compatibility

### ✔ Zero file storage on Sruja servers

It acts **only as a connector**, not a data store.

---

# 🧱 1. High-Level Architecture

```
                ┌────────────────────────────────┐
                │         GitHub Cloud           │
                └────────────────────────────────┘
       ▲                 ▲                  ▲
       │                 │                  │
  GitHub App       GitHub API         GitHub Webhooks
       │                 │                  │
       ▼                 ▼                  ▼
┌────────────────────────────────────────────────────────┐
│                    Sruja Backend                        │
│   (No architecture files stored — only metadata)        │
└────────────────────────────────────────────────────────┘
       ▲                 ▲                  ▲
       │                 │                  │
       ▼                 │                  ▼
Browser Git Client  ←────┘               Web Studio UI
(Local clone)
```

---

# 🔐 2. GitHub App Permissions Spec (Least-Privilege)

### Required Permissions:

#### **A. Repository permissions**

| Permission        | Access       | Why                          |
| ----------------- | ------------ | ---------------------------- |
| Contents          | Read & Write | Pull/Push architecture files |
| Metadata          | Read-only    | Repo metadata                |
| Pull Requests     | Read & Write | PR comments, statuses        |
| Commit Statuses   | Write        | Architecture policy checks   |
| Checks            | Read & Write | Sruja CI-like validation     |
| Issues (optional) | Read & Write | Architecture tasks           |
| Webhooks          | Admin        | Install/remove repo hooks    |

#### **B. Organization-level** (optional)

| Permission     | Access | Why                  |
| -------------- | ------ | -------------------- |
| Members        | Read   | Team sync (optional) |
| Administration | None   | Stay minimal         |

---

# 🛂 3. Authentication Flow (App-Based)

### Step 1 — User installs Sruja GitHub App

* Scoped to org or single repo
* App gets Installation ID
* No access token yet

### Step 2 — Backend exchanges Installation ID for JWT → App token

* **short-lived** token (10 min)
* Used to access GitHub API

### Step 3 — Client-side operations

Browser obtains temporary pre-signed URLs for:

* reading files
* pushing commits (write through Sruja App proxy)

### Step 4 — Browser clones repo locally

No file passes through Sruja backend except diffs when needed for validation.

---

# 📦 4. GitHub App Components

```
GitHub App
 ├── Installation Handler
 ├── Token Exchange Service
 ├── Webhook Processor
 ├── PR Check Runner
 ├── Policy Validator
 └── Deployment Validator
```

---

# 📯 5. Webhook Events Consumed by Sruja

Webhook processor handles:

### ✔ `push`

On updates to architecture branches:

* Trigger Sruja Policy Engine
* Trigger Sruja Kernel validation (optional)
* Update diagrams in PR preview UI

### ✔ `pull_request`

On PR opened/updated/merged:

* Fetch diff of architecture files
* Generate architecture diff
* Run validation & policy checks
* Post status checks
* Post AI insights (optional)

### ✔ `check_run`

Manual re-runs from GitHub UI.

### ✔ `workflow_run` (optional)

Integrate with GitHub Actions.

---

# 🧠 6. App Token Model

### App tokens are short-lived:

* Issued via GitHub JWT → Access Token flow
* Max lifetime: 10 minutes
* Never stored on disk
* Cached in memory, rotated frequently

### Browser never gets full GitHub tokens.

It receives **pre-authorized signed URLs** for:

* fetching raw content
* pushing small patches

All operations proxy through Sruja Backend → GitHub API.

---

# 🧲 7. Branch Naming Conventions

Sruja App recommends:

```
architecture/main
architecture/<variant-name>
architecture/<proposal-id>
architecture/<user-session-id>
```

PRs use:

```
feature/arch/<ticket-id>
refactor/arch/<domain>
```

---

# 🧪 8. Status Checks on PRs

Sruja App posts checks like:

### ✔ Architectural Validation

* broken relationships
* missing systems
* invalid C4 structure

### ✔ Policy Compliance

* forbidden imports
* incorrect event flows
* security requirements not met

### ✔ Variant Drift Detection

* differences between base & variant branches
* incompatible overlays

### ✔ AI Architecture Review (optional)

* Summary
* Suggested improvements
* Missing components/services

Status examples:

```
✓ Structure Valid
✓ Policies Passed
⚠ Variant Drift Detected (see report)
✗ Architecture Violates rule: domain_boundary
```

---

# 🗂 9. PR Review UI in Studio

Users can open PR directly in Sruja:

```
1. PR → checkout branch into local browser FS
2. Kernel loads both HEAD & BASE
3. Compute:
   - diagram diff
   - IR diff
   - semantic diff
4. Show UI for:
   - comments
   - inline annotation
   - resolving
5. Submit review via GitHub API
```

No server stores files.

---

# 🧭 10. PR Checks Execution Architecture

```
GitHub → Webhook → Sruja Backend → Kernel (WASM) → Validation → GitHub Checks API
```

Backend never stores repo content.
It fetches necessary files directly from GitHub on-demand.

Files fetched:

* changed DSL files
* referenced DSL files (minimal set)
* metadata files

Then Kernel runs:

* parse
* validate
* diff
* policy evaluation

No diagram rendering needed on backend (optional).

---

# 🪪 11. Multi-Org and Multi-Repo Support

The App can be installed:

* Per organization
* Per repo
* Across multiple repos
* For private GitHub Enterprise instances

Each workspace links:

```
Workspace → InstallationID → Repo → Branch
```

---

# 🔐 12. Security Design Overview

### ✔ Zero user code stored

### ✔ Zero architecture content stored

### ✔ Zero persistent files on server

### ✔ Tokens encrypted + short-lived

### ✔ All sync via GitHub API

### ✔ File access limited to allowed repos

### ✔ Full audit logging (actions, token use)

### ✔ SSO compatibility

### ✔ Enterprise GitHub App compatible

---

# 🏢 13. Enterprise (On-Prem) Mode

For self-hosted GitHub Enterprise:

* Install Sruja App inside org
* Backend configured with GHE URL
* All API calls stay within private network
* AI can run via local MCP agent
* No external traffic
* Full isolation

---

# 🧬 14. App vs OAuth Model Comparison

| Feature                   | GitHub App | OAuth            |
| ------------------------- | ---------- | ---------------- |
| Per-repo permissions      | ✔          | ❌ coarse-grained |
| Install once, org-wide    | ✔          | ❌                |
| Status checks             | ✔          | ❌                |
| Webhooks                  | ✔          | ❌                |
| Enterprise GitHub         | ✔          | partial          |
| Minimum permissions       | ✔          | ❌                |
| Better security           | ✔          | ❌                |
| Trusted for CI-like flows | ✔          | ❌                |

Conclusion: **GitHub App is strongly preferred**.

---

# ⭐ Final Summary — GitHub App Integration

The Sruja GitHub App provides:

### ✔ secure, minimal-permission access

### ✔ PR-level architecture checks

### ✔ semantic + policy validation

### ✔ webhook-driven updates

### ✔ enterprise GitHub compatibility

### ✔ zero server-side architecture storage

### ✔ offline editing + local-first sync

### ✔ variant-aware architecture workflows

### ✔ AI-assisted PR reviews

This is the **ideal integration model** for enterprise-grade architectural modeling.

---













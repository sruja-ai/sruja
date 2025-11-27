# WASM Execution Model

[← Back to Notebooks Index](./README.md)

## Overview

The Sruja Kernel can be compiled to **WebAssembly (WASM)** for browser-based execution, enabling:

- ✅ JupyterLab (browser-based)
- ✅ VS Code Web
- ✅ Any in-browser notebook
- ✅ Any web IDE
- ✅ Embedding in custom web apps
- ✅ Low-latency AI-assisted architecture workflows

## Architecture

```
Browser (UI) ——— WASM Bridge ——— Sruja Kernel (Go → WASM)
```

### WASM Kernel Responsibilities

- ✅ DSL parsing
- ✅ IR building & updating
- ✅ Validation
- ✅ Diagrams
- ✅ SrujaQL queries
- ✅ Simulation
- ✅ Snapshots / variants
- ✅ AI integration hooks

### Browser Responsibilities

- ✅ Notebook UI
- ✅ Rendering outputs (SVG, Mermaid, tables, diagnostics)
- ✅ AI UI
- ✅ Kernel messaging protocol
- ✅ Persistence (.ipynb file)

## Execution Model

### Runtime Flow

1. **Kernel loaded into browser**
2. **Kernel initializes inside WebWorker or main thread**
3. **Kernel exports functions through WASM exports**
4. **Browser passes DSL/query code to kernel**
5. **Kernel updates in-memory IR inside WASM memory**
6. **Kernel returns outputs via JSON serialization**
7. **Browser renders results**
8. **State persists inside WASM as long as kernel is alive**

## Lifecycle

```
[Load Page]
   ↓
[Fetch kernel.wasm]
   ↓
[Instantiate WASM Module]
   ↓
[Initialize Go runtime in WASM]
   ↓
[Kernel.Init() — build empty ArchitectureStore]
   ↓
[Kernel.RunCell() for each executed notebook cell]
   ↓
[Kernel produces display outputs + diagnostics]
   ↓
[UI renders result]
```

## Memory & IR Storage

### ArchitectureStore in WASM Memory

The ArchitectureStore (IR) is held **inside WASM memory**, not JSON:

**Benefits:**
- ✅ Low latency
- ✅ Incremental updates
- ✅ No serialization between cells
- ✅ Fast queries
- ✅ Fast diff/snapshot application

**Internal Structure:**
```
KernelState (Go struct in WASM mem)
│
├── SymbolTable
├── ArchitectureStore
├── ValidatorsCache
├── DiagramCache
├── SnapshotManager
└── QL Engine
```

**JSON IR is only used when:**
- Exported for snapshots
- Exported for UI
- Exported for MCP agents

This avoids huge JSON overhead on every cell execution.

## WASM Function Exports

### Core Functions

```javascript
// Initialize kernel
export function init()

// Execute a cell
export function execute(code, cellId)

// Run a query
export function query(q)

// Generate diagram
export function diagram(target)

// Snapshot operations
export function snapshot(name)
export function loadSnapshot(name)

// Variant operations
export function createVariant(name, base)
export function applyVariant(name)

// IR operations
export function exportIR()
export function importIR(json)

// Diagnostics
export function getDiagnostics(cellId)

// LSP features
export function autocomplete(prefix, position, code)
export function inspect(position, code)

// Reset kernel
export function reset()
```

### Usage Example

```javascript
// Browser JavaScript
const result = await kernel.execute(cellSource, cellId)

// Result
{
  outputs: [...],
  diagnostics: [...],
  ir_changed: true
}
```

Outputs reference custom MIME types (SVG, JSON, etc.).

## WASM-JS Bridge

### Option A: Standard Go + WASM

Using standard Go toolchain:

```bash
GOOS=js GOARCH=wasm go build -o kernel.wasm
```

**Produces:**
- `kernel.wasm`
- `wasm_exec.js` (Go runtime)

**Loading:**
```javascript
const go = new Go()
const wasm = await WebAssembly.instantiateStreaming(
  fetch("kernel.wasm"),
  go.importObject
)
go.run(wasm.instance)
```

### Option B: TinyGo

Produces smaller WASM:

- ✅ Faster startup (~5–10ms)
- ✅ Lower memory usage
- ✅ Better for web

**Considerations:**
- Some reflect-heavy libraries may break
- May need patches for Participle parser

**Recommendation:** Start with Go, migrate to TinyGo once stable.

## Data Passing

Because WASM cannot directly return complex structs, the bridge uses **JSON or shared memory buffers**.

### Pattern

1. WASM writes JSON string into memory buffer
2. JS reads memory buffer + parses JSON
3. UI renders result

### Example

```javascript
// Execute cell
kernel.execute("system Billing { ... }", "cell_123")
// → returns JSON string pointer + length
// JS reads → parses → displays SVG + diagnostics
```

## Incremental Execution

**Critical for performance:**

Every DSL cell does **incremental updates**:

```
Before:
  ArchitectureStore = existing model

After cell:
  Apply deltas → new ArchitectureStore
  Run partial validators → update impacted parts
  Generate IR diff for UI (optional)
```

### Caching Strategy

- ✅ Cache diagrams
- ✅ Cache validated regions
- ✅ Cache resolved imports
- ✅ Cache symbol table entries
- ✅ Reuse everything

**Only changed parts are recomputed.**

## Execution per Cell

### Kernel Steps

```
1. Parse DSL → AST
2. Identify scope: system/entity/event
3. Remove previous cell contributions
4. Apply new AST diff
5. Resolve references
6. Update IR
7. Run validators (scoped to impacted nodes)
8. Produce diagnostics
9. Produce diagrams (if requested)
10. Return to UI
```

## Performance Strategies

### Key Optimizations

- ✅ **Maintain IR in memory** (not JSON)
- ✅ **Avoid repeated serialization**
- ✅ **Diff-based validation**
- ✅ **Only run validators on impacted nodes**
- ✅ **Cache symbol table**
- ✅ **Cache diagram intermediate structures**
- ✅ **Use WebWorker** to avoid blocking UI thread
- ✅ **Send only deltas** (not full model)

### TinyGo Optimization

For future speed improvements:
- Use TinyGo for smaller binary
- Faster startup times
- Lower memory footprint

## Multi-Kernel Support

Each notebook tab runs:

- Its own WASM instance
- Its own KernelState
- Its own ArchitectureStore

**Memory footprint:** ~2–20MB depending on model size.

## Debugging & Logging

### WASM → Console Bridge

Expose debug logging:

```javascript
export function enableDebugLogs(boolean)
```

Or write logs via JS's `console` inside WASM runtime:

```javascript
console.log("[SRUJA-KERNEL]", message)
```

## Integration with AI & MCP

WASM kernel exposes:

```javascript
export function getIR()
export function suggestFixesForEvent(eventId)
export function suggestBoundaryEnforcements()
export function diffContracts(a, b)
export function listViolations()
```

**AI layers in JS can:**
- Read IR
- Generate fix suggestions
- Call `kernel.applyPatch()`

## Undo/Redo Support

Because IR changes are small diffs:

```javascript
undoStack.push(ModelPatch)
redoStack.push(ModelPatch)
```

**UI calls:**
```javascript
kernel.applyPatch(patch)
```

## Restarting Kernel

When user restarts kernel:

1. Entire WASM runtime restarts
2. IR resets
3. Notebook UI re-executes necessary cells
4. Snapshots/variants restored from metadata

**Equivalent to restarting Python kernel in Jupyter.**

## Security

### WASM Sandbox

- ✅ WASM sandbox ensures no filesystem/network access
- ✅ External AI calls only through controlled JS layer
- ✅ No direct eval of arbitrary JS
- ✅ Memory boundaries are safe
- ✅ Kernel must sanitize inputs

## Summary

The **WASM Execution Model** for the Sruja Kernel:

✅ Go → WASM kernel compilation
✅ Maintains entire Architecture IR in WASM memory
✅ Incremental updates per DSL cell
✅ Efficient JSON encoding/decoding for UI
✅ Custom MIME outputs for diagrams & diagnostics
✅ Rich extension support
✅ Fast, isolated, secure execution
✅ Multi-instance safe (tabs, windows)
✅ Fully compatible with Jupyter & VS Code notebooks
✅ Integrates AI & MCP tools cleanly

This design is **robust, fast, scalable, and zero-dependency** beyond standard Jupyter/WASM APIs.

## Next Steps

- [Architecture Kernel](./kernel.md) - Kernel implementation details
- [Kernel Messaging Protocol](./kernel-messaging.md) - Jupyter protocol integration


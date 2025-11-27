# Undo/Redo Engine

**Status**: Core Engine  
**Pillars**: Core (History Management)

[← Back to Engines](../README.md)

## Overview

The Undo/Redo Engine provides unified history management across DSL Editor (Monaco), Diagram Editor (ReactFlow), and Architecture Model (IR), enabling seamless undo/redo operations.

**This design gives you a global, coherent history, exactly like VS Code, Figma, D2 Studio, and Excalidraw.**

## Purpose

The Undo/Redo Engine:

- ✅ Synchronizes undo/redo across DSL and Diagram
- ✅ Maintains global architecture history stack
- ✅ Prevents sync loops during undo/redo
- ✅ Preserves cursor position and formatting
- ✅ Supports semantic undo/redo (optional)
- ✅ Integrates with Two-Way Sync Engine

## Why This Is Hard

Most editors fail because they keep *separate* undo stacks:

- Monaco has its own undo stack
- ReactFlow has its own state
- Your global model has its own mutations

This leads to:

- Desync
- Jumping cursor
- Diagram resets
- Lost edits
- Infinite loops

**We fix it by introducing a single unified "Architecture History Stack".**

## Global Architecture History Stack (Single Source of Truth)

We store **history snapshots** of the SimpleModel IR.

```
history: [
  { model, source, timestamp }
]
```

### Zustand store:

```ts
interface HistoryState {
  history: SimpleModel[];
  pointer: number; // which index we are on
  push: (model: SimpleModel) => void;
  undo: () => void;
  redo: () => void;
}
```

## Push to History on REAL User Actions

We only push when:

- DSL → model update originates from user typing
- Diagram → model update originates from user interaction

**Not when UI syncs itself.**

Integrate into your existing sync logic:

```ts
function updateModel(newModel, source) {
  const store = useArchStore.getState();
  const history = useHistory.getState();

  // Only push history on real user edits
  if (source === "dsl" || source === "diagram") {
    history.push(newModel);
  }

  store.setModel(newModel, source);
}
```

This guarantees:

- No double pushes
- No phantom history entries
- No infinite undo loops

## When Undo is Triggered

Undo triggers:

- Global Model changes
- ReactFlow updates
- DSL updates via diff patches
- Cursor restoration
- Layout remains stable

### Example handler (Ctrl+Z):

```ts
function handleUndo() {
  const { pointer, history } = useHistory.getState();
  if (pointer <= 0) return;

  const newModel = history[pointer - 1];

  useArchStore.getState().setModel(newModel, "history");
}
```

### Redo:

```ts
function handleRedo() {
  const { pointer, history } = useHistory.getState();
  if (pointer >= history.length - 1) return;

  const newModel = history[pointer + 1];

  useArchStore.getState().setModel(newModel, "history");
}
```

## Prevent Sync Loops During Undo/Redo

When `source === "history"`:

- Do NOT push new history
- Do NOT reparse DSL
- Do NOT treat as user input

Add in your two-way sync engine:

```ts
if (source === "history") return; // skip DSL parsing & diagram-origin sync
```

## Apply to DSL Editor (Monaco) — Diff-Based

When undo happens:

```ts
if (store.updatingFrom === "history") {
  applySmartPatch(monacoEditor, model); // minimal patch
  return;
}
```

This ensures:

- Cursor stays stable
- Formatting preserved
- Comments preserved
- Undo/redo is additive

## Apply to Diagram (ReactFlow)

ReactFlow simply re-renders from model:

```tsx
const model = useArchStore(s => s.model);
const version = useArchStore(s => s.version);

useEffect(() => {
  if (updatingFrom === "history") {
    setNodes(renderDslModelToFlow(model).nodes);
    setEdges(renderDslModelToFlow(model).edges);
  }
}, [version]);
```

Diagram jumps to previous state with perfect fidelity.

## Command Manager (Optional)

Like Figma, you can implement commands:

```
ADD_NODE
DELETE_NODE
MOVE_NODE
RENAME_NODE
ADD_EDGE
DELETE_EDGE
UPDATE_NAME
```

Each command creates:

- A forward patch
- A backward patch

This gives **semantic undo/redo**, making multi-step actions reversible in one step (drag + rename = atomic).

But this is optional for MVP.

## Keyboard Bindings

With shadcn + React:

```ts
useHotkeys("mod+z", () => handleUndo());
useHotkeys("mod+shift+z", () => handleRedo());
```

## Full Undo/Redo Flow Summary

### DSL Edit
```
User types → parse → model updated → history.push()
Diagram re-renders → smart patch DSL
```

### Diagram Edit
```
User drags/moves/connects → patch model → history.push()
Smart DSL patch applied
```

### Undo
```
Undo → pointer-- → model restored
ReactFlow gets new model
Monaco gets smart patched DSL
```

### Redo
```
Redo → pointer++ → model restored
Diagram + DSL updated via smart patches
```

### Result
⚡ PERFECT synchronous undo/redo across DSL + Diagram + Model.

## MCP API

```
history.undo()
history.redo()
history.canUndo()
history.canRedo()
history.clear()
history.getHistory()
```

## Strategic Value

The Undo/Redo Engine provides:

- ✅ Unified history across all editors
- ✅ No sync loops
- ✅ Preserved user experience
- ✅ Professional editor behavior
- ✅ Reliable undo/redo

**This is critical for user experience and editor quality.**

## Implementation Status

✅ Architecture designed  
✅ History stack specified  
✅ Sync prevention defined  
📋 Implementation in progress

---

*The Undo/Redo Engine provides unified history management across DSL and Diagram editors.*


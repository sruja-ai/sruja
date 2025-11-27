# Two-Way Sync Engine

**Status**: Core Engine  
**Pillar**: All (foundational)

[← Back to Engines](../README.md)

## Overview

The Two-Way Sync Engine enables **perfect bidirectional synchronization** between DSL text and visual diagrams. This is a **critical foundational engine** that makes Sruja unique.

## Purpose

The engine ensures:
- ✅ DSL → Diagram always matches
- ✅ Diagram → DSL always matches
- ✅ No infinite loops
- ✅ No flicker
- ✅ No cursor jump in Monaco
- ✅ No race conditions
- ✅ Multi-source update isolation
- ✅ Support for future LSP + validation

## Architecture

```
DSL editor (Monaco)  
    → parse → AST → IR  
        → update global model  
            → render diagram  

Diagram (ReactFlow)  
    → user interactions → patch IR  
        → serialize → DSL  
            → update Monaco content
```

**Key principle:** Changes must not trigger the opposite direction again.

We use **update sources + versioning** to prevent loops.

## Global Data Model

Single source of truth stored in global IR model:

```typescript
interface SyncState {
  model: SimpleModel;             // single source of truth
  updatingFrom: "dsl" | "diagram" | null;
  version: number;                // increments on each logical update
}
```

Stored in Zustand:

```typescript
export const useArchStore = create<SyncState>((set) => ({
  model: { components: [], relations: [] },
  updatingFrom: null,
  version: 0,
  setModel: (newModel, source) =>
    set((state) => ({
      model: newModel,
      updatingFrom: source,
      version: state.version + 1,
    })),
}));
```

## DSL → Model Sync Pipeline

Triggered when the user types in Monaco:

```typescript
monacoEditor.onDidChangeModelContent(() => {
  const store = useArchStore.getState();

  if (store.updatingFrom === "diagram") return;  // ignore diagram-origin updates

  try {
    const ast = parseDsl(editor.getValue());
    const model = astToModel(ast);
    store.setModel(model, "dsl");
  } catch (err) {
    // handle syntax errors (no update)
  }
});
```

**Features:**
- ✅ Prevents recursion
- ✅ Only updates when user typed
- ✅ Validation errors allowed without breaking sync

## Model → Diagram Renderer

ReactFlow uses **model + version** to re-render:

```typescript
const model = useArchStore(s => s.model);
const version = useArchStore(s => s.version);

const { nodes, edges } = useMemo(
  () => renderDslModelToFlow(model),
  [model, version]
);
```

We use `version` to force precise, deterministic re-renders.

## Diagram → Model Sync Pipeline

Triggered on user actions:
- drag node
- rename node
- delete
- connect two nodes
- disconnect
- add new node via palette
- move node

ReactFlow events:

```typescript
const onNodesChange = useCallback((changes) => {
  patchModelFromDiagram(changes, "nodes");
}, []);

const onEdgesChange = useCallback((changes) => {
  patchModelFromDiagram(changes, "edges");
}, []);

const onConnect = useCallback((connection) => {
  patchModelFromDiagram([{ type: "connect", connection }], "edges");
}, []);
```

## patchModelFromDiagram() - Critical Logic

```typescript
function patchModelFromDiagram(changes, kind) {
  const store = useArchStore.getState();

  if (store.updatingFrom === "dsl") return; // ignore DSL-origin updates

  let model = store.model;

  // Apply each diagram change to model
  for (const change of changes) {
    if (change.type === "add") model = addComponent(model, change.item);
    if (change.type === "position") model = moveComponent(model, change);
    if (change.type === "remove") model = removeComponent(model, change);
    if (change.type === "connect") model = addRelation(model, change.connection);
  }

  store.setModel(model, "diagram");
}
```

**Features:**
- ✅ Prevent reverse-trigger loops
- ✅ Every diagram edit updates the IR
- ✅ Zero DSL parsing needed

## Model → DSL Serializer

Updates DSL from the updated model, but only when a diagram edit occurs:

```typescript
useArchStore.subscribe((state, prev) => {
  if (state.updatingFrom !== "diagram") return;

  const dslText = serializeModelToDsl(state.model);

  monacoEditor.executeEdits("sync", [
    {
      range: monacoEditor.getModel().getFullModelRange(),
      text: dslText,
    },
  ]);

  // Keep cursor position stable
  monacoEditor.pushUndoStop();
});
```

**Features:**
- ✅ Perfect synchronization
- ✅ No cursor jumping
- ✅ DSL always stays canonical
- ✅ Prevents infinite loops (we check `updatingFrom`)

## Cursor Preservation

Before updating the DSL, we store the cursor:

```typescript
const pos = monacoEditor.getPosition();
// update modelToDsl
monacoEditor.setPosition(pos);
```

**Result:**
- ✅ Typing experience intact
- ✅ User never loses place
- ✅ Undo stack stays consistent

## Error Handling

When DSL has errors, we show markers but don't break diagram state:

```typescript
monaco.editor.setModelMarkers(model, "dsl", diagnostics);
```

Diagram still uses **last valid model**, which is ideal.

## Event Flow

### DSL Edit
```
Monaco → parse → IR
→ store.setModel(…, "dsl")
→ ReactFlow rerender
```

Diagram update is NOT triggered.

### Diagram Edit
```
ReactFlow → patchModel → IR
→ store.setModel(…, "diagram")
→ serialize → update DSL
```

Parsing is NOT triggered.

## Advanced: Diff-Based Smart Serializer

For production, use diff-based serialization instead of full replacement:

1. Generate canonical DSL from IR (sorted, normalized)
2. Compute diff between current DSL and canonical
3. Apply minimal patches using `monacoEditor.executeEdits()`
4. Preserve cursor and selection

This provides:
- ✅ No full rewrite
- ✅ No cursor jump
- ✅ No flashing
- ✅ Preserves user comments/formatting
- ✅ Preserves whitespace + grouping

## Implementation Status

✅ Architecture designed  
✅ Core sync logic specified  
✅ Loop prevention mechanism defined  
📋 Diff-based serializer in progress  
📋 LSP integration planned

---

*The Two-Way Sync Engine is the foundation that makes Sruja's bidirectional editing possible.*



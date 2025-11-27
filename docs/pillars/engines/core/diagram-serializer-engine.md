# Diagram Serializer Engine

**Status**: Core Engine  
**Pillars**: Core (Serialization)

[← Back to Engines](../README.md)

## Overview

The Diagram Serializer Engine provides diff-based smart serialization from diagram edits to DSL text, updating only minimal parts of the DSL without full rewrites.

**This is the professional, production-grade approach used in Figma, VSCode TextModel edits, D2 Studio, and JetBrains MPS projections.**

## Purpose

The Diagram Serializer Engine:

- ✅ Updates only minimal parts of DSL text
- ✅ No full rewrite
- ✅ No cursor jump
- ✅ No flashing
- ✅ No losing user comments / formatting
- ✅ Preserves user-added whitespace + grouping

**You get true projectional editing behavior WITHOUT a projectional editor.**

## Why We Need a Smart Serializer

A naive serializer replaces the full DSL:

```
editor.update("full new file")
```

Huge issues:

- Cursor jumps
- Undo/redo breaks
- User comments lost
- Whitespace reflow
- Git diffs polluted
- User changes fight the renderer

We solve all of this with **diff-based granular text application**.

## Architecture Overview

```
IR Model
   ↓
Generate canonical DSL (for comparison)
   ↓
Diff canonical DSL against existing DSL text
   ↓
Find minimal patches
   ↓
Apply monacoEditor.executeEdits()
   ↓
Preserve cursor/selection
```

## Step 1 — Generate Canonical DSL From IR

Canonical DSL is *sorted* and *normalized* but **not written to editor** directly.

Used only for diffing.

```ts
export function canonicalDsl(model: SimpleModel): string {
  const lines: string[] = [];

  // Components sorted by ID
  for (const c of [...model.components].sort((a, b) => a.id.localeCompare(b.id))) {
    lines.push(`${c.id}: ${c.type} "${c.name}"`);
  }

  lines.push("");

  // Relations sorted as well
  for (const r of [...model.relations].sort((a, b) =>
    (a.from + a.to).localeCompare(b.from + b.to)
  )) {
    lines.push(`${r.from} -> ${r.to}`);
  }

  return lines.join("\n");
}
```

- ✔ stable
- ✔ deterministic
- ✔ supports diffing

## Step 2 — Compute Diff Between Current DSL and Canonical

We use **diff-match-patch**, the gold standard.

```ts
import DiffMatchPatch from "diff-match-patch";

const dmp = new DiffMatchPatch();

export function computePatch(oldText: string, newText: string) {
  return dmp.patch_make(oldText, newText);
}
```

## Step 3 — Convert Patch → Monaco Edit Operations

The patch is turned into **incremental edits** instead of replacing whole text.

```ts
export function patchToMonacoEdits(
  patches: DiffMatchPatch.patch_obj[],
  model: monaco.editor.ITextModel
) {
  const edits: monaco.editor.IIdentifiedSingleEditOperation[] = [];

  for (const p of patches) {
    let start = model.getPositionAt(p.start1);
    let end = model.getPositionAt(p.start1 + p.length1);

    edits.push({
      range: new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column),
      text: p.diffs
        .filter(([op]) => op !== -1)
        .map(([op, text]) => (op === 1 ? text : ""))
        .join(""),
      forceMoveMarkers: false,
    });
  }

  return edits;
}
```

## Step 4 — Apply the Minimal Patch to Monaco

```ts
export function applySmartPatch(editor, newModel) {
  const store = useArchStore.getState();
  if (store.updatingFrom !== "diagram") return;

  const oldText = editor.getValue();
  const newText = canonicalDsl(newModel);

  const patches = computePatch(oldText, newText);
  const edits = patchToMonacoEdits(patches, editor.getModel());

  editor.executeEdits("diagram-sync", edits);
  editor.pushUndoStop();
}
```

## Step 5 — Cursor + Selection Preservation

Before applying edits, save cursor:

```ts
const cursor = editor.getPosition();
const selection = editor.getSelection();
```

After applying:

```ts
editor.setPosition(cursor);
editor.setSelection(selection);
```

- ✔ typing experience intact
- ✔ user never loses place
- ✔ undo stack stays consistent

## Integrating Into Sync Engine

Replace the old full-serializer with:

```ts
useArchStore.subscribe((state, prev) => {
  if (state.updatingFrom !== "diagram") return;

  applySmartPatch(monacoEditor, state.model);
});
```

## Behavior Examples

### User drags `api` component

Canonical DSL unchanged.

**No diagram → DSL update happens.**

### User renames API → "Backend API"

Only this line changes:

```
api: service "Backend API"
```

The diff engine applies a **single 1-line edit**.

No full rewrite.

### User adds new service by dragging from palette

Only one new line inserted:

```
auth: service "Auth Service"
```

Relations added also as minimal insertions.

### User deletes a component

Delete only the affected lines.

## What We Gain

- ✔ No full document rewrite
- ✔ No jitter or flicker
- ✔ No cursor jumps
- ✔ Formatting preserved
- ✔ Comments preserved
- ✔ Git diffs clean
- ✔ Users feel "in control"
- ✔ Diagram becomes a projection, not a source of truth

## Path to Future Upgrades

This system is fully compatible with:

- LSP TextDocumentSync
- Multi-file architecture (resolve + patch per file)
- Custom formatting rules
- User-defined DSL formatting
- "Format on save" without breaking diagram sync
- Refactor tools (rename symbol, extract service)

## MCP API

```
serializer.canonical(model)
serializer.patch(oldText, newText)
serializer.apply(editor, patches)
```

## Strategic Value

The Diagram Serializer Engine provides:

- ✅ True projectional editing behavior
- ✅ Minimal text updates
- ✅ Preserved user experience
- ✅ Clean Git diffs
- ✅ Professional editor quality

**This is critical for bidirectional sync quality.**

## Implementation Status

✅ Architecture designed  
✅ Diff algorithm specified  
✅ Monaco integration defined  
📋 Implementation in progress

---

*The Diagram Serializer Engine provides diff-based smart serialization from diagram to DSL.*


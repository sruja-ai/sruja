# Graph → IR Patch Spec

# 📌 Scope
VSCode Extension Studio pipeline: diagram edits in the Webview convert to IR patches, which update DSL for two-way editing; layouting handled by ELK.

# ⭐ Purpose
- Convert every diagram edit into precise IR mutations
- Ensure semantic correctness, minimal patches, stable IDs
- Avoid conflicts; support undo/redo; integrate with Kernel + LSP

# 🧠 Core Concept
Diagram is a view; edits produce IR patches.

# 🧱 Patch Model (Universal)
```ts
Patch {
  op: "add" | "remove" | "update" | "move" | "connect" | "disconnect" | "rename",
  targetId: string,
  field?: string,
  value?: any,
  oldValue?: any,
  context?: any,
  origin: "diagram" | "dsl" | "ai"
}
```
- origin=diagram prevents loops; patches are idempotent; transactional

# 🟦 Patch Routing Flow
Diagram Action → Graph Event → Graph→IR Mapper → validate → Kernel.applyPatch → DSL patch → LSP apply → Kernel reparse → Diagram update

# 🟨 Diagram Edits → IR Patch Rules
- ACTION 1 — Move: `op=move`, field=parent
- ACTION 2 — Rename: `op=rename`, field=name
- ACTION 3 — Metadata update: `op=update`, field=metadata.*
- ACTION 4 — Delete: `op=remove`
- ACTION 5 — Add Node: `op=add` with value payload
- ACTION 6 — Connect: `op=connect` {from,to,verb,label}
- ACTION 7 — Disconnect: `op=disconnect` {from,to}
- ACTION 8 — Relation verb/label update: `op=update` on relation
- ACTION 9 — Drag layout position: diagram-only; stored in layout store
- ACTION 10 — Group operations (merge/split systems): higher-level `update` transformed into node/edge changes

# 🟩 Patch Validation
Validate targetId, parent-type compatibility, policy, cycles, required nodes.
Reject invalid patches with UI error.

# 🟦 Patch Batching
- Wait for drop to emit patch on drag
- Multi-select operations batched into transactions `op=batch`

# 🟪 Undo / Redo Support
Inverse rules: add↔remove, move new→old, update new→old, connect↔disconnect, rename new→old.

# 🟧 Multi-File DSL Support
Use `IRNode.location.file` to update correct DSL file; resolve ambiguity; relations across files handled by origin file or global model file.

# ⭐ Final Summary
- Deterministic, reversible, idempotent patches
- Multi-file support; origin tagging to prevent loops
- Transaction batching; AI-driven structural refactors supported

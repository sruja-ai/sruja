# Two-Way Binding Engine

# 📌 Scope
Applies to the VSCode Extension Studio: two-way DSL↔diagram editing inside the Webview, IR as source of truth, ELK layouting preserved across edits, and LSP/Kernel roundtrips.

# ⭐ 0. Purpose
- Visual edits modify the architecture DSL
- Text edits modify the visual architecture
- IR is the single source of truth
- Changes are small, predictable, reversible
- Live sync across multiple files; conflict-free

# 🧠 Core Principle
IR is authoritative; DSL and Diagram are views.
- DSL → IR updates
- Diagram → IR updates
- IR → DSL & Diagram updates

# 🧱 1. Architecture Overview
```
DSL File → Kernel (IR Root) → Two-Way Binding Bus → Diagram Editor
```
Binding sits between DSL, Diagram Studio UI, and Kernel IR.

# 🧩 2. Data Structures
## 2.1 IR Node Model
- Stable `id`
- `type`, `name`, `attributes`, `location` (DSL SourceLocation), `children`, `meta`

## 2.2 Patch Model
```ts
Patch {
  op: "add" | "remove" | "update" | "move" | "rename",
  targetId: string,
  field?: string,
  value?: any,
  oldValue?: any,
  context?: any
}
```
Flows between Diagram↔IR↔DSL↔Diagram.

# 🟧 3. Event Flow Details
## 3.1 DSL → IR
1. LSP detects change
2. LSP parses DSL → PartialIR
3. Kernel merges deterministically
4. Kernel computes IR patches
5. Kernel broadcasts IR_UPDATE
6. Diagram updates

## 3.2 Diagram → IR
1. Diagram emits UI Patch
2. Validate patch
3. Kernel applies to IR
4. Kernel generates DSL patch
5. LSP applies text edit
6. DSL file updates
7. LSP reparses → IR regenerates
8. Diagram gets new IR

# 🟦 4. IR Merge Rules
- PartialIR: merge fields if id exists, else add node
- Diagram patch: `Kernel.IR.applyPatch(patch)`
- Both converge to same IR

# 🟨 5. DSL Patch Generation
- Structural diff + source mapping
- Input: old IR, new IR, SourceLocations
- Output: minimal textual edits
- Types: update field, move node, add node, remove node
- Implementation: tree-aware rewriter; patch only ranges

# 🟫 6. Diagram Binding Rules
- Reflect IR structure; virtual graph diffing
- IR_UPDATE → reconcile graph incrementally

# 🟪 7. Guarantees
- Convergence, idempotence, no infinite loops
- Achieved via stable ids, deterministic merge, origin tagging, local pause

# 🟩 8. Conflict Resolution
- DSL wins for content; Diagram wins for layout/position
- Combined patches when both sides edit same node

# 🟥 9. Sequence Diagrams
- Diagram → DSL Roundtrip and DSL → Diagram Roundtrip sequences

# 🟦 10. Recommended Libraries
- Webview: Cytoscape.js, ELK.js, React, zustand
- Kernel: Go + WASM, protobuf/JSON IR
- LSP: participle/tree-sitter, incremental sync
- DSL Patch Generator: AST + source map, formatter, minimal replacement

# 🟧 11. MVP Phases
- Phase 1: DSL → IR → Diagram (one-way)
- Phase 2: Diagram → IR → DSL
- Phase 3: Conflict resolution + origin tagging
- Phase 4: Notebook cells
- Phase 5: AI-assisted edits

# 🟣 Final Summary
- IR truth, unified patch model, minimal DSL patching
- Deterministic merges, virtual graph diffing, conflict resolution
- Loop prevention, stable IDs, metadata linking

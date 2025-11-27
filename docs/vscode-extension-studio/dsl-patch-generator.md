# DSL Patch Generator

# 📌 Scope
Operates within the VSCode Extension Studio: transforms IR patches into precise DSL text edits via LSP, enabling two-way editing; layouting remains ELK-driven in the Webview.

# ⭐ Purpose
Transform IR patches into precise DSL text edits in VSCode without rewriting entire files.
- Minimal, stable edits; preserve formatting, comments, ordering
- Multi-file support; idempotent and reversible; LSP incremental sync

# 🧠 Core Principle
Never regenerate files; rewrite only affected ranges using source mappings.
- Kernel stores SourceLocation for each IR node (file, offset, length)
- DSL AST nodes store offsets, lengths, indentation

# 🧱 Inputs and Outputs
Inputs:
- IR Patch
- Current DSL AST with source locations
- File contents for `.sruja`
- Node → File mapping inside IR

Outputs:
- VSCode `TextEdit { file, range, newText }` via LSP `textDocument/applyEdit`

# 🟦 Patch Types Supported
- add: insert new block
- remove: delete block
- update: replace fragment
- move: remove from old → insert into new
- rename: update identifier + references
- connect: insert relation statement
- disconnect: remove relation statement
- batch: multiple edits atomically
- replace: structural refactor

# 🟨 DSL Element Mapping
IR → DSL blocks:
- `system Billing { }`
- `container BillingAPI { }`
- `component PaymentProcessor { }`
- `BillingAPI -> AuthAPI "calls"`
Each block has `SourceLocation` for targeted edits.

# 🟥 Pipeline Overview
IR Patch → Find DSL AST Node → Compute Minimal Text Edit → Apply Formatting Rules → Return TextEdit → LSP Applies → DSL rewritten → Kernel recompiles IR → Diagram updates

# 🟩 Patch Generation Algorithms
- Update Field: replace only string literal or token
- Rename: replace identifier and update all references via `ir.references[targetId]`
- Add Node: insert into parent block before closing brace; match indentation
- Remove Node: delete byte range using SourceLocation
- Move Node: delete from old file; insert into new parent; fix indent and ordering
- Add Relation: append in relation section or create new section
- Remove Relation: delete range
- Update Relation Verb/Label: replace verb or label token
- Batch: apply patches by descending offset to avoid overlaps
- Replace (Structural Refactor): remove old blocks, insert new blocks; larger region rewrites allowed

# 🟧 Formatting Rules
- Preserve user formatting; enforce minimal structure on inserts
- Match parent indentation; single spaces; consistent quotes; optional blank line between blocks to match context

# 🟦 Conflict Resolution
- If file changed during patch apply, retry with fresh AST and source locations

# 🟪 Multi-File Support
- Use `irNode.sourceLocation.file` to pick correct file
- Moves: remove from file A, add to file B
- Cross-file relations: insert in originating file or global file

# 🟫 Undo/Redo Support
- Inverse patches generated using original IR state

# 🟩 Performance Considerations
- Use byte offsets; parse AST once per change; apply all edits in one workspace edit; patch only changed regions; persistent parser state for partial reparses

# ⭐ Final Summary
- Minimal textual edits with source precision; supports all operations
- Rewrite only affected ranges; avoid formatting churn; multi-file; undo/redo; works with VSCode LSP

# Node Inspector Interaction Spec

# 📌 Scope
VSCode Extension Studio inspector: edits produce IR patches that update DSL and diagrams inside the Webview; layouting is ELK-based and preserved through two-way editing.

# ⭐ Purpose
Primary detail view for selected nodes; shows structure, metadata, relations, constraints, DSL mapping, policy, AI suggestions.
Editable fields produce IR patches; actions include move, rename, refactor, delete, add relations.
Works for systems, containers, components, entities, events, datastores, externals, relations.

# 🧠 Inspector Trigger Rules
Opens on node click, search selection, edge click (relation inspector), hierarchy selection. Closes on canvas click or ESC.

# 🧱 Inspector UI Layout
A. Header; B. Summary; C. Editable Fields; D. Metadata Editor; E. Parent/Hierarchy; F. Relations Editor; G. AI Suggestions; H. DSL Source Mapping; I. Actions.

# 🟦 A. Header
Icon + Node Type + name; actions: inline rename, change type, “Open in DSL”. Produces rename patch.

# 🟨 B. Summary Panel
Description, technology, tags, policy violations, inferred confidence; badges visible.

# 🟩 C. Editable Fields (Two-Way Binding)
Change events produce IR update patches; Kernel → DSL patch.

# 🟧 D. Metadata Editor
Add/delete/edit metadata; patches on metadata fields; supports JSON values.

# 🟥 E. Parent / Hierarchy
Shows parent, siblings, children; changing parent emits `move` patch.

# 🟪 F. Relations Editor
Incoming/outgoing relations; actions: add/edit/delete, jump to target, AI explain. Patches: update/disconnect/connect.

# 🟫 G. AI Suggestions Panel
AI suggests improve description, rename, missing relations, infer components, classify domain, decompose, merge similar, policy annotations. Apply suggestion → IR patch.

# 🟦 H. DSL Source Mapping
File, line snippet; “Open in Editor”; updates via two-way binding.

# 🟩 I. Actions
- Systems: Add Container, Add External, AI boundary analysis, Move System
- Containers: Add Component, Convert, AI decompose, Delete
- Components: Convert to container, Extract, Delete

# 🟦 Interaction Rules
- Save on Blur: edits auto-save to IR
- Undo/Redo: inspector edits push patches
- Multi-Select Editing: batch tags/metadata/description
- Locked Nodes: prevent edits; show lock reason
- Policy Violations: show warnings; one-click AI fix

# 🟥 Events Emitted by Inspector
INSPECTOR_UPDATE, INSPECTOR_RENAME, INSPECTOR_MOVE, INSPECTOR_METADATA_CHANGE, INSPECTOR_RELATION_ADD/UPDATE/DELETE, INSPECTOR_AI_APPLY.

# ⭐ Final Summary
Editable details for all nodes; IR patches for changes; DSL sync; metadata/relations/hierarchy; AI augmentation; multi-view; source mapping; policy warnings; multi-node edits; undo/redo.

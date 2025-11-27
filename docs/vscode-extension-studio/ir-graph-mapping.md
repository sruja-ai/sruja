# IR → Graph Mapping

# 📌 Scope
Mapping for the VSCode Extension Studio Webview: IR→graph rendering with ELK layouting and reverse mapping to support two-way DSL↔diagram editing.

# ⭐ Purpose
- Convert IR nodes/relations into graph nodes/edges
- Preserve stable identities
- Support multiple view modes
- Hierarchical grouping
- Render validation & policy metadata
- Support incremental updates
- Provide reverse mapping (Graph → IR)

# 🧠 Core Concept
Graph is a visual projection of IR; fully reactive, reversible, diff-aware.

# 🧱 IR Structure (Input)
IRNode: id, type, name, parent, metadata, children, relations.
Supports systems, containers, components, entities, events, external systems, datastores, queues.

# 🖼️ Graph Structure (Output)
Graph: nodes, edges, groups, `viewMode`.

# 📌 Graph Node Model
- id, label, type, shape, style, parentGroupId
- metadata: irId, description, tags, inferred, confidence, violations

# 🔗 Graph Edge Model
- id, source, target, type, label, style, metadata (irRelationId, inferred, violations)

# 🧩 Group (Parent) Model
- id, label, type, childrenIds, style

# 🌐 View Modes
- C4 System, C4 Container, C4 Component
- Event Flow: producer → event → consumer
- Domain (DDD): bounded contexts, entities, aggregates, services
- Contract View: endpoints, commands, queries, responses, event contracts

# 🟦 Mapping Rules
- Stable IDs: GraphNode.id = IRNode.id; GraphEdge.id = hash(src+dst+type)
- Parent/Child → Grouping: `parentGroupId = IRNode.parent`
- Filter nodes by view mode
- Filter relations by visibility rules per view
- Metadata mapping: inferred → dashed, violations → red, tags → badges, confidence → opacity
- Optional hide nodes without edges

# 🟧 Incremental Updates
- Add/remove/update/move nodes; add/remove/update relations
- Use Cytoscape `batch()` for efficient updates

# 🟩 Reverse Mapping (Graph → IR)
- GraphNodes carry `metadata.irId`
- Translates diagram actions into IR patches deterministically

# 🟨 Mapping Pipeline
```
ir → mapIRToGraph(viewMode) → GraphModel → Cytoscape Adapter → Rendered Diagram
```

# 🟥 Performance Rules
- Compound nodes or hierarchical layout
- rAF batching and throttling
- Memoized mapping
- Precompute bounds in worker
- Diff-driven updates only

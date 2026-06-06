# Sruja Graph

> Architecture knowledge graph with analysis algorithms.

## Purpose

Stores the full architecture knowledge graph — nodes (architecture elements), edges (relationships), decisions (ADRs), policies, requirements, incidents, and learnings. Provides graph analysis algorithms: centrality, coupling, SCC detection, treewidth, BM25 retrieval, hybrid retrieval, and context scoring.

## Responsibilities

- Store and query architecture elements and relationships
- Compute centrality metrics (betweenness, closeness, PageRank, eigenvector)
- Analyze coupling between components
- Detect strongly connected components (cycles)
- BM25 sparse retrieval for text search
- Hybrid retrieval combining graph traversal and text search
- Compute AI-readiness context scores
- Manage learning entries for agent memory

## Dependencies

- **Internal**: Sruja_Language, Sruja_Graph_Core, Sruja_Scan (native only)
- **External**: serde, serde_json, chrono, uuid

## Key Types

- `KnowledgeGraph` — Main graph container
- `GraphNode` / `ArchitectureNode` — Graph node (architecture element)
- `GraphEdge` / `ArchitectureEdge` — Graph edge (relationship)
- `Decision`, `DecisionStatus` — Architecture decision records
- `LearningEntry`, `LearningKind` — Agent memory entries
- `SystemGraph` — Cross-repo system graph
- `SparseIndex` — BM25 text index
- `ContextScore` — AI-readiness score

## Code Locations

- `crates/sruja-graph/` — Graph crate
- `src/graph.rs` — Core graph implementation
- `src/centrality.rs` — Centrality analysis
- `src/learning.rs` — Agent memory types
- `src/bm25.rs` — BM25 sparse retrieval

---
*Last updated: 2026-06-06*

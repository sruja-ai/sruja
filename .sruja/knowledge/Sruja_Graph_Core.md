# Sruja Graph Core

> Domain-agnostic graph traits for architecture analysis.

## Purpose

Defines abstract traits (`ContextNode`, `ContextEdge`, `ContextGraph`) that decouple graph analyzers from concrete graph implementations. Allows Sruja's analysis algorithms (centrality, blast radius) to operate on any directed graph, not just the software architecture graph.

## Responsibilities

- Define `ContextNode` trait for graph nodes
- Define `ContextEdge` trait for graph edges
- Define `ContextGraph` trait for graph containers
- Provide generic `CentralityAnalyzer` implementation
- Compute blast radius on any graph implementation

## Dependencies

- **Internal**: None (leaf crate)
- **External**: serde, serde_json, hashbrown

## Key Types

- `ContextNode` — Trait for graph nodes (requires id, label, kind)
- `ContextEdge` — Trait for graph edges (requires source, target, kind)
- `ContextGraph` — Trait for graph containers (requires nodes, edges, adjacency)
- `CentralityAnalyzer` — Generic centrality computation
- `BlastRadiusResult` — Blast radius analysis result
- `ArchitecturalHotspot` — High-centrality node identification

## Code Locations

- `crates/sruja-graph-core/` — Core traits crate
- `src/lib.rs` — Trait definitions
- `src/centrality.rs` — Generic centrality analyzer

---
*Last updated: 2026-06-06*

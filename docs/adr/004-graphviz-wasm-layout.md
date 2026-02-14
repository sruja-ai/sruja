# ADR 004: Use Graphviz WASM for C4 Model Layouts

Date: 2024-12-23

## Status

Accepted. Current extension uses **Rust WASM** for export (e.g. Mermaid/DOT); Graphviz WASM is the chosen approach for **layout** (positioning nodes) where a dedicated layout engine is needed (e.g. book tooling or future visual tooling).

## Context

We need a reliable, deterministic layout engine for rendering C4 Model diagrams. The layout must support hierarchical graphing (subgraphs for compounds/containers) and orthogonal edge routing that minimizes crossings.

## Options Considered

1.  **Dagre (JavaScript)**:
    - Pros: Pure JS, small bundle size.
    - Cons: Unmaintained, poor support for compound nodes (clusters), edges often overlap nodes.

2.  **ELK (Eclipse Layout Kernel)**:
    - Pros: Extremely powerful, layered algorithms (elkjs).
    - Cons: High complexity to configure, large WASM/JS footprint, "Layered" algorithm is good but often places nodes too wide.

3.  **Graphviz (WASM via @hpcc-js/wasm)**:
    - Pros: Industry standard (DOT language), excellent support for clusters (subgraphs), stable deterministic output, "dot" engine is optimized for hierarchical DAGs.
    - Cons: Large WASM binary (~1MB), async loading required.

## Decision

We chose **Graphviz via WASM**.

## Consequences

- **Performance**: Layout is fast enough for typical C4 diagrams (< 100 nodes), but initialization has a slight overhead for fetching WASM.
- **Implementation**: We must handle async loading/projection.
- **DX**: Debugging layouts involves inspecting generated DOT strings, which is a well-understood format.
- **Asset Management**: Where Graphviz WASM is used, ensure the WASM binary is correctly served (e.g. `public/wasm` or equivalent).

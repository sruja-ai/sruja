# Sruja Implementation Audit: Paper vs. Product

This audit evaluates whether Sruja is a "real" engineering tool or just a "on-paper" visualizer.

## 1. The "Actually Useful" Core

Sruja goes beyond standard Architecture-as-Code (AaC) tools in several key areas:

### A. Architectural Linting (Real Utility)

The `pkg/engine` contains a sophisticated concurrent validation engine. Unlike simple schema validators, it performs:

- **Cycle Detection**: Uses optimized iterative DFS with resource pooling to detect circular dependencies at scale.
- **Layer Enforcement**: Validates that "Downstream" components don't call "Upstream" components.
- **Orphan Detection**: Identifies disconnected components that represent "ghost" services.
- **Governance Mapping**: Ensures every requirement and ADR is uniquely identified and linked.

### B. High-Fidelity IDE Support

The `pkg/lsp` implementation is a professional-grade Language Server. It supports:

- **Semantic Renaming**: Renaming a system updates all its relationships across the workspace.
- **Document Links**: Click-to-nav between DSL files and external data.
- **Hover Intelligence**: Shows technology stacks and descriptions when hovering over nodes.
- **Semantic Tokens**: Provides high-quality syntax highlighting that understands the context of the model.

### C. Geometric Quality Control

The `dot` exporter uses a unique `quality.go` system that parses SVG output to measure:

- **Edge Crossings**: Mathematically identifies where lines cross (minimizing visual clutter).
- **Parent-Child Containment**: Geometrically verifies that child nodes are physically inside their parents' bounds.
- **Rank Alignment**: Ensures horizontal/vertical alignment for a professional "blueprint" feel.

---

## 2. Identified Gaps (The "On-Paper" Limits)

### A. Behavioral Simulation

While `story` and `scenario` keywords exist and are parsed correctly, they are currently **descriptive**.

- **Gap**: There is no "Simulator" to run a story through the model to check for policy violations or latency accumulation.
- **Status**: Useful for documentation and Designer animations, but not yet for formal verification.

### B. SLO & Scale Predictive Power

The DSL supports `slo` and `scale` blocks, and the engine validates their format (percentages, durations).

- **Gap**: The values aren't currently used to calculate "theoretical system max throughput" or identify bottlenecks.
- **Status**: Static documentation only.

### C. Minor Placeholders

- **Layout Engine**: The quality metrics code has a minor placeholder for "Label Overlap" detection. This means labels might occasionally overlap nodes in dense diagrams without a score penalty.

---

## 3. Implementation Quality Verdict

**Verdict: Actually Useful.**

Sruja is not a "toy" tool. The implementation of the **concurrent engine, resource pooling, and full LSP** is done properly and follows high-performance Go standards.

It is "actually useful" as a **Governance and Design Enforcement tool**. It moves architecture from "drawing a picture" to "validating a contract." The gaps are primarily in **behavioral execution/simulation**, which are natural next steps for a maturing platform.

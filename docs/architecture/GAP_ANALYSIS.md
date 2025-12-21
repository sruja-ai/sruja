# Architecture Gap Analysis Report: Sruja/LikeC4

## Executive Summary
The current Sruja codebase demonstrates a high-quality foundation with a robust DSL (AST) and validation engine. It exceeds standard C4 implementations by including **SLOs**, **Governance**, and **Contracts** in the core model. However, to achieve "FAANG-level" quality for large-scale enterprise architecture, three critical gaps must be addressed: **Dynamic Behavioral Modeling**, **Scalable Module System**, and **Visualizing non-functional requirements**.

## 1. Critical Feature Gaps (High Priority)

### 1.1 Dynamic Behavioral Modeling (Missing)
**Status**: 🔴 Not Implemented
- **Gap**: The AST and Frontend lack support for "Dynamic" or "Sequence" views.
- **Why it matters**: FAANG-level architecture requires modeling *behavior* (how components interact over time), not just *structure* (static boxes). Without this, users cannot model complex flows like "User Login" or "Order Processing".
- **Recommendation**: Implement `dynamic view` in DSL and a Sequence Diagram renderer.

### 1.2 Module System & Imports (Missing)
**Status**: 🔴 Not Implemented
- **Gap**: `pkg/language` lacks `imports` or library concepts. All models seem to assume a single workspace context without clear dependency management between repositories.
- **Why it matters**: Large organizations need to split architecture models across multiple repositories (e.g., each Team owns their Service's C4 model).
- **Recommendation**: Implement the "Sruja Modules" design (from user's previous context).

## 2. Experience Gaps (Medium Priority)

### 2.1 Visualization of Advanced Features
**Status**: 🟡 Partially Implemented
- **Gap**: While the Backend parses **SLOs** (availability, latency) and **Contracts**, the Frontend (`SystemNode.tsx`, `DeploymentNode.tsx`) does not visualize them.
- **Observation**: `GovernanceBadge` exists, but there is no "SLO Badge" or "Contract View".
- **Why it matters**: Operational data (SLOs) is a key differentiator for Sruja. Hiding it reduces value.
- **Recommendation**: Add visual indicators for SLO targets vs current status on the nodes.

### 2.3 Animations & "Wow" Factor
**Status**: 🔴 Not Implemented
- **Gap**: Static rendering with minimal transitions. No edge animations or flow visualization.
- **Why it matters**: "FAANG-level" quality implies a premium, fluid user experience. Animations communicate *change* and *flow* better than static lines.
- **Recommendation**:
    -   **Edge Animations**: Moving particles common in C4 flow diagrams.
    -   **Layout Transitions**: Smooth morphing when expanding/collapsing nodes (Layout stability).
    -   **Micro-interactions**: Hover states, pulses for alerts/SLO violations.

### 2.2 Deployment Modeling Depth
**Status**: 🟡 Basic
- **Gap**: `DeploymentNode` AST exists, but the frontend component is a thin wrapper.
- **Why it matters**: Production architectures are defined by *where* things run (Structure + Infrastructure).
- **Recommendation**: Enhance Deployment views to show capacity, region, and instance count visually.

## 3. Architecture & Quality (Low Priority / Maintenance)

### 3.1 Frontend-Backend Decoupling
- **Status**: 🟢 Good
- The split between `pkg/language` (Go) and `packages/diagram` (TS) is clean.

### 3.2 Testing
- **Status**: 🟡 In Progress
- Coverage is improving, but integration tests for likely conflicting features (e.g. valid references across split files) will be needed for Modules.

## Proposed Roadmap

1.  **Phase 1: Structure & Scale**
    *   Implement **Sruja Modules** (Filesystem & Git-based imports).
    *   Refactor LSP to support multi-file/multi-repo indexes.

2.  **Phase 2: Behavior**
    *   Implement AST for `dynamic { ... }` views.
    *   Implement Sequence Diagram renderer in Frontend.

3.  **Phase 3: Operations, Intelligence & UX**
    *   **Animations**: Implement edge flow animations and layout morphing (using `framer-motion` or React Flow Pro features).
    *   Visualize SLOs on Diagrams (Heatmap view?).
    *   Visualize Contracts/API definitions.

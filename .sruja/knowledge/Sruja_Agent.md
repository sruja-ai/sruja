# Sruja Agent

> Agentic memory management and Memory-Aware Test-Time Scaling (MaTTS).

## Purpose

Provides agentic memory management and MaTTS trajectories. The memory system stores "learnings" (hypotheses, outcomes, guardrails) from AI-driven architectural experiments. MaTTS generates multiple parallel evaluation trajectories for the same goal, runs them in isolation (git worktrees), and distills higher-quality guardrails by contrasting success vs. failure outcomes.

## Responsibilities

- Store and retrieve learning entries (hypotheses, outcomes, guardrails)
- Curate stale or low-utility learnings
- Generate MaTTS trajectories for parallel evaluation
- Run trajectories in isolated git worktrees
- Contrast trajectory outcomes to distill guardrails
- Provide search across learnings

## Dependencies

- **Internal**: Sruja_Diagnostics, Sruja_Graph, Sruja_Language
- **External**: serde, serde_json, thiserror, chrono, fs2

## Key Types

- `AgenticMemory` — Memory store interface
- `LearningEntry`, `LearningKind`, `LearningPatch` — Learning types (re-exported from sruja-graph)
- `CurationReport`, `LowUtilityEntry`, `StaleEntry` — Curation results
- `TrajectoryRunner` — MaTTS trajectory executor
- `TrajectoryOutcome`, `TrajectoryStatus` — Trajectory results
- `ContrastResult` — Success/failure contrast analysis

## Code Locations

- `crates/sruja-agent/` — Agent crate
- `src/memory/` — Memory management (curation, search, storage)
- `src/matts.rs` — MaTTS trajectory runner
- `src/executor.rs` — Trajectory executor trait

---
*Last updated: 2026-06-06*

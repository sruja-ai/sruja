# sruja-agent

Persistent memory and learning management for Sruja AI agents.

## Overview

`sruja-agent` provides a persistent "memory" for AI agents working with Sruja. It records the history of architectural experiments, their outcomes, and derived guardrails. This enables future agent sessions to learn from past mistakes and replicate successful patterns.

## Key Concepts

### Agentic Memory
A JSON-backed store (`.sruja/agent_memory.json`) that tracks `LearningEntry` objects.

### Learning Entry
A single record containing:
- **Context:** The task or refactoring being performed.
- **Hypothesis:** What the agent believed the change would achieve.
- **Outcome:** Whether the experiment succeeded or failed.
- **Guardrail Advice:** Actionable advice to avoid repeating failures or to ensure success.
- **Affected Elements:** The architectural components involved.

## Usage

### In Rust Code

```rust
use sruja_agent::{AgenticMemory, LearningEntry, ExperimentOutcome};
use std::path::Path;

let repo_root = Path::new(".");
let mut memory = AgenticMemory::load(repo_root)?;

memory.add_learning(LearningEntry {
    timestamp: chrono::Utc::now(),
    context: "Refactoring API layer".to_string(),
    hypothesis: "Moving logic to sruja-engine will reduce CLI bloat".to_string(),
    outcome: ExperimentOutcome::Success,
    reason: None,
    guardrail_advice: "Always move validation logic to the engine".to_string(),
    affected_elements: vec!["Sruja.API".to_string()],
});

memory.save(repo_root)?;
```

### Via CLI

The `sruja-cli` provides commands to interact with this memory:

```bash
# View all architectural learnings
sruja agent history

# Filter history by a specific component
sruja agent history --id Sruja.API

# Clear agentic memory
sruja agent clear --force
```

## Testing

```bash
cargo test -p sruja-agent
```

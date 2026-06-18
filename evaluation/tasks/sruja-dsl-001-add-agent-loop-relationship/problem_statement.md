# DSL Update: Add missing Agent loop relationship to repo.sruja

## Summary

The `Sruja.Agent` container in `repo.sruja` describes "Agentic memory, MaTTS trajectories, learning loop, and cognitive architecture" but does not mention the autonomous agent loop (`sruja agent loop`), which is the primary entry point for autonomous coding tasks. Additionally, the Agent container lacks a relationship to the CLI for the loop command, and the CLI doesn't list the agent loop as a command it provides.

## Current state

The Agent container (line 29-33 in `repo.sruja`):
```sruja
Agent = container "Agent & Memory" {
    technology "Rust"
    description "Agentic memory, MaTTS trajectories, learning loop, and cognitive architecture for AI-assisted development"
    tags ["agent", "memory", "cognitive"]
}
```

The CLI container lists "manages memory and learning" (line 56) but doesn't mention the autonomous loop.

## What to change

1. **Update Agent container description** to include the autonomous agent loop:
   - New description: "CLI-first autonomous agent with the full observe-act-verify-critique-replan loop, agentic memory, MaTTS trajectories, and cognitive architecture for AI-assisted development"
   - Keep existing tags

2. **Add a relationship** from CLI to Agent for the loop command:
   ```sruja
   CLI -> Agent "Drives autonomous loop"
   ```
   This should be placed near the existing `CLI -> Agent "Manages memory and learning"` relationship.

3. **Verify** with `sruja lint repo.sruja` after the change.

## Acceptance criteria

- `sruja lint repo.sruja` passes
- `sruja drift -r .` shows no new violations
- The Agent container description includes "autonomous agent" and "loop"
- The new CLI -> Agent relationship exists in the output
- No other containers or relationships are modified

---
title: "Multi-Agent Orchestration"
weight: 20
summary: "Supervisor, Hierarchical, and Mesh architectures."
difficulty: "advanced"
topic: "agentic-ai"
estimatedTime: "20 mins"
---

# Multi-Agent Orchestration

For complex domains, a single agent can get confused. **Multi-Agent Systems (MAS)** split responsibilities among specialized agents.

## Supervisor Pattern

A central "Supervisor" agent routes tasks to worker agents and aggregates results.

```sruja
import { * } from 'sruja.ai/stdlib'


Supervisor = container "Orchestrator"

Coder = container "Coding Agent" {
description "Writes and executes code"
}

Writer = container "Documentation Agent" {
description "Writes summaries"
}

Supervisor -> Coder "Delegates coding tasks"
Supervisor -> Writer "Delegates writing tasks"
Coder -> Supervisor "Returns result"
Writer -> Supervisor "Returns result"

view index {
include *
}
```

## Hierarchical Teams

Agents can manage other agents, forming a tree structure. This is useful for large-scale operations like software development (Manager -> Tech Lead -> Developer).

## Network/Mesh

Agents communicate directly with each other without a central supervisor. This is more decentralized but harder to debug. Sruja's relationship visualization shines here by mapping the allowable communication paths.

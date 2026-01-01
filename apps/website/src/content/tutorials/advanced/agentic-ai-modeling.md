---
title: "Agentic AI Modeling"
weight: 40
summary: "Model agent orchestration, tools, and memory using Sruja DSL."
tags: ["ai", "agents", "rag", "modeling"]
difficulty: "advanced"
estimatedTime: "30 min"
---

# Agentic AI Modeling

This tutorial shows how to model agent-based systems with orchestrators, planners, executors, tools, and memory.

## Core Structure

```sruja
import { * } from 'sruja.ai/stdlib'


AgentSystem = system "Agentic System" {
Orchestrator = container "Agent Orchestrator"
Planner = container "Planner"
Executor = container "Executor"
Tools = container "Tooling API"
Memory = database "Long-Term Memory"
}

User = person "User"

User -> AgentSystem.Orchestrator "Requests task"
AgentSystem.Orchestrator -> AgentSystem.Planner "Plans steps"
AgentSystem.Orchestrator -> AgentSystem.Executor "Delegates execution"
AgentSystem.Executor -> AgentSystem.Tools "Calls tools"
AgentSystem.Executor -> AgentSystem.Memory "Updates state"

view index {
include *
}
```

## Add Governance

```sruja
Guardrails = policy "Safety Policies" {
  description "Limit tool calls, enforce approvals, track risky operations"
}

R1 = requirement functional "Explain actions"
R2 = requirement constraint "No PII exfiltration"
```

## Integrate RAG

```sruja
import { * } from 'sruja.ai/stdlib'


AgentSystem = system "Agent System" {
  Executor = container "Executor"
}

RAG = system "Retrieval-Augmented Generation" {
  Retriever = container "Retriever"
  Generator = container "Generator"
  VectorDB = database "VectorDB"
}

AgentSystem.Executor -> RAG.Retriever "Fetch contexts"
RAG.Retriever -> RAG.VectorDB "Search"
RAG.Generator -> AgentSystem.Executor "Produce answer"
```

## Next Steps

- Explore `examples/pattern_agentic_ai.sruja` and `examples/pattern_rag_pipeline.sruja`
- Add scenarios to capture common workflows
- Use views to present developer vs. executive perspectives

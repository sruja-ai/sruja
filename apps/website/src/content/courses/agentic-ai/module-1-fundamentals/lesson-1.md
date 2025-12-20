---
title: "What is Agentic AI?"
weight: 10
summary: "Defining Agentic AI and its shift from static chains to dynamic loops."
difficulty: "intermediate"
topic: "agentic-ai"
estimatedTime: "10 mins"
---

# What is Agentic AI?

Traditional LLM applications often follow a linear chain: Prompt -> LLM -> Output. **Agentic AI** breaks this linearity by introducing a control loop where the model decides **what to do next**.

## The Control Loop

An agent typically operates in a loop:
1.  **Observe**: Read input or environment state.
2.  **Reason**: Decide on an action (using an LLM).
3.  **Act**: Execute the action (call a tool).
4.  **Reflect**: Observe the result of the action.
5.  **Repeat**: Continue until the goal is met.

## Agent vs. Chain

| Feature | Chain (e.g., LangChain Runnable) | Agent |
| :--- | :--- | :--- |
| **Control Flow** | Hardcoded by developer | Determined dynamically by LLM |
| **Flexibility** | Rigid, predictable | Adaptive, handles ambiguity |
| **Failure Recovery** | Often brittle (fails if one step fails) | Can self-correct and retry |
| **Complexity** | Lower | Higher (requires guardrails) |

## Why Sruja for Agents?

Modeling agents is complex because relationships are often dynamic. Sruja helps by:
*   **Visualizing Dependencies**: Showing which agents use which tools.
*   **Defining Boundaries**: separating the cognitive engine (LLM) from the execution layer (Tools).
*   **Documenting Flows**: Tracing the decision loop.

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  Agent = component "Research Agent"
  LLM = component "Model Provider"
  Tool = component "Search Tool"

  Agent -> LLM "Reasons next step"
  Agent -> Tool "Executes action"
  Tool -> Agent "Returns observation"
}

views {
  view index {
    include *
  }
}
```

---
title: "Core Components: Agents, Tools, Memory"
weight: 20
summary: "Deep dive into the anatomy of an AI agent."
difficulty: "intermediate"
topic: "agentic-ai"
estimatedTime: "15 mins"
---

# Core Components

Every agentic system consists of a few fundamental building blocks.

## 1. The Agent (The Brain)
The core logic that orchestrates the workflow. It holds the "system prompt" or persona and manages the context window.

## 2. Tools (The Hands)
Capabilities exposed to the agent. These can be:
*   **APIs**: Weather, Stock Prices, Internal Databases.
*   **Functions**: Calculator, Code Interpreter.
*   **Retrievers**: RAG search against vector databases.

## 3. Memory (The Context)
*   **Short-term Memory**: The current conversation history and scratchpad of thoughts.
*   **Long-term Memory**: Vector databases or persistent storage for recalling past interactions.

## Modeling in Sruja

We can map these components to Sruja elements:

*   **Agent** -> `container` or `component`
*   **Tool** -> `component` or external `system`
*   **Memory** -> `datastore`

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
  AgentSystem = system "Customer Support Bot" {
    Brain = container "Orchestrator" {
      description "Main control loop"
    }

    Memory = container "Context Store" {
      ShortTerm = component "Conversation History"
      LongTerm = component "Vector DB"
    }

    Tools = container "Toolbelt" {
      CRM = component "CRM Connector"
      KB = component "Knowledge Base"
    }

    Brain -> Tools.CRM "Look up user"
    Brain -> Tools.KB "Search policy"
    Brain -> Memory.ShortTerm "Read/Write context"
  }
}

views {
  view index {
    include *
  }
}
```

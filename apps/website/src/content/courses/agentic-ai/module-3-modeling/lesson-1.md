---
title: "Modeling Strategies"
weight: 10
summary: "Choosing the right abstraction level."
difficulty: "advanced"
topic: "agentic-ai"
estimatedTime: "15 mins"
---

# Modeling Strategies

How should you represent an agent in Sruja? It depends on the scope of your diagram.

## Level 1: System Context

If your AI is a product that users interact with, model it as a **System**.

```sruja
import { * } from 'sruja.ai/stdlib'


User = person "User"
AI_Assistant = system "Support Bot"

User -> AI_Assistant "Chats with"

view index {
include *
}
```

## Level 2: Container View

If you are designing the internals, agents are often **Containers** (deployable units).

```sruja
import { * } from 'sruja.ai/stdlib'


AI_Assistant = system "AI Assistant" {
Router = container "Router Agent"
Search = container "Search Agent"
VectorDB = database "Memory"
}
```

## Level 3: Component View

If you are designing a single agent's logic, the specific tools and chains are **Components**.

```sruja
import { * } from 'sruja.ai/stdlib'


AI_Assistant = system "AI Assistant" {
SearchAgent = container "Search Agent" {
  Planner = component "ReAct Logic"
  GoogleTool = component "Search API"
  Scraper = component "Web Scraper"
}
}
```

## Using Metadata

Use metadata to capture AI-specific details:

```sruja
container GPT4Agent {
  metadata {
    model "gpt-4-turbo"
    temperature "0.7"
    max_tokens "4096"
    cost_per_1k_input "$0.01"
  }
}
```

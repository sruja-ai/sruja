---
title: "The ReAct Pattern"
weight: 10
summary: "Modeling the Reason + Act loop."
difficulty: "advanced"
topic: "agentic-ai"
estimatedTime: "15 mins"
---

# The ReAct Pattern

**ReAct** (Reasoning + Acting) is a prompting strategy where the model explicitly generates:
1.  **Thought**: Reasoning about the current state.
2.  **Action**: The tool call to make.
3.  **Observation**: The result of the tool call.

This loop continues until the agent decides it has enough information to answer.

## Sruja Model

We can model this flow using a `scenario` or `story` in Sruja to visualize the sequence.

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
  component Agent
  component Tool
  component User

  story ReActLoop "Answering a Question" {
    User -> Agent "Ask: What is the weather in SF?"
    
    // Step 1
    Agent -> Agent "Thought: I need to check weather"
    Agent -> Tool "Action: WeatherAPI(SF)"
    Tool -> Agent "Observation: 15°C, Cloudy"
    
    // Step 2
    Agent -> Agent "Thought: I have the answer"
    Agent -> User "Answer: It's 15°C and cloudy."
  }
}

views {
  view index {
    include *
  }
}
```

This visualization helps stakeholders understand the latency and cost implications of the multiple steps involved in a single user request.

---
title: "Governance and Safety"
weight: 20
summary: "Defining constraints and policies for autonomous agents."
difficulty: "advanced"
topic: "agentic-ai"
estimatedTime: "15 mins"
---

# Governance and Safety

Autonomous agents can be unpredictable. Architecture-as-Code allows us to define **constraints** to ensure safety.

## Defining Requirements
Use `requirement` blocks to specify safety properties.

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
  container Agent
  container BankAPI

  Agent -> BankAPI "Transfers funds"

  requirement HumanLoop functional "Transfers > $1000 must require human approval"
  requirement PII constraint "No PII should be sent to external LLM providers"
}

views {
  view index {
    include *
  }
}
```

## Policy as Code
You can enforce rules about which agents can access which tools.

```sruja
// Example of a prohibited relationship
// Agent -> ProductionDB "Direct Write" 
// ^ This could be flagged by a linter rule
```

## Guardrails
Model your guardrails explicitly as components that intercept messages.

```sruja
container AgentSystem {
  component UserProxy "Input Guardrail"
  component LLM
  component OutputGuard "Output Validator"

  UserProxy -> LLM "Sanitized Input"
  LLM -> OutputGuard "Raw Output"
  OutputGuard -> UserProxy "Safe Response"
}
```

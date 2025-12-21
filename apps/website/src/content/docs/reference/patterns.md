---
title: "Architecture Patterns"
weight: 53
summary: "Reusable patterns: request/response, event-driven, saga, CQRS."
---

# Architecture Patterns

## Request/Response

```sruja
specification {
  element system
  element container
  element datastore
}

model {
  App = system "App" {
    Web = container "Web"
    API = container "API"
    DB = datastore "Database"
  }
  
  App.Web -> App.API "Calls"
  App.API -> App.DB "Reads/Writes"
}

views {
  view index {
    include *
  }
}
```

## Event-Driven

```sruja
specification {
  element system
  element container
}

model {
  Orders = system "Order System" {
    OrderSvc = container "Order Service"
    PaymentSvc = container "Payment Service"
  }
  
  Orders.OrderSvc -> Orders.PaymentSvc "OrderCreated event"
  Orders.PaymentSvc -> Orders.OrderSvc "PaymentConfirmed event"
}

views {
  view index {
    include *
  }
}
```

## Saga

```sruja
specification {
  element system
  element container
}

model {
  Orders = system "Order System" {
    OrderSvc = container "Order Service"
    InventorySvc = container "Inventory Service"
    PaymentSvc = container "Payment Service"
  }
  
  scenario CreateOrderSaga "Order Creation Saga" {
    Orders.OrderSvc -> Orders.InventorySvc "Reserves stock"
    Orders.InventorySvc -> Orders.OrderSvc "Confirms reserved"
    Orders.OrderSvc -> Orders.PaymentSvc "Charges payment"
    Orders.PaymentSvc -> Orders.OrderSvc "Confirms charged"
  }
}

views {
  view index {
    include *
  }
}
```

## CQRS

```sruja
specification {
  element system
  element container
  element datastore
}

model {
  App = system "App" {
    CommandAPI = container "Command API"
    QueryAPI = container "Query API"
    ReadDB = datastore "Read Database"
    WriteDB = datastore "Write Database"
  }
  
  App.CommandAPI -> App.WriteDB "Writes"
  App.QueryAPI -> App.ReadDB "Reads"
}

views {
  view index {
    include *
  }
}
```

## RAG (Retrieval-Augmented Generation)

```sruja
specification {
  element system
  element container
  element datastore
}

model {
  AIQA = system "AI Q&A" {
    Indexer = container "Indexer"
    Retriever = container "Retriever"
    Generator = container "Generator"
    VectorDB = datastore "Vector Store"
  }
  
  AIQA.Indexer -> AIQA.VectorDB "Writes embeddings"
  AIQA.Retriever -> AIQA.VectorDB "Searches"
  AIQA.Generator -> AIQA.Retriever "Fetches contexts"
}
```

See `examples/pattern_rag_pipeline.sruja` for a production-ready model.

## Agentic Orchestration

```sruja
specification {
  element system
  element container
  element datastore
}

model {
  AgentSystem = system "Agent System" {
    Orchestrator = container "Agent Orchestrator"
    Planner = container "Planner"
    Executor = container "Executor"
    Tools = container "Tooling API"
    Memory = datastore "Long-Term Memory"
  }
  
  AgentSystem.Orchestrator -> AgentSystem.Planner "Plans tasks"
  AgentSystem.Orchestrator -> AgentSystem.Executor "Executes steps"
  AgentSystem.Executor -> AgentSystem.Tools "Calls tools"
  AgentSystem.Executor -> AgentSystem.Memory "Updates state"
}

views {
  view index {
    include *
  }
}
```

See `examples/pattern_agentic_ai.sruja` for a complete agent graph.

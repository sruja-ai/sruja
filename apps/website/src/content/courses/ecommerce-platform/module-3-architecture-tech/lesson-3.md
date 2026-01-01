---
title: "Lesson 3: API-First Design"
weight: 3
summary: "Design-first API development using OpenAPI and Sruja together."
---

# Lesson 3: API-First Design

Before frontend and backend teams start working, they need to agree on the API. This is where **API-First Design** comes in.

## API-First Design

Instead of writing code and then documenting it, we define the API schema first using **OpenAPI**. This allows frontend devs to mock the API while backend devs build it.

## Sruja's Role: Architecture Modeling

Sruja models **which services exist and how they connect**. For detailed API schemas (endpoints, request/response structures), use **OpenAPI/Swagger**.

```sruja
import { * } from 'sruja.ai/stdlib'


customer = person "Customer"

ecommerce = system "E-Commerce Platform" {
  api = container "Core API" {
    technology "Go, Gin"
    // API schemas defined in openapi.yaml
  }

  orderDB = database "Order Database" {
    technology "PostgreSQL"
  }

  api -> orderDB "reads and writes to"
}

customer -> ecommerce.api "uses"

view index {
  title "E-Commerce Architecture"
  include *
}
```

## Best Practice: Separation of Concerns

1. **Sruja**: Models architecture (services, containers, relationships)
2. **OpenAPI**: Defines API schemas (endpoints, request/response structures)
3. **Together**: Architecture shows the big picture, OpenAPI shows the details

## Why this matters

1. **Right tool for the job**: Architecture modeling vs. API specification
2. **Industry standard**: OpenAPI is widely supported by tools and frameworks
3. **Code Generation**: Generate Go structs, TypeScript interfaces, and client SDKs from OpenAPI

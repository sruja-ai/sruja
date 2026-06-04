---
title: "Lesson 2: Selecting the Stack"
weight: 2
summary: "Choosing technologies and documenting them in Containers."
---

# Lesson 2: Selecting the Stack

We have our domains. Now we need to pick the tools to build them.

## The Stack

1.  **Frontend**: Next.js (React) - Great for SEO and performance.
2.  **Backend**: Rust - Performance, safety, and great tooling for services.
3.  **Database**: PostgreSQL - Reliable, ACID compliant (critical for money).

## Modeling in Sruja

We define these choices in our `container` and `database` definitions.

```sruja
import { * } from 'sruja.ai/stdlib'

Platform = system "E-Commerce Platform" {
WebApp = container "Storefront & Admin" {
  technology "Next.js, TypeScript"
  description "The user-facing application."
}

API = container "Core API" {
  technology "Rust, Axum"
  description "REST API handling business logic."
}

Database = database "Primary DB" {
  technology "PostgreSQL 15"
  description "Stores orders, products, and users."
}

WebApp -> API "JSON/HTTPS"
API -> Database "SQL/TCP"
}
```

By documenting `technology`, we make it clear to new developers what skills they need.

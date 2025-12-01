---
title: "Lesson 2: Selecting the Stack"
weight: 2
summary: "Choosing technologies and documenting them in Containers."
---

# Lesson 2: Selecting the Stack

We have our domains. Now we need to pick the tools to build them.

## The Stack
1.  **Frontend**: Next.js (React) - Great for SEO and performance.
2.  **Backend**: Go (Golang) - High performance, great concurrency for e-commerce.
3.  **Database**: PostgreSQL - Reliable, ACID compliant (critical for money).

## Modeling in Sruja
We define these choices in our `container` definitions.

```sruja
system Platform "E-Commerce Platform" {
    
    container WebApp "Storefront & Admin" {
        technology "Next.js, TypeScript"
        description "The user-facing application."
    }

    container API "Core API" {
        technology "Go, Gin"
        description "REST API handling business logic."
    }

    container Database "Primary DB" {
        technology "PostgreSQL 15"
        description "Stores orders, products, and users."
    }

    WebApp -> API "JSON/HTTPS"
    API -> Database "SQL/TCP"
}
```

By documenting `technology`, we make it clear to new developers what skills they need.

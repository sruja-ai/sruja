---
title: "DSL Basics"
weight: 20
summary: "Learn Sruja syntax: systems, containers, persons, relations, and descriptions."
tags: ["dsl", "modeling"]
---

# DSL Basics

Sruja is an architecture DSL. This tutorial introduces its core elements.

## Elements

```sruja
import { * } from 'sruja.ai/stdlib'


Shop = system "Shop API" {
  WebApp = container "Web" {
    description "Gateway layer"
  }
  CatalogSvc = container "Catalog"
  MainDB = database "Database"
}

User = person "User"

User -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.CatalogSvc "Routes"
Shop.CatalogSvc -> Shop.MainDB "Reads/Writes"

view index {
  include *
}
```

## Descriptions and Metadata

```sruja
import { * } from 'sruja.ai/stdlib'


Payments = system "Payments" {
  description "Handles payments and refunds"
  // metadata
  metadata {
    team "FinTech"
    tier "critical"
  }
}
```

## Component‑level Modeling

```sruja
import { * } from 'sruja.ai/stdlib'


App = system "App" {
  Web = container "Web" {
    Cart = component "Cart"
  }
}
```

## Next Steps

- Learn [Deployment Modeling](tutorials/advanced/deployment-modeling.md) for infrastructure perspective
- Test yourself: see [Beginner path](../../docs/beginner-path.md) for optional quiz references.

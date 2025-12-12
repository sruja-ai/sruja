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
architecture "Shop" {
  system ShopSystem "API" {
    container WebApp "Web" {
      description "Gateway layer"
    }
    container CatalogSvc "Catalog"
    datastore MainDB "Database"
  }

  person User "User"

  User -> ShopSystem.WebApp "Uses"
  ShopSystem.WebApp -> ShopSystem.CatalogSvc "Routes"
  ShopSystem.CatalogSvc -> ShopSystem.MainDB "Reads/Writes"
}
```

## Descriptions and Metadata

```sruja
system Payments "Payments" {
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
system App {
  container Web {
    component Cart "Cart"
  }
}
```

## Next Steps

 - Learn [Deployment Modeling](/tutorials/advanced/deployment-modeling) for infrastructure perspective
- Take the quiz: [DSL Basics Quiz](/quizzes/dsl-basics)

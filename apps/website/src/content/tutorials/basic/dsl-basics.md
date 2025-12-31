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
person = kind "Person"
system = kind "System"
container = kind "Container"
datastore = kind "Datastore"

shop = system "Shop API" {
webApp = container "Web" {
  description "Gateway layer"
}
catalogSvc = container "Catalog"
mainDB = datastore "Database"
}

user = person "User"

user -> shop.webApp "Uses"
shop.webApp -> shop.catalogSvc "Routes"
shop.catalogSvc -> shop.mainDB "Reads/Writes"

view index {
include *
}
```

## Descriptions and Metadata

```sruja
system = kind "System"

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
system = kind "System"
container = kind "Container"
component = kind "Component"

App = system "App" {
Web = container "Web" {
  Cart = component "Cart"
}
}
```

## Next Steps

- Learn [Deployment Modeling](/tutorials/advanced/deployment-modeling) for infrastructure perspective
- Take the quiz: [DSL Basics Quiz](/quizzes/dsl-basics)

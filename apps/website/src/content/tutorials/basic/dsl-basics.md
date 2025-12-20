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
specification {
  element person
  element system
  element container
  element datastore
}

model {
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
}

views {
  view index {
    include *
  }
}
```

## Descriptions and Metadata

```sruja
specification {
  element system
}

model {
  Payments = system "Payments" {
    description "Handles payments and refunds"
    // metadata
    metadata {
      team "FinTech"
      tier "critical"
    }
  }
}
```

## Component‑level Modeling

```sruja
specification {
  element system
  element container
  element component
}

model {
  App = system "App" {
    Web = container "Web" {
      Cart = component "Cart"
    }
  }
}
```

## Next Steps

 - Learn [Deployment Modeling](/tutorials/advanced/deployment-modeling) for infrastructure perspective
- Take the quiz: [DSL Basics Quiz](/quizzes/dsl-basics)

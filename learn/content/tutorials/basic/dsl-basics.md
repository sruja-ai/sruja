---
title: "DSL Basics"
weight: 20
summary: "Learn Sruja syntax: systems, containers, persons, relations, and descriptions."
tags: [dsl]
aliases: ["/tutorials/dsl-basics/"]
---

# DSL Basics

Sruja is an architecture DSL. This tutorial introduces its core elements.

## Elements

```sruja
architecture "Shop" {
  system API "API" {
    container Web "Web" {
      description "Gateway layer"
    }
    container Catalog "Catalog"
    datastore DB "Database"
  }

  person User "User"

  User -> API.Web "Uses"
  API.Web -> API.Catalog "Routes"
  API.Catalog -> API.DB "Reads/Writes"
}
```

## Descriptions and Metadata

```sruja
system Payments "Payments" {
  description "Handles payments and refunds"
  // metadata
  metadata {
    team: "FinTech"
    tier: "critical"
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

## Module and Context (Logical Grouping)

Sruja supports logical grouping with `module` (primary) or `context` (DDD alias):

```sruja
architecture "Shop" {
  system API {
    // module is the primary keyword for logical grouping
    module Orders {
      component OrderController
    }
    
    // context is an alias (DDD style)
    context Payments {
      component PaymentProcessor
    }
  }
}
```

**Note**: `module`, `context`, and `boundedContext` are aliases - use whichever feels natural.

## Unified `data` Keyword

The `data` keyword is context-aware - it means different things in different contexts:

```sruja
architecture "Shop" {
  system API {
    module Orders {
      // In module: Domain model
      data Order {
        id string
        total float
      }
    }
    
    // In datastore: Database table
    datastore DB {
      data User {
        id string
        email string
      }
    }
    
    // In api: Request/response schema
    api PlaceOrder {
      request Order  // Uses data Order as schema
    }
  }
}
```

**Simple Insight**: One keyword (`data`), multiple meanings based on context. Reduces vocabulary size.


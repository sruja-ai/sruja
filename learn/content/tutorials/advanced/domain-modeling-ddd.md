---
title: "Domain Modeling (DDD)"
weight: 60
summary: "Model entities, value objects, and domain events with Sruja. Understand when to use domain vs system."
tags: [ddd]
aliases: ["/tutorials/domain-modeling-ddd/"]
---

# Domain Modeling (DDD)

Sruja supports modeling domain entities, value objects, and events using the `domain` keyword.

## System vs Domain: Different Perspectives

**Important**: `system` and `domain` are **not alternatives** - they represent **different perspectives**:

| Perspective | Keyword | Purpose | When to Use |
| :--- | :--- | :--- | :--- |
| **Physical/Deployment** | `system` | Technical architecture, deployment units | Modeling how the system is deployed and runs |
| **Logical/Business** | `domain` | Business domain, bounded contexts | Modeling business logic and domain concepts |

**They can coexist** in the same architecture:

```sruja
architecture "E-Commerce" {
    // Physical view: How it's deployed
    system ShopAPI {
        container WebApp "Web Application"
        container Database "PostgreSQL Database"
    }
    
    // Logical view: Business domain
    domain ECommerce {
        context Orders {
            aggregate Order {
                entity OrderLineItem {
                    name string
                    quantity int
                }
                valueObject Money {
                    amount float
                    currency string
                }
            }
        }
    }
}
```

**Key Insight**: Use `system` when thinking about **deployment and technology**. Use `domain` when thinking about **business logic and domain modeling**. They complement each other, not replace each other.

## Entities & Value Objects

```sruja
architecture "Shop" {
  domain Sales "Sales" {
    context Orders {
      aggregate Order {
        entity OrderLineItem {
          name string
          quantity int
        }
        valueObject Money {
          amount float
          currency string
        }
      }
    }
  }
}
```

## Domain Events

```sruja
event OrderPlaced {
  orderId string
  customerId string
}
```

## When to Use Domain vs System

### Use `system` when:
- Modeling deployment and technology choices
- Showing how the system runs (containers, services)
- Focusing on infrastructure and scalability

### Use `domain` when:
- Modeling business logic and concepts
- Showing bounded contexts and aggregates
- Focusing on domain-driven design

### Use both when:
- You need both perspectives (recommended for larger systems)
- Different teams need different views (DevOps vs Developers)
- You want a complete architectural picture


---
title: "Lesson 1: Microservices Architecture"
weight: 1
summary: "Monolith vs Microservices, Service Boundaries."
---

# Lesson 1: Microservices Architecture

## Monolith vs. Microservices

### Monolithic Architecture

A single application where all functionality is packaged together.

- **Pros:** Simple to develop, deploy, and test initially.
- **Cons:** Hard to scale specific parts, tight coupling, single point of failure.

### Microservices Architecture

A collection of small, independent services that communicate over a network.

- **Pros:** Independent scaling, technology diversity, fault isolation.
- **Cons:** Distributed system complexity, network latency, data consistency challenges.

## Defining Service Boundaries

The hardest part of microservices is deciding where to draw the lines. Common strategies include:

- **Business Capability:** Group by what the business does (e.g., Billing, Shipping).
- **Subdomain:** Group by Domain-Driven Design (DDD) subdomains.

---

## 🛠️ Sruja Perspective: Modeling Microservices

In Sruja, microservices are typically modeled as separate `container` items within a `system`, or even as separate `system` items if they are large enough.

### Basic Example

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

OrderSystem = system "Order Management" {
    OrderService = container "Order Service" {
        technology "Go"
        description "Handles order placement and tracking."
    }
    OrderDB = database "Order Database" {
        technology "PostgreSQL"
    }
    OrderService -> OrderDB "Reads/Writes"
}

InventorySystem = system "Inventory Management" {
    InventoryService = container "Inventory Service" {
        technology "Java"
        description "Tracks stock levels."
    }
    InventoryDB = database "Inventory Database" {
        technology "PostgreSQL"
    }
    InventoryService -> InventoryDB "Reads/Writes"
}

// Inter-service communication
Customer -> OrderSystem.OrderService "Places order"
OrderSystem.OrderService -> InventorySystem.InventoryService "Reserves stock"

// Requirements drive architecture
requirement R1 functional "Must handle 10k orders/day"
requirement R2 performance "Order placement < 500ms"
requirement R3 scalability "Scale order processing independently"

// Document decisions
adr ADR001 "Split into microservices" {
    status "Accepted"
    context "Need independent scaling for order vs inventory"
    decision "Separate OrderSystem and InventorySystem"
    consequences "Better scalability, network latency overhead"
}

view index {
title "System Overview"
include *
}

// Developer perspective: Focus on services and APIs
view developer {
title "Developer View - Service Architecture"
include OrderSystem OrderSystem.OrderService OrderSystem.OrderDB
include InventorySystem InventorySystem.InventoryService InventorySystem.InventoryDB
exclude Customer
}

// Product perspective: Focus on user experience
view product {
title "Product View - User Journey"
include Customer
include OrderSystem
exclude InventorySystem InventorySystem.InventoryDB
}

// Data flow perspective: Show data dependencies
view dataflow {
title "Data Flow View"
include OrderSystem.OrderService OrderSystem.OrderDB
include InventorySystem.InventoryService InventorySystem.InventoryDB
exclude Customer
}
```

### Key Benefits of Multiple Views

1. **Different Audiences**: Developers see technical details, product managers see user flows
2. **Reduced Complexity**: Each view focuses on what matters for that perspective
3. **Better Communication**: Stakeholders get diagrams tailored to their needs
4. **Documentation**: Multiple views serve as different types of documentation

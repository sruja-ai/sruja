---
title: "Lesson 1: Microservices Architecture"
weight: 1
summary: "Monolith vs Microservices, Service Boundaries."
---

# Lesson 1: Microservices Architecture

## Monolith vs. Microservices

### Monolithic Architecture
A single application where all functionality is packaged together.
*   **Pros:** Simple to develop, deploy, and test initially.
*   **Cons:** Hard to scale specific parts, tight coupling, single point of failure.

### Microservices Architecture
A collection of small, independent services that communicate over a network.
*   **Pros:** Independent scaling, technology diversity, fault isolation.
*   **Cons:** Distributed system complexity, network latency, data consistency challenges.

## Defining Service Boundaries

The hardest part of microservices is deciding where to draw the lines. Common strategies include:
*   **Business Capability:** Group by what the business does (e.g., Billing, Shipping).
*   **Subdomain:** Group by Domain-Driven Design (DDD) subdomains.

---

## 🛠️ Sruja Perspective: Modeling Microservices

In Sruja, microservices are typically modeled as separate `container` items within a `system`, or even as separate `system` items if they are large enough.

```sruja
architecture "E-Commerce Platform" {
    system OrderSystem "Order Management" {
        container OrderService "Order Service" {
            technology "Go"
            description "Handles order placement and tracking."
        }
    }

    system InventorySystem "Inventory Management" {
        container InventoryService "Inventory Service" {
            technology "Java"
            description "Tracks stock levels."
        }
    }

    // Inter-service communication
    // Inter-service communication
    OrderService -> InventoryService "Reserves stock"
}
```

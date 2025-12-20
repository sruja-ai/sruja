---
title: "Lesson 1: The \"Good\" Problem (Traffic Spikes)"
weight: 1
summary: "Refactoring from Monolith to Microservices when necessary."
---

# Lesson 1: The "Good" Problem

You have too many users. Your single database is melting. It's time to scale.

## The Bottleneck
Our monitoring (Module 5) shows that `Inventory` checks are 80% of the database load.

## The Refactor: Splitting the Monolith
We decide to extract `Inventory` into its own microservice with its own database.

### Step 1: Update the Architecture
We change `Inventory` from a logical domain inside the monolith to a physical `system`.

```sruja
// Before
domain Inventory { ... }

// After
specification {
  element system
  element container
}

model {
  InventoryService = system "Inventory Microservice" {
    API = container "Inventory API"
    Database = container "Inventory DB"
  }
}
```

### Step 2: Update the Contracts
The `OrderService` can no longer call `Inventory` functions directly. It must make a gRPC call.

```sruja
specification {
  element system
  element container
}

model {
  OrderService = system "Order Service" {
    // ...
    OrderService -> InventoryService "gRPC CheckStock"
  }
}
```

## Why Sruja helps
Refactoring is dangerous. Sruja helps you visualize the *impact* of the change before you write code. You can see exactly which other systems depend on Inventory and ensure you don't break them.

---
title: "Lesson 4: Contracts & APIs"
weight: 4
summary: "Design stable APIs and event schemas for the platform."
---

# Lesson 4: Contracts & APIs

## Why Contracts?
Contracts define stable interfaces between services; they reduce coupling and surprises.

## Sruja: API & Event Contracts

```sruja
architecture "E-Commerce" {
  contracts {
    api CheckoutAPI {
      version "v1"
      endpoint "/checkout" method "POST"
      request "Cart"
      response "Order"
    }

    event OrderPlaced {
      version "v1"
      topic "orders.placed"
      payload "Order"
    }
  }
}
```

## Practice
- Define an API contract for `AddToCart`.
- Add an `OrderCancelled` event with payload fields.


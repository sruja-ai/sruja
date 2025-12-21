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
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
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

views {
  view index {
    include *
  }
}
```

## Practice
- Define an API contract for `AddToCart`.
- Add an `OrderCancelled` event with payload fields.


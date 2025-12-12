---
title: "Scenario"
weight: 22
summary: "Describe behavioral flows as steps between elements."
---

# Scenario

Scenarios describe behavioral flows as ordered steps. They focus on interactions rather than data pipelines.

## Syntax

```sruja
architecture "Checkout" {
  person Customer
  system Shop {
    container WebApp
    container API
    datastore DB
  }

  scenario CheckoutFlow "User Checkout" {
    Customer -> Shop.WebApp "adds items to cart"
    Shop.WebApp -> Shop.API "submits cart"
    Shop.API -> Shop.DB "validates and reserves stock"
    Shop.API -> Shop.WebApp "returns confirmation"
    Shop.WebApp -> Customer "displays success"
  }
}
```

## Scenario vs Flow

Sruja supports two similar constructs for modeling interactions:

- **`scenario`**: Models behavioral flows - what happens when a user interacts with the system (user stories, use cases)
- **`flow`**: Models data flows - how data moves through the system (Data Flow Diagrams, DFD-style)

Both use the same syntax with relations between elements, but serve different purposes:

```sruja
// Scenario: User behavior
scenario Checkout "User Checkout" {
  Customer -> Shop.WebApp "adds items to cart"
  Shop.WebApp -> Shop.API "submits cart"
}

// Flow: Data flow
flow OrderProcess "Order Processing" {
  Customer -> Shop "Order Details"
  Shop -> Shop.API "Processes"
  Shop.API -> Shop.DB "Save Order"
}
```

**When to use:**
- Use `scenario` for user journeys, business processes, and behavioral flows
- Use `flow` for data pipelines, ETL processes, and system-to-system data flows

## Tips

- Keep step labels short and action‑oriented.
- Use fully qualified names when referring outside the current context.
- Use `scenario` for behavior; use `flow` for data flows; use relations for structure.

## See Also

- [Layering](/docs/concepts/layering)
- [Validation](/docs/concepts/validation)

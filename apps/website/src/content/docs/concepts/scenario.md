---
title: "Scenario"
weight: 6
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

## Tips

- Keep step labels short and action‑oriented.
 - Use fully qualified names when referring outside the current context.
 - Use `scenario` for behavior; use relations for structure.

## See Also

- [Layering](/docs/concepts/layering)
- [Validation](/docs/concepts/validation)

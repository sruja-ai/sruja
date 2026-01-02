---
title: "Scenario"
weight: 22
summary: "Describe behavioral flows as steps between elements."
---

# Scenario

Scenarios describe behavioral flows as ordered steps. They focus on interactions rather than data pipelines.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


Customer = person "Customer"
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
  DB = database "Database"
}

// Scenarios using flat syntax
CheckoutFlow = scenario "User Checkout" {
  step Customer -> Shop.WebApp "adds items to cart"
  step Shop.WebApp -> Shop.API "submits cart"
  step Shop.API -> Shop.DB "validates and reserves stock"
  step Shop.API -> Shop.WebApp "returns confirmation"
  step Shop.WebApp -> Customer "displays success"
}

// 'story' is an alias for 'scenario'
LoginStory = story "User Login" {
  step Customer -> Shop.WebApp "enters credentials"
  step Shop.WebApp -> Shop.API "validates user"
}

view index {
  include *
}
```

## Aliases & Semantics

Sruja provides three keywords that are **structurally identical** (sharing the same underlying AST definition and syntax) but convey different semantic intent:

- **`scenario`**: Models behavioral flows (e.g., Use Cases, User Journeys).
- **`story`**: An alias for `scenario` (e.g., User Stories).
- **`flow`**: Models data movement (e.g., Data Flow Diagrams).

While the syntax is the same, using the appropriate keyword helps readers understand the _nature_ of the interaction being modeled.

```sruja
import { * } from 'sruja.ai/stdlib'


Customer = person "Customer"
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
  DB = database "Database"
}

// Scenario: User behavior
Checkout = scenario "User Checkout" {
  Customer -> Shop.WebApp "adds items to cart"
  Shop.WebApp -> Shop.API "submits cart"
}

// Flow: Data flow
OrderProcess = flow "Order Processing" {
  Customer -> Shop "Order Details"
  Shop -> Shop.API "Processes"
  Shop.API -> Shop.DB "Save Order"
}
```

**When to use:**

- Use `scenario` for user journeys, business processes, and behavioral flows
- Use `flow` for data pipelines, ETL processes, and system-to-system data flows

## Tips

- Keep step labels short and action‑oriented
- Use fully qualified names when referring outside the current context
- Use `scenario` or `story` for behavior; use `flow` for data flows; use relations for structure

## See Also

- [Layering](/docs/concepts/layering)
- [Validation](/docs/concepts/validation)

---
title: "Behavior and depends_on"
weight: 21
summary: "Describe component behavior and upstream dependencies explicitly."
---

# Behavior and depends_on

Use `behavior` to document responsibilities; use `depends_on` to note upstreams.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


App = system "App" {
  API = container "API" {
    CheckoutService = component "Checkout Service"
  }

  DB = database "Database" { tags ["internal"] }
}

// dependency relations at architecture level
App.API.CheckoutService -> App.DB "reads/writes"
App.API.CheckoutService -> PaymentGateway "calls"

PaymentGateway = system "Payment Gateway" { tags ["external"] }

```

## Guidance

- Keep behavior bullets action‑oriented and concise.
- Use `depends_on` to surface operational and failure domains.
- Mark external boundaries with `tags ["external"]` on systems.

## Related

- `relations` for runtime calls
- `tags` and metadata for classification

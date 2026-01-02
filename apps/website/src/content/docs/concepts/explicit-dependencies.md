---
title: "Explicit Dependencies"
weight: 21
summary: "Note upstream dependencies explicitly."
---

# Explicit Dependencies

Use `depends_on` to note upstreams when a full relation is overkill or for static dependencies.

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

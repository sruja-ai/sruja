---
title: "Behavior and depends_on"
weight: 21
summary: "Describe component behavior and upstream dependencies explicitly."
---

# Behavior and depends_on

Use `behavior` to document responsibilities; use `depends_on` to note upstreams.

## Syntax

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
  system App {
    container API {
      component CheckoutService {}
    }

    datastore DB { tags ["internal"] }
  }

  // dependency relations at architecture level
  App.API.CheckoutService -> App.DB "reads/writes"
  App.API.CheckoutService -> PaymentGateway "calls"

  system PaymentGateway { tags ["external"] }

}
```

## Guidance
- Keep behavior bullets action‑oriented and concise.
- Use `depends_on` to surface operational and failure domains.
- Mark external boundaries with `tags ["external"]` on systems.

## Related
- `relations` for runtime calls
- `tags` and metadata for classification

---
title: "Contracts, Constraints, Conventions"
weight: 43
summary: "Specify APIs/events/data, architectural constraints, and shared conventions."
---

# Contracts, Constraints, Conventions

Use these blocks to formalize interfaces, limits, and team agreements.

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
  contracts {
    api CheckoutAPI {
      version "v1"
      endpoint "/checkout" method "POST"
    }

    event OrderPlaced {
      version "v1"
      topic "orders.placed"
      payload "Order"
    }

    data Order {
      version "v1"
      schema {
        entries {
          key "id" type string
          key "total" type number
        }
      }
    }
  }

  constraints {
    rule "No direct WebApp -> DB writes"
    rule "Only managed Postgres for relational data"
  }

  conventions {
    naming "kebab-case for services"
    tracing "W3C trace context propagated across services"
  }

}
```

## Guidance
- Keep `contracts` versioned and narrow; prefer stable event schemas.
- Use `constraints` for firm boundaries (security, compliance, architecture).
- Capture team `conventions` to improve consistency across services.

## Related
- `policy` for governance documents
- `adr` for rationale behind constraints

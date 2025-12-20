---
title: "Scale"
weight: 32
summary: "Express capacity ranges and metrics for components and systems."
---

# Scale

Use `scale` to capture expected capacity, concurrency, or throughput ranges.

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
      scale {
        metric "req/s"
        min 200
        max 2000
      }
    }
  }
}

views {
  view index {
    include *
  }
}
```

## Guidance
- Choose a single clear `metric` per block (e.g., `req/s`, `users`, `messages/s`).
- Use realistic bounds to guide design and SLOs.
- Pair with `slo` and `requirements` to encode targets.

## Related
- `slo` for availability/latency/errorRate/throughput targets
- `requirements` for performance expectations


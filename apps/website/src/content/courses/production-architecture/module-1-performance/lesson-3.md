---
title: "Lesson 3: SLO Enforcement in Practice"
weight: 3
summary: "Use SLOs to drive scaling, alerting, and design trade‑offs."
---

# Lesson 3: SLO Enforcement in Practice

## SLO‑Driven Operations
Use SLOs to set thresholds for alerts and capacity changes.

## Sruja: Model SLOs & Validate

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
  system API {
    container Gateway {
      scale { metric "req/s" min 500 max 5000 }
    }
  }

  slo {
    availability { target "99.95%" window "30 days" }
    latency { p95 "150ms" window "7 days" }
    errorRate { target "< 0.05%" window "30 days" }
    throughput { target "3000 req/s" window "1 hour" }
  }
}

views {
  view index {
    include *
  }
}
```

```bash
sruja lint payments.sruja
```

## Practice
- Define SLOs for your critical path; ensure `scale` bounds meet `throughput`.
- Set alert thresholds aligned to SLO windows.


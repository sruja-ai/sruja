---
title: "Lesson 4: SLOs & Scale Integration"
weight: 4
summary: "Define SLOs and align scale to meet targets."
---

# Lesson 4: SLOs & Scale Integration

## Why SLOs?
SLOs set measurable targets (availability, latency, error rate, throughput). They guide capacity and design.

## Sruja: SLO + Scale

```sruja
architecture "E-Commerce Platform" {
  system Shop {
    container API {
      scale {
        metric "req/s"
        min 200
        max 2000
      }
    }
  }

  slo {
    availability { target "99.9%" window "30 days" }
    latency { p95 "200ms" window "7 days" }
    errorRate { target "< 0.1%" window "30 days" }
    throughput { target "1000 req/s" window "1 hour" }
  }
}
```

## Practice
- Set `p95` and `availability` targets for the API.
- Adjust `scale` bounds to keep `throughput` above target.


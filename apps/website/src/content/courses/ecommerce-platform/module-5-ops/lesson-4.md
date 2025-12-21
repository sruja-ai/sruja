---
title: "Lesson 4: Ops SLOs & Monitoring"
weight: 4
summary: "Set SLO targets and align alerts and dashboards."
---

# Lesson 4: Ops SLOs & Monitoring

## SLOs in Ops
Translate business expectations into measurable targets; build dashboards around them.

## Sruja: Define SLOs

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
  slo {
    availability { target "99.9%" window "30 days" }
    latency { p95 "200ms" window "7 days" }
    errorRate { target "< 0.1%" window "30 days" }
    throughput { target "1000 req/s" window "1 hour" }
  }
}

views {
  view index {
    include *
  }
}
```

## Practice
- Set `p95` latency targets for checkout and search.
- Map alerts to SLO windows; define runbooks for breaches.


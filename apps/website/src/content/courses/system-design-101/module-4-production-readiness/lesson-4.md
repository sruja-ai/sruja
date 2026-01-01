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
import { * } from 'sruja.ai/stdlib'


ECommerce = system "E-Commerce Platform" {
API = container "API Service" {
  technology "Go"

  // Scale configuration aligned with SLOs
  scale {
    metric "req/s"
    min 200
    max 2000
  }

  // SLOs define what "good" looks like
  slo {
    availability {
      target "99.9%"
      window "30 days"
      current "99.95%"
    }
    latency {
      p95 "200ms"
      p99 "500ms"
      window "7 days"
      current {
        p95 "180ms"
        p99 "450ms"
      }
    }
    errorRate {
      target "< 0.1%"
      window "30 days"
      current "0.05%"
    }
    throughput {
      target "1000 req/s"
      window "1 hour"
      current "950 req/s"
    }
  }
}

Database = database "PostgreSQL" {
  technology "PostgreSQL"
  slo {
    availability {
      target "99.9%"
      window "30 days"
    }
    latency {
      p95 "50ms"
      p99 "100ms"
    }
  }
}

API -> Database "Reads/Writes"
}

view index {
title "Production System with SLOs"
include *
}

// SLO monitoring view
view slos {
title "SLO Monitoring View"
include ECommerce.API ECommerce.Database
}
```

### Key Integration Points

1. **Scale aligns with SLOs**: Min/max replicas support throughput targets
2. **SLOs guide monitoring**: Define what metrics to track and alert on
3. **Current vs Target**: Track progress toward SLO targets
4. **Multiple SLO types**: Availability, latency, error rate, throughput

## Practice

- Set `p95` and `availability` targets for the API.
- Adjust `scale` bounds to keep `throughput` above target.

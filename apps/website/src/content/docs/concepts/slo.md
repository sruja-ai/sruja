---
title: "Service Level Objectives (SLO)"
weight: 52
summary: "Define performance and reliability targets for your systems."
---

# Service Level Objectives (SLO)

Service Level Objectives (SLOs) define measurable targets for system performance and reliability. Sruja allows you to document SLOs directly in your architecture model, keeping performance requirements close to your design.

## Syntax

SLOs are defined at the system or container level using a `slo` block:

```sruja
import { * } from 'sruja.ai/stdlib'


API = system "API Service" {
  WebServer = container "Web Server" {
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
}
```

## Example

```sruja
import { * } from 'sruja.ai/stdlib'


PaymentService = system "Payment Service" {
  PaymentAPI = container "Payment API" {
    technology "Go"
    slo {
      availability {
        target "99.99%"
        window "30 days"
      }
      latency {
        p95 "100ms"
        p99 "250ms"
        window "7 days"
      }
      errorRate {
        target "< 0.01%"
        window "30 days"
      }
    }
  }
}

view index {
  include *
}
```

## Benefits

- **Documentation**: SLOs are part of your architecture, not separate documents
- **Validation**: Can be validated against actual metrics
- **Communication**: Clear performance expectations for stakeholders
- **Planning**: Helps with capacity planning and resource allocation

## See Also

- [Container](/docs/concepts/container)
- [System](/docs/concepts/system)
- [Metadata & Tags](/docs/concepts/metadata-and-tags)

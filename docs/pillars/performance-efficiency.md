# Performance Efficiency Pillar

This document describes how Sruja supports the Performance Efficiency pillar of the Well-Architected Framework.

[← Back to Pillars Index](./README.md)

## Overview

**Performance Efficiency** focuses on using IT and computing resources efficiently to meet system requirements, and evolving technology choices over time.

**Includes:**
- Selecting appropriate resource types/sizes
- Monitoring performance
- Evolving technology
- Automating changes

---

## Core Support (Basic)

The core DSL includes basic performance features:

### Basic Latency

```sruja
system MyService {
  latency {
    target: "<200ms"
  }
}
```

### Basic Throughput

```sruja
system MyService {
  throughput {
    target: "1000 rps"
  }
}
```

### Scaling

```sruja
system MyService {
  scaling {
    min: 2
    max: 10
  }
}
```

---

## Advanced Extensions

### Performance DSL

Complete performance modeling:

```sruja
performance {
  latency APIService {
    p50: 22ms
    p90: 50ms
    p99: 120ms
  }

  throughput APIService {
    max_rps: 2000
    sustained_rps: 600
  }

  scaling APIService {
    min: 2
    max: 30
    target_rps: 1500
    metric: "cpu"
    cooldown: "5m"
  }

  sli latency_sli {
    target: APIService.latency.p99
  }

  slo CheckoutSLO {
    sli: latency_sli
    objective: "<200ms"
    period: "30d"
  }

  performance_budget CheckoutBudget {
    max_latency: 200ms
    max_hops: 5
    max_payload: 10mb
  }

  workload MorningTraffic {
    pattern: "cyclical"
    peak_rps: 5000
    offpeak_rps: 800
  }

  perf_path CheckoutFlow {
    steps: [APIService, AuthService, OrderService, PaymentService]
  }

  capacity Database {
    max_qps: 5000
    storage_limit: 4tb
    iops_limit: 15000
  }
}
```

### Resource Selection

```sruja
resources {
  compute APIServiceCompute {
    type: "ec2"
    instance_type: "c5.xlarge"
    rationale: "CPU-optimized for API processing"
  }

  database DatabaseResource {
    type: "rds"
    instance_type: "db.r5.2xlarge"
    storage_type: "gp3"
  }
}
```

### Caching

```sruja
caching {
  cache RedisCache {
    type: "redis"
    ttl: "1h"
    strategy: "write-through"
  }

  cdn CloudFrontCDN {
    provider: "aws-cloudfront"
    cache_behavior: "cache-first"
  }
}
```

### Performance Optimization

```sruja
optimization {
  database_indexing {
    table: "orders"
    indexes: ["user_id", "created_at", "status"]
  }

  query_optimization {
    query: "GetUserOrders"
    optimization: "use_index"
  }
}
```

---

## Integration with Other Pillars

### Operational Excellence
- Performance monitoring
- Performance alerting

### Reliability
- Performance under load
- Degradation strategies

### Cost Optimization
- Right-sizing resources
- Cost-performance trade-offs

---

## Validation Rules

Performance validation includes:

- ✅ All services must have latency targets
- ✅ Critical paths must have SLOs
- ✅ Resources must be right-sized
- ✅ Caching strategies must be defined
- ✅ Scaling policies must be configured

---

## Engines

### Core Engines
- **Basic Latency Engine** - Latency target validation
- **Basic Throughput Engine** - Throughput target checks
- **Basic Scaling Engine** - Min/max scaling validation

### Advanced Engines
- **Performance Engine** - Full performance DSL support
- **Latency Analysis Engine** - P50, P90, P99 analysis
- **Throughput Analysis Engine** - RPS, QPS, TPS analysis
- **SLO/SLI Engine** - Service level objectives/indicators
- **Performance Budget Engine** - Performance budget tracking
- **Bottleneck Detection Engine** - Identify performance bottlenecks
- **Capacity Planning Engine** - Resource capacity forecasting
- **Workload Analysis Engine** - Traffic pattern analysis
- **Performance Optimization Engine** - Query, algorithm optimization
- **Caching Strategy Engine** - Cache placement and strategy
- **Resource Right-Sizing Engine** - Optimal resource selection
- **Scaling Strategy Engine** - Horizontal/vertical scaling
- **Load Balancing Engine** - Load distribution strategies
- **Resource Selection Engine** - Match tech to requirements
- **Technology Fit Engine** - Technology recommendation
- **Architecture Ranking Engine** - Score scenarios for optimal choice

See: [Engines by Pillars](./engines.md#performance-efficiency-pillar) for complete list

---

## DSL Reference

See: [DSL Extensions - Performance](../specs/dsl-extensions.md#2-performance-dsl)

---

## Examples

- [Performance Example](../examples/performance/)
- [Scaling Strategies](../examples/scaling/)

---

*Performance Efficiency ensures your architecture uses resources efficiently while meeting performance requirements.*


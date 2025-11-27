# DSL Extensions Reference

This document provides a comprehensive reference for all DSL extensions covering various aspects of software engineering.

**Note**: DSL extensions are organized around the [Well-Architected Framework pillars](../pillars/README.md). See the [pillar mapping](../pillars/mapping.md) for how extensions map to pillars.

[← Back to Documentation Index](../README.md)

## Overview

Sruja DSL is designed to be extensible, allowing you to model not just architecture structure, but also:

- **Resilience & Reliability** - Timeouts, retries, circuit breakers, health checks
- **Performance** - Latency, throughput, scaling, SLIs/SLOs
- **Cost & FinOps** - Cost models, budgets, optimization
- **Security** - Authentication, authorization, encryption, compliance
- **Observability** - Metrics, logging, tracing, monitoring
- **Compliance & Governance** - Policies, rules, controls, approvals
- **And more...**

Each extension is optional and can be used independently or combined as needed.

---

## 1. Resilience & Reliability DSL

Model fault tolerance, retry policies, circuit breakers, and reliability patterns.

### Basic Syntax

```sruja
// Timeout configuration
timeout APIServiceTimeout {
  duration: "2s"
  enforce: true
}

// Retry policy
retry_policy ExponentialRetry {
  attempts: 3
  backoff: "exponential"
  base_delay: "100ms"
  max_delay: "5s"
  jitter: true
}

// Circuit breaker
circuit_breaker PaymentCB {
  failure_threshold: "50%"
  open_duration: "30s"
  half_open_requests: 5
  fallback: "default_payment"
}

// Health check
health_check FullHealth {
  endpoint: "/health"
  liveness: "/health/live"
  readiness: "/health/ready"
  interval: "10s"
  timeout: "2s"
}

// Autoscaling
autoscale CheckoutAS {
  min: 2
  max: 20
  metric: "cpu"
  target: "60%"
}

// Resilience profile (combines all)
resilience CheckoutResilience {
  service: CheckoutService
  timeout: "2s"
  retry_policy: ExponentialRetry
  circuit_breaker: PaymentCB
  health: FullHealth
  autoscale: CheckoutAS
}
```

### Full Grammar

See the conversation file for complete Ohm grammar covering:
- Timeouts (global and endpoint-specific)
- Retry policies (exponential, linear, custom)
- Circuit breakers
- Bulkheads
- Rate limiting
- Backpressure
- Degradation strategies
- Auto-healing
- Failover strategies
- Error budgets
- Reliability scoring
- Failure scenarios

---

## 2. Performance DSL

Model latency, throughput, scaling, and performance characteristics.

### Basic Syntax

```sruja
// Latency specification
latency APIService {
  p50: 22ms
  p90: 50ms
  p99: 120ms
}

// Throughput
throughput APIService {
  max_rps: 2000
  sustained_rps: 600
}

// Scaling configuration
scaling APIService {
  min: 2
  max: 30
  target_rps: 1500
  metric: "cpu"
  cooldown: "5m"
}

// SLI/SLO
sli latency_sli {
  target: APIService.latency.p99
}

slo CheckoutSLO {
  sli: latency_sli
  objective: "<200ms"
  period: "30d"
}

// Performance budget
performance_budget CheckoutBudget {
  max_latency: 200ms
  max_hops: 5
  max_payload: 10mb
}

// Workload patterns
workload MorningTraffic {
  pattern: "cyclical"
  peak_rps: 5000
  offpeak_rps: 800
}

// Performance path
perf_path CheckoutFlow {
  steps: [APIService, AuthService, OrderService, PaymentService]
}
```

### Full Grammar

See the conversation file for complete grammar covering:
- Latency (p50, p90, p99)
- Throughput (max, sustained, burst)
- Concurrency limits
- Queue modeling
- Vertical and horizontal scaling
- SLIs and SLOs
- Error budgets
- Performance budgets
- Workload patterns
- Throughput paths
- Bottleneck identification
- Capacity planning
- Network latency/bandwidth
- Rate limiting
- Backpressure
- Retry and circuit breaker (performance-focused)

---

## 3. Cost & FinOps DSL

Model costs, budgets, and financial operations.

### Basic Syntax

```sruja
// Cost model
cost_model APIServiceCost {
  compute: "$0.10/hour"
  storage: "$0.05/GB/month"
  network: "$0.01/GB"
}

// Budget
budget MonthlyBudget {
  limit: "$5000"
  period: "1 month"
  alert_threshold: "80%"
}

// Cost per environment
cost EnvironmentCost {
  environment: "production"
  estimated_monthly: "$2000"
}

// Per-request cost
cost_per_request CheckoutRequest {
  service: CheckoutService
  estimated_cost: "$0.001"
}
```

### Features

- Cost models for services
- Budget definitions and tracking
- Environment-specific costing
- Per-request cost modeling
- Cost optimization strategies
- Cost allocation

---

## 4. Security DSL

Model security requirements, authentication, authorization, and compliance.

### Basic Syntax

```sruja
// Authentication
authentication OAuth2Auth {
  type: "OAuth2"
  provider: "Auth0"
  scopes: ["read", "write"]
}

// Authorization
authorization RBAC {
  model: "role-based"
  roles: ["admin", "user", "guest"]
}

// Encryption
encryption DataEncryption {
  at_rest: "AES-256"
  in_transit: "TLS 1.3"
}

// Security policy
security_policy PaymentPolicy {
  requires: [encryption, authentication, audit_logging]
  compliance: ["PCI-DSS", "GDPR"]
}
```

### Features

- Authentication mechanisms
- Authorization models
- Encryption (at rest, in transit)
- Security policies
- Compliance requirements
- Audit logging
- Data classification
- Access controls

---

## 5. Observability DSL

Model metrics, logging, tracing, and monitoring.

### Basic Syntax

```sruja
// Metrics
metrics APIMetrics {
  latency: true
  throughput: true
  error_rate: true
  custom: ["queue_depth", "cache_hit_rate"]
}

// Logging
logging APILogging {
  level: "info"
  format: "json"
  retention: "30d"
  destinations: ["cloudwatch", "elasticsearch"]
}

// Tracing
tracing APITracing {
  enabled: true
  sampling_rate: "10%"
  backend: "jaeger"
}

// Monitoring
monitoring APIMonitoring {
  alerts: [
    { metric: "error_rate", threshold: "1%", severity: "critical" },
    { metric: "latency_p99", threshold: "500ms", severity: "warning" }
  ]
}
```

### Features

- Metrics definition
- Logging configuration
- Distributed tracing
- Alert definitions
- Dashboard specifications
- Monitoring targets

---

## 6. Compliance & Governance DSL

Model policies, rules, controls, and governance requirements.

### Basic Syntax

```sruja
// Policy
policy DataRetentionPolicy {
  rule: "all_pii_data"
  retention_period: "7 years"
  compliance: ["GDPR", "CCPA"]
}

// Governance rule
governance_rule NoDirectDBAccess {
  severity: error
  message: "Services must not access database directly"
  check: service.database_access == "via_repository"
}

// Compliance requirement
compliance PCI_DSS {
  standard: "PCI-DSS v3.2.1"
  requirements: [
    "encryption_at_rest",
    "encryption_in_transit",
    "access_controls",
    "audit_logging"
  ]
}
```

### Features

- Policy definitions
- Governance rules
- Compliance standards
- Approval workflows
- Audit requirements
- Regulatory mappings

---

## 7. Systems Thinking DSL

Model system behavior, feedback loops, and causal dynamics.

### Basic Syntax

```sruja
system "Payment-Load-Loop" {
  concepts {
    Traffic
    Latency
    Retries
    Load
  }

  causal {
    Traffic +-> Latency delay 200ms
    Latency +-> Retries
    Retries +-> Load
    Load +-> Traffic
  }

  loops {
    R1 reinforcing {
      Traffic -> Latency
      Latency -> Retries
      Retries -> Load
      Load -> Traffic
    }
  }

  stocks {
    PendingRequests initial 0
  }

  flows {
    Incoming -> PendingRequests rate rps("API")
    Processed -> PendingRequests rate rps("Worker")
  }
}
```

### Features

- Causal relationships (with polarity and delays)
- Feedback loops (reinforcing and balancing)
- Stocks and flows (system dynamics)
- Constraints and goals
- Architecture mapping

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [Systems Thinking DSL](./dsl-systems-thinking.md) - Complete guide

---

## 8. DSL Evolution

The DSL is designed to evolve over time:

### DSL v0.1 (MVP)
- Basic architecture elements (systems, containers, components)
- Simple relationships
- Basic metadata

### DSL v1 (Current)
- Full architecture modeling
- Requirements and ADRs
- User journeys
- Multi-layer abstraction

### DSL v2 (Planned)
- All extension DSLs (resilience, performance, security, etc.)
- Advanced patterns

### DSL v3 (Future)
- Systems thinking
- Causal models
- System dynamics simulation
- Advanced scenarios

---

## Using Extensions

### Enable Extensions

Extensions are opt-in. Enable them in your DSL:

```sruja
workspace {
  extensions: [
    "resilience",
    "performance",
    "security"
  ]
  
  model {
    // Your architecture
  }
  
  resilience {
    // Resilience definitions
  }
  
  performance {
    // Performance definitions
  }
}
```

### Validation

Each extension can have its own validation rules:

```sruja
validation {
  resilience: {
    require_timeout: true
    require_health_check: true
  }
  performance: {
    require_slo: true
    require_capacity: true
  }
}
```

---

## Grammar References

For complete grammar specifications, see:
- [DSL Specification](./dsl-specification.md) - Core DSL grammar
- Conversation file - Detailed extension grammars (Ohm format)

---

## Examples

See `docs/examples/` for complete examples using various DSL extensions.

---

## Implementation Status

| Extension | Status | Notes |
|-----------|--------|-------|
| Core Architecture | ✅ Implemented | Basic DSL in v0.1.0 |
| Resilience | 📋 Planned | Grammar defined |
| Performance | 📋 Planned | Grammar defined |
| Cost/FinOps | 📋 Planned | Grammar defined |
| Security | 📋 Planned | Grammar defined |
| Observability | 📋 Planned | Grammar defined |
| Compliance | 📋 Planned | Grammar defined |

---

## References

- [Core DSL Specification](./dsl-specification.md)
- [Implementation Plan](../implementation-plan.md)
- [Validation Engine](../guides/validation-engine.md)


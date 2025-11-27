# Reliability Pillar

This document describes how Sruja supports the Reliability pillar of the Well-Architected Framework.

[← Back to Pillars Index](./README.md)

## Overview

**Reliability** is the ability of a workload to perform its function correctly and consistently, including recovering from failures and meeting demands.

**Includes:**
- Distributed system design
- Redundancy
- Planning for change
- Handling failure

---

## Core Support (Basic)

The core DSL includes basic reliability features:

### Basic Retry

```sruja
system MyService {
  retry {
    attempts: 3
    backoff: "exponential"
  }
}
```

### Timeout

```sruja
system MyService {
  timeout: "2s"
}
```

### Circuit Breaker

```sruja
system MyService {
  circuit_breaker {
    failure_threshold: "50%"
    open_duration: "30s"
  }
}
```

---

## Advanced Extensions

### Resilience & Reliability DSL

Complete reliability modeling:

```sruja
resilience {
  timeout APIServiceTimeout {
    duration: "2s"
    enforce: true
  }

  retry_policy ExponentialRetry {
    attempts: 3
    backoff: "exponential"
    base_delay: "100ms"
    max_delay: "5s"
    jitter: true
  }

  circuit_breaker PaymentCB {
    failure_threshold: "50%"
    open_duration: "30s"
    half_open_requests: 5
    fallback: "default_payment"
  }

  bulkhead CheckoutBulkhead {
    max_concurrent: 300
    queue_size: 500
  }

  health_check FullHealth {
    endpoint: "/health"
    liveness: "/health/live"
    readiness: "/health/ready"
    interval: "10s"
    timeout: "2s"
  }

  autoscale CheckoutAS {
    min: 2
    max: 20
    metric: "cpu"
    target: "60%"
  }

  failover PaymentFailover {
    primary: "us-east"
    secondary: "us-west"
    policy: "active-passive"
  }
}
```

### Redundancy

```sruja
redundancy {
  database_primary {
    region: "us-east-1"
    replicas: 2
  }

  database_replica {
    region: "us-west-1"
    async_replication: true
  }
}
```

### Failure Scenarios

```sruja
failure_scenario PaymentDown {
  simulate: "PaymentService returns 500"
  expected_behavior: "Checkout applies fallback"
  recovery: "automatic"
}
```

### Chaos Engineering

```sruja
chaos {
  experiment "DatabaseLatency" {
    target: Database
    injection: "latency"
    duration: "5m"
    expected_impact: "degraded_performance"
  }
}
```

---

## Integration with Other Pillars

### Operational Excellence
- Reliability monitoring
- Failure detection and response

### Security
- Secure failure modes
- Security as reliability factor

### Performance Efficiency
- Performance under failure
- Degradation strategies

---

## Validation Rules

Reliability validation includes:

- ✅ Critical services must have redundancy
- ✅ All external calls must have timeouts
- ✅ Retry policies must be defined
- ✅ Circuit breakers must be configured
- ✅ Health checks must be implemented
- ✅ Failure scenarios must be documented

---

## Engines

### Core Engines
- **Basic Retry Engine** - Retry policy validation
- **Basic Timeout Engine** - Timeout requirement checks
- **Basic Circuit Breaker Engine** - Circuit breaker configuration

### Advanced Engines
- **Resilience Engine** - Full resilience DSL support
- **Health Check Engine** - Liveness, readiness, startup probes
- **Retry Policy Engine** - Exponential, linear, custom retries
- **Circuit Breaker Engine** - Failure threshold management
- **Bulkhead Engine** - Resource isolation
- **Rate Limiting Engine** - Request rate control
- **Backpressure Engine** - Flow control
- **Degradation Engine** - Graceful degradation strategies
- **Auto-healing Engine** - Automatic recovery
- **Autoscaling Engine** - CPU, latency, event-driven scaling
- **Failover Engine** - Active-passive, active-active strategies
- **Failure Scenario Engine** - Failure mode modeling
- **Chaos Engineering Engine** - Chaos testing automation
- **Incident Propagation Simulator** - MTTR, cascade failures
- **Stress Simulation Engine** - Load, failure, scaling tests
- **Change Impact Simulator** - Simulates change propagation
- **Redundancy Engine** - Multi-region, multi-AZ planning
- **High Availability Engine** - HA configuration validation

See: [Engines by Pillars](./engines.md#reliability-pillar) for complete list

---

## DSL Reference

See: [DSL Extensions - Resilience](../specs/dsl-extensions.md#1-resilience--reliability-dsl)

---

## Examples

- [Resilience Example](../examples/resilience/)
- [Failure Scenarios](../examples/failure-scenarios/)

---

*Reliability ensures your architecture performs correctly and consistently, even when things go wrong.*


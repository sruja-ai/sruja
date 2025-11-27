# Operational Excellence Pillar

This document describes how Sruja supports the Operational Excellence pillar of the Well-Architected Framework.

[← Back to Pillars Index](./README.md)

## Overview

**Operational Excellence** focuses on running and monitoring systems, and continually improving processes and procedures.

**Key concepts:**
- Operations as code
- Frequent small reversible changes
- Anticipating failure
- Learning from failures

---

## Core Support (Basic)

The core DSL includes basic operational excellence features:

### Health Checks

```sruja
system MyService {
  health_check {
    endpoint: "/health"
    interval: "30s"
  }
}
```

### Basic Observability

```sruja
system MyService {
  observability {
    metrics: ["latency", "error_rate"]
    logging: "json"
  }
}
```

### Change Management

```sruja
adr ADR001: "Use blue-green deployments"
  status: accepted
  rationale: "Enable zero-downtime deployments"
```

---

## Advanced Extensions

### Observability DSL

Full observability modeling:

```sruja
observability {
  metrics APIMetrics {
    latency: true
    throughput: true
    error_rate: true
    custom: ["queue_depth", "cache_hit_rate"]
  }

  logging APILogging {
    level: "info"
    format: "json"
    retention: "30d"
    destinations: ["cloudwatch", "elasticsearch"]
  }

  tracing APITracing {
    enabled: true
    sampling_rate: "10%"
    backend: "jaeger"
  }

  monitoring APIMonitoring {
    alerts: [
      { metric: "error_rate", threshold: "1%", severity: "critical" },
      { metric: "latency_p99", threshold: "500ms", severity: "warning" }
    ]
  }
}
```

### Operations as Code

Model operational procedures:

```sruja
operations {
  runbook "DatabaseFailover" {
    steps: [
      "Check primary database health",
      "Promote replica to primary",
      "Update DNS records",
      "Verify application connectivity"
    ]
    automation: "terraform"
  }
}
```

### Deployment Strategies

```sruja
deployment BlueGreen {
  strategy: "blue-green"
  rollback: "automatic"
  health_check: "required"
}
```

### Incident Response

```sruja
incident_response {
  oncall_rotation: "team-platform"
  escalation: ["team-lead", "cto"]
  runbooks: ["DatabaseFailover", "ServiceRestart"]
}
```

---

## Integration with Other Pillars

### Reliability
- Health checks enable reliability
- Monitoring detects failures early

### Security
- Security monitoring and alerting
- Audit logging

### Performance Efficiency
- Performance monitoring
- Resource utilization tracking

---

## Validation Rules

Operational Excellence validation includes:

- ✅ All services must have health checks
- ✅ Critical services must have monitoring
- ✅ All changes must have ADRs
- ✅ Runbooks must exist for critical operations
- ✅ Deployment strategies must be defined

---

## Engines

### Core Engines
- **Basic Health Check Engine** - Health check validation
- **Basic Metrics Engine** - Latency, error rate tracking
- **Basic Logging Engine** - Log format validation

### Advanced Engines
- **Observability Engine (ATOE)** - Full observability DSL support
- **Metrics Collection Engine** - Custom metrics definition
- **Logging Aggregation Engine** - Centralized logging
- **Distributed Tracing Engine** - Request tracing
- **Alerting Engine** - Alert definition and routing
- **Dashboard Generation Engine** - Auto-generate dashboards
- **Runbook Engine** - Operational procedure automation
- **Deployment Strategy Engine** - Blue-green, canary, rolling
- **Incident Response Engine** - Automated incident handling
- **CI/CD Integration Engine** - Validate in pipelines
- **Change Impact Analyzer** - Assess change impact
- **Rollback Controller** - Safe rollback plans

See: [Engines by Pillars](./engines.md#operational-excellence-pillar) for complete list

---

## DSL Reference

See: [DSL Extensions - Observability](../specs/dsl-extensions.md#5-observability-dsl)

---

## Examples

- [Observability Example](../examples/observability/)
- [Deployment Strategies](../examples/deployments/)

---

*Operational Excellence ensures your architecture is runnable, monitorable, and continuously improvable.*


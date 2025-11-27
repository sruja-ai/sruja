# Core DSL - Well-Architected Basics

This document describes the core DSL features that provide basic support for all six Well-Architected Framework pillars (general software architecture principles, not cloud-specific).

[← Back to Pillars Index](./README.md)

## Overview

The **Core DSL** provides minimal but solid support for all six pillars, enabling you to get started quickly while maintaining architectural quality.

This is the foundation - you can add advanced extensions later as needed.

---

## Core Features by Pillar

### 1. Operational Excellence (Basic)

```sruja
workspace {
  model {
    system MyService {
      // Basic health check
      health_check: "/health"
      
      // Basic observability
      metrics: ["latency", "error_rate"]
      logging: "json"
    }
  }
}
```

**What's included:**
- Health check endpoints
- Basic metrics (latency, error rate)
- Logging format
- ADR support for change documentation

---

### 2. Security (Basic)

```sruja
workspace {
  model {
    system MyService {
      // Basic authentication
      authentication: "OAuth2"
      
      // Basic encryption
      encryption: {
        at_rest: "AES-256"
        in_transit: "TLS 1.3"
      }
      
      // Security tags
      tags: ["encrypted", "audited"]
    }
  }
}
```

**What's included:**
- Authentication type
- Encryption (at rest, in transit)
- Security tags
- Basic compliance tags

---

### 3. Reliability (Basic)

```sruja
workspace {
  model {
    system MyService {
      // Basic retry
      retry: {
        attempts: 3
        backoff: "exponential"
      }
      
      // Basic timeout
      timeout: "2s"
      
      // Basic circuit breaker
      circuit_breaker: {
        failure_threshold: "50%"
        open_duration: "30s"
      }
    }
  }
}
```

**What's included:**
- Retry policies
- Timeouts
- Circuit breakers
- Basic health checks

---

### 4. Performance Efficiency (Basic)

```sruja
workspace {
  model {
    system MyService {
      // Basic latency target
      latency: "<200ms"
      
      // Basic throughput
      throughput: "1000 rps"
      
      // Basic scaling
      scaling: {
        min: 2
        max: 10
      }
    }
  }
}
```

**What's included:**
- Latency targets
- Throughput targets
- Basic scaling (min/max)
- Resource type hints

---

### 5. Cost Optimization (Basic)

```sruja
workspace {
  model {
    system MyService {
      // Basic cost estimate
      cost: "$500/month"
      
      // Cost tags
      tags: ["cost-center:engineering"]
    }
  }
}
```

**What's included:**
- Cost estimates
- Cost center tags
- Environment-based costing hints

---

### 6. Sustainability (Basic)

```sruja
workspace {
  model {
    system MyService {
      // Basic resource efficiency
      resource_efficiency: "high"
      
      // Basic carbon tracking
      carbon_footprint: "50 kg CO2/month"
    }
  }
}
```

**What's included:**
- Resource efficiency indicators
- Basic carbon footprint tracking
- Green computing hints

---

## Complete Core Example

```sruja
workspace {
  model {
    system PaymentService {
      // Operational Excellence
      health_check: "/health"
      metrics: ["latency", "error_rate", "throughput"]
      logging: "json"
      
      // Security
      authentication: "OAuth2"
      encryption: {
        at_rest: "AES-256"
        in_transit: "TLS 1.3"
      }
      tags: ["pci-dss", "encrypted"]
      
      // Reliability
      retry: {
        attempts: 3
        backoff: "exponential"
      }
      timeout: "2s"
      circuit_breaker: {
        failure_threshold: "50%"
        open_duration: "30s"
      }
      
      // Performance Efficiency
      latency: "<200ms"
      throughput: "2000 rps"
      scaling: {
        min: 2
        max: 20
      }
      
      // Cost Optimization
      cost: "$800/month"
      tags: ["cost-center:payments"]
      
      // Sustainability
      resource_efficiency: "high"
      carbon_footprint: "60 kg CO2/month"
    }
  }
}
```

---

## When to Use Core vs Extensions

### Use Core When:
- ✅ Building MVP architectures
- ✅ Learning the DSL
- ✅ Simple architectures
- ✅ Quick prototypes
- ✅ Basic documentation needs

### Use Extensions When:
- ✅ Enterprise architectures
- ✅ Complex requirements
- ✅ Detailed modeling needed
- ✅ Advanced validation required
- ✅ Compliance requirements

---

## Migration Path

1. **Start with Core** - Use basic pillar support
2. **Identify Gaps** - Determine which pillars need more detail
3. **Add Extensions** - Enable specific extension DSLs
4. **Iterate** - Refine as architecture evolves

---

## Validation

Core DSL includes basic validation for all pillars:

- ✅ Operational Excellence: Health checks required
- ✅ Security: Encryption required for sensitive data
- ✅ Reliability: Timeouts required for external calls
- ✅ Performance: Latency targets recommended
- ✅ Cost: Cost estimates recommended
- ✅ Sustainability: Resource efficiency recommended

---

## Core Engines

The core includes these basic engines:

- **DSL Parser Engine** - Parse DSL to AST
- **Model Composer** - Build unified architecture model
- **Two-Way Sync Engine** - Diagram ↔ DSL sync
- **Basic Validation Engine** - Essential validation rules
- **Basic Health Check Engine** - Health check validation
- **Basic Metrics Engine** - Latency, error rate tracking
- **Basic Security Engine** - Authentication, encryption checks
- **Basic Resilience Engine** - Retry, timeout, circuit breaker
- **Basic Performance Engine** - Latency, throughput targets
- **Basic Cost Engine** - Cost tracking
- **Basic Sustainability Engine** - Resource efficiency

See: [Engines by Pillars](./engines.md) for complete engine catalog

---

## Next Steps

- [Operational Excellence](./operational-excellence.md) - Advanced observability
- [Security](./security.md) - Complete security DSL
- [Reliability](./reliability.md) - Full resilience DSL
- [Performance Efficiency](./performance-efficiency.md) - Complete performance DSL
- [Cost Optimization](./cost-optimization.md) - Full FinOps DSL
- [Sustainability](./sustainability.md) - Complete sustainability DSL
- [Engines by Pillars](./engines.md) - Complete engine catalog

---

*Start simple with Core, add extensions as you grow.*


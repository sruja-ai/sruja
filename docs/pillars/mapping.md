# DSL Extensions to Pillars Mapping

This document maps DSL extensions to Well-Architected Framework pillars.

[← Back to Pillars Index](./README.md)

## Overview

Sruja DSL extensions are organized around the six Well-Architected Framework pillars. This document shows how each extension maps to pillars.

---

## Extension to Pillar Mapping

### Operational Excellence Pillar

**Core DSL Extensions:**
- Observability DSL (metrics, logging, tracing, monitoring)
- Operations as Code DSL
- Deployment Strategies DSL
- Incident Response DSL
- Runbooks DSL

**Related Extensions:**
- ADR DSL (change documentation)
- Requirements DSL (operational requirements)

**See**: [Operational Excellence Pillar](./operational-excellence.md)

---

### Security Pillar

**Core DSL Extensions:**
- Security DSL (authentication, authorization, encryption)
- IAM DSL (identity and access management)
- Network Security DSL
- Data Classification DSL
- Compliance DSL (PCI-DSS, GDPR, etc.)

**Related Extensions:**
- Governance DSL (security policies)
- Audit DSL (security auditing)

**See**: [Security Pillar](./security.md)

---

### Reliability Pillar

**Core DSL Extensions:**
- Resilience & Reliability DSL (timeouts, retries, circuit breakers)
- Health Checks DSL
- Redundancy DSL
- Failure Scenarios DSL
- Chaos Engineering DSL

**Related Extensions:**
- Performance DSL (performance under failure)
- Systems Thinking DSL (failure propagation)

**See**: [Reliability Pillar](./reliability.md)

---

### Performance Efficiency Pillar

**Core DSL Extensions:**
- Performance DSL (latency, throughput, scaling)
- Resource Selection DSL
- Caching DSL
- Performance Optimization DSL
- Capacity Planning DSL

**Related Extensions:**
- Cost Optimization DSL (cost-performance trade-offs)
- Sustainability DSL (resource efficiency)

**See**: [Performance Efficiency Pillar](./performance-efficiency.md)

---

### Cost Optimization Pillar

**Core DSL Extensions:**
- Cost & FinOps DSL (cost models, budgets)
- Cost Allocation DSL
- Cost Optimization Strategies DSL
- Cost Analysis DSL

**Related Extensions:**
- Performance DSL (right-sizing)
- Sustainability DSL (efficiency = cost savings)

**See**: [Cost Optimization Pillar](./cost-optimization.md)

---

### Sustainability Pillar

**Core DSL Extensions:**
- Sustainability DSL (carbon footprint, resource efficiency)
- Green Computing DSL
- Carbon Accounting DSL
- Green Architecture Patterns DSL

**Related Extensions:**
- Cost Optimization DSL (efficiency = sustainability)
- Performance DSL (efficient performance)

**See**: [Sustainability Pillar](./sustainability.md)

---

## Cross-Pillar Extensions

Some extensions span multiple pillars:

### Systems Thinking DSL
- **Operational Excellence**: System behavior understanding
- **Reliability**: Failure propagation modeling
- **Performance**: Performance dynamics
- **Cost**: Cost dynamics

### Governance DSL
- **Security**: Security policies
- **Operational Excellence**: Operational policies
- **Cost**: Cost governance
- **All Pillars**: Cross-cutting governance

### Requirements DSL
- **All Pillars**: Requirements can map to any pillar

### ADR DSL
- **All Pillars**: Decisions can relate to any pillar

---

## Implementation Status

| Pillar | Core Support | Advanced Extensions | Status |
|--------|-------------|-------------------|--------|
| Operational Excellence | ✅ Basic | 📋 Full DSL | Grammar defined |
| Security | ✅ Basic | 📋 Full DSL | Grammar defined |
| Reliability | ✅ Basic | 📋 Full DSL | Grammar defined |
| Performance Efficiency | ✅ Basic | 📋 Full DSL | Grammar defined |
| Cost Optimization | ✅ Basic | 📋 Full DSL | Grammar defined |
| Sustainability | ✅ Basic | 📋 Full DSL | Grammar defined |

---

## Using Pillars in Your Architecture

### Example: All Pillars in One Model

```sruja
workspace {
  model {
    system PaymentService {
      // Operational Excellence
      health_check: "/health"
      metrics: ["latency", "error_rate"]
      
      // Security
      authentication: "OAuth2"
      encryption: { at_rest: "AES-256", in_transit: "TLS 1.3" }
      
      // Reliability
      retry: { attempts: 3, backoff: "exponential" }
      timeout: "2s"
      circuit_breaker: { failure_threshold: "50%" }
      
      // Performance Efficiency
      latency: "<200ms"
      throughput: "2000 rps"
      scaling: { min: 2, max: 20 }
      
      // Cost Optimization
      cost: "$800/month"
      
      // Sustainability
      resource_efficiency: "high"
      carbon_footprint: "60 kg CO2/month"
    }
  }
}
```

### Example: Enable Advanced Extensions

```sruja
workspace {
  extensions: [
    "observability",      // Operational Excellence
    "security",           // Security
    "resilience",        // Reliability
    "performance",        // Performance Efficiency
    "cost",              // Cost Optimization
    "sustainability"     // Sustainability
  ]
  
  // ... model definitions with full DSL support
}
```

---

## Validation by Pillar

Each pillar has validation rules:

- **Operational Excellence**: Health checks, monitoring required
- **Security**: Encryption, authentication required for sensitive data
- **Reliability**: Timeouts, retries required for external calls
- **Performance Efficiency**: Latency targets recommended
- **Cost Optimization**: Cost estimates recommended
- **Sustainability**: Resource efficiency recommended

---

## Next Steps

1. **Start with Core** - [Core DSL](./core.md) - Basics of all pillars
2. **Choose Pillars** - Focus on pillars most relevant to your architecture
3. **Add Extensions** - Enable advanced DSLs as needed
4. **Validate** - Run pillar-specific validation

---

*This mapping helps you understand how DSL extensions align with industry-standard architecture frameworks.*


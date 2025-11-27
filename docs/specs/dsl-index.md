# DSL Index & Quick Reference

This document provides a quick reference index to all DSL features and extensions.

[← Back to Documentation Index](../README.md)

## Core DSL

### Design Philosophy

**Important**: Read [Core DSL vs Extensions](./core-vs-extensions.md) to understand what belongs in core DSL vs extensions.

**Core principles:**
- ✅ Core DSL = Pure architecture structure (stable, universal)
- ✅ Extensions = Domain/org/cloud-specific concerns (via plugins)
- ✅ Metadata = Freeform key-value pairs for infinite extension

### Basic Architecture Modeling
- **Systems** - High-level software systems
- **Containers** - Applications, databases, queues
- **Components** - Logical modules
- **Relations** - Dependencies and interactions
- **Metadata** - Technology, descriptions, tags, freeform extension points

See: 
- [Core DSL vs Extensions](./core-vs-extensions.md) - **Design philosophy and boundaries**
- [DSL Overview](./dsl-overview.md) - High-level introduction
- [DSL Specification](./dsl-specification.md) - Complete grammar reference

---

## Extension DSLs

### 1. Resilience & Reliability DSL
**Purpose**: Model fault tolerance and reliability patterns

**Key Features**:
- Timeouts (global, endpoint-specific)
- Retry policies (exponential, linear, custom)
- Circuit breakers
- Bulkheads
- Rate limiting
- Backpressure
- Degradation strategies
- Health checks (liveness, readiness, startup)
- Auto-healing
- Autoscaling (CPU, latency, event-driven)
- Failover strategies
- Error budgets
- Reliability scoring
- Failure scenarios

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [DSL Extensions](./dsl-extensions.md#1-resilience--reliability-dsl)

---

### 2. Performance DSL
**Purpose**: Model performance characteristics and requirements

**Key Features**:
- Latency (p50, p90, p99)
- Throughput (max, sustained, burst)
- Concurrency limits
- Queue modeling
- Scaling (horizontal, vertical)
- SLIs and SLOs
- Error budgets
- Performance budgets
- Workload patterns (cyclical, burst, steady)
- Throughput paths
- Bottleneck identification
- Capacity planning
- Network latency/bandwidth
- Rate limiting
- Backpressure
- Per-operation performance

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [DSL Extensions](./dsl-extensions.md#2-performance-dsl)

---

### 3. Cost & FinOps DSL
**Purpose**: Model costs, budgets, and financial operations

**Key Features**:
- Cost models (compute, storage, network)
- Budget definitions
- Environment-specific costing
- Per-request cost modeling
- Cost optimization strategies
- Cost allocation

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [DSL Extensions](./dsl-extensions.md#3-cost--finops-dsl)

---

### 4. Security DSL
**Purpose**: Model security requirements and compliance

**Key Features**:
- Authentication mechanisms
- Authorization models (RBAC, ABAC)
- Encryption (at rest, in transit)
- Security policies
- Compliance requirements (PCI-DSS, GDPR, etc.)
- Audit logging
- Data classification
- Access controls

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [DSL Extensions](./dsl-extensions.md#4-security-dsl)

---

### 5. Observability DSL
**Purpose**: Model metrics, logging, tracing, and monitoring

**Key Features**:
- Metrics definition
- Logging configuration
- Distributed tracing
- Alert definitions
- Dashboard specifications
- Monitoring targets

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [DSL Extensions](./dsl-extensions.md#5-observability-dsl)

---

### 6. Compliance & Governance DSL
**Purpose**: Model policies, rules, controls, and governance

**Key Features**:
- Policy definitions
- Governance rules
- Compliance standards
- Approval workflows
- Audit requirements
- Regulatory mappings

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [DSL Extensions](./dsl-extensions.md#6-compliance--governance-dsl)

---

### 7. Systems Thinking DSL
**Purpose**: Model system behavior, feedback loops, and causal dynamics

**Key Features**:
- Causal relationships (with polarity and delays)
- Feedback loops (reinforcing and balancing)
- Stocks and flows (system dynamics)
- Constraints and goals
- Architecture mapping
- Causal loop diagrams
- System dynamics simulation
- Leverage point detection
- Trade-off exploration

**Status**: 📋 Grammar defined, implementation planned

**Reference**: [Systems Thinking DSL](./dsl-systems-thinking.md) - Complete guide

**Note**: This is a major differentiator - no other architecture tool supports systems thinking modeling.

---

## Additional DSL Features

### Requirements DSL
- Epics
- Features
- User Stories
- Use Cases
- Acceptance Criteria
- Business Rules
- Actors/Personas
- Traceability links

**Status**: ✅ Partially implemented (basic requirements in v0.1.0)

---

### ADR DSL
- ADR definitions
- Status tracking (proposed, accepted, rejected, deprecated)
- Supersession tracking
- Impact analysis
- Versioning
- Integration with architecture elements

**Status**: ✅ Partially implemented (basic ADRs in v0.1.0)

---

### User Journey DSL
- Journey steps
- Flow definitions
- Actor interactions
- Traceability to architecture

**Status**: 📋 Planned

---

## DSL Evolution

### v0.1 (Current/MVP)
- ✅ Basic architecture elements
- ✅ Simple relationships
- ✅ Requirements (basic)
- ✅ ADRs (basic)
- ✅ Mermaid compilation

### v1 (Planned)
- 📋 Full extension support
- 📋 Advanced validation
- 📋 LSP integration
- 📋 Visual editor

### v2 (Future)
- 📋 Systems thinking
- 📋 Causal models
- 📋 Advanced simulation
- 📋 AI-powered generation

---

## Quick Syntax Reference

### Basic Architecture
```sruja
system MySystem {
  container MyContainer {
    component MyComponent
  }
}

MyComponent -> OtherComponent "Uses"
```

### Requirements
```sruja
requirement R1: functional "Must handle 10k RPS"
```

### ADRs
```sruja
adr ADR001: "Use microservices architecture"
```

### Resilience
```sruja
timeout ServiceTimeout { duration: "2s" }
retry_policy RetryPolicy { attempts: 3 }
circuit_breaker PaymentCB { failure_threshold: "50%" }
```

### Performance
```sruja
latency APIService { p99: 120ms }
throughput APIService { max_rps: 2000 }
slo CheckoutSLO { objective: "<200ms" }
```

---

## Finding More Information

- **Design Philosophy**: [Core DSL vs Extensions](./core-vs-extensions.md) - **Must read!**
- **Metadata Model**: [Metadata Model](./metadata-model.md) - **Extension mechanism**
- **Getting Started**: [DSL Overview](./dsl-overview.md)
- **Complete Grammar**: [DSL Specification](./dsl-specification.md)
- **Extension Details**: [DSL Extensions](./dsl-extensions.md)
- **Examples**: [Examples Directory](../examples/)
- **Implementation**: [Phase 1 Core](../implementation/phase1-core.md)

---

## Grammar Sources

Complete grammar definitions are available in:
- [DSL Specification](./dsl-specification.md) - Core grammar
- `mermaid-conversation.md` - Detailed extension grammars (Ohm format)

The conversation file contains extensive grammar definitions for all extensions in Ohm format, ready for parser implementation.

---

*This index is a living document and will be updated as new DSL features are added.*


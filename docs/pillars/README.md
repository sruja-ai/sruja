# Well-Architected Framework Pillars

Sruja DSL is organized around the **Well-Architected Framework** pillars (inspired by AWS WAF), providing a structured approach to architecture modeling for **any software system** - cloud, on-premise, hybrid, or distributed.

**Note**: Sruja is a **general-purpose software architecture tool**, not tied to any specific cloud provider. The Well-Architected Framework pillars are used purely as an organizational structure for comprehensive architecture modeling.

[← Back to Documentation Index](../README.md)

## The Six Pillars

### 1. [Operational Excellence](./operational-excellence/)
Running and monitoring systems, and continually improving processes and procedures.

**Key concepts**: Operations as code, frequent small reversible changes, anticipating failure, learning from failures.

### 2. [Security](./security/)
Protecting data, systems, and assets while delivering business value.

**Key principles**: Strong identity foundation, layered security, traceability, automation of security controls, protecting data in transit and at rest.

### 3. [Reliability](./reliability/)
The ability of a workload to perform its function correctly and consistently, including recovering from failures and meeting demands.

**Includes**: Distributed system design, redundancy, planning for change, handling failure.

### 4. [Performance Efficiency](./performance-efficiency/)
Using IT and computing resources efficiently to meet system requirements, and evolving technology choices over time.

**Includes**: Selecting appropriate resource types/sizes, monitoring performance, evolving technology, automating changes.

### 5. [Cost Optimization](./cost-optimization/)
Avoiding unneeded cost, understanding spending, selecting resources right, scaling to meet business needs without overspending.

### 6. [Sustainability](./sustainability/)
Focusing on minimizing environmental impacts, efficient use of resources, shared responsibility for sustainability in cloud architectures.

---

## Documentation Structure

### Core (Basics of All Pillars)
The core DSL includes minimal but solid support for all 6 pillars:

- **Operational Excellence**: Basic observability, health checks
- **Security**: Basic authentication, encryption concepts
- **Reliability**: Basic retry, timeout, circuit breaker
- **Performance Efficiency**: Basic latency, throughput
- **Cost Optimization**: Basic cost tracking
- **Sustainability**: Basic resource efficiency

See: [Core DSL](../specs/dsl-overview.md)

### Advanced Extensions
Each pillar has advanced, vertical-specific extensions:

- **Operational Excellence**: Full observability DSL, monitoring, alerting, runbooks
- **Security**: Complete security DSL, compliance, governance
- **Reliability**: Full resilience DSL, chaos engineering, failure scenarios
- **Performance Efficiency**: Complete performance DSL, scaling, optimization
- **Cost Optimization**: Full FinOps DSL, cost modeling, budgets
- **Sustainability**: Resource efficiency, carbon footprint, green architecture

See: [DSL Extensions](../specs/dsl-extensions.md)

---

## Quick Navigation

| Pillar | Core Support | Advanced Extensions | Status |
|--------|-------------|-------------------|--------|
| [Operational Excellence](./operational-excellence.md) | ✅ Basic | 📋 Full DSL | Planned |
| [Security](./security.md) | ✅ Basic | 📋 Full DSL | Planned |
| [Reliability](./reliability.md) | ✅ Basic | 📋 Full DSL | Planned |
| [Performance Efficiency](./performance-efficiency.md) | ✅ Basic | 📋 Full DSL | Planned |
| [Cost Optimization](./cost-optimization.md) | ✅ Basic | 📋 Full DSL | Planned |
| [Sustainability](./sustainability.md) | ✅ Basic | 📋 Full DSL | Planned |

See also: 
- [Core DSL](./core.md) - Basics of all pillars
- [Extension Mapping](./mapping.md) - How extensions map to pillars
- [Engines by Pillars](./engines.md) - Complete engine catalog organized by pillars

---

## How to Use

1. **Start with Core** - Use basic pillar support for MVP architectures
2. **Add Extensions** - Enable advanced DSL extensions as needed
3. **Mix and Match** - Use only the extensions you need
4. **Validate** - Each pillar has validation rules

---

*This structure aligns Sruja with industry-standard architecture frameworks while maintaining flexibility for custom needs.*


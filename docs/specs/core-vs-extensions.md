# Core DSL vs Extensions

**Status**: Canonical Design Principle  
**Version**: 1.0  
**Last Updated**: 2025-01-XX

[← Back to DSL Specification](./dsl-specification.md)

---

## 🎯 Design Philosophy

The Sruja DSL follows a **core + extensions** architecture to ensure:

* **Stability** - Core DSL remains stable over years
* **Future-proofing** - New capabilities don't require grammar changes
* **Extensibility** - Domain/org/cloud-specific concerns via plugins
* **Simplicity** - Users learn a small, universal core
* **Tooling maturity** - LSP, diagrams, validation work on stable foundation

This pattern is used by Terraform, Structurizr DSL, GraphQL, OpenAPI, CDK, Kubernetes CRDs.

---

## ✅ Core DSL (Canonical, Stable, Universal)

The Core DSL models **only architectural structure** - not policies, operational details, runtime behaviors, or environment configs.

### 1. Architecture Scoping

Defines the top-level boundary.

```dsl
architecture "E-Commerce Platform" { ... }
```

**Rationale**: Universal, required, fundamental.

---

### 2. Elements (Structural Building Blocks)

These define the **static topology** of the architecture.

#### Systems

```dsl
system Billing { 
  label "Billing System"
  description "Handles all billing operations"
}
```

#### Containers

```dsl
container BillingAPI {
  label "Billing API"
  technology "Go"
}
```

#### Components

```dsl
component InvoiceGenerator {
  label "Invoice Generator"
  technology "Go"
}
```

#### Persons

```dsl
person Customer "End Customer"
```

#### DataStores

```dsl
datastore InvoiceDB "Invoice Database"
```

#### Queues

```dsl
queue BillingEvents "Billing Event Queue"
```

**Rationale**: These are the heart of any architecture DSL (C4, UML, SysML, Arc42 all agree).

---

### 3. Relationships

Semantic edges between elements.

```dsl
BillingAPI -> InvoiceDB writes "Writes invoices"
Frontend -> BillingAPI calls "Makes API calls"
```

**Rationale**: Absolutely core for understanding architecture topology.

---

### 4. Journeys

Behavioral flows across structural elements.

```dsl
journey CheckoutFlow {
  title "Customer Checkout Journey"
  steps {
    Customer -> Cart "Adds items"
    Cart -> PaymentAPI "Processes payment"
    PaymentAPI -> OrderDB "Saves order"
  }
}
```

**Rationale**: User interactions and system flows are fundamental architectural concerns.

---

### 5. ADRs (Architecture Decision Records)

Document architectural decisions.

```dsl
adr ADR001 "Use JWT for authentication"
```

**Rationale**: Universal, stable, essential for architecture governance.

---

### 6. Requirements

Structural or high-level requirements for architectural reasoning.

```dsl
requirement R1 performance "p95 latency < 200ms"
requirement R2 security "Must encrypt data in transit"
requirement R3 functional "Must support 10k concurrent users"
requirement R4 constraint "Must use PostgreSQL"
```

**Rationale**: Part of architectural reasoning, not fine-grained runtime policies.

---

### 7. Imports

Compose architecture landscapes from multiple files.

```dsl
import "billing.sruja" as Billing
import "inventory.sruja" as Inventory

Billing.BillingAPI -> Inventory.InventoryAPI calls "Checks availability"
```

**Rationale**: Essential for multi-file, multi-domain, multi-team, system-of-systems modeling.

---

### 8. Metadata (Generic, Freeform)

Every element supports metadata for infinite extension without grammar changes.

```dsl
system Billing {
  label "Billing System"
  metadata {
    owner: "Platform Team"
    cost_center: "Finance"
    compliance_level: "PCI-DSS"
  }
}
```

**Rationale**: Enables extensions without modifying the core grammar.

---

## 📊 Core DSL Summary

### Structural Language

* `architecture` - Top-level boundary
* `system` - High-level software system
* `container` - Application or data store within a system
* `component` - Logical module within a container
* `person` - External user or actor
* `datastore` - Data persistence location
* `queue` - Message queue or event stream

### Structural Properties

* `label` - Human-readable name
* `description` - Detailed description
* `technology` - Technology stack (optional)
* `tags` - Categorization tags
* `metadata` - Freeform key-value pairs

### Relationships

* Unidirectional: `A -> B "label"`
* Bidirectional: `A <-> B "label"` (via journeys)
* Verb-qualified: `A -> B calls "label"`
* Labeled: `A -> B "describes interaction"`

### Composition

* `import` - Import other architecture files
* Cross-architecture references: `ModuleA.System -> ModuleB.System`

### Governance

* `requirement` - Functional, performance, security, constraint requirements
* `adr` - Architecture Decision Records

### Flows

* `journey` - User or system journey
* `journey.steps` - Sequence of interactions

---

## ❌ What SHOULD NOT be Core

These change constantly, vary by organization, and shouldn't be fixed in grammar:

### Operational Policies

* ❌ Rate limits
* ❌ Auth policies
* ❌ Caching rules
* ❌ Circuit breaker policies
* ❌ Timeout rules
* ❌ Retry policies

### Deployment & Configuration

* ❌ Deployment configs
* ❌ Environment-specific settings
* ❌ Runtime scaling policies
* ❌ Platform constraints

### Cloud-Specific Resources

* ❌ AWS Lambda configurations
* ❌ Azure Functions settings
* ❌ GCP Cloud Run configs
* ❌ API Gateway specific configs

### Observability & Operations

* ❌ Logging policies
* ❌ Metrics collection
* ❌ Tracing configuration
* ❌ Alerting rules

### Business & Organizational

* ❌ Cost modeling details
* ❌ Team ownership (use metadata)
* ❌ Budget constraints
* ❌ SLIs/SLOs beyond top-level

### Implementation Details

* ❌ Code-level policies
* ❌ Framework-specific configs
* ❌ Library versions
* ❌ Build configurations

**All of these belong in:**
* `metadata` blocks
* `requirement` statements (if architectural)
* `adr` records (if decision-related)
* **Extension plugins** (for validation, codegen, diagrams)

---

## 🪄 Extensions (Plugin Domain)

Extensions handle anything that is:

* **Domain-specific** - Healthcare, finance, e-commerce patterns
* **Cloud-specific** - AWS, Azure, GCP resources
* **Org-specific** - Company policies, standards
* **Operational** - Deployment, monitoring, scaling
* **Policy-based** - Security policies, compliance rules
* **Implementation detail** - Framework configs, code-level concerns
* **Environment-dependent** - Dev/staging/prod differences

### Extension Examples

#### Rate Limiting Extension

```dsl
system BillingAPI {
  metadata {
    rate_limit: "1000 req/min"
    rate_limit_window: "60s"
    rate_limit_strategy: "token_bucket"
  }
}
```

Plugin can:
* Validate rate limit syntax
* Generate API Gateway configs
* Generate monitoring dashboards
* Export rate limit policies

#### Cloud Mapping Extension

```dsl
container BillingAPI {
  technology "Go"
  metadata {
    deployment_target: "AWS Lambda"
    runtime: "go1.21"
    memory: "512MB"
    timeout: "30s"
  }
}
```

Plugin can:
* Generate CloudFormation/Terraform
* Generate CDK code
* Validate against cloud quotas
* Cost estimation

#### Security Extension

```dsl
system BillingAPI {
  metadata {
    authentication: "OAuth2"
    authorization: "RBAC"
    encryption_at_rest: "AES-256"
    encryption_in_transit: "TLS 1.3"
    compliance: ["PCI-DSS", "SOC2"]
  }
}
```

Plugin can:
* Validate security policies
* Generate security documentation
* Export compliance reports
* Generate security test cases

#### Cost Modeling Extension

```dsl
system BillingAPI {
  metadata {
    cost_model: {
      compute: "AWS Lambda"
      requests_per_month: 1000000
      avg_duration_ms: 200
      memory_mb: 512
    }
  }
}
```

Plugin can:
* Calculate monthly costs
* Generate cost reports
* Optimize resource allocation
* Budget alerts

---

## 🏆 Core vs Extension Matrix

| Concern | Core DSL | Extension |
|---------|----------|-----------|
| Systems, Containers, Components | ✅ | ❌ |
| Persons, DataStores, Queues | ✅ | ❌ |
| Relationships | ✅ | ❌ |
| Journeys | ✅ | ❌ |
| ADRs | ✅ | ❌ |
| Requirements (architectural) | ✅ | ❌ |
| Imports | ✅ | ❌ |
| Metadata (freeform) | ✅ | ❌ |
| Rate limits | ❌ | ✅ |
| Auth policies | ❌ | ✅ |
| Caching rules | ❌ | ✅ |
| Circuit breakers | ❌ | ✅ |
| Deployment configs | ❌ | ✅ |
| Cloud resources | ❌ | ✅ |
| Observability configs | ❌ | ✅ |
| Cost modeling | ❌ | ✅ |
| Team ownership | ❌ | ✅ (metadata) |
| Compliance details | ❌ | ✅ (metadata) |

---

## 🔧 Extension Implementation

### Plugin Architecture

Extensions are implemented as **plugins** that:

1. **Parse** metadata from Core DSL
2. **Validate** extension-specific rules
3. **Generate** code/configs/diagrams
4. **Transform** Core DSL to target formats

### Plugin Example: Rate Limit Extension

```go
// Plugin validates and generates rate limit configs
type RateLimitPlugin struct {
  metadata map[string]interface{}
}

func (p *RateLimitPlugin) Validate(arch *Architecture) error {
  // Validate rate limit syntax in metadata
}

func (p *RateLimitPlugin) Generate(arch *Architecture) ([]byte, error) {
  // Generate API Gateway rate limit config
}
```

### Extension Registry

```go
// Register extensions
registry.Register("rate-limit", NewRateLimitPlugin())
registry.Register("aws", NewAWSPlugin())
registry.Register("security", NewSecurityPlugin())
```

---

## 📝 Best Practices

### 1. Use Metadata for Extensions

```dsl
system BillingAPI {
  metadata {
    // Extension-specific configs
    rate_limit: "1000/min"
    cache_ttl: "300s"
  }
}
```

### 2. Use Requirements for Architectural Constraints

```dsl
requirement R1 performance "API must respond in <200ms"
requirement R2 security "Must support OAuth2"
```

### 3. Use ADRs for Decisions

```dsl
adr ADR001 "Use Redis for caching instead of in-memory cache"
```

### 4. Use Plugins for Code Generation

```bash
sruja compile billing.sruja --plugin rate-limit --plugin aws
```

### 5. Keep Core DSL Stable

Don't add new keywords for operational concerns. Use metadata + plugins instead.

---

## 🚀 Future Extensions

Potential extension areas:

* **Resilience Patterns** - Circuit breakers, retries, timeouts
* **Performance** - Caching, CDN, database query optimization
* **Security** - AuthN/AuthZ, encryption, compliance
* **Cost Optimization** - Cost modeling, budget alerts
* **Cloud Providers** - AWS, Azure, GCP specific resources
* **Domain Patterns** - Healthcare, finance, e-commerce specific patterns
* **Observability** - Logging, metrics, tracing configs
* **CI/CD** - Deployment pipelines, testing strategies

All via **plugins**, not core grammar changes.

---

## 📚 References

This design follows patterns from:

* **Terraform** - Core HCL + provider plugins
* **Structurizr DSL** - Core structure + workspace extensions
* **GraphQL** - Core schema + directives/extensions
* **OpenAPI** - Core spec + vendor extensions
* **CDK** - Core constructs + cloud-specific modules
* **Kubernetes CRDs** - Core resources + custom resources

---

## ✅ Summary

**Core DSL = Pure architecture structure, relationships, decisions, requirements, and journeys.**

**Everything else = Extensions via metadata + plugins.**

This ensures:
* ✅ Future-proof
* ✅ Simple for users
* ✅ Universal applicability
* ✅ Stable over time
* ✅ Infinitely extensible
* ✅ Enterprise-ready
* ✅ Safe from DSL bloat

---

**Next Steps:**
- [ ] Implement metadata block parsing
- [ ] Design plugin system architecture
- [ ] Create plugin SDK
- [ ] Build example plugins (rate-limit, AWS, security)
- [ ] Document plugin development guide

[← Back to DSL Specification](./dsl-specification.md)

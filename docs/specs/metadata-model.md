# Metadata Model

**Status**: Core DSL Feature  
**Version**: 1.0  
**Last Updated**: 2025-01-XX

[← Back to Documentation Index](../README.md)  
[Core DSL vs Extensions](./core-vs-extensions.md) - Metadata as extension mechanism

---

## 🌟 Overview

Metadata is the **central extension mechanism** that makes the Sruja DSL:

* ✅ Extensible without grammar changes
* ✅ Future-proof
* ✅ Plugin-friendly
* ✅ Domain-agnostic
* ✅ Safe from complexity bloat

Metadata allows users to attach **arbitrary structured information** to any DSL element without modifying the parser, AST, or tooling.

---

## 🧱 What is Metadata?

Metadata is **freeform key-value pairs** attached to any element:

```dsl
container BillingAPI {
    technology "Go"
    
    metadata {
        team: "Payments"
        tier: "critical"
        rate_limit_per_ip: "50/s"
        error_budget: "99.5%"
        cloud: "aws"
        lambda_memory: "512MB"
        tracing: "enabled"
    }
}
```

### Characteristics

* **Freeform** - Any key-value pairs
* **String-based** (v1) - Values are strings
* **Nested maps/arrays** (v2, optional) - Future enhancement
* **Uniform** - Available on all elements
* **Optional** - Not required, but always available

---

## 📋 Metadata Syntax

### Basic Syntax

```dsl
element ID "Label" {
    metadata {
        key1: "value1"
        key2: "value2"
    }
}
```

### On All Element Types

```dsl
architecture "My System" {
    metadata {
        version: "1.0"
        owner: "Platform Team"
    }
    
    system BillingAPI {
        metadata {
            team: "Payments"
            tier: "critical"
        }
    }
    
    container API {
        metadata {
            rate_limit: "100/s"
        }
    }
    
    component InvoiceGenerator {
        metadata {
            language: "Go"
        }
    }
    
    datastore InvoiceDB {
        metadata {
            engine: "PostgreSQL"
            version: "14"
        }
    }
    
    queue Events {
        metadata {
            provider: "RabbitMQ"
        }
    }
    
    person Customer {
        metadata {
            persona: "end-user"
        }
    }
}
```

---

## 🔌 How Plugins Use Metadata

Metadata enables plugins to extend the DSL without modifying the core grammar.

### 1. Validation Plugins

Validate metadata values:

```dsl
metadata {
    rate_limit_per_ip: "50/s"
}
```

```go
func (p RateLimitValidator) Validate(element *Element) error {
    rl, ok := element.MetaString("rate_limit_per_ip")
    if ok && !isValidRateLimit(rl) {
        return fmt.Errorf("invalid rate limit format: %s", rl)
    }
    return nil
}
```

### 2. Diagram Plugins

Control diagram rendering:

```dsl
metadata {
    color: "red"
    shape: "hexagon"
    highlight: "true"
}
```

Plugin can:
* Apply custom colors
* Use custom shapes
* Highlight elements
* Hide/show elements

### 3. Exporter Plugins

Generate configuration files:

```dsl
container APIGW {
    metadata {
        rate_limit: "100/s"
        auth: "jwt"
        upstream: "BillingAPI"
    }
}
```

Plugin generates Kong/API Gateway configs.

### 4. Code Generation Plugins

Drive code generation:

```dsl
metadata {
    generate_client: "true"
    generate_stub: "false"
    language: "typescript"
}
```

### 5. Policy-as-Code Plugins

Express and enforce policies:

```dsl
metadata {
    pii: "true"
    encryption: "required"
    retention_days: "365"
    compliance: "GDPR"
}
```

### 6. Cloud Mapping Plugins

Map to cloud resources:

```dsl
container API {
    metadata {
        deployment_target: "AWS Lambda"
        runtime: "go1.21"
        memory: "512MB"
        timeout: "30s"
    }
}
```

Plugin generates Terraform/CloudFormation/CDK.

---

## 🛠 Implementation

### AST Structure

```go
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`
    Value string `parser:"@String"`
}

type ElementWithMetadata struct {
    Metadata []*MetaEntry `parser:"( 'metadata' '{' @@* '}' )?"`
}
```

### Helper API

```go
// Get metadata as string
func (e *Element) MetaString(key string) (string, bool)

// Get metadata as boolean
func (e *Element) MetaBool(key string) bool

// Get all metadata with prefix
func (e *Element) MetaMap(prefix string) map[string]string

// Check if metadata key exists
func (e *Element) HasMeta(key string) bool

// Get all metadata
func (e *Element) AllMetadata() map[string]string
```

---

## 📚 Best Practices

### 1. Naming Conventions

Use clear, consistent key names:

```dsl
metadata {
    team: "Payments"
    owner: "Platform Team"
    rate_limit_per_ip: "50/s"
    deployment_target: "AWS Lambda"
}
```

### 2. Keep Keys Flat

Avoid nested structures in v1:

✅ Good:
```dsl
metadata {
    team: "Payments"
    tier: "critical"
}
```

❌ Avoid (until v2):
```dsl
metadata {
    team.name: "Payments"
    team.owner: "John Doe"
}
```

### 3. Use Metadata for Extensions Only

Don't use metadata for core architectural structure:

✅ Good:
```dsl
metadata {
    rate_limit: "100/s"
    cloud_provider: "aws"
}
```

❌ Avoid:
```dsl
metadata {
    system_type: "microservice"  // Use element type instead
}
```

### 4. Document Metadata Keys

Teams should document their metadata schemas:

```markdown
# Team Metadata Schema

- `team` - Team name (required for containers)
- `tier` - Service tier: critical, standard, low
- `rate_limit_per_ip` - Format: "N/s" or "N/m" or "N/h"
```

---

## 🧩 Plugin Categories Using Metadata

### Validation Plugins

* Rate limit validation
* Required field checking
* Format validation
* Compliance rules

### Diagram Plugins

* Custom rendering
* Highlighting
* Filtering
* Annotation

### Exporter Plugins

* Terraform generation
* CloudFormation templates
* Helm charts
* Kubernetes manifests
* API Gateway configs

### Code Generation Plugins

* Client SDKs
* Server stubs
* Test generation
* Documentation

### Policy Plugins

* Security policies
* Compliance checks
* Governance rules
* Approval workflows

### Cloud Mapping Plugins

* AWS resources
* Azure resources
* GCP resources
* Infrastructure as Code

---

## 🎯 Common Metadata Patterns

### Team & Ownership

```dsl
metadata {
    team: "Platform Team"
    owner: "john.doe@example.com"
    cost_center: "ENG-001"
}
```

### Performance & Scaling

```dsl
metadata {
    rate_limit: "1000/s"
    autoscale_min: "2"
    autoscale_max: "10"
    cpu_limit: "1000m"
    memory_limit: "2Gi"
}
```

### Security & Compliance

```dsl
metadata {
    encryption_at_rest: "AES-256"
    encryption_in_transit: "TLS 1.3"
    compliance: "PCI-DSS"
    pii: "true"
    retention_days: "365"
}
```

### Cloud Configuration

```dsl
metadata {
    cloud_provider: "aws"
    region: "us-east-1"
    deployment_target: "Lambda"
    runtime: "go1.21"
    memory: "512MB"
    timeout: "30s"
}
```

### Observability

```dsl
metadata {
    tracing: "enabled"
    log_level: "info"
    metrics: "prometheus"
    alert_severity: "critical"
}
```

### Cost & Budget

```dsl
metadata {
    cost_center: "ENG-001"
    monthly_budget: "1000"
    cost_allocation: "per-service"
}
```

---

## 🚀 Future Enhancements (v2)

### Nested Metadata

```dsl
metadata {
    team: {
        name: "Platform"
        owner: "john@example.com"
    }
    limits: {
        rate: "100/s"
        burst: "200/s"
    }
}
```

### Typed Values

```dsl
metadata {
    port: 8080
    enabled: true
    tags: ["critical", "api"]
}
```

### Metadata Schemas

```dsl
schema team_metadata {
    team: string (required)
    owner: string (required)
    tier: enum("critical", "standard", "low")
}
```

---

## ✅ Summary

**Metadata = Infinite extensibility without grammar changes**

* ✅ Attach to any element
* ✅ Freeform key-value pairs
* ✅ Plugin consumption
* ✅ Zero parser impact
* ✅ Stable AST

**Use metadata for:**
* Team/organizational info
* Operational policies
* Cloud configurations
* Compliance requirements
* Extension-specific data

**Keep in core DSL:**
* Element types (system, container, etc.)
* Relationships
* Labels and descriptions
* Requirements and ADRs

---

[← Back to Documentation Index](../README.md)  
[Core DSL vs Extensions](./core-vs-extensions.md)

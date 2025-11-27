# DSL Overview

This document provides a high-level overview of the Sruja DSL language and its capabilities.

[← Back to Documentation Index](../README.md)

## What is Sruja DSL?

Sruja DSL is a **Domain-Specific Language** for modeling software architecture. It allows you to:

- Define architecture in plain text (Git-friendly)
- Model at multiple abstraction levels (VHLD, HLD, LLD)
- Include requirements, ADRs, and user journeys
- Specify non-functional aspects (performance, security, resilience)
- Compile to diagrams (Mermaid, and eventually custom UI)
- Enable AI integration via MCP

## Core Philosophy

### 1. Code-First
Architecture is defined in text files (`.sruja`), not just diagrams. This enables:
- Version control with meaningful diffs
- Code review workflows
- CI/CD integration
- Collaboration via Git

### 2. Bidirectional
Changes in DSL ⇄ changes in diagrams (future: visual editor)

### 3. Extensible
The DSL can be extended with domain-specific extensions:
- Resilience patterns
- Performance characteristics
- Security requirements
- Cost models
- And more...

### 4. Multi-Layer
Support for different abstraction levels:
- **VHLD** (Very High-Level Design) - Business domains, high-level services
- **HLD** (High-Level Design) - Technology choices, runtime components
- **LLD** (Low-Level Design) - Code modules, API specs, class structures

## Basic Example

```sruja
workspace {
  model {
    system User "End User"
    
    system API "API Service" {
      container WebApp "Web Application" {
        technology "React"
      }
      
      container Database "PostgreSQL Database" {
        technology "PostgreSQL 14"
      }
    }
    
    User -> WebApp "Uses"
    WebApp -> Database "Reads/Writes"
  }
  
  requirements {
    R1: functional "Must handle 10k concurrent users"
    R2: constraint "Must use PostgreSQL"
  }
  
  adrs {
    ADR001: "Use microservices architecture for scalability"
  }
}
```

## Language Structure

### 1. Workspace
Top-level container for all architecture definitions.

### 2. Model
Core architecture elements:
- **Systems** - High-level software systems
- **Containers** - Applications, databases, message queues
- **Components** - Logical modules within containers
- **Relations** - Dependencies and interactions

### 3. Requirements
Functional and non-functional requirements linked to architecture.

### 4. ADRs
Architecture Decision Records documenting key decisions.

### 5. Extensions
Optional extensions for specialized modeling:
- Resilience & Reliability
- Performance
- Security
- Cost & FinOps
- Observability
- Compliance & Governance

## Key Features

### Hierarchical Modeling
Model at different levels of detail:

```sruja
system ECommerce {
  container WebApp {
    component ShoppingCart
    component Checkout
    component ProductCatalog
  }
  
  container PaymentService {
    component PaymentProcessor
    component FraudDetection
  }
}
```

### Relationships
Define dependencies and interactions:

```sruja
ShoppingCart -> Checkout "Proceeds to"
Checkout -> PaymentService "Processes payment"
PaymentService -> FraudDetection "Validates"
```

### Metadata
Add technology, descriptions, and tags:

```sruja
container Database {
  technology "PostgreSQL 14"
  description "Primary data store for user and order data"
  tags: ["persistence", "sql", "relational"]
}
```

### Requirements Traceability
Link requirements to architecture:

```sruja
requirement R1: functional "Must handle 10k RPS"
  implements: [APIService, LoadBalancer]
```

### ADR Integration
Document decisions:

```sruja
adr ADR003 {
  title: "Use PostgreSQL for primary data store"
  context: "Need ACID transactions and complex queries"
  decision: "PostgreSQL 14"
  consequences: ["Strong consistency", "Complex scaling"]
}
```

## DSL Evolution

### v0.1 (MVP) - Current
- Basic architecture elements
- Simple relationships
- Requirements and ADRs
- Mermaid compilation

### v1 (Planned)
- Full extension support
- Advanced validation
- LSP integration
- Visual editor

### v2 (Future)
- Systems thinking
- Causal models
- Advanced simulation
- AI-powered generation

## Compilation Targets

Currently:
- **Mermaid C4** - Primary target for MVP

Future:
- **Custom UI** - Native diagram renderer
- **PlantUML** - Alternative diagram format
- **D2** - Another diagram format
- **JSON/YAML** - Programmatic access

## Getting Started

1. **Install Sruja**: `go install github.com/sruja-ai/sruja/cmd/sruja@latest`
2. **Create a file**: `example.sruja`
3. **Write DSL**: Use the syntax above
4. **Compile**: `sruja compile example.sruja`
5. **View**: Open the generated Mermaid file

## Next Steps

- [DSL Specification](./dsl-specification.md) - Complete grammar reference
- [DSL Extensions](./dsl-extensions.md) - Specialized DSL extensions
- [Examples](../examples/) - Real-world examples
- [Implementation](../implementation/phase1-core.md) - How it's built

---

*The DSL is designed to grow with your needs - start simple, add complexity as required.*


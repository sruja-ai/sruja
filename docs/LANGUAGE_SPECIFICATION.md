# Sruja Language Specification

This document provides a complete specification of the Sruja architecture-as-code language for AI code assistants and developers.

## Overview

Sruja is a domain-specific language (DSL) for defining software architecture models. It supports C4 model concepts (systems, containers, components), requirements, ADRs, scenarios, flows, policies, SLOs, and more.

## Language Grammar

### File Structure

Sruja uses a **flat syntax** — all declarations are top-level, no wrapper blocks required.

```sruja
// Elements
User = person "User"
Shop = system "E-commerce Shop"

// Relationships
User -> Shop "uses"

// Governance
R1 = requirement functional "Must handle 10k users"
SecurityPolicy = policy "Encrypt all data" category "security"
```

### Element Kinds

Before using elements like `person`, `system`, `container`, etc., you must declare them as **kinds**. This establishes the vocabulary of element types available in your architecture.

```sruja
// Standard C4 kinds (required at top of file)
person = kind "Person"
system = kind "System"
container = kind "Container"
component = kind "Component"
database = kind "Database"
datastore = kind "Datastore"  // Note: Use 'database' keyword in element declarations, 'datastore' is for custom kinds
queue = kind "Queue"
```

**Why kinds?** This allows Sruja to:

- Validate that you're using recognized element types
- Enable custom element types for domain-specific modeling
- Provide LSP autocompletion for your declared kinds

#### Custom Kinds

You can define custom element types for your domain:

```sruja
// Custom kinds for microservices
microservice = kind "Microservice"
eventBus = kind "Event Bus"
gateway = kind "API Gateway"

// Now use them
Catalog = microservice "Catalog Service"
Kafka = eventBus "Kafka Cluster"
```

### Imports

Import kinds and tags from the standard library or other Sruja files.

#### Standard Library Import

```sruja
// Import all from stdlib
import { * } from 'sruja.ai/stdlib'

// Now you can use person, system, container, etc. without defining them
User = person "User"
Shop = system "Shop"
```

#### Named Imports

```sruja
// Import specific kinds only
import { person, system, container } from 'sruja.ai/stdlib'

User = person "User"
Shop = system "Shop"
```

#### Relative Imports

```sruja
// Import from a local file
import { * } from './shared-kinds.sruja'
```

**Note**: When using imports, you don't need to redeclare the imported kinds.

### Elements

#### Persons

```sruja
User = person "User" {
    description "End user of the system"
}
```

#### Systems

```sruja
MySystem = system "My System" {
    description "Optional description"
    metadata {
        key "value"
        tags ["tag1", "tag2"]
    }
    slo {
        availability {
            target "99.9%"
            window "30d"
            current "99.95%"
        }
    }
}
```

#### Containers

```sruja
MyContainer = container "My Container" {
    technology "Technology stack"
    description "Optional description"
    version "1.0.0"
    tags ["api", "backend"]
    scale {
        min 3
        max 10
        metric "cpu > 80%"
    }
    slo {
        latency {
            p95 "200ms"
            p99 "500ms"
        }
    }
}
```

#### Components

```sruja
MyComponent = component "My Component" {
    technology "Technology"
    description "Optional description"
    scale {
        min 1
        max 5
    }
}
```

#### Data Stores

```sruja
MyDB = database "My Database" {
    technology "PostgreSQL"
    description "Optional description"
}
```

#### Queues

```sruja
MyQueue = queue "My Queue" {
    technology "RabbitMQ"
    description "Optional description"
}
```

### Relationships

```sruja
// Basic relationship
From -> To "Label"

// Nested element references use dot notation
System.Container -> System.Container.Component "calls"

// With tags
From -> To "Label" [tag1, tag2]
```

### Requirements

```sruja
R1 = requirement functional "Description"
R2 = requirement nonfunctional "Description"
R3 = requirement constraint "Description"
R4 = requirement performance "Description"
R5 = requirement security "Description"

// With body block
R6 = requirement functional "Description" {
    description "Detailed description"
    metadata {
        priority "high"
    }
}
```

### ADRs (Architectural Decision Records)

```sruja
ADR001 = adr "Title" {
    status "accepted"
    context "What situation led to this decision"
    decision "What was decided"
    consequences "Trade-offs, gains, and losses"
}
```

### Scenarios and Flows

#### Scenarios

```sruja
MyScenario = scenario "Scenario Title" {
    step User -> System.WebApp "Credentials"
    step System.WebApp -> System.DB "Verify"
}

// 'story' is an alias for 'scenario'
CheckoutStory = story "User Checkout Flow" {
    step User -> ECommerce.CartPage "adds item to cart"
}
```

**Note**: The `step` keyword is recommended for clarity, but optional. Both syntaxes work:

- With `step`: `step User -> System.WebApp "action"`
- Without `step`: `User -> System.WebApp "action"` (inside scenario block)

#### Flows (DFD-style data flows)

```sruja
OrderProcess = flow "Order Processing" {
    step Customer -> Shop.WebApp "Order Details"
    step Shop.WebApp -> Shop.Database "Save Order"
    step Shop.Database -> Shop.WebApp "Confirmation"
}
```

**Note**: Flows use the same syntax as scenarios. The `step` keyword is recommended for clarity.

### Metadata

```sruja
metadata {
    key "value"
    anotherKey "another value"
    tags ["tag1", "tag2"]
}
```

### Overview Block

```sruja
overview {
    summary "High-level summary of the architecture"
    audience "Target audience for this architecture"
    scope "What is covered in this architecture"
    goals ["Goal 1", "Goal 2"]
    nonGoals ["What is explicitly out of scope"]
    risks ["Risk 1", "Risk 2"]
}
```

### SLO (Service Level Objectives)

```sruja
slo {
    availability {
        target "99.9%"
        window "30 days"
        current "99.95%"
    }
    latency {
        p95 "200ms"
        p99 "500ms"
        window "7 days"
        current {
            p95 "180ms"
            p99 "420ms"
        }
    }
    errorRate {
        target "0.1%"
        window "7 days"
        current "0.08%"
    }
    throughput {
        target "10000 req/s"
        window "peak hour"
        current "8500 req/s"
    }
}
```

SLO blocks can be defined at:

- Architecture level (top-level)
- System level
- Container level

### Scale Block

```sruja
scale {
    min 3
    max 10
    metric "cpu > 80%"
}
```

Scale blocks can be defined at:

- Container level
- Component level

### Deployment

```sruja
deployment Prod "Production" {
    node AWS "AWS" {
        node USEast1 "US-East-1" {
            infrastructure LB "Load Balancer"
            containerInstance Shop.API
        }
    }
}
```

### Governance

#### Policies

```sruja
policy SecurityPolicy "Enforce TLS 1.3" category "security" enforcement "required"

// Or with body block
policy DataRetentionPolicy "Retain data for 7 years" {
    category "compliance"
    enforcement "required"
    description "Detailed policy description"
}
```

#### Constraints

```sruja
constraints {
    "Constraint description"
    "Another constraint"
}
```

#### Conventions

```sruja
conventions {
    "Convention description"
    "Another convention"
}
```

### Views (Optional)

Views are **optional** — if not specified, standard C4 views are automatically generated.

```sruja
view index {
    title "System Context"
    include *
}

view container_view of Shop {
    title "Shop Containers"
    include Shop.*
    exclude Shop.WebApp
    autolayout lr
}

styles {
    element "Database" {
        shape "cylinder"
        color "#ff0000"
    }
}
```

### View Types

- `index` - System context view (C4 L1)
- `container` - Container view (C4 L2)
- `component` - Component view (C4 L3)
- `deployment` - Deployment view

### View Expressions

- `include *` - Include all elements in scope
- `include Element1 Element2` - Include specific elements
- `exclude Element1` - Exclude specific elements
- `autolayout "lr"|"tb"|"auto"` - Layout direction hint

## Implied Relationships

Relationships are automatically inferred when child relationships exist:

```sruja
User -> API.WebApp "Uses"
// Automatically infers: User -> API
```

This reduces boilerplate while maintaining clarity.

## Complete Example

```sruja
// Element Kinds (required)
person = kind "Person"
system = kind "System"
container = kind "Container"
component = kind "Component"
datastore = kind "Datastore"  // Note: Use 'database' keyword in element declarations, 'datastore' is for custom kinds

// Overview
overview {
    summary "E-commerce platform architecture"
    audience "Development team"
    scope "Core shopping and payment functionality"
}

// Elements
Customer = person "Customer"
Admin = person "Administrator"

Shop = system "E-commerce Shop" {
    description "High-performance e-commerce platform"

    WebApp = container "Web Application" {
        technology "React"
        Cart = component "Shopping Cart"
        Checkout = component "Checkout Service"
    }

    API = container "API Gateway" {
        technology "Node.js"
        scale {
            min 3
            max 10
        }
        slo {
            latency {
                p95 "200ms"
                p99 "500ms"
            }
        }
    }

    DB = database "PostgreSQL Database" {
        technology "PostgreSQL 14"
    }
}

// Relationships
Customer -> Shop.WebApp "Browses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> Shop.DB "Reads/Writes"

// Requirements
R1 = requirement functional "Must support 10k concurrent users"
R2 = requirement constraint "Must use PostgreSQL"

// ADRs
ADR001 = adr "Use microservices architecture" {
    status "accepted"
    context "Need to scale different parts independently"
    decision "Adopt microservices architecture"
    consequences "Gain: Independent scaling. Trade-off: Increased complexity"
}

// Policies
SecurityPolicy = policy "Enforce TLS 1.3" {
    category "security"
    enforcement "required"
}

// Constraints and Conventions
constraints {
    "All APIs must use HTTPS"
    "Database must be encrypted at rest"
}

conventions {
    "Use RESTful API design"
    "Follow semantic versioning"
}

// Scenarios
PurchaseScenario = scenario "User purchases item" {
    step Customer -> Shop.WebApp "Adds item to cart"
    step Shop.WebApp -> Shop.API "Submits order"
    step Shop.API -> Shop.DB "Saves order"
}

// Views (optional - auto-generated if omitted)
view index {
    title "System Context"
    include *
}

view container_view of Shop {
    title "Shop Containers"
    include Shop.*
}
```

## Key Rules

1. **Flat Syntax**: All declarations are top-level, no `specification {}`, `model {}`, or `views {}` wrapper blocks
2. **IDs**: Must be unique within their scope
3. **References**: Use dot notation (e.g., `System.Container`)
4. **Relations**: Can be defined anywhere (implied relationships are automatically inferred)
5. **Metadata**: Freeform key-value pairs
6. **Descriptions**: Optional string values
7. **Views**: Optional — C4 views are automatically generated if not specified
8. **SLOs**: Can be defined at architecture, system, or container level
9. **Scale**: Can be defined at container or component level

## Common Patterns

### C4 Model Levels

- **Level 1 (System Context)**: Systems and persons
- **Level 2 (Container)**: Containers within systems
- **Level 3 (Component)**: Components within containers

## Resources

- Official Documentation: https://sruja.ai
- Examples: `examples/` directory
- Grammar: Defined in `pkg/language/` package

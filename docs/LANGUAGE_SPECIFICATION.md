# Sruja Language Specification

This document provides a complete specification of the Sruja architecture-as-code language for AI code assistants and developers.

## Overview

Sruja is a domain-specific language (DSL) for defining software architecture models. It supports C4 model concepts (systems, containers, components), requirements, ADRs, scenarios, flows, policies, SLOs, and more.

## Language Grammar

### File Structure

A Sruja file can contain:

- An `architecture` block (recommended for complete models)
- Top-level elements (for reusable components)

### Core Syntax

Sruja uses the LikeC4 syntax structure:

```sruja
specification {
    // Element definitions
    element person
    element system
    element container
    element component
}

model {
    // Model architecture and relationships
    customer = person "Customer"
    backend = system "Backend System"
    
    customer -> backend "uses"
}

views {
    // C4 Views
    view index {
        include *
    }
}
```

### Elements

#### Systems

```sruja
system ID "Label" {
    description "Optional description"
    metadata {
        key "value"
        tags ["tag1", "tag2"]
    }
    // Containers, components, datastores, queues, etc.
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
container ID "Label" {
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
    // Components
}
```

#### Components

```sruja
component ID "Label" {
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
datastore ID "Label" {
    technology "Database type"
    description "Optional description"
}
```

#### Queues

```sruja
queue ID "Label" {
    technology "Queue technology"
    description "Optional description"
}
```

#### Persons

```sruja
person ID "Label" {
    description "Optional description"
}
```

#### Relations

```sruja
From -> To "Label"
// When referring to nested elements, use dot notation:
System.Container -> System.Container.Component "Label"
// Or with verb
From -> To verb "Label"
// Or with tags
From -> To "Label" [tag1, tag2]
```

### Requirements and ADRs

#### Requirements

```sruja
requirement ID functional "Description"
requirement ID nonfunctional "Description"
requirement ID constraint "Description"
requirement ID performance "Description"
requirement ID security "Description"

// With body block
requirement ID functional "Description" {
    type "functional"
    description "Detailed description"
    metadata {
        priority "high"
    }
}
```

Constraints:

- Requirements are declared at the architecture root only.
- Declarations at system/container/component level are deprecated and ignored by exporters and UI.

#### ADRs (Architectural Decision Records)

```sruja
adr ID "Title" {
    status "accepted"
    context "What situation led to this decision"
    decision "What was decided"
    consequences "Trade-offs, gains, and losses"
}
```

Constraints:

- ADRs are declared at the architecture root only.
- Declarations at system/container/component level are deprecated and ignored by exporters and UI.

### Scenarios and Flows

#### Scenarios

```sruja
scenario "Scenario Title" {
    User -> System.WebApp "Credentials"
    System.WebApp -> System.DB "Verify"
}

// Or with ID
scenario Checkout "User Checkout Flow" {
    User -> ECommerce.CartPage "adds item to cart"
    ECommerce.CartPage -> ECommerce "clicks checkout"
}

// 'story' is an alias for 'scenario'
story Checkout "User Checkout Flow" {
    User -> ECommerce.CartPage "adds item to cart"
}
```

#### Flows (DFD-style data flows)

```sruja
flow OrderProcess "Order Processing" {
    Customer -> Shop.WebApp "Order Details"
    Shop.WebApp -> Shop.Database "Save Order"
    Shop.Database -> Shop.WebApp "Confirmation"
}
```

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

- Architecture level
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

### Contracts

```sruja
contracts {
    contract CreateOrder api {
        version "1.0"
        endpoint "/api/orders"
        method "POST"
        request {
            customerId string
            items array
        }
        response {
            orderId string
            status string
        }
        errors ["INVALID_REQUEST", "OUT_OF_STOCK"]
    }
}
```

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
policy SecurityPolicy "Enforce TLS 1.3 for all external communications" category "security" enforcement "required"

// Or with body block
policy DataRetentionPolicy "Retain order data for 7 years" {
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

Views are **optional** - if not specified, standard C4 views are automatically generated. Views block is only needed for customization.

```sruja
views {
    container Shop "API Focus" {
        include Shop.API Shop.DB
        exclude Shop.WebApp
        autolayout lr
    }

    styles {
        element "Database" {
            shape "cylinder"
            color "#ff0000"
        }
    }
}
```

### View Types

- `systemContext` - System context view (C4 L1)
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
specification {
    element person
    element system
    element container
    element component
    element datastore
}

model {
    customer = person "Customer"
    admin = person "Administrator"

    shop = system "E-commerce Shop" {
        description "High-performance e-commerce platform"
        
        webApp = container "Web Application" {
            technology "React"
            cart = component "Shopping Cart"
            checkout = component "Checkout Service"
        }

        api = container "API Gateway" {
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

        db = datastore "PostgreSQL Database" {
            technology "PostgreSQL 14"
        }
    }

    // Relationships
    customer -> shop.webApp "Browses"
    shop.webApp -> shop.api "Calls"
    shop.api -> shop.db "Reads/Writes"

    // Sruja Extensions (Governance & Requirements)
    requirement R1 functional "Must support 10k concurrent users"
    requirement R2 constraint "Must use PostgreSQL"

    adr ADR001 "Use microservices architecture" {
        status "accepted"
        context "Need to scale different parts independently"
        decision "Adopt microservices architecture"
        consequences "Gain: Independent scaling. Trade-off: Increased complexity"
    }

    policy SecurityPolicy "Enforce TLS 1.3" category "security" enforcement "required"

    constraints {
        "All APIs must use HTTPS"
        "Database must be encrypted at rest"
    }

    conventions {
        "Use RESTful API design"
        "Follow semantic versioning"
    }

    scenario "User purchases item" {
        customer -> shop.webApp "Adds item to cart"
        shop.webApp -> shop.api "Submits order"
        shop.api -> shop.db "Saves order"
    }

    flow PaymentFlow "Payment processing" {
        shop.webApp -> shop.api "Validate payment method"
        shop.api -> paymentGateway "Process payment" // Identifying external system
    }
}

views {
    view index {
        title "System Context"
        include *
    }
    
    view container_view of shop {
        title "Shop Containers"
        include shop.*
    }
}
```

## Key Rules

1. **IDs**: Must be unique within their scope
2. **References**: Use dot notation (e.g., `System.Container`)
3. **Relations**: Can be defined at any level (implied relationships are automatically inferred)
4. **Metadata**: Freeform key-value pairs
5. **Descriptions**: Optional string values
6. **Views**: Optional - C4 views are automatically generated if not specified
7. **Requirements and ADRs**: Must be declared at architecture root level only
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

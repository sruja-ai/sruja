# Sruja Language Specification

This document provides a complete specification of the Sruja architecture-as-code language for AI code assistants and developers.

## Overview

Sruja is a domain-specific language (DSL) for defining software architecture models. It supports C4 model concepts (systems, containers, components), requirements, ADRs, scenarios, flows, and more.

## Language Grammar

### File Structure

A Sruja file can contain:
- An `architecture` block (recommended for complete models)
- Top-level elements (for reusable components)
- Imports

### Core Syntax

```sruja
architecture "Name" {
    // Elements go here
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
    // Containers, components, etc.
}
```

#### Containers
```sruja
container ID "Label" {
    technology "Technology stack"
    description "Optional description"
    // Components
}
```

#### Components
```sruja
component ID "Label" {
    technology "Technology"
    description "Optional description"
}
```

#### Data Stores
```sruja
datastore ID "Label" {
    technology "Database type"
}
```

#### Queues
```sruja
queue ID "Label" {
    technology "Queue technology"
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
// Or with technology
From -> To "Label" {
    technology "Protocol"
}
// Or with tags
From -> To "Label" [tag1, tag2]
```

### Requirements and ADRs

#### Requirements
```sruja
requirement ID functional "Description"
requirement ID nonfunctional "Description"
requirement ID constraint "Description"
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
story Checkout "User Checkout Flow" {
    User -> ECommerce.CartPage "adds item to cart"
    ECommerce.CartPage -> ECommerce "clicks checkout"
}
```

#### Flows (DFD-style data flows)
```sruja
// Use scenario/story for DFD-style flows
scenario OrderProcess "Order Processing" {
    Customer -> Shop.WebApp "Order Details"
    Shop.WebApp -> Shop.Database "Save Order"
    Shop.Database -> Shop.WebApp "Confirmation"
}
```

### Imports

```sruja
import "path/to/file.sruja"
```

### Metadata

```sruja
metadata {
    key "value"
    anotherKey "another value"
    tags ["tag1", "tag2"]
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
policy "Policy ID" "Policy Title" {
    description "Policy description"
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
}
```

## Complete Example

```sruja
architecture "E-commerce Platform" {
    person Customer "Customer"
    person Admin "Administrator"
    
    system Shop "E-commerce Shop" {
        container WebApp "Web Application" {
            technology "React"
            component Cart "Shopping Cart"
            component Checkout "Checkout Service"
        }
        
        container API "API Gateway" {
            technology "Node.js"
        }
        
        datastore DB "PostgreSQL Database" {
            technology "PostgreSQL 14"
        }
    }
    
    Customer -> Shop.WebApp "Browses"
    Shop.WebApp -> Shop.API "Calls"
    Shop.API -> Shop.DB "Reads/Writes"
    
    requirement R1 functional "Must support 10k concurrent users"
    requirement R2 constraint "Must use PostgreSQL"
    
    adr ADR001 "Use microservices architecture" {
        status "accepted"
        context "Need to scale different parts independently"
        decision "Adopt microservices architecture"
        consequences "Gain: Independent scaling. Trade-off: Increased complexity"
    }
    
    scenario "User purchases item" {
        Customer -> Shop.WebApp "Adds item to cart"
        Shop.WebApp -> Shop.API "Submits order"
        Shop.API -> Shop.DB "Saves order"
    }
    
    scenario PaymentFlow "Payment processing" {
        WebApp -> API "Validate payment method"
        API -> PaymentGateway "Process payment"
        PaymentGateway -> API "Update order status"
    }
}
```

## Views (Optional)

Views are **optional** - if not specified, standard C4 views are automatically generated. Views block is only needed for customization.

```sruja
views {
    container Shop "API Focus" {
        include Shop.API Shop.DB
        exclude Shop.WebApp
        autolayout "lr"
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

## Key Rules

1. **IDs**: Must be unique within their scope
2. **References**: Use dot notation (e.g., `System.Container`)
3. **Relations**: Can be defined at any level (implied relationships are automatically inferred)
4. **Metadata**: Freeform key-value pairs
5. **Descriptions**: Optional string values
6. **Views**: Optional - C4 views are automatically generated if not specified

## Common Patterns

### C4 Model Levels
- **Level 1 (System Context)**: Systems and persons
- **Level 2 (Container)**: Containers within systems
- **Level 3 (Component)**: Components within containers

## Resources

- Official Documentation: https://sruja.ai
- Examples: `examples/` directory
- Grammar: Defined in `pkg/language/ast.go`

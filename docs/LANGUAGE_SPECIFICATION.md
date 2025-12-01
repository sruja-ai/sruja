# Sruja Language Specification

This document provides a complete specification of the Sruja architecture-as-code language for AI code assistants and developers.

## Overview

Sruja is a domain-specific language (DSL) for defining software architecture models. It supports C4 model concepts (systems, containers, components), Domain-Driven Design (DDD), requirements, ADRs, and more.

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
        key: "value"
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
// Or with technology
From -> To "Label" {
    technology "Protocol"
}
```

### Domain-Driven Design (DDD)

#### Domains
```sruja
domain "Domain Name" {
    context "Context Name" {
        aggregate "Aggregate Name" {
            entities {
                "Entity1"
                "Entity2"
            }
            valueObjects {
                "ValueObject1"
            }
        }
        entities {
            "Entity1"
        }
        valueObjects {
            "ValueObject1"
        }
        events {
            "Event1"
        }
    }
}
```

### Requirements and ADRs

#### Requirements
```sruja
requirement ID functional "Description"
requirement ID nonfunctional "Description"
requirement ID constraint "Description"
```

#### ADRs (Architectural Decision Records)
```sruja
adr ID "Title" {
    description "Decision description"
    status "proposed" | "accepted" | "deprecated" | "superseded"
}
```

### Scenarios and Flows

#### Scenarios
```sruja
scenario "Scenario Title" {
    step 1 "First step"
    step 2 "Second step"
    // Steps reference elements: User -> API "Calls"
}
```

#### Flows
```sruja
system API {
    flow "Flow Name" {
        step 1 "Step description"
        step 2 "Next step"
    }
}
```

### Imports

```sruja
import "path/to/file.sruja"
```

### Metadata

```sruja
metadata {
    key: "value"
    anotherKey: "another value"
}
```

### Deployment

```sruja
deployment "Environment Name" {
    system API {
        container WebApp {
            instances 3
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
    }
    
    scenario "User purchases item" {
        step 1 "Customer adds item to cart"
        step 2 "Customer proceeds to checkout"
        step 3 "Payment is processed"
    }
}
```

## Key Rules

1. **IDs**: Must be unique within their scope
2. **References**: Use dot notation (e.g., `System.Container`)
3. **Relations**: Can be defined at any level
4. **Metadata**: Freeform key-value pairs
5. **Descriptions**: Optional string values

## Common Patterns

### C4 Model Levels
- **Level 1 (System Context)**: Systems and persons
- **Level 2 (Container)**: Containers within systems
- **Level 3 (Component)**: Components within containers

### DDD Modeling
- Use `domain` for bounded contexts
- Use `aggregate` for aggregate roots
- Use `entities` and `valueObjects` for domain models

## Resources

- Official Documentation: https://sruja.ai
- Examples: `examples/` directory
- Grammar: Defined in `pkg/language/ast.go`


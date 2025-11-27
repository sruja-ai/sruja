# Sruja Notebook Examples

[← Back to Notebooks Index](../README.md)

## Overview

This directory contains practical examples demonstrating how to use Sruja Architecture Notebooks for real-world architecture design and validation.

## Example Notebooks

### 1. Basic Architecture Design
**File:** `basic-architecture.ipynb`

A simple e-commerce system demonstrating:
- System and container definitions
- Component relationships
- Basic validation
- Diagram generation

**Key Concepts:**
- DSL cell syntax
- Incremental architecture building
- Querying the model

### 2. Domain-Driven Design
**File:** `ddd-example.ipynb`

A complete DDD example showing:
- Domain entities and aggregates
- Domain events
- Lifecycle definitions
- Event simulation

**Key Concepts:**
- Entity lifecycle modeling
- Event-driven architecture
- State transition validation

### 3. Microservices Architecture
**File:** `microservices.ipynb`

A microservices system with:
- Multiple bounded contexts
- Service communication patterns
- API contracts
- Deployment considerations

**Key Concepts:**
- System decomposition
- Service boundaries
- Contract definitions

### 4. Architecture Variants
**File:** `variants-example.ipynb`

Demonstrates architecture experimentation:
- Creating snapshots
- Building variants
- Comparing alternatives
- Merging changes

**Key Concepts:**
- Snapshot management
- Variant creation
- Diff and merge operations

### 5. Validation and Governance
**File:** `validation-example.ipynb`

Comprehensive validation example:
- Selective validation
- Custom validation rules
- Policy enforcement
- Diagnostic interpretation

**Key Concepts:**
- Validation commands
- Diagnostic messages
- Policy compliance

## Quick Start Examples

### Example 1: Simple System Definition

```sruja
// Define a simple system
system PaymentService {
  description "Handles payment processing"
  
  container API {
    description "REST API for payment operations"
    technology "Node.js"
  }
  
  container Database {
    description "Payment data storage"
    technology "PostgreSQL"
  }
  
  relation API -> Database {
    type uses
    description "Stores payment records"
  }
}
```

### Example 2: Query the Architecture

```sruja
// Find all containers in PaymentService
find containers in PaymentService

// Select systems with specific technology
select systems where technology = "Node.js"
```

### Example 3: Generate Diagram

```sruja
// Generate Mermaid diagram
diagram PaymentService format mermaid

// Generate D2 diagram for all systems
diagram all format d2
```

### Example 4: Validate Architecture

```sruja
// Validate entire architecture
validate all

// Validate specific system
validate PaymentService

// Validate with specific rules
validate PaymentService rules: naming, contracts
```

### Example 5: Create Snapshot

```sruja
// Create snapshot of current state
%snapshot create v1.0 "Initial architecture"

// List snapshots
%snapshot list

// Load a snapshot
%snapshot load v1.0
```

### Example 6: Event Simulation

```sruja
// Define entity with lifecycle
entity Order {
  lifecycle {
    CREATED -> PROCESSING
    PROCESSING -> SHIPPED
    PROCESSING -> CANCELLED
    SHIPPED -> DELIVERED
  }
}

// Define event with lifecycle effect
event OrderShipped {
  lifecycle_effect {
    Order.PROCESSING -> Order.SHIPPED
  }
}

// Simulate event sequence
simulate Order from CREATED events: OrderProcessed, OrderShipped
```

## Best Practices

### 1. Incremental Development
- Start with high-level systems
- Add containers and components gradually
- Validate at each step

### 2. Use Snapshots
- Create snapshots before major changes
- Use variants for experimentation
- Document decisions in ADRs

### 3. Leverage Queries
- Use queries to explore the model
- Find dependencies and relationships
- Validate assumptions

### 4. Generate Diagrams
- Generate diagrams frequently
- Use different formats for different audiences
- Filter diagrams by scope

### 5. Validate Early
- Run validation after each change
- Address diagnostics promptly
- Use selective validation for speed

## Common Patterns

### Pattern 1: Layered Architecture

```sruja
system Application {
  container Presentation {
    description "User interface layer"
  }
  
  container Business {
    description "Business logic layer"
  }
  
  container Data {
    description "Data access layer"
  }
  
  relation Presentation -> Business
  relation Business -> Data
}
```

### Pattern 2: API Gateway

```sruja
system Microservices {
  container Gateway {
    description "API Gateway"
  }
  
  container ServiceA {
    description "Service A"
  }
  
  container ServiceB {
    description "Service B"
  }
  
  relation Gateway -> ServiceA
  relation Gateway -> ServiceB
}
```

### Pattern 3: Event-Driven

```sruja
system EventDriven {
  container Producer {
    description "Event producer"
  }
  
  container Queue {
    description "Message queue"
    type Queue
  }
  
  container Consumer {
    description "Event consumer"
  }
  
  relation Producer -> Queue
  relation Queue -> Consumer
}
```

## Troubleshooting

### Issue: Validation Errors
**Solution:** Check diagnostics, fix syntax errors, validate incrementally

### Issue: Diagram Not Rendering
**Solution:** Verify diagram format, check cell output type, ensure valid model

### Issue: Query Returns Empty
**Solution:** Verify model has been built, check query syntax, use `%ir` to inspect

### Issue: Variant Merge Conflicts
**Solution:** Review conflicts, resolve manually, use `%variant diff` to inspect

## Next Steps

1. Explore the example notebooks
2. Try modifying examples
3. Create your own architecture
4. Experiment with variants
5. Share your notebooks!

---

**Note:** Example notebooks are provided as `.ipynb` files that can be opened in JupyterLab, VSCode, or any Jupyter-compatible environment.


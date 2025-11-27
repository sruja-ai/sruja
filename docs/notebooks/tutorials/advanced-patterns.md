# Advanced Patterns in Sruja Notebooks

[← Back to Notebooks Index](../README.md)

## Overview

This tutorial covers advanced patterns and techniques for using Sruja Notebooks effectively in complex architecture scenarios.

## Pattern 1: Multi-System Architecture

### Defining Multiple Systems

```sruja
// E-commerce platform
system ECommerce {
  container WebStore {
    description "Customer-facing web store"
  }
  
  container OrderService {
    description "Order processing service"
  }
  
  container PaymentService {
    description "Payment processing"
  }
  
  relation WebStore -> OrderService
  relation OrderService -> PaymentService
}

// Inventory system
system Inventory {
  container InventoryAPI {
    description "Inventory management API"
  }
  
  container WarehouseDB {
    description "Warehouse database"
  }
  
  relation InventoryAPI -> WarehouseDB
}

// Cross-system relationship
relation ECommerce.OrderService -> Inventory.InventoryAPI {
  type uses
  description "Check inventory availability"
}
```

### Querying Across Systems

```sruja
// Find all systems
find systems

// Find relations between systems
find relations where from contains "ECommerce" and to contains "Inventory"
```

## Pattern 2: Domain-Driven Design

### Entities and Aggregates

```sruja
domain ECommerce {
  entity Order {
    description "Customer order aggregate"
    
    lifecycle {
      CREATED -> CONFIRMED
      CONFIRMED -> SHIPPED
      SHIPPED -> DELIVERED
      CONFIRMED -> CANCELLED
    }
  }
  
  entity Payment {
    description "Payment entity"
    
    lifecycle {
      PENDING -> AUTHORIZED
      AUTHORIZED -> CAPTURED
      AUTHORIZED -> FAILED
    }
  }
}
```

### Domain Events

```sruja
event OrderCreated {
  description "Order was created"
  
  lifecycle_effect {
    Order.CREATED -> Order.CONFIRMED
  }
}

event PaymentAuthorized {
  description "Payment was authorized"
  
  lifecycle_effect {
    Payment.PENDING -> Payment.AUTHORIZED
  }
}
```

### Simulating Complex Flows

```sruja
// Simulate order lifecycle
simulate Order from CREATED events: OrderCreated, OrderShipped, OrderDelivered

// Simulate payment with failure scenario
simulate Payment from PENDING events: PaymentAuthorized, PaymentFailed
```

## Pattern 3: Microservices Patterns

### Service Mesh Pattern

```sruja
system Microservices {
  container ServiceMesh {
    description "Service mesh for inter-service communication"
  }
  
  container UserService {
    description "User management service"
  }
  
  container ProductService {
    description "Product catalog service"
  }
  
  container OrderService {
    description "Order processing service"
  }
  
  // All services communicate via service mesh
  relation UserService -> ServiceMesh
  relation ProductService -> ServiceMesh
  relation OrderService -> ServiceMesh
}
```

### API Gateway Pattern

```sruja
system APIArchitecture {
  container Gateway {
    description "API Gateway"
  }
  
  container AuthService {
    description "Authentication service"
  }
  
  container BusinessService1 {
    description "Business service 1"
  }
  
  container BusinessService2 {
    description "Business service 2"
  }
  
  // Gateway routes to services
  relation Gateway -> AuthService
  relation Gateway -> BusinessService1
  relation Gateway -> BusinessService2
}
```

## Pattern 4: Event-Driven Architecture

### Event Sourcing Pattern

```sruja
system EventSourced {
  container EventStore {
    description "Event store"
    type Queue
  }
  
  container CommandHandler {
    description "Command handler"
  }
  
  container QueryHandler {
    description "Query handler"
  }
  
  container ReadModel {
    description "Read model database"
  }
  
  // Commands flow to event store
  relation CommandHandler -> EventStore
  
  // Events flow to read model
  relation EventStore -> ReadModel
  
  // Queries read from read model
  relation QueryHandler -> ReadModel
}
```

### CQRS Pattern

```sruja
system CQRS {
  container CommandSide {
    description "Command side (writes)"
  }
  
  container QuerySide {
    description "Query side (reads)"
  }
  
  container WriteDB {
    description "Write database"
  }
  
  container ReadDB {
    description "Read database"
  }
  
  relation CommandSide -> WriteDB
  relation QuerySide -> ReadDB
  relation WriteDB -> ReadDB {
    type uses
    description "Replication"
  }
}
```

## Pattern 5: Architecture Variants

### Creating Variants for Comparison

```sruja
// Base architecture - monolithic
%snapshot create monolithic "Monolithic approach"

system MonolithicApp {
  container Application {
    description "Single monolithic application"
  }
}

// Variant 1 - Microservices
%variant create microservices monolithic "Microservices approach"

system MicroservicesApp {
  container UserService {}
  container ProductService {}
  container OrderService {}
}

// Variant 2 - Modular monolith
%variant create modular monolithic "Modular monolith"

system ModularApp {
  container UserModule {}
  container ProductModule {}
  container OrderModule {}
}
```

### Comparing Variants

```sruja
// Compare microservices variant with base
%variant diff microservices

// Compare modular variant with base
%variant diff modular

// Merge successful variant
%variant merge microservices
```

## Pattern 6: Validation Strategies

### Selective Validation

```sruja
// Validate only naming conventions
validate all rules: naming

// Validate specific system
validate ECommerce rules: contracts, dependencies

// Validate with custom scope
validate ECommerce.OrderService
```

### Validation Workflow

```sruja
// 1. Quick validation after each change
validate all

// 2. Comprehensive validation before snapshot
validate all rules: all

// 3. Focused validation for specific concerns
validate ECommerce rules: security, performance
```

## Pattern 7: Documentation in Notebooks

### Architecture Decision Records (ADRs)

```markdown
# ADR-001: Choosing Microservices Architecture

## Status
Accepted

## Context
We need to scale our application and support multiple teams.

## Decision
We will adopt a microservices architecture.

## Consequences
- Better scalability
- Team autonomy
- Increased operational complexity
```

### Living Documentation

```sruja
// System documentation
system PaymentService {
  description """
    Payment processing service.
    
    Responsibilities:
    - Process payment requests
    - Handle payment authorization
    - Manage payment state
    
    Non-functional requirements:
    - Must process 1000 TPS
    - 99.9% uptime required
  """
}
```

## Pattern 8: Query Patterns

### Dependency Analysis

```sruja
// Find all dependencies of a system
find relations where from = "PaymentService"

// Find what depends on a system
find relations where to = "PaymentService"

// Find circular dependencies
find relations where from = to
```

### Technology Analysis

```sruja
// Find all Node.js components
select containers where technology = "Node.js"

// Find all databases
select containers where type = "DataStore"

// Find all external services
select containers where type = "ExternalService"
```

## Pattern 9: Incremental Refinement

### Start High-Level

```sruja
// Iteration 1: High-level systems
system ECommerce {}
system Inventory {}
```

### Add Containers

```sruja
// Iteration 2: Add containers
system ECommerce {
  container WebApp {}
  container API {}
  container DB {}
}
```

### Add Components

```sruja
// Iteration 3: Add components
system ECommerce {
  container API {
    component OrderController {}
    component PaymentController {}
  }
}
```

### Add Details

```sruja
// Iteration 4: Add relations and details
system ECommerce {
  container API {
    component OrderController {}
  }
  
  container DB {}
  
  relation API -> DB
}
```

## Pattern 10: Testing Architecture Changes

### Before/After Comparison

```sruja
// Before: Create snapshot
%snapshot create before "Before refactoring"

// Make changes
system RefactoredSystem {
  // ... new architecture ...
}

// After: Compare
%snapshot create after "After refactoring"

// Use queries to compare
find systems
find containers
find relations
```

## Best Practices Summary

1. **Start Simple**: Begin with high-level abstractions
2. **Iterate**: Refine incrementally
3. **Validate**: Check after each change
4. **Document**: Use markdown cells for decisions
5. **Experiment**: Use variants for alternatives
6. **Query**: Explore your model frequently
7. **Visualize**: Generate diagrams regularly
8. **Snapshot**: Save stable versions

## Next Steps

- Explore [Example Notebooks](../examples/README.md)
- Review [Complete Feature List](../COMPLETED-FEATURES.md)
- Check [Implementation Status](../IMPLEMENTATION-STATUS.md)

---

**Master these patterns to become proficient with Sruja Architecture Notebooks!**


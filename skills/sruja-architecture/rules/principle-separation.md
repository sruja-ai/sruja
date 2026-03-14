# principle-separation

## Why It Matters

Separation of concerns is a fundamental architectural principle that improves maintainability, testability, and flexibility. When each component has a single, well-defined responsibility, the architecture becomes easier to understand and modify.

## When to Apply

Always apply separation of concerns when:

- Designing new architectures
- Refactoring existing systems
- Identifying components for microservices
- Organizing business logic

## Correct Approach

### Example 1: Separated Responsibilities

```sruja
OrderManagement = system "Order Management" {
  OrderService = container "Order API" {
    technology "Node.js"
    description "Handles order lifecycle operations"
  }

  PaymentService = container "Payment Service" {
    technology "Python"
    description "Processes payment transactions"
  }

  InventoryService = container "Inventory Service" {
    technology "Go"
    description "Manages product inventory"
  }
}
```

### Example 2: Single Responsibility Components

```sruja
Frontend = container "Web Frontend" {
  technology "React"
  description "User interface and presentation layer"
}

API = container "API Gateway" {
  technology "Kong"
  description "Routing, authentication, rate limiting"
}

Backend = container "Business Service" {
  technology "Java + Spring Boot"
  description "Core business logic and use cases"
}

Database = database "Database" {
  technology "PostgreSQL"
  description "Data persistence and storage"
}
```

## Incorrect Approach

```sruja
// ❌ One container doing everything
Ecommerce = container "E-Commerce App" {
  technology "Node.js"
  description "Frontend, backend, payments, inventory, all in one"
}

Ecommerce -> Database "everything"
```

## Common Mistakes

1. **God Components**: Single container handling multiple concerns
   - ❌ API, Worker, and Scheduler all in one
   - ✅ Split into focused containers

2. **Mixed Layers**: UI, business logic, and data access in same component
   - ❌ Frontend directly accessing database
   - ✅ Frontend → API → Database

3. **Tight Coupling**: Direct dependencies on implementations
   - ❌ Component calling specific database directly
   - ✅ Using interfaces and abstraction layers

## Additional Context

Separation of concerns relates to:

- `principle-layered` - Organize into clear architectural layers
- `principle-cohesion-coupling` - Balance cohesion and coupling
- `pattern-hexagonal` - Ports and adapters pattern
- `anti-god-component` - Avoid monolithic components

## References

- Single Responsibility Principle (SOLID)
- Clean Architecture by Robert C. Martin
- Domain-Driven Design by Eric Evans
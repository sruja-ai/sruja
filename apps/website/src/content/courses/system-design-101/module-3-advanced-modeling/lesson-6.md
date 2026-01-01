---
title: "Lesson 6: Advanced DSL Features"
weight: 6
summary: "Master views, scenarios, flows, and element kinds to create production-ready models."
---

# Lesson 6: Advanced DSL Features

The Sruja DSL provides a **flat syntax** where all declarations—from element kinds to views—are top-level. This lesson covers the advanced capabilities that make your models more maintainable, understandable, and useful.

## Kinds and Types: Your Foundation

Before creating instances of your architecture (like a "Database"), you must establish what **kinds** of elements exist. This isn't just documentation—it provides real benefits:

### Benefits

1. **Early Validation**: Catches typos in element types before runtime
2. **Better Tooling**: Enables autocomplete, validation, and refactoring
3. **Documentation**: Makes available element types explicit
4. **Organization**: Separates structure definition from instantiation

```sruja
import { * } from 'sruja.ai/stdlib'


// Now you can use any of the declared element types
Customer = person "Customer"
App = system "Application" {
API = container "API"
DB = datastore "Database"
}
```

### Best Practice

Declare all element types you'll use upfront. This makes your model self-documenting and enables better tooling support.

## Multiple Views: One Model, Many Perspectives

Use `view` blocks to create custom perspectives from your architecture. This is essential for communicating with different audiences. Unlike some other tools, Sruja allows defining views anywhere in your file, though keeping them at the bottom is a common convention.

### Real-World Example: E-Commerce Platform

```sruja
import { * } from 'sruja.ai/stdlib'


Customer = person "Customer"
Admin = person "Administrator"

ECommerce = system "E-Commerce Platform" {
WebApp = container "Web Application" {
  CartComponent = component "Shopping Cart"
  ProductComponent = component "Product Catalog"
}
API = container "API Service" {
  OrderController = component "Order Controller"
  PaymentController = component "Payment Controller"
}
OrderDB = datastore "Order Database"
ProductDB = datastore "Product Database"
Cache = datastore "Redis Cache"
EventQueue = queue "Event Queue"
}

PaymentGateway = system "Payment Gateway" {
metadata {
    tags ["external"]
  }
}

Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.OrderDB "Stores orders"
ECommerce.API -> ECommerce.Cache "Caches queries"
ECommerce.API -> PaymentGateway "Processes payments"

// Executive view: High-level business context
view executive {
title "Executive Overview"
include Customer
include Admin
include ECommerce
include PaymentGateway
exclude ECommerce.WebApp
exclude ECommerce.API
exclude ECommerce.OrderDB
exclude ECommerce.ProductDB
exclude ECommerce.Cache
exclude ECommerce.EventQueue
}

// Architect view: Container-level architecture
view architect {
title "Architectural View"
include ECommerce
include ECommerce.WebApp
include ECommerce.API
include ECommerce.OrderDB
include ECommerce.ProductDB
include ECommerce.Cache
include ECommerce.EventQueue
include PaymentGateway
exclude Customer
exclude Admin
}

// Developer view: Component-level implementation
view developer {
title "Developer View"
include ECommerce.WebApp
include ECommerce.API
include ECommerce.OrderDB
include ECommerce.ProductDB
include ECommerce.Cache
exclude Customer
exclude Admin
exclude PaymentGateway
}

// Data flow view: Focus on data dependencies
view dataflow {
title "Data Flow View"
include ECommerce.API
include ECommerce.OrderDB
include ECommerce.ProductDB
include ECommerce.Cache
include ECommerce.EventQueue
exclude Customer
exclude Admin
exclude ECommerce.WebApp
exclude PaymentGateway
}

// User journey view: Customer experience
view userjourney {
title "User Journey View"
include Customer
include ECommerce.WebApp
include ECommerce.API
include PaymentGateway
exclude Admin
exclude ECommerce.OrderDB
exclude ECommerce.ProductDB
exclude ECommerce.Cache
exclude ECommerce.EventQueue
}

// Default view: Complete system
view index {
title "Complete System View"
include *
}
```

### View Strategies

1. **By Audience**: Executive, Architect, Developer, Product Manager
2. **By Concern**: Data flow, Security, Performance, User experience
3. **By Layer**: Context, Container, Component
4. **By Feature**: Checkout flow, User management, Analytics

## Scenarios: Modeling User Journeys

Scenarios model behavioral flows—what happens when users interact with your system. They're perfect for documenting user stories and use cases.

### Example: Checkout Flow

```sruja
import { * } from 'sruja.ai/stdlib'


Customer = person "Customer"

ECommerce = system "E-Commerce System" {
WebApp = container "Web Application"
API = container "API Service"
OrderDB = datastore "Order Database"
}

Inventory = system "Inventory System" {
InventoryService = container "Inventory Service"
}

PaymentGateway = system "Payment Gateway" {
metadata {
    tags ["external"]
  }
}

// Model the checkout journey
// EXPECTED_FAILURE: Layer violation (scenarios model bidirectional user journeys)
CheckoutFlow = scenario "User Checkout Journey" {
Customer -> ECommerce.WebApp "Adds items to cart"
ECommerce.WebApp -> ECommerce.API "Submits checkout"
ECommerce.API -> Inventory.InventoryService "Reserves stock"
Inventory.InventoryService -> ECommerce.API "Confirms availability"
ECommerce.API -> PaymentGateway "Processes payment"
PaymentGateway -> ECommerce.API "Confirms payment"
ECommerce.API -> ECommerce.OrderDB "Saves order"
ECommerce.API -> ECommerce.WebApp "Returns confirmation"
ECommerce.WebApp -> Customer "Shows order confirmation"
}

// Alternative happy path
CheckoutSuccess = scenario "Successful Checkout" {
Customer -> ECommerce.WebApp "Completes checkout"
ECommerce.WebApp -> ECommerce.API "Processes order"
ECommerce.API -> Customer "Confirms order"
}

// Error scenario
CheckoutFailure = scenario "Checkout Failure" {
Customer -> ECommerce.WebApp "Attempts checkout"
ECommerce.WebApp -> ECommerce.API "Validates order"
ECommerce.API -> Inventory.InventoryService "Checks stock"
Inventory.InventoryService -> ECommerce.API "Out of stock"
ECommerce.API -> ECommerce.WebApp "Returns error"
ECommerce.WebApp -> Customer "Shows out of stock message"
}

view index {
include *
}
```

### When to Use Scenarios

- **User Stories**: Document how users interact with your system
- **Use Cases**: Model specific business processes
- **Error Handling**: Document failure paths and recovery
- **Integration Testing**: Define test scenarios

## Flows: Modeling Data Pipelines

Flows model data-oriented processes—how data moves through your system. Use them for ETL, streaming, and batch processing.

### Example: Analytics Pipeline

```sruja
// EXPECTED_FAILURE: Layer violation (flows model bidirectional data movement)
import { * } from 'sruja.ai/stdlib'


Analytics = system "Analytics Platform" {
IngestionService = container "Data Ingestion"
ProcessingService = container "Data Processing"
QueryService = container "Query Service"
EventStream = queue "Event Stream"
RawDataDB = datastore "Raw Data Store"
ProcessedDataDB = datastore "Processed Data Warehouse"
}

// Data flow: Event ingestion pipeline
EventIngestion = flow "Event Ingestion Pipeline" {
Analytics.IngestionService -> Analytics.EventStream "Publishes events"
Analytics.EventStream -> Analytics.ProcessingService "Streams events"
Analytics.ProcessingService -> Analytics.RawDataDB "Stores raw data"
Analytics.ProcessingService -> Analytics.ProcessedDataDB "Stores processed data"
Analytics.QueryService -> Analytics.ProcessedDataDB "Queries analytics"
}

// Batch processing flow
BatchProcessing = flow "Daily Batch Processing" {
Analytics.RawDataDB -> Analytics.ProcessingService "Extracts daily data"
Analytics.ProcessingService -> Analytics.ProcessingService "Transforms data"
Analytics.ProcessingService -> Analytics.ProcessedDataDB "Loads aggregated data"
}

view index {
include *
}
```

### Scenario vs Flow

- **Scenario**: Behavioral flows (user actions, business processes)
- **Flow**: Data flows (ETL, streaming, batch processing)

## Integrating Requirements, ADRs, and Policies

Sruja's flat syntax makes it easy to integrate requirements, ADRs, and policies directly into your architecture model as top-level declarations.

### Complete Example

```sruja
import { * } from 'sruja.ai/stdlib'


Customer = person "Customer"

// Requirements drive architecture
R1 = requirement "Must handle 10k concurrent users" { tags ["functional"] }
R2 = requirement "API response < 200ms p95" { tags ["performance"] }
R3 = requirement "Scale to 1M users" { tags ["scalability"] }
R4 = requirement "All PII encrypted at rest" { tags ["security"] }

// Architecture decisions documented as ADRs
ADR001 = adr "Use microservices for independent scaling" {
status "Accepted"
context "Need to scale order processing independently from inventory"
decision "Split into OrderService and InventoryService"
consequences "Better scalability, increased network complexity"
}

ADR002 = adr "Use PostgreSQL for strong consistency" {
status "Accepted"
context "Need ACID transactions for financial data"
decision "Use PostgreSQL instead of NoSQL"
consequences "Strong consistency, SQL complexity"
}

// Architecture that satisfies requirements
ECommerce = system "E-Commerce Platform" {
API = container "API Service" {
  technology "Go"
  description "Satisfies R1, R2, R3"
  // adr ADR001 ADR002

  slo {
    availability {
      target "99.99%"
      window "30 days"
    }
    latency {
      p95 "200ms"
      p99 "500ms"
    }
  }
}

OrderDB = datastore "Order Database" {
  technology "PostgreSQL"
  description "Satisfies R4 - encrypted at rest"
  // adr ADR002
}
}

// Policy enforcement
SecurityPolicy = policy "All databases must be encrypted" {
category "security"
enforcement "required"
description "Compliance requirement for PII data"
}

view index {
include *
}
```

## Best Practices

1. **Explicit Kinds**: Import or declare all element kinds upfront.
2. **Use Multiple Views**: Create views for different audiences and concerns
3. **Document with Scenarios**: Model user journeys and business processes
4. **Model Data Flows**: Use flows for ETL and data pipelines
5. **Link Requirements**: Connect requirements to architecture decisions
6. **Document Decisions**: Use ADRs to explain why, not just what
7. **Define SLOs**: Model service level objectives for production systems

## Next Steps

Now that you understand the advanced features, you can create production-ready models that:

- Communicate effectively with different audiences
- Document user journeys and data flows
- Link requirements to architecture decisions
- Enable automated validation and governance

👉 **[Module 4: Production Readiness](../module-4-production-readiness/lesson-1)** - Learn how to make your architecture production-ready.

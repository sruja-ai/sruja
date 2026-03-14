# Sruja Architecture DSL - Complete Guide

Comprehensive guide for software architecture design using Sruja DSL. This document is compiled from individual rules and contains all patterns, principles, and best practices for AI agents generating Sruja architectures.

**Syntax:** Sruja uses **flat, top-level declarations** — no `architecture "Name" { }` wrapper. Declare kinds (or use `import { * } from 'sruja.ai/stdlib'`), then elements and relationships at the top level. Use PascalCase for element IDs.

## Quick Start

When generating Sruja architecture DSL:

1. **Identify all external actors** (users, systems, services)
2. **Define major systems** and their boundaries
3. **Break down into containers** (services, applications)
4. **Define datastores** (databases, caches, queues)
5. **Map relationships** with clear labels and protocols
6. **Apply architectural patterns** appropriate to the use case
7. **Check for anti-patterns** and fix them
8. **Validate trade-offs** and document decisions

---

## Architectural Principles

### Separation of Concerns

Split systems into logical components based on responsibility. Each component should have a single, well-defined purpose. Avoid mixing business logic with infrastructure concerns.

**When to Apply:** Always apply when designing new architectures or refactoring.

**Example:**

```sruja
OrderManagement = system "Order Management" {
  description "Handles order lifecycle and payments"

  OrderAPI = container "Order API" {
    technology "Node.js"
    description "Handles order lifecycle"
  }

  PaymentService = container "Payment Service" {
    technology "Python"
    description "Processes payments"
  }

  InventoryService = container "Inventory Service" {
    technology "Go"
    description "Manages inventory"
  }
}
```

### Layered Architecture

Organize into clear layers: Presentation → Application → Domain → Infrastructure

**Example:**

```sruja
WebFrontend = container "Web Frontend" {
  technology "React"
  description "User interface"
}

APIGateway = container "API Gateway" {
  technology "Express"
  description "HTTP API endpoints"
}

BusinessService = container "Business Service" {
  technology "Node.js"
  description "Core business logic"
}

DataService = container "Data Service" {
  technology "Node.js"
  description "Data access layer"
}

Database = database "Database" {
  technology "PostgreSQL"
  description "Data persistence"
}
```

### Bounded Contexts

Group related functionality into distinct contexts. Each context has its own domain model.

**Example:**

```sruja
UserManagement = system "User Management" {
  description "User identity and profile"
  UserService = container "User Service" { ... }
}

OrderProcessing = system "Order Processing" {
  description "Order lifecycle"
  OrderService = container "Order Service" { ... }
}

Payments = system "Payments" {
  description "Payment processing"
  PaymentService = container "Payment Service" { ... }
}
```

### Dependency Rule

Dependencies should point inward. Use dependency inversion: depend on abstractions, not concretes.

### Cohesion vs Coupling

- **High cohesion**: Related functionality grouped together
- **Low coupling**: Minimal dependencies between components

---## Component Types

### Person (Human Actors Only)

Use **only for human** external actors that interact with the system. Do not use person for external software (APIs, SaaS, backends)—use **system** for those, with optional `tags ["external"]`.

**When to Use:**

- Users (Admin, Customer, Guest)
- Administrators, operators, support
- Stakeholders (report viewers, managers)
- Developers or API consumers when they are human users

**Example:**

```sruja
User = person "End User" {
  description "Customer using application"
}

Admin = person "Administrator" {
  description "System administrator"
}

// External software: use system, not person (see System section)
```

### System (Major Boundaries)

Use for high-level system boundaries: your own systems and **external software** (APIs, SaaS, control planes, destinations, transformers). For external systems you don't own, add `tags ["external"]` or `tags ["vendor"]` when useful.

**Example:**

```sruja
OrderSystem = system "Order Management" {
  description "Handles order lifecycle"
}

ExternalSystem = system "External Inventory" {
  description "Third-party inventory system"
  tags ["external"]
}
```

### Container (Deployable Units)

Use for deployable units (processes, services, applications).

**When to Use:**

- API services (REST, GraphQL, gRPC)
- Background workers/job processors
- Message consumers/producers
- Web applications

**Example:**

```sruja
APIService = container "Order API" {
  technology "Node.js + Express"
  description "RESTful API for orders"
}

Worker = container "Order Processor" {
  technology "Python + Celery"
  description "Background worker"
}
```

### Datastore (Storage/Cache)

Use for persistent storage or cache.

**Example:**

```sruja
Database = database "Orders DB" {
  technology "PostgreSQL"
  description "Primary database"
}

Cache = database "Cache" {
  technology "Redis"
  description "Application cache"
}

Queue = queue "Message Queue" {
  technology "RabbitMQ"
  description "Event streaming"
}
```

---

## Architectural Patterns

### Monolith with Modular Boundaries

Single deployable unit with clear internal module boundaries.

**Use when:**

- Small teams (1-10 developers)
- Simple domain
- Rapid development needed
- Building MVP

**Example:**

```sruja
Application = system "Application" {
  description "Single deployable unit with clear module boundaries"

  APIGateway = container "API Gateway" {
    technology "Node.js"
    description "Single entry point"
  }

  UserModule = container "User Module" {
    technology "Node.js"
    description "User management"
  }

  ProjectModule = container "Project Module" {
    technology "Node.js"
    description "Project management"
  }

  Database = database "Database" {
    technology "PostgreSQL"
    description "Central database"
  }
}

Application.APIGateway -> Application.UserModule "gRPC (internal)"
Application.APIGateway -> Application.ProjectModule "gRPC (internal)"
Application.UserModule -> Application.Database "SQL"
Application.ProjectModule -> Application.Database "SQL"
```

### Microservices

Multiple independent services communicating via APIs/events.

**Use when:**

- Multiple teams
- Complex domain
- Independent scaling needed
- Different technologies per service

**Example:**

```sruja
UserService = container "User Service" {
  technology "Go"
  description "User management"
}

OrderService = container "Order Service" {
  technology "Node.js"
  description "Order processing"
}

PaymentService = container "Payment Service" {
  technology "Python"
  description "Payment processing"
}

EventStore = queue "Event Store" {
  technology "Kafka"
  description "Event streaming"
}

UserService -> OrderService "REST API"
OrderService -> PaymentService "REST API"
OrderService -> EventStore "publishes events"
PaymentService -> EventStore "publishes events"
```

### Event-Driven Architecture

Event producers and consumers with async messaging.

**Use when:**

- Real-time processing
- Loose coupling needed
- Eventual consistency acceptable

**Example:**

```sruja
EventStore = queue "Kafka" {
  technology "Kafka"
  description "Central event stream"
}

OrderService = container "Order Service" {
  technology "Node.js"
  description "Publishes order events"
}

AnalyticsService = container "Analytics Service" {
  technology "Python"
  description "Consumes events for analytics"
}

OrderService -> EventStore "publishes events"
AnalyticsService -> EventStore "consumes events"
```

### CQRS (Command Query Responsibility Segregation)

Separate read and write models.

**Example:**

```sruja
EventStore = queue "Event Store" {
  technology "Kafka"
  description "Event stream"
}

WriteModel = container "Write Service" {
  technology "Node.js"
  description "Handles commands"
}

WriteDB = database "Write Database" {
  technology "PostgreSQL"
  description "Normalized write model"
}

ReadModel = container "Read Service" {
  technology "Go"
  description "Optimized for queries"
}

ReadDB = database "Read Database" {
  technology "Elasticsearch"
  description "Denormalized read model"
}

WriteModel -> WriteDB "writes"
WriteModel -> EventStore "publishes events"
ReadModel -> EventStore "subscribes"
ReadModel -> ReadDB "reads"
```

### Hexagonal Architecture (Ports and Adapters)

Domain core with no external dependencies, ports as interfaces, adapters as implementations.

**Example:**

```sruja
DomainCore = container "Domain Core" {
  technology "Java"
  description "Business logic (no external deps)"
}

HTTPAdapter = container "HTTP Adapter" {
  technology "Spring Boot"
  description "REST API implementation"
}

DBAdapter = container "Database Adapter" {
  technology "Spring Data"
  description "PostgreSQL implementation"
}

HTTPAdapter -> DomainCore "uses"
DBAdapter -> DomainCore "implements"
```

---

## Relationship Guidelines

### Synchronous Relationships

Use `->` for requests/responses requiring immediate feedback.

**Protocols:** HTTPS, HTTP/2, gRPC, REST API, GraphQL

**Example:**

```sruja
Web -> API "HTTPS"
API -> Database "PostgreSQL (JDBC)"
API -> ExternalService "REST API"
```

### Asynchronous Relationships

Use `->` with event labels for messaging.

**Labels:** "publishes events to", "subscribes to", "emits events"

**Example:**

```sruja
Producer -> Kafka "publishes events"
Consumer -> Kafka "consumes events"
Worker -> Queue "subscribes"
```

### Relationship Labels

Be specific, include protocol and purpose.

**DO:**

- Be specific: "HTTPS" vs "API call"
- Include protocol: "REST API", "gRPC"
- Show purpose: "reads from", "writes to", "publishes events"
- Indicate direction: "reads from", "writes to"

**DON'T:**

- Vague labels: "uses", "connects to"
- Inconsistent naming conventions

**Good Examples:**

```sruja
Frontend -> API "HTTPS"
API -> Database "reads/writes"
API -> MessageQueue "publishes events"
Worker -> MessageQueue "consumes events"
```

---

## Anti-Patterns to Avoid

### God Component

Single container doing too many responsibilities.

**❌ Wrong:**

```sruja
God = container "Everything" {
  technology "Node.js"
  description "Auth, orders, payments, inventory, notifications"
}
```

**✅ Correct:**

```sruja
AuthService = container "Auth Service" { ... }
OrderService = container "Order Service" { ... }
PaymentService = container "Payment Service" { ... }
```

### Direct Database Access from Multiple Layers

Frontend, worker, API all accessing database directly.

**❌ Wrong:**

```sruja
Frontend -> Database "SQL"
Worker -> Database "SQL"
API -> Database "SQL"
```

**✅ Correct:**

```sruja
Frontend -> API "HTTPS"
Worker -> API "REST API"
API -> Database "SQL"
```

### Circular Dependencies

Service A → Service B → Service A

**❌ Wrong:**

```sruja
ServiceA -> ServiceB "calls"
ServiceB -> ServiceA "calls"
```

**✅ Correct:**

```sruja
ServiceA -> CommonService "uses"
ServiceB -> CommonService "uses"
```

### Tight Coupling

Components calling specific implementations directly.

**❌ Wrong:**

```sruja
Service -> PostgreSQLDirect "uses"
```

**✅ Correct:**

```sruja
Service -> DataLayer "uses"
DataLayer -> PostgreSQL "implements"
```

### Orphan Components

Components with no relationships.

**❌ Wrong:**

```sruja
Orphan = container "No Relationships" {
  description "Unused component"
}
```

**✅ Correct:**

```sruja
Active = container "Used Service" {
  description "Has clear purpose and connections"
}

API -> Active "REST API"
```

---

## Trade-offs and Decisions

### Monolith vs Microservices

| Aspect            | Monolith                    | Microservices                  |
| ----------------- | --------------------------- | ------------------------------ |
| Development Speed | Fast initially, slows later | Slower initially, faster later |
| Scalability       | Scale everything            | Scale individually             |
| Technology        | Single stack                | Multiple technologies          |
| Deployment        | Single unit                 | Multiple units                 |
| Complexity        | Low                         | High                           |
| Cost              | Low initially               | Higher overhead                |
| Team Size         | Best for small teams        | Best for large teams           |

**Choose Monolith when:**

- Team size < 10 developers
- Simple domain
- Time-to-market critical
- Building MVP

**Choose Microservices when:**

- Multiple teams
- Complex domain
- Different scaling needs
- Different technologies needed

### Synchronous vs Asynchronous

**Synchronous (HTTP/gRPC):**

- ✅ Simpler to reason about
- ✅ Immediate feedback
- ✅ Consistent state
- ❌ Coupling (consumer must be available)
- ❌ Limited scalability
- ❌ Cascading failures

**Asynchronous (Events):**

- ✅ Loose coupling
- ✅ Better scalability
- ✅ Better fault tolerance
- ❌ Complexity (event ordering, retries)
- ❌ Eventual consistency
- ❌ Harder debugging

**Choose Synchronous when:**

- Real-time requirements
- Immediate feedback needed
- Consistency is critical

**Choose Asynchronous when:**

- Loose coupling needed
- Scalability is priority
- Fault tolerance important
- Eventual consistency acceptable

---

## Common Architectural Scenarios

### E-Commerce Platform

```sruja
Customer = person "Customer" {
  description "End user of the e-commerce platform"
}

Application = system "Application" {
  description "Core e-commerce application"

  WebFrontend = container "Web Frontend" {
    technology "React"
    description "User interface"
  }

  APIGateway = container "API Gateway" {
    technology "Kong"
    description "Routing and auth"
  }

  UserService = container "User Service" {
    technology "Go"
    description "User management"
  }

  OrderService = container "Order Service" {
    technology "Node.js"
    description "Order processing"
  }

  PaymentService = container "Payment Service" {
    technology "Python"
    description "Payment processing"
  }

  InventoryService = container "Inventory Service" {
    technology "Java"
    description "Inventory management"
  }

  NotificationService = container "Notification Service" {
    technology "Node.js"
    description "Email, SMS, push"
  }

  Database = database "Database" {
    technology "PostgreSQL"
    description "Primary database"
  }

  Cache = database "Cache" {
    technology "Redis"
    description "Caching layer"
  }

  EventStore = queue "Event Store" {
    technology "Kafka"
    description "Event streaming"
  }
}

Stripe = system "Stripe" {
  description "External payment service"
  tags ["external"]
}

Customer -> Application.WebFrontend "HTTPS"
Application.WebFrontend -> Application.APIGateway "HTTPS"
Application.APIGateway -> Application.UserService "REST API"
Application.APIGateway -> Application.OrderService "REST API"
Application.OrderService -> Application.PaymentService "REST API"
Application.OrderService -> Application.InventoryService "REST API"
Application.OrderService -> Application.NotificationService "REST API"
Application.UserService -> Application.Database "SQL"
Application.OrderService -> Application.Database "SQL"
Application.PaymentService -> Application.Database "SQL"
Application.UserService -> Application.Cache "Redis"
Application.OrderService -> Application.Cache "Redis"
Application.PaymentService -> Stripe "REST API"
Application.PaymentService -> Application.EventStore "publishes events"
Application.NotificationService -> Application.EventStore "consumes events"
```

### Real-Time Analytics

```sruja
Analytics = system "Analytics" {
  description "Real-time analytics platform"

  Collector = container "Data Collector" {
    technology "Python"
    description "Ingests events from sources"
  }

  StreamProcessor = container "Stream Processor" {
    technology "Kafka Streams"
    description "Real-time processing"
  }

  Database = database "Time Series DB" {
    technology "InfluxDB"
    description "Stores metrics"
  }

  API = container "Analytics API" {
    technology "Go"
    description "Query interface"
  }

  Dashboard = container "Dashboard" {
    technology "React"
    description "Visualization"
  }
}

Analytics.Collector -> Analytics.StreamProcessor "publishes events"
Analytics.StreamProcessor -> Analytics.Database "writes metrics"
Analytics.API -> Analytics.Database "reads metrics"
Analytics.Dashboard -> Analytics.API "GraphQL"
```

---

## Checklist for Valid Architectures

- [ ] All components have clear purposes and descriptions
- [ ] Relationships show clear data flow and protocols
- [ ] No circular dependencies
- [ ] No orphan components
- [ ] Appropriate architectural patterns applied
- [ ] Security considerations included
- [ ] Scalability addressed
- [ ] Technology choices justified
- [ ] Trade-offs acknowledged
- [ ] Follows separation of concerns
- [ ] High cohesion within components
- [ ] Low coupling between components
- [ ] Relationships use specific, descriptive labels

---

## Prompt Templates for AI Generation

### Generate from Requirements

```
Generate Sruja architecture for [description]:

Requirements:
- [List requirements]

Consider:
- [Architecture patterns to apply]
- [Scalability needs]
- [Security requirements]

Provide:
- Complete Sruja DSL
- Explanation of architectural decisions
- Trade-offs made
```

### Refactor Existing

```
Review and refactor this Sruja architecture:

[PASTE DSL]

Issues:
- [List problems]

Refactor to:
- [Goals]
- Apply best practices
- Remove anti-patterns
```

### Add Feature

```
Add [feature] to this architecture:

[PASTE EXISTING DSL]

Feature requirements:
- [Requirements]

Update architecture appropriately.
```

---

## Resources

- **Documentation:** https://sruja.ai/docs
- **Language Spec:** docs/LANGUAGE_SPECIFICATION.md
- **Examples:** examples/
- **GitHub:** https://github.com/sruja-ai/sruja
---

**Version:** Aligns with Sruja repo release (see GitHub Releases).

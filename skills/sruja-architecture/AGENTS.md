# Sruja Architecture DSL - Complete Guide

Comprehensive guide for software architecture design using Sruja DSL. This document is compiled from individual rules and contains all patterns, principles, and best practices for AI agents generating Sruja architectures.

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
architecture "E-Commerce" {
  system "Order Management" {
    order_service = container "Order API" {
      technology "Node.js"
      description "Handles order lifecycle"
    }

    payment_service = container "Payment Service" {
      technology "Python"
      description "Processes payments"
    }

    inventory_service = container "Inventory Service" {
      technology "Go"
      description "Manages inventory"
    }
  }
}
```

### Layered Architecture

Organize into clear layers: Presentation → Application → Domain → Infrastructure

**Example:**

```sruja
web_frontend = container "Web Frontend" {
  technology "React"
  description "User interface"
}

api_gateway = container "API Gateway" {
  technology "Express"
  description "HTTP API endpoints"
}

business_service = container "Business Service" {
  technology "Node.js"
  description "Core business logic"
}

data_service = container "Data Service" {
  technology "Node.js"
  description "Data access layer"
}

database = database "Database" {
  technology "PostgreSQL"
  description "Data persistence"
}
```

### Bounded Contexts

Group related functionality into distinct contexts. Each context has its own domain model.

**Example:**

```sruja
system "User Management" {
  user_service = container "User Service" { ... }
}

system "Order Processing" {
  order_service = container "Order Service" { ... }
}

system "Payments" {
  payment_service = container "Payment Service" { ... }
}
```

### Dependency Rule

Dependencies should point inward. Use dependency inversion: depend on abstractions, not concretes.

### Cohesion vs Coupling

- **High cohesion**: Related functionality grouped together
- **Low coupling**: Minimal dependencies between components

---## Component Types

### Person (External Actors)

Use for external entities that interact with the system.

**When to Use:**

- Users (Admin, Customer, Guest)
- External systems (Payment Gateway, SaaS)
- Third-party services (Analytics, Monitoring)

**Example:**

```sruja
user = person "End User" {
  description "Customer using application"
}

admin = person "Administrator" {
  description "System administrator"
}

stripe = person "Stripe" {
  description "External payment processing"
}
```

### System (Major Boundaries)

Use for high-level system boundaries representing major domains.

**Example:**

```sruja
order_system = system "Order Management" {
  description "Handles order lifecycle"
}

external_system = system "External Inventory" {
  description "Third-party inventory system"
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
api_service = container "Order API" {
  technology "Node.js + Express"
  description "RESTful API for orders"
}

worker = container "Order Processor" {
  technology "Python + Celery"
  description "Background worker"
}
```

### Datastore (Storage/Cache)

Use for persistent storage or cache.

**Example:**

```sruja
database = database "Orders DB" {
  technology "PostgreSQL"
  description "Primary database"
}

cache = database "Cache" {
  technology "Redis"
  description "Application cache"
}

queue = queue "Message Queue" {
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
architecture "Project Management" {
  system "Application" {
    api_gateway = container "API Gateway" {
      technology "Node.js"
      description "Single entry point"
    }

    user_module = container "User Module" {
      technology "Node.js"
      description "User management"
    }

    project_module = container "Project Module" {
      technology "Node.js"
      description "Project management"
    }

    database = database "Database" {
      technology "PostgreSQL"
      description "Central database"
    }
  }

  api_gateway -> user_module "gRPC (internal)"
  api_gateway -> project_module "gRPC (internal)"

  user_module -> database "SQL"
  project_module -> database "SQL"
}
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
architecture "E-Commerce" {
  user_service = container "User Service" {
    technology "Go"
    description "User management"
  }

  order_service = container "Order Service" {
    technology "Node.js"
    description "Order processing"
  }

  payment_service = container "Payment Service" {
    technology "Python"
    description "Payment processing"
  }

  event_store = queue "Event Store" {
    technology "Kafka"
    description "Event streaming"
  }

  user_service -> order_service "REST API"
  order_service -> payment_service "REST API"
  order_service -> event_store "publishes events"
  payment_service -> event_store "publishes events"
}
```

### Event-Driven Architecture

Event producers and consumers with async messaging.

**Use when:**

- Real-time processing
- Loose coupling needed
- Eventual consistency acceptable

**Example:**

```sruja
event_store = queue "Kafka" {
  description "Central event stream"
}

producer = container "Order Service" {
  technology "Node.js"
  description "Publishes order events"
}

consumer = container "Analytics Service" {
  technology "Python"
  description "Consumes events for analytics"
}

producer -> event_store "publishes events"
consumer -> event_store "consumes events"
```

### CQRS (Command Query Responsibility Segregation)

Separate read and write models.

**Example:**

```sruja
write_model = container "Write Service" {
  technology "Node.js"
  description "Handles commands"
}

write_db = database "Write Database" {
  technology "PostgreSQL"
  description "Normalized write model"
}

read_model = container "Read Service" {
  technology "Go"
  description "Optimized for queries"
}

read_db = database "Read Database" {
  technology "Elasticsearch"
  description "Denormalized read model"
}

write_model -> write_db "writes"
write_model -> event_store "publishes events"
read_model -> event_store "subscribes"
read_model -> read_db "reads"
```

### Hexagonal Architecture (Ports and Adapters)

Domain core with no external dependencies, ports as interfaces, adapters as implementations.

**Example:**

```sruja
domain_core = container "Domain Core" {
  technology "Java"
  description "Business logic (no external deps)"
}

http_adapter = container "HTTP Adapter" {
  technology "Spring Boot"
  description "REST API implementation"
}

db_adapter = container "Database Adapter" {
  technology "Spring Data"
  description "PostgreSQL implementation"
}

http_adapter -> domain_core "uses"
db_adapter -> domain_core "implements"
```

---

## Relationship Guidelines

### Synchronous Relationships

Use `->` for requests/responses requiring immediate feedback.

**Protocols:** HTTPS, HTTP/2, gRPC, REST API, GraphQL

**Example:**

```sruja
web -> api "HTTPS"
api -> database "PostgreSQL (JDBC)"
api -> external_service "REST API"
```

### Asynchronous Relationships

Use `->` with event labels for messaging.

**Labels:** "publishes events to", "subscribes to", "emits events"

**Example:**

```sruja
producer -> kafka "publishes events"
consumer -> kafka "consumes events"
worker -> queue "subscribes"
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
frontend -> api "HTTPS"
api -> database "reads/writes"
api -> message_queue "publishes events"
worker -> message_queue "consumes events"
```

---

## Anti-Patterns to Avoid

### God Component

Single container doing too many responsibilities.

**❌ Wrong:**

```sruja
god = container "Everything" {
  technology "Node.js"
  description "Auth, orders, payments, inventory, notifications"
}
```

**✅ Correct:**

```sruja
auth_service = container "Auth Service" { ... }
order_service = container "Order Service" { ... }
payment_service = container "Payment Service" { ... }
```

### Direct Database Access from Multiple Layers

Frontend, worker, API all accessing database directly.

**❌ Wrong:**

```sruja
frontend -> database "SQL"
worker -> database "SQL"
api -> database "SQL"
```

**✅ Correct:**

```sruja
frontend -> api "HTTPS"
worker -> api "REST API"
api -> database "SQL"
```

### Circular Dependencies

Service A → Service B → Service A

**❌ Wrong:**

```sruja
service_a -> service_b "calls"
service_b -> service_a "calls"
```

**✅ Correct:**

```sruja
service_a -> common_service "uses"
service_b -> common_service "uses"
```

### Tight Coupling

Components calling specific implementations directly.

**❌ Wrong:**

```sruja
service -> postgresql_direct "uses"
```

**✅ Correct:**

```sruja
service -> data_layer "uses"
data_layer -> postgresql "implements"
```

### Orphan Components

Components with no relationships.

**❌ Wrong:**

```sruja
orphan = container "No Relationships" {
  description "Unused component"
}
```

**✅ Correct:**

```sruja
active = container "Used Service" {
  description "Has clear purpose and connections"
}

api -> active "REST API"
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
architecture "E-Commerce" {
  user = person "Customer"

  system "Application" {
    web_frontend = container "Web Frontend" {
      technology "React"
      description "User interface"
    }

    api_gateway = container "API Gateway" {
      technology "Kong"
      description "Routing and auth"
    }

    user_service = container "User Service" {
      technology "Go"
      description "User management"
    }

    order_service = container "Order Service" {
      technology "Node.js"
      description "Order processing"
    }

    payment_service = container "Payment Service" {
      technology "Python"
      description "Payment processing"
    }

    inventory_service = container "Inventory Service" {
      technology "Java"
      description "Inventory management"
    }

    notification_service = container "Notification Service" {
      technology "Node.js"
      description "Email, SMS, push"
    }

    database = database "Database" {
      technology "PostgreSQL"
      description "Primary database"
    }

    cache = database "Cache" {
      technology "Redis"
      description "Caching layer"
    }

    event_store = queue "Event Store" {
      technology "Kafka"
      description "Event streaming"
    }
  }

  stripe = person "Stripe" {
    description "External payment service"
  }

  user -> web_frontend "HTTPS"
  web_frontend -> api_gateway "HTTPS"

  api_gateway -> user_service "REST API"
  api_gateway -> order_service "REST API"

  order_service -> payment_service "REST API"
  order_service -> inventory_service "REST API"
  order_service -> notification_service "REST API"

  user_service -> database "SQL"
  order_service -> database "SQL"
  payment_service -> database "SQL"

  user_service -> cache "Redis"
  order_service -> cache "Redis"

  payment_service -> stripe "REST API"
  payment_service -> event_store "publishes events"
  notification_service -> event_store "consumes events"
}
```

### Real-Time Analytics

```sruja
architecture "Analytics Platform" {
  system "Analytics" {
    collector = container "Data Collector" {
      technology "Python"
      description "Ingests events from sources"
    }

    stream_processor = container "Stream Processor" {
      technology "Kafka Streams"
      description "Real-time processing"
    }

    database = database "Time Series DB" {
      technology "InfluxDB"
      description "Stores metrics"
    }

    api = container "Analytics API" {
      technology "Go"
      description "Query interface"
    }

    dashboard = container "Dashboard" {
      technology "React"
      description "Visualization"
    }
  }

  collector -> stream_processor "publishes events"
  stream_processor -> database "writes metrics"
  api -> database "reads metrics"
  dashboard -> api "GraphQL"
}
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
- **Discord:** https://discord.gg/VNrvHPV5

---

**Version:** 1.0.0
**Last Updated:** 2025-02-07
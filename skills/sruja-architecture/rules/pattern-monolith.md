# pattern-monolith

## Why It Matters

A monolithic architecture is a single deployable unit containing all functionality. It's ideal for small teams, rapid development, and simple domains. Understanding when to use (or avoid) monoliths is critical for architectural success.

## When to Apply

Choose a monolithic architecture when:

- Team size is small (1-10 developers)
- Domain complexity is low to moderate
- Time-to-market is a priority
- You're building an MVP or prototype
- Scaling requirements are predictable
- You prefer simplicity over flexibility

## Correct Approach

### Example 1: Modular Monolith

```sruja
architecture "Project Management System" {
  system "Application" {
    # API Gateway / Entry Point
    api_gateway = container "API Gateway" {
      technology "Node.js + Express"
      description "Single entry point with routing and authentication"
    }

    # Business Modules
    user_module = container "User Module" {
      technology "Node.js"
      description "User management and authentication"
    }

    project_module = container "Project Module" {
      technology "Node.js"
      description "Project and workspace management"
    }

    task_module = container "Task Module" {
      technology "Node.js"
      description "Task creation and tracking"
    }

    # Shared Resources
    database = datastore "Primary Database" {
      technology "PostgreSQL"
      description "Central database for all modules"
    }

    cache = datastore "Cache Layer" {
      technology "Redis"
      description "Shared cache for performance"
    }
  }

  # Clear module boundaries internally
  api_gateway -> user_module "gRPC (internal)"
  api_gateway -> project_module "gRPC (internal)"
  api_gateway -> task_module "gRPC (internal)"

  user_module -> database "SQL"
  project_module -> database "SQL"
  task_module -> database "SQL"

  # All share the same cache
  user_module -> cache "Redis"
  project_module -> cache "Redis"
  task_module -> cache "Redis"
}
```

### Example 2: Layered Monolith

```sruja
architecture "Web Application" {
  system "Monolith" {
    # Presentation Layer
    web_frontend = container "Web Frontend" {
      technology "React + Vite"
      description "User interface and presentation"
    }

    api_layer = container "API Layer" {
      technology "Express.js"
      description "HTTP API endpoints and routing"
    }

    # Application Layer
    user_service = container "User Service" {
      technology "Node.js"
      description "User-related business logic"
    }

    order_service = container "Order Service" {
      technology "Node.js"
      description "Order processing business logic"
    }

    # Infrastructure Layer
    database = datastore "Database" {
      technology "PostgreSQL"
      description "Data persistence"
    }

    message_queue = datastore "Message Queue" {
      technology "RabbitMQ"
      description "Internal messaging"
    }
  }

  web_frontend -> api_layer "HTTPS"
  api_layer -> user_service "HTTP"
  api_layer -> order_service "HTTP"

  user_service -> database "SQL"
  order_service -> database "SQL"

  user_service -> message_queue "publishes events"
  order_service -> message_queue "consumes events"
}
```

## Incorrect Approach

```sruja
# ❌ Everything in one container
app = container "Everything" {
  technology "Node.js"
  description "Frontend, API, database, all in one"
}
```

## Common Mistakes

1. **Lack of Internal Boundaries**
   - ❌ No separation between modules or layers
   - ✅ Define clear module boundaries even within monolith

2. **Shared Database Anti-Pattern**
   - ❌ Multiple monoliths sharing same database
   - ✅ Each system has its own database or schema

3. **Ignoring Modularity**
   - ❌ Tightly coupled code with no clear boundaries
   - ✅ Modular design that can be extracted later

4. **Premature Scaling Efforts**
   - ❌ Building microservices architecture for small team
   - ✅ Start with monolith, split when needed

## Advantages of Monolith

✅ **Simpler Development**

- Easier to understand codebase
- No inter-service communication complexity
- Shared libraries and utilities

✅ **Easier Deployment**

- Single deployable unit
- Simple CI/CD pipeline
- No distributed system issues

✅ **Faster Development**

- No need for service discovery
- No distributed transactions
- Simpler debugging and monitoring

✅ **Lower Overhead**

- Less infrastructure
- Lower operational cost
- Fewer moving parts

## Disadvantages of Monolith

❌ **Scalability Limitations**

- Scale entire application, not individual parts
- Harder to optimize performance hotspots

❌ **Technology Lock-in**

- Must use single technology stack
- Hard to adopt new technologies

❌ **Maintenance Challenges**

- Harder to maintain as codebase grows
- Potential for monolithic code

❌ **Team Scalability**

- More developers → more conflicts
- Harder to parallelize work

## When to Migrate to Microservices

Consider migrating from monolith to microservices when:

- Team grows beyond 10-15 developers
- Multiple teams working independently
- Different parts have different scaling needs
- Different parts need different technologies
- Deployment frequency varies by feature
- Reliability isolation is needed

## Additional Context

The monolith vs microservices decision is a classic architectural trade-off. See `tradeoff-monolith-vs-microservices` for detailed comparison.

Related rules:

- `pattern-microservices` - When to use distributed architecture
- `principle-separation` - Even monoliths need separation of concerns
- `anti-god-component` - Avoid monolithic components within the monolith

## References

- Building Microservices by Sam Newman
- Monolith to Microservices Evolution
- Modular Monolith Architecture Pattern
- Domain-Driven Design: Bounded Contexts
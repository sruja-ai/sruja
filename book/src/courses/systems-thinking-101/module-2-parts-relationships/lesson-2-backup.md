# Lesson 2: Sruja Elements

## Learning Goals

- Learn Sruja's core element types
- Understand when to use each type
- Model parts with proper Sruja syntax

## Sruja Element Types

Sruja provides element types that map to the C4 model:

| Element Type | Purpose                             | C4 Level |
| ------------ | ----------------------------------- | -------- |
| `person`     | Users, stakeholders                 | Level 1  |
| `system`     | Software systems, services          | Level 2  |
| `container`  | Applications, databases, queues     | Level 3  |
| `component`  | Modules, services within containers | Level 4  |

## Person

Use `person` to represent humans who interact with your system.

### Basic Syntax

```sruja
person = kind "Person"

User = person "End User"
Admin = person "Administrator"
Support = person "Customer Support"
```

### With Details

```sruja
Customer = person "Customer" {
  description "End users who purchase products"
  metadata {
    type ["external"]
    priority "high"
  }
}
```

### When to Use

- End users (customers, employees)
- Stakeholders (managers, business owners)
- Support teams
- External users (API consumers)

### Examples

```sruja
// Internal users
Developer = person "Developer"
ProductManager = person "Product Manager"

// External users
APIConsumer = person "API Consumer"
Partner = person "Business Partner"
```

## System

Use `system` to represent standalone software systems.

### Basic Syntax

```sruja
system = kind "System"

Shop = system "E-Commerce Platform"
APIGateway = system "API Gateway"
PaymentGateway = system "Payment Gateway"
```

### With Details

```sruja
ECommerce = system "E-Commerce Platform" {
  description "Platform for buying and selling products"
  metadata {
    version "2.0"
    team ["platform-team"]
  }
  slo {
    availability {
      target "99.9%"
    }
  }
}
```

### When to Use

- Main application you're building
- External systems you depend on
- Third-party services
- Separate software products

### Examples

```sruja
// Your systems
Platform = system "Sruja Platform"
Dashboard = system "Analytics Dashboard"

// External systems
Stripe = system "Stripe"
AWS = system "Amazon Web Services"
GitHub = system "GitHub"
```

## Container

Use `container` to represent applications, databases, and other deployable units within a system.

### Basic Syntax

```sruja
container = kind "Container"

WebApp = container "Web Application"
API = container "API Service"
DB = database "Database"
Queue = queue "Message Queue"
```

### With Details

```sruja
WebApp = container "Web Application" {
  technology "React"
  description "Single-page application"
  version "3.1.0"
  tags ["frontend", "typescript"]
  scale {
    min 2
    max 10
  }
  slo {
    latency {
      p95 "200ms"
    }
  }
}
```

### When to Use

- Web applications (React, Vue, Angular)
- API services (Node.js, Rust, Python)
- Databases (PostgreSQL, MongoDB)
- Message queues (Kafka, RabbitMQ)
- Caches (Redis, Memcached)

### Database vs Datastore

```sruja
// Database (relational)
UserDB = database "User Database" {
  technology "PostgreSQL"
}

// Datastore (document)
CacheDB = datastore "Redis Cache" {
  technology "Redis"
}
```

## Component

Use `component` to represent modules or services within a container.

### Basic Syntax

```sruja
component = kind "Component"

AuthService = component "Authentication Service"
UserService = component "User Service"
OrderController = component "Order Controller"
```

### With Details

```sruja
AuthService = component "Authentication Service" {
  technology "Rust"
  description "Handles user login and registration"
  scale {
    min 1
    max 5
  }
}
```

### When to Use

- Service modules within a monolith
- Controllers in MVC architecture
- Domain services
- Library components
- Utilities and helpers

## Complete Example

```sruja
import { * } from 'sruja.ai/stdlib'

// People (Level 1)
Customer = person "Customer"
Administrator = person "Administrator"

// Systems (Level 2)
ECommerce = system "E-Commerce Platform"
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Containers (Level 3)
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
  }

  API = container "API Service" {
    technology "Node.js"
    scale {
      min 3
      max 10
    }

    // Components (Level 4)
    ProductService = component "Product Service"
    CartService = component "Cart Service"
    OrderService = component "Order Service"
    PaymentService = component "Payment Service"
  }

  Database = database "PostgreSQL" {
    technology "PostgreSQL 14"
  }

  Cache = queue "Redis" {
    technology "Redis 7"
  }
}
```

## Naming Conventions

### Element IDs

Use PascalCase for element IDs:

```sruja
Good:
Customer = person "Customer"
WebApp = container "Web App"

Bad:
customer = person "Customer"  // SnakeCase
WEBAPP = container "Web App"  // ALL CAPS
```

### Display Names

Use descriptive names with proper capitalization:

```sruja
Good:
User = person "End User"
API = container "API Service"

Bad:
User = person "user"  // Lowercase
API = container "api"  // Lowercase
```

### Consistency

Be consistent across your architecture:

```sruja
Good:
UserService = component "User Service"
OrderService = component "Order Service"
PaymentService = component "Payment Service"

Bad:
UserService = component "User Service"
Order = component "Order"
Payment = component "Payment Service"
```

## Adding Metadata

Use metadata to add context:

```sruja
API = container "API Service" {
  technology "Rust"
  version "2.0.1"
  tags ["backend", "api"]
  metadata {
    team ["platform-team"]
    repository "github.com/company/api"
  }
}
```

## Exercise

Model the following using Sruja elements:

> "A blog platform has readers and writers. The main system contains a web frontend (Next.js), API backend (Python/FastAPI), and database (PostgreSQL). The API has user management, post management, and comment services."

**Create elements for:**

- People
- Systems
- Containers
- Components

## Key Takeaways

1. **Four core element types**: person, system, container, component
2. **Match type to C4 level**: Each has a specific purpose
3. **Add details**: technology, description, tags, scale, SLOs
4. **Follow naming conventions**: Be consistent
5. **Use metadata**: Add context for your team

## Next Lesson

In [Lesson 3](lesson-3.md), you'll learn how to connect parts with relationships.

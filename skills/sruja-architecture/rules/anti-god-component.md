# anti-god-component

## Why It Matters

A "god component" is a single component that handles too many responsibilities. This violates the single responsibility principle, makes code hard to maintain, test, and understand, and creates a bottleneck for scaling.

## When to Apply

Check for god components when:

- Reviewing existing architectures
- Refactoring monolithic systems
- Identifying performance bottlenecks
- Planning microservice extraction
- Designing new systems

## Correct Approach

### Example 1: Split by Responsibility

```sruja
# ❌ Anti-pattern: God Component
architecture "Bad E-Commerce" {
  god_container = container "Everything" {
    technology "Node.js"
    description "User auth, orders, payments, inventory, notifications, all in one"
  }

  god_container -> database "SQL"
}

# ✅ Correct: Split into focused containers
architecture "Good E-Commerce" {
  system "E-Commerce" {
    api_gateway = container "API Gateway" {
      technology "Kong"
      description "Routing, authentication, rate limiting"
    }

    user_service = container "User Service" {
      technology "Node.js"
      description "User management and authentication"
    }

    order_service = container "Order Service" {
      technology "Go"
      description "Order processing and management"
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
      description "Email, SMS, push notifications"
    }

    database = datastore "Database" {
      technology "PostgreSQL"
      description "Data persistence"
    }
  }

  api_gateway -> user_service "REST API"
  api_gateway -> order_service "REST API"

  user_service -> database "SQL"
  order_service -> database "SQL"
  payment_service -> database "SQL"
  inventory_service -> database "SQL"
}
```

### Example 2: Split by Layer

```sruja
# ❌ Anti-pattern: Everything in one layer
architecture "Bad Web App" {
  web_container = container "Web App" {
    technology "Node.js"
    description "UI, API, business logic, data access, caching"
  }
}

# ✅ Correct: Separate concerns into layers
architecture "Good Web App" {
  system "Web Application" {
    # Presentation Layer
    web_frontend = container "Web Frontend" {
      technology "React"
      description "User interface"
    }

    api_gateway = container "API Gateway" {
      technology "Express.js"
      description "HTTP API endpoints"
    }

    # Application Layer
    business_service = container "Business Service" {
      technology "Node.js"
      description "Business logic and use cases"
    }

    # Infrastructure Layer
    data_service = container "Data Service" {
      technology "Node.js"
      description "Data access and caching"
    }

    cache = datastore "Cache" {
      technology "Redis"
      description "Caching layer"
    }

    database = datastore "Database" {
      technology "PostgreSQL"
      description "Data persistence"
    }
  }

  web_frontend -> api_gateway "HTTPS"
  api_gateway -> business_service "HTTP"
  business_service -> data_service "HTTP"
  data_service -> cache "Redis"
  data_service -> database "SQL"
}
```

## Signs of God Components

1. **Too Many Responsibilities**
   - Handles multiple unrelated concerns
   - "Does everything" in description
   - Massive code file or module

2. **High Coupling**
   - Depends on many other components
   - Has many incoming and outgoing relationships
   - Central point of failure

3. **Hard to Test**
   - Requires complex setup for unit tests
   - Tests are slow and flaky
   - Hard to mock dependencies

4. **Performance Bottleneck**
   - All requests go through single component
   - Scaling requires scaling entire component
   - Can't optimize individual functions

5. **Frequent Changes**
   - Changes for unrelated features
   - High merge conflict rate
   - Many developers working on same code

## Common God Component Patterns

### 1. Controller-As-God

```sruja
# ❌ Controller doing everything
controller = container "Main Controller" {
  technology "Node.js"
  description "Handles all requests, business logic, data access, caching"
}
```

### 2. Service-As-God

```sruja
# ❌ Service handling all business logic
service = container "Business Service" {
  technology "Java"
  description "Users, orders, payments, inventory, all business logic"
}
```

### 3. API-As-God

```sruja
# ❌ API gateway with too much logic
gateway = container "API Gateway" {
  technology "Node.js"
  description "Routing, auth, rate limiting, validation, transformation, enrichment"
}
```

## How to Fix God Components

### 1. Split by Responsibility

Identify distinct responsibilities and create separate containers:

- Authentication → User Service
- Orders → Order Service
- Payments → Payment Service
- Inventory → Inventory Service
- Notifications → Notification Service

### 2. Split by Layer

Separate concerns into architectural layers:

- Presentation → Frontend, API Gateway
- Application → Business Services, Use Cases
- Infrastructure → Data Access, Caching, External APIs

### 3. Split by Domain

Use bounded contexts to split:

- User Management Context
- Order Management Context
- Payment Context
- Inventory Context

### 4. Extract Common Functionality

Move shared code to separate components:

- Logging → Logging Service
- Monitoring → Monitoring Service
- Caching → Cache Layer
- Event Bus → Message Queue

## Additional Context

God components are a major anti-pattern that leads to:

- Unmaintainable code
- Difficult testing
- Performance issues
- Team bottlenecks
- High technical debt

Related rules:

- `principle-separation` - Separation of concerns
- `anti-tight-coupling` - Reducing component dependencies
- `principle-cohesion-coupling` - Balancing cohesion and coupling
- `pattern-microservices` - Splitting into services

## References

- Single Responsibility Principle (SOLID)
- Code Smell: God Object
- Refactoring: Extract Class
- Microservices: Extract Service Pattern
- Domain-Driven Design: Bounded Contexts

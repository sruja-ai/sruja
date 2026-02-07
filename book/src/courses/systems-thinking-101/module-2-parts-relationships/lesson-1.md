# Lesson 1: Identifying Parts

## Learning Goals

- Learn how to identify the key parts of a system
- Understand the different types of components
- Practice identifying parts from requirements

## What Are Parts?

In systems thinking, "parts" are the distinct components that make up a system. In software architecture, these are the building blocks: users, systems, services, databases, queues, and more.

## The C4 Model Hierarchy

Sruja follows the C4 model, which provides a clear hierarchy of parts:

```
Level 1: Person (Users, stakeholders)
  ↓
Level 2: System (Software systems)
  ↓
Level 3: Container (Applications, databases, services)
  ↓
Level 4: Component (Modules, classes, libraries)
```

## Identifying Parts: Step by Step

### Step 1: Start with People

Who interacts with the system?

**Example Requirements:**

> "Customers can browse products, add to cart, and checkout. Administrators can manage inventory and view reports."

**People identified:**

- Customer
- Administrator

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"
Administrator = person "Administrator"
```

### Step 2: Identify Systems

What software systems are involved?

**From requirements:**

- E-commerce platform (the main system)
- Payment gateway (external)
- Email service (external)

```sruja
ECommerce = system "E-Commerce Platform"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
```

### Step 3: Break Down Systems into Containers

What applications, services, and databases make up each system?

**For E-Commerce:**

- Web Application (React frontend)
- API Service (Node.js backend)
- Database (PostgreSQL)
- Cache (Redis)

```sruja
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
  Cache = queue "Redis Cache"
}
```

### Step 4: Break Down Containers into Components (Optional)

What modules or components make up each container?

**For API Service:**

- Product Service
- Cart Service
- Order Service
- Payment Service

```sruja
API = container "API Service" {
  ProductService = component "Product Service"
  CartService = component "Cart Service"
  OrderService = component "Order Service"
  PaymentService = component "Payment Service"
}
```

## Common Patterns

### Pattern 1: Frontend-Backend-Database

```sruja
App = system "Application" {
  Frontend = container "Web App" {
    technology "React"
  }
  Backend = container "API Service" {
    technology "Node.js"
  }
  Database = database "Database" {
    technology "PostgreSQL"
  }
}
```

### Pattern 2: Microservices

```sruja
App = system "Microservice Application" {
  APIGateway = container "API Gateway"
  UserService = container "User Service"
  OrderService = container "Order Service"
  NotificationService = container "Notification Service"
}
```

### Pattern 3: Event-Driven

```sruja
App = system "Event-Driven System" {
  Producer = container "Event Producer"
  Consumer = container "Event Consumer"
  MessageQueue = queue "Kafka Cluster"
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Oversimplification

**Don't:**

```sruja
// Too simple, loses important detail
App = system "The App"
```

**Do:**

```sruja
// Shows structure
App = system "The App" {
  Frontend = container "Frontend"
  Backend = container "Backend"
  Database = database "Database"
}
```

### Anti-Pattern 2: Over-Engineering

**Don't:**

```sruja
// Too detailed, hard to understand
App = system "The App" {
  Frontend = container "Frontend" {
    Header = component "Header"
    Body = component "Body"
    Footer = component "Footer"
  }
}
```

**Do:**

```sruja
// Right level of detail
App = system "The App" {
  Frontend = container "Frontend"
  Backend = container "Backend"
}
```

### Anti-Pattern 3: Mixing Levels

**Don't:**

```sruja
// Inconsistent level of detail
App = system "The App" {
  Frontend = container "Frontend"
  UserService = component "User Service"  // Skips container level
  Database = database "Database"
}
```

**Do:**

```sruja
// Consistent hierarchy
App = system "The App" {
  Frontend = container "Frontend"
  Backend = container "Backend" {
    UserService = component "User Service"
  }
  Database = database "Database"
}
```

## Exercise: Identify Parts

Read these requirements and identify the parts:

> "A project management tool allows team members to create tasks, assign them to others, and track progress. Managers can view reports and approve tasks. The system sends email notifications for task assignments and due dates. Task data is stored in a database. The system integrates with Slack for notifications."

**Identify:**

1. People: ******\_******
2. Systems: ******\_******
3. Containers: ******\_******

## Key Takeaways

1. **Start with people**: Who uses the system?
2. **Use the C4 hierarchy**: Person → System → Container → Component
3. **Match detail to audience**: Not every diagram needs components
4. **Avoid anti-patterns**: Don't oversimplify or over-engineer
5. **Be consistent**: Use the same level of detail throughout

## Next Lesson

In [Lesson 2](lesson-2.md), you'll learn how to use Sruja's element types to model the parts you've identified.

---
title: "Lesson 2: Sruja Elements"
weight: 2
summary: "Using person, system, container, and component"
time: "5 min"
---

# Your Toolkit: Sruja's Element Types

Now that you know how to identify parts, let's talk about how to actually define them in Sruja. Think of element types as your vocabulary—the building blocks you'll use to describe any software architecture.

The beauty of Sruja is that it gives you exactly four element types. Not ten, not twenty—just four. This might feel limiting at first, but I promise you'll appreciate the simplicity. With these four types, you can model any software system from a simple blog to a distributed microservices platform.

Let's dive in.

## Learning Goals

By the end of this lesson, you'll be able to:

- Confidently use all four Sruja element types
- Know exactly when to use each type (and when not to)
- Write clean, consistent element definitions
- Add meaningful details that make your diagrams useful

## The Four Building Blocks

Here's the complete toolkit. You'll use these four types over and over again:

| Element Type | Purpose | When I Use It |
|--------------|---------|---------------|
| `person` | Humans who interact with the system | Every architecture starts here |
| `system` | Standalone software systems | The main systems you build and depend on |
| `container` | Deployable units within systems | Applications, databases, queues—the things you actually deploy |
| `component` | Modules within containers | Only when you need to show internal architecture |

Notice how each maps to a level in the C4 hierarchy? Person is Level 1, System is Level 2, Container is Level 3, Component is Level 4. They work together in a hierarchy, not as independent pieces.

## Person: It Always Starts With People

Every system exists to serve humans. That's why `person` is your starting point. Don't skip it—even if it feels obvious. Having people in your diagram grounds everything in reality.

### The Basics

Here's how simple it is:

```sruja
Customer = person "Customer"
Admin = person "Administrator"
Support = person "Customer Support"
```

That's it. You define an ID (like `Customer`) and a display name (like `"Customer"`). Simple, readable, and immediately clear.

### Adding Context

Sometimes a name isn't enough. You might want to add more details:

```sruja
Customer = person "Customer" {
  description "End users who purchase products"
  metadata {
    type ["external"]
    priority "high"
  }
}
```

I use descriptions when the name alone might be ambiguous. For example, "Admin" could mean a system administrator, a content admin, or a superuser. Adding a description removes that ambiguity.

### Real-World Examples

Here are some people I've modeled in different projects:

```sruja
// partial
// Internal users
Developer = person "Developer"
ProductManager = person "Product Manager"
DataAnalyst = person "Data Analyst"

// External users
APIConsumer = person "API Consumer"
Partner = person "Business Partner"
Vendor = person "Vendor"

// Support roles
SupportAgent = person "Customer Support"
SalesRep = person "Sales Representative"
```

One thing I've learned: don't overthink this. If someone interacts with your system, model them as a person. You don't need to capture every single role—just the ones that matter for your diagram's purpose.

## System: The Big Picture

Systems are the major software units. Think of them as the "big boxes" in your architecture—the main applications, third-party services, and platforms you depend on.

### Defining Systems

Here's the basic syntax:

```sruja
ECommerce = system "E-Commerce Platform"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
```

### Adding Details That Matter

When I'm documenting systems for a team, I often add more context:

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

I love that Sruja lets you add SLOs (Service Level Objectives) directly to systems. This makes your architecture diagrams not just visual—they become living documentation that tells the story of what you've promised to deliver.

### Your Systems vs. External Systems

One distinction I always make:

```sruja
// partial
// Systems you own and maintain
Platform = system "Sruja Platform"
Dashboard = system "Analytics Dashboard"

// Systems you depend on (third-party)
Stripe = system "Stripe"
AWS = system "Amazon Web Services"
GitHub = system "GitHub"
```

I usually tag external systems with `metadata { tags ["external"] }` so anyone looking at the diagram immediately understands which systems you control and which you don't. This is crucial for understanding dependencies and risk.

## Container: What You Actually Deploy

This is where things get practical. Containers are the deployable units—web apps, APIs, databases, caches. These are the things you actually deploy to production.

### Understanding Containers

The name "container" can be confusing because people think of Docker containers. In C4 and Sruja, "container" just means "a deployable unit." It could be:
- A web application
- An API service
- A database
- A message queue
- A cache

### The Different Container Types

Sruja gives you specific types for common containers:

```sruja
// partial
// Regular containers
WebApp = container "Web Application"
API = container "API Service"

// Databases and datastores
UserDB = database "User Database"
CacheDB = datastore "Redis Cache"

// Message queues
Kafka = queue "Kafka Cluster"
RabbitMQ = queue "RabbitMQ"
```

I use `database` for relational databases (PostgreSQL, MySQL) and `datastore` for non-relational storage (Redis, MongoDB). The distinction matters because the behavior and considerations are different.

### Adding Rich Details

When documenting containers for developers, I add a lot of useful information:

```sruja
// partial
API = container "API Service" {
  technology "Node.js"
  description "RESTful API handling all business logic"
  version "3.1.0"
  tags ["backend", "typescript"]
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

This tells developers everything they need to know: what technology it uses, what it does, how it scales, and what performance targets it's supposed to meet. This is way more useful than a simple box diagram.

## Component: Looking Inside (When You Need To)

Components are optional. I'll say that again: **components are optional**. Only use them when your audience needs to see inside a container.

### When to Use Components

Add components when:
- You're documenting for developers who need implementation details
- A container is complex (more than 5-6 logical parts)
- Different teams own different parts of the same container
- You need to show internal architecture for a design review

Skip components when:
- You're talking to business stakeholders
- The container is simple enough to explain at a high level
- The internal structure is still evolving

### Defining Components

Here's how you model them:

```sruja
// partial
API = container "API Service" {
  AuthService = component "Authentication Service"
  ProductService = component "Product Service"
  CartService = component "Cart Service"
  OrderService = component "Order Service"
}
```

### Adding Details to Components

You can add the same details to components that you add to containers:

```sruja
// partial
AuthService = component "Authentication Service" {
  technology "Rust"
  description "Handles user login, registration, and JWT validation"
  scale {
    min 1
    max 5
  }
  slo {
    latency {
      p99 "100ms"
    }
  }
}
```

## Putting It All Together

Let me show you a complete example that uses all four element types. This is a typical e-commerce platform:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// Level 1: People
Customer = person "Customer" {
  description "End users purchasing products"
}
Administrator = person "Administrator" {
  description "Staff managing inventory and orders"
}

// Level 2: Systems
ECommerce = system "E-Commerce Platform" {
  description "Main platform for online sales"
}
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}
EmailService = system "Email Service" {
  metadata {
    tags ["external"]
  }
}

// Level 3: Containers
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
    description "Single-page application for customers"
  }

  API = container "API Service" {
    technology "Node.js"
    description "RESTful API handling business logic"
    scale {
      min 3
      max 10
    }

    // Level 4: Components (inside API)
    ProductService = component "Product Service"
    CartService = component "Cart Service"
    OrderService = component "Order Service"
    PaymentService = component "Payment Service"
  }

  Database = database "PostgreSQL" {
    technology "PostgreSQL 14"
    description "Primary data store"
  }

  Cache = database "Redis Cache" {
    technology "Redis 7"
    description "Caching layer for performance"
  }
}
```

See how everything nests naturally? People at the top, systems next, containers within systems, and components within containers. This hierarchy makes it easy to understand the architecture at any level of detail.

## Naming Conventions That Work

After years of modeling systems, I've settled on some naming conventions that keep things consistent and readable.

### Element IDs: Use PascalCase

```sruja
// partial
// Good
Customer = person "Customer"
WebApp = container "Web App"
UserService = component "User Service"

// Avoid these
customer = person "Customer"      // SnakeCase is harder to read
WEBAPP = container "Web App"      // ALL CAPS feels like shouting
webapp = container "Web App"      // Lowercase is inconsistent
```

### Display Names: Be Descriptive

```sruja
// partial
// Good
User = person "End User"
API = container "API Service"

// Avoid these
User = person "user"              // Lowercase looks unprofessional
API = container "api"             // Lowercase looks like a typo
```

### Consistency Within a Diagram

This is the one that trips people up most often. If you name one thing a "Service," call all similar things "Service":

```sruja
// partial
// Good: Consistent naming
UserService = component "User Service"
OrderService = component "Order Service"
PaymentService = component "Payment Service"

// Bad: Inconsistent naming
UserService = component "User Service"
Order = component "Order"              // Should be "Order Service"
Payment = component "Payment Service"  // Inconsistent with "Order"
```

I create a style guide for each project that lists the naming conventions we're using. It sounds tedious, but it saves so much time and confusion in the long run.

## Using Metadata Effectively

Metadata is where you add context that makes your diagrams actually useful. Don't add metadata just for the sake of it—add metadata that helps your audience understand what they're looking at.

```sruja
// partial
API = container "API Service" {
  technology "Rust"
  version "2.0.1"
  tags ["backend", "api"]
  metadata {
    team ["platform-team"]
    repository "github.com/company/api"
    documentation "docs.company.com/api"
  }
}
```

I include team ownership information because it helps people know who to talk to when they have questions. I include repository links because it connects the architecture to the actual code. These details transform a static diagram into a living piece of documentation.

## What to Remember

Sruja gives you four element types—person, system, container, component—and they're all you need. The key is knowing when to use each one:

- **Person**: Always include these. Every system serves humans.
- **System**: Include the main systems you're building and the ones you depend on.
- **Container**: Include the deployable units within your systems.
- **Component**: Only include these when your audience needs implementation details.

Add metadata that provides context. Use naming conventions consistently. Keep it simple and your diagrams will be clear and useful.

Everything else flows from these principles.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding:

### Question 1

You're modeling a blog platform with these requirements:

> "The platform has readers and writers. The main system contains a web frontend (Next.js), API backend (Python/FastAPI), and database (PostgreSQL). The API has user management, post management, and comment services."

At the **container** level, which elements should you include?

**A)** Reader, Writer, User Management, Post Management, Comment Management
**B)** Web Frontend, API Backend, Database
**C)** User Management, Post Management, Comment Management
**D)** Blog Platform, Next.js, FastAPI, PostgreSQL

<details>
<summary>Click to see the answer</summary>

**Answer: B) Web Frontend, API Backend, Database**

Containers are the **deployable units** within a system. The web frontend, API backend, and database are all things you'd deploy. Readers and writers are people, not containers. User management, post management, and comment management are components (internal modules within the API), not containers. Next.js, FastAPI, and PostgreSQL are technologies, not elements.

The key insight: containers = what you actually deploy. Everything else is either higher-level (systems, people) or lower-level (components).
</details>

---

### Question 2

When should you model components within a container?

**A)** Always—components provide the most detailed view
**B)** Only when using microservices architecture
**C)** When your audience needs to understand internal architecture or when a container is complex
**D)** Never—containers provide sufficient detail for all use cases

<details>
<summary>Click to see the answer</summary>

**Answer: C) When your audience needs to understand internal architecture or when a container is complex**

Components are an **optional** level of detail. You should include them when:

- You're documenting for developers who need implementation details
- A container has grown complex (more than 5-6 logical parts)
- Different teams own different parts of the same container
- You're doing a detailed design review

You should **skip** components when:

- You're presenting to stakeholders who don't need implementation details
- The container is simple enough to explain at a high level
- The internal structure is still changing

The golden rule: match the level of detail to your audience's needs. Don't show components to business stakeholders. Do show components to developers who need to understand how the system is built.
</details>

---

## What's Next?

Now you know how to define all the parts of your system. But parts alone don't tell the whole story. Systems are defined by how their parts interact and connect.

In the next lesson, you'll learn about relationships—how to model the connections between your elements. You'll learn to write clear, meaningful labels that describe exactly how parts communicate with each other.


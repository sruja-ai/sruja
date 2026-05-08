---
title: "Lesson 4: Bringing Order to Chaos: Hierarchy and Nesting"
weight: 4
summary: "Organizing system parts with clear structure"
time: "5 min"
---

# Bringing Order to Chaos: Hierarchy and Nesting

Imagine walking into a house where everything is dumped in one giant room—furniture, kitchen appliances, clothes, tools, books. You could find what you're looking for eventually, but it would be frustrating and inefficient. That's what an unorganized architecture feels like.

Hierarchy and nesting are your tools for bringing order to chaos. They help you organize your system's parts in a way that makes sense, is easy to understand, and scales as your system grows.

In this lesson, you'll learn how to structure your systems effectively using the C4 model's hierarchy. You'll discover when to nest, when to keep things flat, and how to create diagrams that tell a clear story at any level of detail.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand the C4 model's four-level hierarchy
- Know when to nest components and when to keep them separate
- Reference nested elements using dot notation
- Create systems that are consistent and easy to navigate
- Choose the right level of detail for your audience

## The C4 Hierarchy: Your Foundation

Let's revisit the C4 model's hierarchy because it's the foundation for everything we'll discuss. Think of it like building a city:

```
People (Level 1)
  ↓ The citizens who live there
  
Systems (Level 2)
  ↓ The buildings and facilities
  
Containers (Level 3)
  ↓ The rooms and departments within buildings
  
Components (Level 4)
  ↓ The furniture and equipment within rooms
```

Each level contains the one below it. People interact with systems, systems contain containers, containers contain components. This nested structure is what makes architectures clear and navigable.

I've found this metaphor really helps people understand why we nest things. You don't put a couch (component) directly in the middle of the street (system). It belongs in a living room (container), which is inside a house (system). The same principle applies to software architecture.

## Nesting Containers in Systems

Systems contain containers. This is where most of the action happens.

### The Basic Pattern

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
  }

  API = container "API Service" {
    technology "Node.js"
  }

  Database = database "PostgreSQL" {
    technology "PostgreSQL 14"
  }

  Cache = database "Redis Cache" {
    technology "Redis 7"
  }
}
```

### Why Nesting Matters

When I first started modeling systems, I sometimes put everything at the same level—all containers, no systems. It seemed simpler, but it created confusion:

- **Who owns what?** Without systems, ownership is unclear
- **What belongs together?** Related containers end up scattered
- **How do I reference things?** Dot notation becomes meaningless

Nesting containers in systems solves all these problems. It's clear what belongs together, who owns it, and how to reference it.

### A Real-World Example

Let me show you a more realistic example from a project I worked on—a healthcare platform:

```sruja
HealthcarePlatform = system "Healthcare Platform" {
  PatientPortal = container "Patient Portal" {
    technology "React"
    description "Web app for patients to manage appointments"
  }

  DoctorDashboard = container "Doctor Dashboard" {
    technology "Vue.js"
    description "Web app for doctors to view patient records"
  }

  API = container "API Gateway" {
    technology "Kong"
    description "Routes requests to microservices"
  }

  EHR = container "EHR System" {
    technology "Java"
    description "Electronic health records system"
  }
}
```

See how each container has a clear purpose? The system groups them together and tells you they're all part of the same platform. Anyone looking at this diagram immediately understands the big picture.

## Nesting Components in Containers

This is where you need to be careful. Components are powerful, but only when used appropriately.

### The Decision Framework

I use this simple framework to decide whether to add components:

**Add components when:**
- You're documenting for developers who need implementation details
- A container is complex (more than 5-6 logical parts)
- Different teams own different parts of the same container
- You need to show internal architecture for a design review

**Skip components when:**
- You're presenting to stakeholders who don't need technical details
- The container is simple enough to explain at a high level
- The internal structure is still evolving and changing

### When to Nest Components

Let me show you an example where components make sense:

```sruja
// partial
API = container "API Service" {
  technology "Node.js"
  description "RESTful API handling all business logic"

  // Authentication and authorization
  AuthService = component "Authentication Service" {
    technology "Node.js"
    description "Handles login, registration, JWT tokens"
  }

  // Domain services
  UserService = component "User Service" {
    description "User profile and preferences"
  }
  
  ProductService = component "Product Service" {
    description "Product catalog and inventory"
  }
  
  CartService = component "Cart Service" {
    description "Shopping cart and checkout"
  }
  
  OrderService = component "Order Service" {
    description "Order processing and fulfillment"
  }

  // Integration services
  PaymentService = component "Payment Service" {
    description "Payment gateway integration"
  }
  
  NotificationService = component "Notification Service" {
    description "Email and SMS notifications"
  }
}
```

This makes sense because:
- The API is complex with multiple distinct services
- Different teams might own different services
- Developers need to understand the internal architecture

### When to Keep It Simple

Now, here's an example where I would **not** use components:

```sruja
// partial
SimpleAPI = container "Simple API" {
  technology "Node.js"
  description "Basic CRUD API for a small application"
}
```

Why no components? Because:
- It's simple enough to explain at the container level
- The audience (stakeholders) doesn't need implementation details
- The internal structure might change as we iterate

The key insight: **match the level of detail to your audience's needs**.

## Referencing Nested Elements

Once you have nested elements, you need to know how to reference them. Sruja uses dot notation, which is intuitive once you get the hang of it.

### The Reference Patterns

```sruja
// partial
// Level 1 to Level 2 (Person to System)
Customer -> ECommerce "Uses platform"

// Level 2 to Level 3 (System to Container)
Customer -> ECommerce.PatientPortal "Manages appointments"
Doctor -> ECommerce.DoctorDashboard "Views patient records"

// Level 3 to Level 3 (Container to Container)
ECommerce.PatientPortal -> ECommerce.API "Submits requests"
ECommerce.API -> ECommerce.EHR "Retrieves patient data"

// Level 3 to Level 4 (Container to Component)
ECommerce.API -> ECommerce.API.AuthService "Validates user"

// Level 4 to Level 4 (Component to Component)
ECommerce.API.AuthService -> ECommerce.API.UserService "Fetches user profile"
```

### Cross-System References

What if you need to reference something inside another system? Dot notation still works:

```sruja
// partial
// Reference a service inside an external system
ECommerce.API.PaymentService -> Stripe.ChargeService "Process payment"

// Reference a database inside another system
ECommerce.API -> AnalyticsDB.DataWarehouse "Push analytics data"
```

This level of detail can be useful when debugging integrations or documenting specific API contracts.

## Consistency: The Golden Rule

If there's one thing I've learned from years of modeling systems, it's this: **consistency matters more than any individual decision**.

### Consistency in Nesting

Don't mix levels inconsistently:

```sruja
// partial
// Bad: Inconsistent nesting
App = system "App" {
  Frontend = container "Frontend"
  Backend = container "Backend" {
    API = component "API Service"
  }
  Database = database "Database"
}

// Good: Consistent nesting
App = system "App" {
  Frontend = container "Frontend" {
    UI = component "UI Components"
  }
  Backend = container "Backend" {
    API = component "API Service"
  }
  Database = database "Database"
}
```

In the bad example, why does Backend have components but Frontend doesn't? Is Frontend simpler? Did we forget to break it down? The inconsistency creates confusion. The good example shows a consistent approach.

### Consistency in Naming

Use consistent naming conventions across your hierarchy:

```sruja
// partial
// Good: Consistent suffixes
UserService = component "User Service"
OrderService = component "Order Service"
PaymentService = component "Payment Service"

// Bad: Inconsistent suffixes
UserService = component "User Service"
Order = component "Order"
Payment = component "Payment Service"
```

In the bad example, why is it "User Service" but just "Order"? The inconsistency makes people wonder if there's a meaningful difference. Spoiler: there usually isn't.

## Pitfalls I've Encountered (So You Don't Have To)

Let me share some mistakes I've made and seen others make. Hopefully, you can avoid them.

### Pitfall 1: Deep Nesting

```sruja
// partial
// Bad: Too deep (5+ levels)
App = system "App" {
  Frontend = container "Frontend" {
    Layout = component "Layout" {
      Header = component "Header" {
        Navigation = component "Navigation" {
          Menu = component "Menu"
        }
      }
    }
  }
}
```

This is unreadable. Who can understand a 5-level deep structure? Nobody. The diagram becomes useless because it's too complex.

**Solution:** Keep nesting to 3-4 levels max. If you're going deeper, you're probably in implementation territory, not architecture.

### Pitfall 2: Orphaned Elements

```sruja
// partial
// Bad: Component without container
Shop = system "Shop"
WebApp = container "Web App"
API = container "API"
Database = database "Database"
AuthService = component "Auth Service"  // Orphan!
```

Where does AuthService belong? Is it in WebApp? In API? Orphaned elements confuse everyone who looks at the diagram.

**Solution:** Every component must belong to a container. Every container must belong to a system. No exceptions.

### Pitfall 3: Flat Everything

```sruja
// partial
// Bad: Everything at same level
Customer = person "Customer"
WebApp = container "Web App"
API = container "API"
Database = database "Database"
AuthService = component "Auth Service"
```

This shows no structure. Is WebApp part of API? Are they siblings? Who knows?

**Solution:** Show the hierarchy. Nest things properly. Make it clear what belongs to what.

## Creating Views at Different Levels

One of the most powerful features of Sruja is the ability to create views at different hierarchy levels. This lets you tell the right story to the right audience.

### Level 1: System Context View

```sruja
view system_context {
  title "System Context"
  include *
}
```

This view shows everything: people, systems, external dependencies. Perfect for stakeholders who want the big picture.

### Level 2: System View

```sruja
view system_view of ECommerce {
  title "E-Commerce System"
  include ECommerce
}
```

This view shows a single system and all its containers. Great for developers who need to understand how a specific system is structured.

### Level 3: Container View

```sruja
// partial
view container_view of ECommerce.API {
  title "API Containers"
  include ECommerce.API.*
  exclude ECommerce.API.Database
}
```

This view shows the internals of a container. Perfect for teams working on that specific service.

### Level 4: Component View

```sruja
// partial
view component_view of ECommerce.API {
  title "API Components"
  include ECommerce.API.*
}
```

This view shows all components within a container. Ideal for detailed design reviews and implementation planning.

The beauty is that you can have multiple views of the same architecture, each tailored to a different audience. The stakeholders get the big picture. Developers get the details. Everyone gets what they need.

## What to Remember

Hierarchy and nesting are about bringing order to complexity. When you're structuring your systems:

- **Follow the C4 hierarchy**: Person → System → Container → Component
- **Nest logically**: Group related parts together
- **Be consistent**: Use the same patterns throughout
- **Match detail to audience**: Don't show components to stakeholders
- **Create multiple views**: Tell different stories to different audiences
- **Keep it simple**: Avoid deep nesting and over-complexity

If you take away one thing, let it be this: **a well-structured hierarchy makes complex systems understandable**. Everything else flows from that principle.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding:

### Question 1

You're modeling a simple mobile app with these requirements:

> "Users can create profiles, post content, and like posts. The app has a mobile frontend, API backend, and database."

At what level should you stop modeling? Should you include components?

**A)** Yes, always include components for completeness
**B)** No, containers are sufficient for this system
**C)** Include components for the API but not the mobile frontend
**D)** Include components only if using microservices

<details>
<summary>Click to see the answer</summary>

**Answer: B) No, containers are sufficient for this system**

Here's why:
- The system is simple (mobile frontend, API, database)
- Three containers is manageable complexity
- There's no indication that the API is internally complex
- Components would add unnecessary detail for a straightforward system

The decision framework I use:
- **Is the container complex?** No, it sounds simple.
- **Is the audience developers?** Maybe, but even developers don't always need component-level detail.
- **Will it add clarity?** Probably not—it might just add noise.

If this system grows and the API becomes complex with 10+ services, then yes, add components. But for now? Keep it at the container level.

</details>

---

### Question 2

You're referencing nested elements in your architecture. Which reference is correctly formatted?

**A)** `WebApp.API.Database "Queries data"`
**B)** `API.AuthService -> UserService "Gets profile"`
**C)** `Shop.API.ProductService -> CartService "Adds item"`
**D)** `System.Container.Service -> External.Service "Calls"`

<details>
<summary>Click to see the answer</summary>

**Answer: B) API.AuthService -> UserService "Gets profile"**

Let's analyze each option:

**A)** `WebApp.API.Database "Queries data"` — Wrong. This implies WebApp has an API, which has a Database. But typically, API and Database are siblings within the System, not nested. It should probably be `System.Database`.

**B)** `API.AuthService -> UserService "Gets profile"` — Correct! This shows a component (AuthService) within a container (API) communicating with another component (UserService) within the same container. Perfect dot notation.

**C)** `Shop.API.ProductService -> CartService "Adds item"` — Almost correct, but incomplete. CartService should be fully qualified as `Shop.API.CartService`. Otherwise, it's unclear where CartService lives.

**D)** `System.Container.Service -> External.Service "Calls"` — Wrong on multiple levels. External systems shouldn't have components in your architecture model—you don't control them. Also, it's unclear where External.Service is.

The key insight: **use full dot notation when referencing nested elements**. Make it clear where every element lives in the hierarchy.

</details>

---

## What's Next?

Congratulations! You've completed Module 2: Parts and Relationships. You now have a complete toolkit for:

- Identifying the key parts of any system
- Modeling parts using Sruja's element types
- Defining meaningful relationships between parts
- Organizing everything with clear hierarchy and structure

You can now create diagrams that tell stories about how systems work—stories that are clear, consistent, and useful for your audience.

In the next module, you'll learn about **boundaries**—how to define where one system ends and another begins. This is crucial for understanding dependencies, managing complexity, and designing systems that are decoupled and maintainable.

See you there!

---

## Module 2 Complete!

You've now mastered the fundamentals of modeling systems. Here's what you've learned:

**Lesson 1: Identifying Parts**
- Start with people, then systems, then containers, then components
- Use the C4 hierarchy as your guide
- Match the level of detail to your audience

**Lesson 2: Sruja Elements**
- Four element types: person, system, container, component
- Add details that matter: technology, description, SLOs
- Use naming conventions consistently

**Lesson 3: Defining Relationships**
- Write labels that tell a story
- Use tags to add meaningful context
- Reference nested elements with dot notation

**Lesson 4: Hierarchy and Nesting**
- Structure your systems consistently
- Avoid deep nesting and over-complexity
- Create multiple views for different audiences

You're ready to tackle more advanced concepts. Let's continue!

---
title: "Lesson 7: Views Best Practices"
weight: 7
summary: "Master the art of creating effective views for different audiences and purposes."
---

# Lesson 7: Views Best Practices

Views are one of the most powerful features in Sruja DSL. They let you create multiple perspectives from a single model, making your architecture documentation accessible to different audiences. This lesson covers best practices for creating effective views.

## The Power of Views

A single architecture model can serve:

- **Executives**: High-level business context
- **Product Managers**: Feature and user journey focus
- **Architects**: Technical design and patterns
- **Developers**: Implementation details
- **Operations**: Deployment and monitoring concerns
- **Security**: Compliance and threat modeling

## View Creation Strategy

### 1. Start with Your Audience

Before creating a view, ask:

- **Who will use this view?** (Executive, Developer, Ops)
- **What questions do they need answered?** (Cost, Performance, Security)
- **What level of detail do they need?** (Context, Container, Component)

### 2. Use Include/Exclude Strategically

```sruja
element person
element system
element container
element component
element datastore

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
}

PaymentGateway = system "Payment Gateway" {
external true
}

Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.OrderDB "Stores orders"
ECommerce.API -> PaymentGateway "Processes payments"

// Executive view: Business context only
view executive {
title "Executive Overview"
include Customer Admin
include ECommerce PaymentGateway
exclude ECommerce.WebApp ECommerce.API ECommerce.OrderDB ECommerce.ProductDB
exclude ECommerce.WebApp.CartComponent ECommerce.WebApp.ProductComponent
exclude ECommerce.API.OrderController ECommerce.API.PaymentController
description "High-level business context for executives"
}

// Architect view: Container-level architecture
view architect {
title "Architectural View"
include ECommerce ECommerce.WebApp ECommerce.API
include ECommerce.OrderDB ECommerce.ProductDB
include PaymentGateway
exclude Customer Admin
exclude ECommerce.WebApp.CartComponent ECommerce.WebApp.ProductComponent
exclude ECommerce.API.OrderController ECommerce.API.PaymentController
description "Container-level architecture for architects"
}

// Developer view: Component-level implementation
view developer {
title "Developer View"
include ECommerce.WebApp ECommerce.WebApp.CartComponent ECommerce.WebApp.ProductComponent
include ECommerce.API ECommerce.API.OrderController ECommerce.API.PaymentController
include ECommerce.OrderDB ECommerce.ProductDB
exclude Customer Admin PaymentGateway
description "Component-level details for developers"
}
```

### 3. Create Concern-Specific Views

Focus on specific concerns: performance, security, data flow, deployment.

```sruja
// Performance view: Components with performance characteristics
view performance {
title "Performance View"
include ECommerce.API ECommerce.OrderDB
exclude Customer Admin ECommerce.WebApp
description "Focuses on performance-critical components"
}

// Security view: External interactions and data stores
view security {
title "Security View"
include ECommerce.API PaymentGateway ECommerce.OrderDB
exclude Customer Admin ECommerce.WebApp
description "Highlights security boundaries and external systems"
}

// Data flow view: Data dependencies
view dataflow {
title "Data Flow View"
include ECommerce.API ECommerce.OrderDB ECommerce.ProductDB
exclude Customer Admin ECommerce.WebApp PaymentGateway
description "Shows how data flows through the system"
}
```

## View Naming Conventions

Use clear, descriptive names that indicate the view's purpose:

### Good Names

- `executive` - Clear audience
- `dataflow` - Clear concern
- `deployment` - Clear purpose
- `security-audit` - Specific use case

### Avoid

- `view1`, `view2` - Not descriptive
- `temp` - Temporary views should be removed
- `test` - Test views shouldn't be in production models

## View Organization Patterns

### Pattern 1: By Audience

Create views for each stakeholder group.

```sruja
view executive { /* ... */ }
view product { /* ... */ }
view architect { /* ... */ }
view developer { /* ... */ }
view operations { /* ... */ }
```

### Pattern 2: By Concern

Create views for different technical concerns.

```sruja
view performance { /* ... */ }
view security { /* ... */ }
view dataflow { /* ... */ }
view deployment { /* ... */ }
```

### Pattern 3: By Layer

Create views for different C4 model layers.

```sruja
view context { /* System context */ }
view container { /* Container diagram */ }
view component { /* Component diagram */ }
```

### Pattern 4: By Feature

Create views for specific features or domains.

```sruja
view checkout { /* Checkout flow */ }
view search { /* Search functionality */ }
view analytics { /* Analytics pipeline */ }
```

## Best Practices

### 1. Always Include an Index View

```sruja
view index {
title "Complete System View"
include *
description "Complete system overview"
}
```

### 2. Use Descriptive Titles

```sruja
view executive {
  title "Executive Overview - Business Context"
  // ...
}
```

### 3. Add Descriptions

```sruja
view architect {
  title "Architectural View"
  description "Container-level architecture showing system boundaries and interactions"
  // ...
}
```

### 4. Keep Views Focused

Each view should answer a specific set of questions. If a view tries to answer too many questions, split it into multiple views.

### 5. Document View Purpose

Use comments or descriptions to explain why a view exists and when to use it.

```sruja
// Use this view for:
// - Executive presentations
// - Business stakeholder discussions
// - High-level architecture reviews
view executive {
title "Executive Overview"
// ...
}
```

## Common View Patterns

### Executive Dashboard View

```sruja
view executive {
  title "Executive Dashboard"
  include Customer Admin
  include ECommerce PaymentGateway
  exclude ECommerce.WebApp ECommerce.API
  exclude ECommerce.OrderDB ECommerce.ProductDB
}
```

### Technical Architecture View

```sruja
view technical {
  title "Technical Architecture"
  include ECommerce ECommerce.WebApp ECommerce.API
  include ECommerce.OrderDB ECommerce.ProductDB
  exclude Customer Admin
}
```

### User Journey View

```sruja
view userjourney {
  title "User Journey View"
  include Customer
  include ECommerce.WebApp ECommerce.API
  include PaymentGateway
  exclude Admin ECommerce.OrderDB ECommerce.ProductDB
}
```

### Deployment View

```sruja
view deployment {
  title "Deployment View"
  include ECommerce.WebApp ECommerce.API
  include ECommerce.OrderDB
  exclude Customer Admin PaymentGateway
}
```

## View Maintenance

### When to Create New Views

- New stakeholder group needs different perspective
- New concern emerges (e.g., compliance)
- Feature-specific view needed for documentation

### When to Remove Views

- View is no longer used
- View duplicates another view
- View is outdated and not maintained

### When to Update Views

- Architecture changes affect view scope
- New components need to be included/excluded
- View purpose changes

## Advanced: View Composition

You can create views that build on other views by carefully selecting elements:

```sruja
// Base view: All containers
view containers {
title "Container View"
include ECommerce.WebApp ECommerce.API
include ECommerce.OrderDB ECommerce.ProductDB
}

// Extended view: Containers + external systems
view containers-extended {
title "Container View with External Systems"
include ECommerce.WebApp ECommerce.API
include ECommerce.OrderDB ECommerce.ProductDB
include PaymentGateway
}
```

## Real-World Example: E-Commerce Platform

```sruja
element person
element system
element container
element component
element datastore
element queue

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
external true
}

Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.OrderDB "Stores orders"
ECommerce.API -> ECommerce.Cache "Caches queries"
ECommerce.API -> PaymentGateway "Processes payments"

// Complete system
view index {
title "Complete System View"
include *
}

// Executive: Business context
view executive {
title "Executive Overview"
include Customer Admin
include ECommerce PaymentGateway
exclude ECommerce.WebApp ECommerce.API ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
}

// Product: User journeys
view product {
title "Product View - User Journeys"
include Customer
include ECommerce.WebApp ECommerce.API
include PaymentGateway
exclude Admin ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
}

// Architect: Container architecture
view architect {
title "Architectural View"
include ECommerce ECommerce.WebApp ECommerce.API
include ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
include PaymentGateway
exclude Customer Admin
}

// Developer: Component details
view developer {
title "Developer View"
include ECommerce.WebApp ECommerce.WebApp.CartComponent ECommerce.WebApp.ProductComponent
include ECommerce.API ECommerce.API.OrderController ECommerce.API.PaymentController
include ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache
exclude Customer Admin PaymentGateway
}

// Operations: Deployment and monitoring
view operations {
title "Operations View"
include ECommerce.WebApp ECommerce.API
include ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
exclude Customer Admin PaymentGateway
}

// Data flow: Data dependencies
view dataflow {
title "Data Flow View"
include ECommerce.API ECommerce.OrderDB ECommerce.ProductDB ECommerce.Cache ECommerce.EventQueue
exclude Customer Admin ECommerce.WebApp PaymentGateway
}

// Performance: Performance-critical components
view performance {
title "Performance View"
include ECommerce.API ECommerce.Cache ECommerce.OrderDB
exclude Customer Admin ECommerce.WebApp ECommerce.ProductDB ECommerce.EventQueue PaymentGateway
}
```

## Summary

Views are a powerful tool for making architecture documentation accessible to different audiences. Follow these best practices:

1. **Start with your audience** - Know who will use the view
2. **Use include/exclude strategically** - Focus on what matters
3. **Create concern-specific views** - Performance, security, data flow
4. **Use clear naming** - Descriptive names that indicate purpose
5. **Document view purpose** - Explain why the view exists
6. **Keep views focused** - Each view should answer specific questions
7. **Maintain views** - Update or remove views as architecture evolves

👉 **[Module 4: Production Readiness](../module-4-production-readiness/lesson-1)** - Learn how to make your architecture production-ready.

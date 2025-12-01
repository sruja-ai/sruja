---
title: "Systems Thinking"
weight: 25
summary: "Learn to model systems holistically: parts, boundaries, flows, feedback loops, and context."
tags: [systems-thinking, modeling]
aliases: ["/tutorials/systems-thinking/"]
---

# Systems Thinking

Systems thinking helps you understand how components interact as part of a whole. Sruja supports five core systems thinking concepts.

## 1. Parts and Relationships

Systems thinking starts with understanding **what** the system contains (parts) and **how** they connect (relationships).

```sruja
architecture "E-Commerce" {
  person Customer "End User"
  
  system Shop "E-Commerce System" {
    container WebApp "Web Application" {
      technology "React"
    }
    
    container API "API Service" {
      technology "Go"
    }
    
    datastore DB "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  // Relationships show how parts interact
  Customer -> Shop.WebApp "Uses"
  Shop.WebApp -> Shop.API "Calls"
  Shop.API -> Shop.DB "Reads/Writes"
}
```

**Key insight**: Identify the parts first, then define how they relate.

## 2. Boundaries

Boundaries define what's **inside** the system vs. what's **outside** (the environment).

```sruja
architecture "E-Commerce" {
  // Inside boundary: System contains these components
  system Shop {
    container WebApp
    container API
    datastore DB
  }
  
  // Outside boundary: External entities
  person Customer "End User"
  person Admin "System Administrator"
  
  system PaymentGateway "Third-party Payment Service" {
    external  // Explicitly marks external boundary
  }
  
  // Relationships cross boundaries
  Customer -> Shop.WebApp "Uses"          // External → Internal
  Shop.API -> PaymentGateway "Processes"  // Internal → External
}
```

**Key insight**: Use `system` to define internal boundaries, `person` and `external` for external boundaries.

## 3. Flows

Flows show how information and data **move through** the system. Sruja supports two flow styles:

### Data Flow Diagram (DFD) Style

Use `flow` for data flow diagrams:

```sruja
architecture "Order Processing" {
  person Customer
  system Shop {
    container WebApp
    container API
    datastore DB
  }
  system PaymentGateway { external }
  
  // DFD-style flow
  flow OrderProcess "Order Processing Flow" {
    Customer -> WebApp "Submits Order"
    WebApp -> API "Sends Order Data"
    API -> DB "Saves Order"
    API -> PaymentGateway "Charges Payment"
    PaymentGateway -> API "Returns Confirmation"
    API -> DB "Updates Order Status"
    API -> WebApp "Returns Result"
    WebApp -> Customer "Shows Confirmation"
  }
}
```

### User Story/Scenario Style

Use `story` or `scenario` for behavioral flows:

```sruja
story Checkout "User Checkout Flow" {
  Customer -> "Cart Page" "adds items to cart"
  "Cart Page" -> WebApp "clicks checkout"
  WebApp -> API "validates cart"
  API -> DB "checks inventory"
  DB -> API "returns stock status"
  API -> PaymentGateway "processes payment"
  PaymentGateway -> API "confirms payment"
  API -> DB "creates order"
  API -> WebApp "returns order confirmation"
  WebApp -> Customer "displays success message"
}
```

**Key insight**: Use `flow` for data flows (DFD), `story`/`scenario` for behavioral flows (BDD).

## 4. Feedback Loops

Feedback loops show how actions create **reactions** that affect future actions. Cycles are valid patterns in Sruja.

### Simple Feedback Loop

```sruja
architecture "User Interaction" {
  person User
  system App {
    container WebApp
    container API
  }
  
  // Feedback loop: User action → System response → User reaction
  User -> WebApp "Submits Form"
  WebApp -> API "Validates"
  API -> WebApp "Returns Validation Result"
  WebApp -> User "Shows Feedback"
  // The feedback affects user's next action (completing the loop)
}
```

### System Feedback Loop

```sruja
architecture "Inventory Management" {
  person Admin
  system Shop {
    container API
    datastore Inventory
  }
  
  // Event-driven feedback loop
  API -> Inventory "Updates Stock"
  Inventory -> API "Notifies Low Stock"
  API -> Admin "Sends Alert"
  Admin -> API "Adjusts Inventory"
  // Creates feedback: API ↔ Inventory ↔ Admin
}
```

**Key insight**: Cycles model natural feedback loops, event-driven patterns, and mutual dependencies. They're valid architectural patterns.

## 5. Context

Context defines the **environment** the system operates in - external dependencies, stakeholders, and surrounding systems.

```sruja
architecture "E-Commerce Platform" {
  // Internal system
  system Shop {
    container WebApp
    container API
    datastore DB
  }
  
  // Context: Stakeholders
  person Customer "End User"
  person Admin "System Administrator"
  person Support "Customer Support"
  
  // Context: External dependencies
  system PaymentGateway "Third-party Payment" {
    external
  }
  
  system EmailService "Email Notifications" {
    external
  }
  
  system AnalyticsService "Usage Analytics" {
    external
  }
  
  // Context relationships
  Customer -> Shop "Uses"
  Admin -> Shop "Manages"
  Support -> Shop "Monitors"
  Shop -> PaymentGateway "Depends on"
  Shop -> EmailService "Sends notifications"
  Shop -> AnalyticsService "Tracks usage"
}
```

**Key insight**: Context includes all external entities and dependencies that affect or are affected by your system.

## Putting It All Together

Here's a complete example combining all five concepts:

```sruja
architecture "Systems Thinking Example" {
  // 1. PARTS AND RELATIONSHIPS
  person Customer "End User"
  person Admin "System Administrator"
  
  system ECommerce "E-Commerce System" {
    container WebApp "Web Application" {
      technology "React"
    }
    container API "API Service" {
      technology "Go"
    }
    datastore DB "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  // 2. BOUNDARIES
  system PaymentGateway "Third-party Payment Service" {
    external  // External boundary
  }
  
  // 3. FLOWS
  flow OrderProcess "Order Processing Flow" {
    Customer -> WebApp "Submits Order"
    WebApp -> API "Sends Order Data"
    API -> DB "Saves Order"
    API -> PaymentGateway "Charges Payment"
    PaymentGateway -> API "Returns Confirmation"
    API -> DB "Updates Order Status"
    API -> WebApp "Returns Result"
    WebApp -> Customer "Shows Confirmation"
  }
  
  // 4. FEEDBACK LOOPS
  Customer -> WebApp "Submits Form"
  WebApp -> API "Validates"
  API -> WebApp "Returns Validation Result"
  WebApp -> Customer "Shows Feedback"
  
  API -> DB "Updates Inventory"
  DB -> API "Notifies Low Stock"
  API -> Admin "Sends Alert"
  Admin -> API "Adjusts Inventory"
  
  // 5. CONTEXT
  person Support "Customer Support"
  system EmailService "Email Notifications" {
    external
  }
  
  Customer -> ECommerce "Uses"
  Admin -> ECommerce "Manages"
  Support -> ECommerce "Monitors"
  ECommerce -> PaymentGateway "Depends on"
  ECommerce -> EmailService "Sends notifications"
}
```

## Why Systems Thinking Matters

- **Holistic understanding**: See the whole system, not just parts
- **Natural patterns**: Model real-world interactions and feedback
- **Clear boundaries**: Understand what's in scope vs. context
- **Flow visualization**: See how data and information move
- **Valid cycles**: Feedback loops are natural, not errors

## Next Steps

- Try the complete example: `examples/systems_thinking.sruja`
- Explore [Domain Modeling (DDD)](/tutorials/domain-modeling-ddd/) for business logic perspective
- Learn [Deployment Modeling](/tutorials/deployment-modeling/) for infrastructure perspective


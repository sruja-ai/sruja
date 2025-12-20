---
title: "Systems Thinking"
weight: 25
summary: "Learn to model systems holistically: parts, boundaries, flows, feedback loops, and context."
tags: ["systems", "modeling"]
---

# Systems Thinking

Systems thinking helps you understand how components interact as part of a whole. Sruja supports five core systems thinking concepts.

## 1. Parts and Relationships

Systems thinking starts with understanding **what** the system contains (parts) and **how** they connect (relationships).

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  Customer = person "End User"
  
  Shop = system "E-Commerce System" {
    WebApp = container "Web Application" {
      technology "React"
    }
    
    API = container "API Service" {
      technology "Go"
    }
    
    DB = datastore "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  // Relationships show how parts interact
  Customer -> Shop.WebApp "Uses"
  Shop.WebApp -> Shop.API "Calls"
  Shop.API -> Shop.DB "Reads/Writes"
}

views {
  view index {
    include *
  }
}
```

**Key insight**: Identify the parts first, then define how they relate.

## 2. Boundaries

Boundaries define what's **inside** the system vs. what's **outside** (the environment).

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  // Inside boundary: System contains these components
  system Shop {
    container WebApp
    container API
    datastore DB
  }
  
  // Outside boundary: External entities
  Customer = person "End User"
  Admin = person "System Administrator"
  
  PaymentGateway = system "Third-party Payment Service" {
    metadata {
      tags ["external"]
    }
  }
  
  // Relationships cross boundaries
  Customer -> Shop.WebApp "Uses"
  Shop.API -> PaymentGateway "Processes"
}

views {
  view index {
    include *
  }
}
```

**Key insight**: Use `system` to define internal boundaries, `person` and `external` for external boundaries.

## 3. Flows

Flows show how information and data **move through** the system. Sruja supports two flow styles:

### Data Flow Diagram (DFD) Style

Use `scenario` for data-oriented flows:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  person Customer
  system Shop {
    container WebApp
    container API
    datastore DB
  }
  system PaymentGateway {
    metadata {
      tags ["external"]
    }
  }
  
  scenario OrderProcess "Order Processing" {
    Customer -> Shop.WebApp "Submits Order"
    Shop.WebApp -> Shop.API "Sends Order Data"
    Shop.API -> Shop.DB "Saves Order"
    Shop.API -> PaymentGateway "Charges Payment"
    Shop.API -> Shop.WebApp "Returns Result"
    Shop.WebApp -> Customer "Shows Confirmation"
  }
}

views {
  view index {
    include *
  }
}
```

### User Story/Scenario Style

Use `scenario` for behavioral flows:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  Customer = person "End User"
  ECommerce = system "E-Commerce System" {
    CartPage = container "Shopping Cart Page"
    WebApp = container "Web Application"
    API = container "API Service"
    DB = datastore "Database"
  }
  PaymentGateway = system "Payment Service" {
    metadata {
      tags ["external"]
    }
  }
  
  story Checkout "User Checkout Flow" {
    Customer -> ECommerce.CartPage "adds items to cart"
    ECommerce.CartPage -> ECommerce.WebApp "clicks checkout"
    ECommerce.WebApp -> ECommerce.API "validates cart"
    ECommerce.API -> ECommerce.DB "checks inventory"
    ECommerce.DB -> ECommerce.API "returns stock status"
    ECommerce.API -> PaymentGateway "processes payment"
    PaymentGateway -> ECommerce.API "confirms payment"
    ECommerce.API -> ECommerce.DB "creates order"
    ECommerce.API -> ECommerce.WebApp "returns order confirmation"
    ECommerce.WebApp -> Customer "displays success message"
  }
}

views {
  view index {
    include *
  }
}
```

**Key insight**: Use `flow` for data flows (DFD), `story`/`scenario` for behavioral flows (BDD).

## 4. Feedback Loops

Feedback loops show how actions create **reactions** that affect future actions. Cycles are valid patterns in Sruja.

### Simple Feedback Loop

```sruja
// EXPECTED_FAILURE: Layer violation
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  User = person "End User"
  App = system "Application" {
    WebApp = container "Web Application"
    API = container "API Service"
  }
  
  // Feedback loop: User action → System response → User reaction
  User -> App.WebApp "Submits Form"
  App.WebApp -> App.API "Validates"
  App.API -> App.WebApp "Returns Validation Result"
  App.WebApp -> User "Shows Feedback"
  // The feedback affects user's next action (completing the loop)

}
```

### System Feedback Loop

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  person Admin
  system Shop {
    container API
    datastore Inventory
  }
  
  // Event-driven feedback loop
  Shop.API -> Shop.Inventory "Updates Stock"
  Shop.Inventory -> Shop.API "Notifies Low Stock"
  Shop.API -> Admin "Sends Alert"
  Admin -> Shop.API "Adjusts Inventory"
  // Creates feedback: API ↔ Inventory ↔ Admin
}

views {
  view index {
    include *
  }
}
```

**Key insight**: Cycles model natural feedback loops, event-driven patterns, and mutual dependencies. They're valid architectural patterns.

## 5. Context

Context defines the **environment** the system operates in - external dependencies, stakeholders, and surrounding systems.

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  // Internal system
  system Shop {
    WebApp = container "Web Application"
    API = container "API Service"
    DB = datastore "Database"
  }
  
  // Context: Stakeholders
  Customer = person "End User"
  Admin = person "System Administrator"
  Support = person "Customer Support"
  
  // Context: External dependencies
  PaymentGateway = system "Third-party Payment" {
    metadata {
      tags ["external"]
    }
  }
  
  EmailService = system "Email Notifications" {
    metadata {
      tags ["external"]
    }
  }
  
  AnalyticsService = system "Usage Analytics" {
    metadata {
      tags ["external"]
    }
  }
  
  // Context relationships
  Customer -> Shop "Uses"
  Admin -> Shop "Manages"
  Support -> Shop "Monitors"
  Shop -> PaymentGateway "Depends on"
  Shop -> EmailService "Sends notifications"
  Shop -> AnalyticsService "Tracks usage"
}

views {
  view index {
    include *
  }
}
```

**Key insight**: Context includes all external entities and dependencies that affect or are affected by your system.

## Putting It All Together

Here's a complete example combining all five concepts:

```sruja
// EXPECTED_FAILURE: Layer violation
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  // 1. PARTS AND RELATIONSHIPS
  Customer = person "End User"
  Admin = person "System Administrator"
  
  ECommerce = system "E-Commerce System" {
    WebApp = container "Web Application" {
      technology "React"
    }
    API = container "API Service" {
      technology "Go"
    }
    DB = datastore "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  // 2. BOUNDARIES
  PaymentGateway = system "Third-party Payment Service" {
    metadata {
      tags ["external"]
    }
  }
  
  // 3. FLOWS
  scenario OrderProcess "Order Processing" {
    Customer -> ECommerce.WebApp "Submits Order"
    ECommerce.WebApp -> ECommerce.API "Sends Order Data"
    ECommerce.API -> ECommerce.DB "Saves Order"
    ECommerce.API -> PaymentGateway "Charges Payment"
    ECommerce.API -> ECommerce.WebApp "Returns Result"
    ECommerce.WebApp -> Customer "Shows Confirmation"
  }
  
  // 4. FEEDBACK LOOPS
  Customer -> ECommerce.WebApp "Submits Form"
  ECommerce.WebApp -> ECommerce.API "Validates"
  ECommerce.API -> ECommerce.WebApp "Returns Validation Result"
  ECommerce.WebApp -> Customer "Shows Feedback"
  
  ECommerce.API -> ECommerce.DB "Updates Inventory"
  ECommerce.DB -> ECommerce.API "Notifies Low Stock"
  ECommerce.API -> Admin "Sends Alert"
  Admin -> ECommerce.API "Adjusts Inventory"
  
  // 5. CONTEXT
  Support = person "Customer Support"
  EmailService = system "Email Notifications" {
    metadata {
      tags ["external"]
    }
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
- Learn [Deployment Modeling](/tutorials/advanced/deployment-modeling) for infrastructure perspective

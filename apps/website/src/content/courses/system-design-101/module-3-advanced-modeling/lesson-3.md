---
title: "Lesson 3: Advanced Scenarios"
weight: 3
summary: "Modeling user journeys and technical sequences with Scenarios."
---

# Lesson 3: Advanced Scenarios

In Sruja, use `scenario` (and its alias `story`) to model runtime interactions: user journeys and technical sequences.

## When to Use Scenarios

- Modeling user interactions
- Modeling technical sequences across elements

### 1. User Flows (Stories)

When modeling a user flow, you focus on the *value* delivered to the user. Sruja provides the `story` keyword (an alias for `scenario`) to make these definitions semantic and clear.

```sruja
specification {
  element person
  element system
  element container
  element datastore
}

model {
    User = person "Customer"
    
    Ticketing = system "Ticketing System" {
        WebApp = container "Web Application" {
            technology "React"
        }
        PaymentService = container "Payment Service" {
            technology "Go"
        }
        TicketDB = datastore "Ticket Database" {
            technology "PostgreSQL"
        }
        
        WebApp -> PaymentService "Processes payment"
        PaymentService -> TicketDB "Stores transaction"
    }
    
    // High-level user flow
    story BuyTicket "User purchases a ticket" {
        User -> Ticketing.WebApp "Selects ticket"
        Ticketing.WebApp -> Ticketing.PaymentService "Process payment" {
            latency "500ms"
            protocol "HTTPS"
        }
        Ticketing.PaymentService -> User "Sends receipt"
    }
}

views {
  view index {
    include *
  }
}
```

Notice how we can add properties like `latency` and `protocol` to steps using the `{ key "value" }` syntax. This adds richness to your model without cluttering the diagram.

### 2. Technical Sequences

When modeling technical sequences, you dive deeper into the architecture, showing how `containers` and `components` interact to fulfill a request. You can stick with the `scenario` keyword here.

```sruja
specification {
  element person
  element system
  element container
  element datastore
}

model {
    User = person "End User"
    
    AuthSystem = system "Authentication System" {
        WebApp = container "Web Application" {
            technology "React"
        }
        AuthServer = container "Auth Server" {
            technology "Node.js, OAuth2"
        }
        Database = datastore "User Database" {
            technology "PostgreSQL"
        }
        
        WebApp -> AuthServer "Validates tokens"
        AuthServer -> Database "Queries user data"
    }
    
    // Detailed technical flow
    scenario AuthFlow "Authentication" "Handles OAuth2 login process" {
        User -> AuthSystem.WebApp "Provides credentials"
        AuthSystem.WebApp -> AuthSystem.AuthServer "Validates token"
        AuthSystem.AuthServer -> AuthSystem.Database "Looks up user"
        AuthSystem.Database -> AuthSystem.AuthServer "Returns user data"
        AuthSystem.AuthServer -> AuthSystem.WebApp "Confirms token valid"
        AuthSystem.WebApp -> User "Shows login success"
    }
}

views {
  view index {
    include *
  }
}
```

## 🛠️ Syntax Flexibility

Sruja offers flexible syntax to suit your needs:

### Simple Syntax
Great for quick sketches or simple flows.

```sruja
specification {
  element person
  element system
  element container
}

model {
    User = person "User"
    
    AuthSystem = system "Auth System" {
        WebApp = container "Web App"
    }
    
    scenario "Login Failure" {
        User -> AuthSystem.WebApp "Enters wrong password"
        AuthSystem.WebApp -> User "Shows error message"
    }
}

views {
  view index {
    include *
  }
}
```

### Formal Syntax
Better for documentation and referencing. Includes an ID and optional description.

```sruja
specification {
  element person
  element system
  element container
  element datastore
}

model {
    Customer = person "Customer"
    
    ECommerce = system "E-Commerce System" {
        Cart = container "Shopping Cart" {
            technology "React"
        }
        Payment = container "Payment Service" {
            technology "Go"
        }
    }
    
    Inventory = system "Inventory System" {
        InventoryService = container "Inventory Service" {
            technology "Java"
        }
    }
    
    Customer -> ECommerce.Cart "Adds items"
    ECommerce.Cart -> Inventory.InventoryService "Checks availability"
    ECommerce.Cart -> ECommerce.Payment "Processes payment"
    
    scenario Checkout "Checkout Process" {
        description "The complete checkout flow including payment and inventory check."
        
        Customer -> ECommerce.Cart "Initiates checkout"
        ECommerce.Cart -> Inventory.InventoryService "Reserves items"
        Inventory.InventoryService -> ECommerce.Cart "Confirms reserved"
        ECommerce.Cart -> ECommerce.Payment "Charges payment"
        ECommerce.Payment -> Customer "Sends confirmation"
    }
}

views {
  view index {
    include *
  }
}
```

---

## Visualizing Scenarios

Studio/Viewer can highlight scenario paths over your static architecture so readers can follow behavior step‑by‑step.

---

## Data-Oriented Scenarios

Use `scenario` for data‑oriented flows too—keep steps focused on interactions and outcomes between elements.

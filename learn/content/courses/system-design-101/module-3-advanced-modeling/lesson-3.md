---
title: "Lesson 3: Advanced Scenarios"
weight: 3
summary: "Modeling both user flows and technical interactions with Scenarios."
---

# Lesson 3: Advanced Scenarios and Flows

In Sruja, you have multiple tools for modeling runtime interactions: `scenario`/`story` for user journeys and technical sequences, and `flow` for data flow diagrams (DFD).

## Two Constructs, Different Purposes

Sruja provides two keywords for different types of flows:

1.  **`scenario`/`story`**: User journeys and technical sequences (BDD-style)
2.  **`flow`**: Data flow diagrams (DFD-style)

### When to Use Each

- **`scenario`/`story`**: When modeling user interactions or technical sequences
- **`flow`**: When modeling how data moves through the system (DFD style)

### 1. User Flows (Stories)

When modeling a user flow, you focus on the *value* delivered to the user. Sruja provides the `story` keyword (an alias for `scenario`) to make these definitions semantic and clear.

```sruja
// High-level user flow
story BuyTicket "User purchases a ticket" {
    User -> WebApp "Selects ticket"
    WebApp -> PaymentService "Process payment" {
        latency "500ms"
        protocol "HTTPS"
    }
    PaymentService -> User "Sends receipt"
}
```

Notice how we can add properties like `latency` and `protocol` to steps using the `{ key "value" }` syntax. This adds richness to your model without cluttering the diagram.

### 2. Technical Sequences

When modeling technical sequences, you dive deeper into the architecture, showing how `containers` and `components` interact to fulfill a request. You can stick with the `scenario` keyword here.

```sruja
// Detailed technical flow
scenario AuthFlow "Authentication" "Handles OAuth2 login process" {
    User -> WebApp "Credentials"
    WebApp -> AuthServer "Validate Token"
    AuthServer -> Database "Lookup User"
    Database -> AuthServer "User Data"
    AuthServer -> WebApp "Token Valid"
    WebApp -> User "Login Success"
}
```

## 🛠️ Syntax Flexibility

Sruja offers flexible syntax to suit your needs:

### Simple Syntax
Great for quick sketches or simple flows.

```sruja
scenario "Login Failure" {
    User -> WebApp "Enters wrong password"
    WebApp -> User "Show error message"
}
```

### Formal Syntax
Better for documentation and referencing. Includes an ID and optional description.

```sruja
scenario Checkout "Checkout Process" {
    description "The complete checkout flow including payment and inventory check."
    
    User -> Cart "Checkout"
    Cart -> Inventory "Reserve Items"
    Inventory -> Cart "Reserved"
    Cart -> Payment "Charge"
}
```

---

## 🎨 D2 Layers: Visualizing Views

Sruja automatically organizes your scenarios into **D2 Layers**. When you generate a diagram, you don't just get a static picture; you get an interactive map.

1.  **Architecture Layer**: The base layer showing your static structure (Systems, Containers).
2.  **Scenario Layers**: Each `scenario` is generated as a separate layer.

This allows you to toggle between the static structure and various runtime flows within the same diagram, giving you a complete 360-degree view of your system!

---

## Data Flow Diagrams (DFD) with `flow`

For data flow modeling, use the `flow` keyword:

```sruja
// Data Flow Diagram (DFD) style
flow OrderProcess "Order Processing Flow" {
    Customer -> Shop "Order Details"
    Shop -> Database "Save Order"
    Database -> Shop "Confirmation"
    Shop -> Customer "Order Confirmed"
}
```

**When to use `flow`:**
- Modeling data movement (DFD style)
- Showing how information flows through the system
- Technical data flow documentation

**When to use `scenario`/`story`:**
- Modeling user journeys
- Showing technical sequences
- BDD-style behavior documentation


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

## Visualizing Scenarios

Studio/Viewer can highlight scenario paths over your static architecture so readers can follow behavior step‑by‑step.

---

## Data-Oriented Scenarios

Use `scenario` for data‑oriented flows too—keep steps focused on interactions and outcomes between elements.

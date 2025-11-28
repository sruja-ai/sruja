---
title: "Lesson 3: Advanced Scenarios"
weight: 3
summary: "Modeling both user flows and technical interactions with Scenarios."
---

# Lesson 3: Advanced Scenarios

In Sruja, the `scenario` construct is your unified tool for modeling all types of runtime interactions. Whether you are describing a high-level user journey or a low-level technical sequence, `scenario` handles it all.

## One Construct, Many Uses

Instead of having separate keywords for different types of flows, Sruja allows you to use `scenario` for:

1.  **User Flows (The "What")**: High-level interactions focusing on value delivery.
2.  **Technical Sequences (The "How")**: Detailed component-to-component interactions.

### 1. User Flows

When modeling a user flow, you typically involve `person` actors and high-level `system` or `container` elements. The goal is to show *what* the user is doing.

```sruja
// High-level user flow
scenario BuyTicket "User purchases a ticket" {
    User -> WebApp "Selects ticket"
    WebApp -> PaymentService "Process payment"
    PaymentService -> User "Sends receipt"
}
```

### 2. Technical Sequences

When modeling technical sequences, you dive deeper into the architecture, showing how `containers` and `components` interact to fulfill a request. This is equivalent to a UML Sequence Diagram.

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


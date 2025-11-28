---
title: "Lesson 3: Journeys and Scenarios"
weight: 3
summary: "Runtime interactions, User stories, and D2 Layers."
---

# Lesson 3: Scenarios vs User Journeys

As you model complex systems, it's important to distinguish between *what the system does for the user* and *how the system components interact technically*.

## User Journeys (The "What")
A **User Journey** describes a high-level scenario from the perspective of an actor (user). It focuses on the value delivered.
*   **Scope:** Often spans multiple systems.
*   **Audience:** Product Managers, Stakeholders, QA.
*   **Example:** "User purchases a ticket."

## Scenarios (The "How")
A **Scenario** illustrates how elements in your static model interact at runtime to fulfill a specific user story or feature. It is essentially a sequence diagram derived from your static model.
*   **Scope:** specific interactions between containers/components.
*   **Audience:** Developers, Architects.
*   **Example:** "API Controller calls Auth Service, then Database."

---

## 🛠️ Sruja Perspective: `journey` vs `scenario`

Sruja supports both concepts natively.

### 1. User Journeys
Use `journey` for high-level flows.

```sruja
journey BuyTicket {
    title "User purchases a ticket"
    steps {
        User -> WebApp "Selects ticket"
        WebApp -> PaymentService "Process payment"
    }
}
```


### 2. Scenarios (Technical Flows)
Use `scenario` for technical system flows, sequence diagrams, and runtime interactions. This replaces the old `dynamic` keyword.

You can use a simple syntax for quick sketches:

```sruja
scenario "Login Failure" {
    User -> WebApp "Enters wrong password"
    WebApp -> User "Show error message"
}
```

Or a more formal syntax with an ID and description for referencing:

```sruja
scenario AuthFlow "Authentication" "Handles OAuth2 login process" {
    User -> WebApp "Credentials"
    WebApp -> AuthServer "Validate Token"
    AuthServer -> WebApp "Token Valid"
    WebApp -> User "Login Success"
}
```

---

## 🎨 D2 Layers: Visualizing Views

Sruja automatically organizes these views into **D2 Layers**. When you generate a diagram, you will see a layer selector (if your viewer supports it) or separate pages for:

1.  **Requirements:** (First Layer) Shows all requirements as `page` shapes.
2.  **Journeys:** Each `journey` gets its own layer with the title displayed.
3.  **Scenarios:** Each `scenario` gets its own layer.
4.  **Architecture:** (Last Layer) The static architecture view.

This allows you to toggle between the static structure, the requirements, and various runtime flows within the same diagram!

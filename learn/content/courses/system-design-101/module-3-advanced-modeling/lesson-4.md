---
title: "Lesson 4: Architectural Views"
weight: 4
summary: "Creating different perspectives with Views (C4 Model)."
---

# Lesson 4: Architectural Views

As your system grows, a single diagram becomes too cluttered. You need different "maps" for different audiences:
*   **Executives:** Need a high-level overview (Context).
*   **Architects:** Need to see service boundaries (Containers).
*   **Developers:** Need to see internal details (Components).

## The `view` Keyword

Sruja allows you to define multiple views of the same underlying model using the `view` keyword. This is heavily inspired by the **C4 Model**.

### 1. Context View (Level 1)
Shows the big picture: System + Users + External Systems.

```sruja
view Context "System Landscape" {
    scope global
    include system
    exclude container
}
```

### 2. Container View (Level 2)
Zooms into a specific system to show its containers (apps, databases).

```sruja
view ContainerView "E-Commerce Containers" {
    scope system ECommerce
    include container
    include system
}
```

### 3. Component View (Level 3)
Zooms into a specific container to show its internal components.

```sruja
view ComponentView "Order Service Internals" {
    scope container OrderService
    include component
    include entity
}
```

## How it Works
*   **`scope`**: Defines the boundary of the view (e.g., `global`, `system X`, `container Y`).
*   **`include`**: Specifies which element types to show (e.g., `system`, `container`, `component`).
*   **`exclude`**: Explicitly hides element types.

By defining views, you can generate multiple diagrams from a single Sruja file, keeping your documentation consistent and tailored to the audience.

---
title: "The C4 Model"
weight: 1
summary: "Understand the core concepts behind Sruja's architecture modeling."
---

# The C4 Model

Sruja is built on the **C4 model**, a hierarchical approach to software architecture diagrams. If you are new to architecture-as-code, it helps to understand these four levels of abstraction.

Think of it like **Google Maps** for your code: you can zoom out to see the whole world (System Context), or zoom in to see individual streets (Code).

## The 4 Levels

### 1. System Context (Level 1)
**"The Big Picture"**

This is the highest level of abstraction. It shows your software system as a single box, and how it interacts with users and other systems (like functional dependencies, email systems, or payment gateways).

*   **Goal:** What is the system, who uses it, and how does it fit into the existing IT landscape?
*   **Audience:** Everyone (Technical & Non-Technical).

```sruja
// A System Context View
system App "My App"
person User "Customer"
system Stripe "Payment Gateway"

User -> App "Uses"
App -> Stripe "Process Payments"
```

### 2. Container (Level 2)
**"The High-Level Technical Building Blocks"**

**Note:** In C4, a "Container" is NOT a Docker container. It represents a deployable unit—something that runs separately. Examples include:
*   A Single-Page Application (SPA)
*   A Mobile App
*   A Server-side API application
*   A Database
*   A File System

*   **Goal:** What are the major technical choices? How do they communicate?
*   **Audience:** Architects, Developers, Ops.

```sruja
system App "My App" {
    container Web "React App"
    container API "Go Service"
    datastore DB "PostgreSQL"
}
```

### 3. Component (Level 3)
**"The Internals"**

Zooming into a Container to see the major structural building blocks. In an API, these might be your controllers, services, or repositories.

*   **Goal:** How is the container structured?
*   **Audience:** Developers.

### 4. Code (Level 4)
**"The Details"**

The actual classes, interfaces, and functions. Sruja focuses mainly on Levels 1, 2, and 3, as Level 4 is best managed by your IDE.

## Key Relationships

The power of C4 is in the **Hierarchical** nature.
*   A **System** defines the boundary.
*   **Containers** live *inside* a System.
*   **Components** live *inside* a Container.

When you define a relationship at a lower level (e.g., `API -> DB`), Sruja automatically understands the relationship at higher levels (e.g., `App -> DB` is implied).

## Why use C4?

1.  **Shared Vocabulary:** "Component" and "Service" often mean different things to different teams. C4 standardizes this.
2.  **Zoom Levels:** Avoids the "one giant messy diagram" problem. You can view the system at the level of detail relevant to you.

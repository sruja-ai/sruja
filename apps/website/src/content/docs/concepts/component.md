---
title: "Component"
weight: 4
summary: "A Component is a grouping of related functionality encapsulated behind a well-defined interface."
---

# Component

A **Component** is a grouping of related functionality encapsulated behind a well-defined interface. Components reside inside Containers.

## Syntax

```sruja
component ID "Label/Name" {
    technology "Technology"
    // ... items
}
```

## Example

```sruja
component AuthController "Authentication Controller" {
    technology "Spring MVC Rest Controller"
    description "Handles user login and registration."
}
```

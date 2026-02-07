---
title: "Component"
weight: 13
summary: "A Component is a grouping of related functionality encapsulated behind a well-defined interface."
---

# Component

A **Component** is a grouping of related functionality encapsulated behind a well-defined interface. Components reside inside Containers.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


ID = component "Label/Name" {
technology "Technology"
// ... items
}
```

## Example

```sruja
import { * } from 'sruja.ai/stdlib'


AuthController = component "Authentication Controller" {
technology "Spring MVC Rest Controller"
description "Handles user login and registration."
}
```

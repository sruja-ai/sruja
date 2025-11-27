---
title: System
weight: 2
---

# System

A **System** represents a software system, which is the highest level of abstraction in the C4 model. A system delivers value to its users, whether they are human or other systems.

## Syntax

```sruja
system ID "Label/Name" {
    description "Optional description"
    // ... contains containers
}
```

## Example

```sruja
system BankingSystem "Internet Banking System" {
    description "Allows customers to view accounts and make payments."
}
```

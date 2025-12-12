---
title: "Person"
weight: 15
summary: "A Person represents a human user of your software system."
---

# Person

A **Person** represents a human user of your software system (e.g., "Customer", "Admin", "Employee").

## Syntax

```sruja
person ID "Label" {
    description "Optional description"
    tags ["tag1", "tag2"]
}
```

## Example

```sruja
person Customer "Bank Customer" {
    description "A customer of the bank with personal accounts."
}
```
```

---
title: "Database"
weight: 16
summary: "A Database represents a database, file system, or any other system that stores data."
---

# Database

A **Database** represents a database, file system, or any other system that stores data. It is a specialized type of Container.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


ID = database "Label" {
    description "Optional description"
    technology "Technology"
}
```

## Example

```sruja
import { * } from 'sruja.ai/stdlib'


DB = database "Main Database" {
    technology "PostgreSQL"
    description "Stores user and order data"
}
```

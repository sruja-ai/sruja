---
title: "DataStore"
weight: 7
summary: "A DataStore represents a database, file system, or any other system that stores data."
---

# DataStore

A **DataStore** represents a database, file system, or any other system that stores data. It is a specialized type of Container.

## Syntax

```sruja
datastore ID "Label" {
    description "Optional description"
    technology "Technology"
}
```

## Example

```sruja
datastore DB "Main Database" {
    technology "PostgreSQL"
    description "Stores user and order data"
}
```

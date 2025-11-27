---
title: DataStore
weight: 7
---

# DataStore

A **DataStore** represents a database, file system, or any other system that stores data. It is a specialized type of Container.

## Syntax

```sruja
datastore ID "Label/Name" {
    technology "Technology"
    description "Description"
    tags ["tag1", "tag2"]
}
```

## Example

```sruja
datastore MainDB "Main Database" {
    technology "PostgreSQL"
    description "Stores all user and transaction data."
}
```

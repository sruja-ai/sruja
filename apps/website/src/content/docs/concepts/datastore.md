---
title: "DataStore"
weight: 16
summary: "A DataStore represents a database, file system, or any other system that stores data."
---

# DataStore

A **DataStore** represents a database, file system, or any other system that stores data. It is a specialized type of Container.

## Syntax

```sruja
datastore = kind "Datastore"

ID = datastore "Label" {
description "Optional description"
technology "Technology"
}
```

## Example

```sruja
datastore = kind "Datastore"

DB = datastore "Main Database" {
technology "PostgreSQL"
description "Stores user and order data"
}
```

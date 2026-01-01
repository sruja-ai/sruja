---
title: "Lesson 2: Databases"
weight: 2
summary: "SQL vs NoSQL, Replication, and Sharding."
---

# Lesson 2: Databases

## SQL vs. NoSQL

### SQL (Relational Databases)

- **Structure:** Structured data with predefined schemas (Tables, Rows, Columns).
- **Query Language:** SQL (Structured Query Language).
- **ACID Compliance:** Strong guarantees for Atomicity, Consistency, Isolation, Durability.
- **Examples:** MySQL, PostgreSQL, Oracle.
- **Best for:** Complex queries, financial transactions.

### NoSQL (Non-Relational Databases)

- **Structure:** Flexible schemas (Key-Value, Document, Graph, Column-Family).
- **Scalability:** Designed for horizontal scaling.
- **Examples:** MongoDB (Document), Redis (Key-Value), Cassandra (Column).
- **Best for:** Rapidly changing data, massive scale, unstructured data.

## Scaling Databases

### Replication

Copying data to multiple servers.

- **Master-Slave:** Writes go to Master, Reads go to Slaves. Good for read-heavy systems.
- **Master-Master:** Writes can go to any node. Complex conflict resolution needed.

### Sharding

Partitioning data across multiple servers (e.g., Users A-M on Server 1, N-Z on Server 2).

- **Pros:** Handles massive data volumes.
- **Cons:** Complex joins, rebalancing data is hard.

---

## 🛠️ Sruja Perspective: Modeling Databases

Sruja allows you to define the type of database and its role in the system.

```sruja
import { * } from 'sruja.ai/stdlib'


UserDB = container "User Database" {
    technology "PostgreSQL"
    tags ["relational", "primary"]
    description "Stores user profiles and authentication data."
}

SessionStore = container "Session Cache" {
    technology "Redis"
    tags ["key-value", "cache"]
    description "Stores active user sessions for fast access."
}

view index {
include *
}
```

---
title: "Lesson 1: Canonical IDs"
weight: 1
summary: "Unique identification across repositories with canonical IDs."
---

# Lesson 1: Canonical IDs

## The Problem: ID Collisions

In a multi-repo system, multiple teams might use the same IDs:

- Repo A: `API` container
- Repo B: `API` container
- Repo C: `API` container

How do we distinguish them?

## Solution: Canonical IDs

Every element gets a **canonical ID** with format:

```
repo_id::local_id
```

Examples:
- `user-service::UserAPI`
- `order-service::OrderAPI`
- `payment-service::PaymentAPI`

## How Canonical IDs Work

### In repo.sruja (local)

```sruja
<!-- partial -->
UserService = system "User Service" {
  UserAPI = container "User API" {
    description "REST API for user management"
  }
}
```

### In system index (composed)

The compose process adds canonical IDs:

```json
{
  "nodes": [
    {
      "canonical_id": "user-service::UserAPI",
      "repo_id": "user-service",
      "local_id": "UserAPI",
      "kind": "container",
      "label": "User API"
    }
  ]
}
```

## Referencing Across Repos

### In DSL imports

```sruja
<!-- partial -->
// In order-service/repo.sruja
import { user-service } from "../bundles/user-service.bundle.json"

OrderService = system "Order Service" {
  OrderProcessor = container "Order Processor"

  // Use canonical reference
  OrderProcessor -> user-service::UserAPI "Validates customer"
}
```

### In queries

```bash
# Query architecture elements using natural language or patterns
sruja query "UserService" -r .

# Query with architecture file
sruja query "API" -r . -a repo.sruja

# Use why command to understand relationships
sruja why "why does OrderService depend on UserService?" -r .
```

## ID Generation Rules

| Element Type | Canonical ID Format | Example |
|-------------|---------------------|---------|
| System | `repo_id::SystemName` | `user-service::UserService` |
| Container | `repo_id::ContainerName` | `user-service::UserAPI` |
| Component | `repo_id::ComponentName` | `user-service::AuthComponent` |

## Best Practices

| Practice | Why |
|----------|-----|
| Use descriptive repo IDs | `user-service` not `us` |
| Consistent naming | Same pattern across teams |
| Don't change IDs | Breaks references |
| Document ID meaning | Helps AI editors |

## Hands-On: Work with Canonical IDs

1. **Publish a bundle and inspect canonical IDs:**
   ```bash
   sruja publish -r . -o repo.bundle.json
   cat repo.bundle.json | jq '.nodes[].canonical_id'
   ```

2. **Query specific elements:**
   ```bash
   sruja query "UserAPI" -r .
   ```

3. **Use impact analysis to see downstream dependencies:**
   ```bash
   sruja impact UserService -r . --depth 2
   ```

## Learning Outcomes

- ✅ Understand what canonical IDs are and why they prevent collisions
- ✅ Explain the format `repo_id::local_id` for different element types
- ✅ Use canonical IDs when referencing components across repositories
- ✅ Apply best practices for naming and maintaining IDs

## Quiz: Test Your Understanding

### Q1: What problem do canonical IDs solve in federated architecture?

A) They speed up bundle composition
B) They prevent naming collisions when multiple repos have components with the same local name
C) They encrypt sensitive architecture data
D) They automatically resolve conflicts

### Q2: What is the format of a canonical ID?

A) `repo-id.local-id` (with a period)
B) `repo_id::local_id` (with double colon)
C) `repo-id/local-id` (with a slash)
D) `local_id@repo_id` (with an at sign)

### Q3: Why should canonical IDs remain stable once established?

A) They are used for authentication
B) Changing them breaks existing references and relationships across repos
C) They are stored in a database
D) They are part of the Git history

## Next Steps

Lesson 2 covers external references between repos.
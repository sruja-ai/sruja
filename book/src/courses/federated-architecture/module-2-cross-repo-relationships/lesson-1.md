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
# Query specific repo's component
sruja query --canonical "user-service::UserAPI"

# Query all components of a repo
sruja query --repo user-service
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

## Next Steps

Lesson 2 covers external references between repos.
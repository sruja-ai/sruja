---
title: "Lesson 3: Ownership & Contracts"
weight: 3
summary: "Track ownership and interfaces across repositories."
---

# Lesson 3: Ownership & Contracts

## Tracking Ownership

Every component should have an owner:

```sruja
UserService = system "User Service" {
  owner "platform-team"

  UserAPI = container "User API" {
    owner "platform-team"
    description "REST API for user management"
  }

  UserDB = database "User Database" {
    owner "platform-team"
    description "PostgreSQL database for users"
  }
}
```

## Contract Exposure

Expose interfaces for other teams:

```sruja
// In user-service/repo.sruja
contract UserServiceContract {
  version "2.0.0"
  owner "platform-team"

  endpoint GET /users/{id} {
    response 200 {
      body {
        id: string
        email: string
        name: string
      }
    }
  }

  endpoint POST /users {
    request {
      email: string
      name: string
    }
    response 201 {
      body {
        id: string
      }
    }
  }
}

UserAPI {
  // Mark as exposing contract
  exposes UserServiceContract
}
```

## Consuming Contracts

```sruja
// In order-service/repo.sruja
import { UserServiceContract } from "./bundles/user-service.bundle.json"

OrderProcessor = container "Order Processor" {
  // Consume via contract
  requires UserServiceContract

  // Implementation detail
  dependency user-service::UserAPI
}
```

## SLA Tracking

```sruja
UserAPI {
  sla {
    availability "99.9%"
    latency_p99 "100ms"
    throughput "5000 req/s"
  }
}
```

## Owner Groups

```yaml
# .sruja/owners.yaml
teams:
  platform-team:
    name "Platform Team"
    email "platform@company.com"
    slack "#team-platform"

  payments-team:
    name "Payments Team"
    email "payments@company.com"
    slack "#team-payments"
```

## Querying Ownership

```bash
# Find all components owned by a team
sruja query --owner platform-team

# Find components without owners
sruja query --no-owner

# Show ownership report
sruja report --ownership
```

## Module Complete!

You've completed Cross-Repo Relationships. You now understand:
- ✅ Canonical ID format and usage
- ✅ Importing and referencing external bundles
- ✅ Tracking ownership across repos
- ✅ Exposing and consuming contracts

Next, Module 3 covers conflict resolution.
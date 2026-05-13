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
# Check drift and see ownership violations
sruja drift -r . -a repo.sruja

# Use ai-context to see team-specific architecture
sruja ai-context -r . --format markdown
```

## Hands-On: Define Ownership

1. **Add ownership metadata to your architecture:**
   ```sruja
   UserService = system "User Service" {
     owner "platform-team"

     UserAPI = container "User API" {
       owner "platform-team"
       description "REST API for user management"
     }
   }
   ```

2. **Validate and check for issues:**
   ```bash
   sruja lint repo.sruja
   sruja drift -r . -a repo.sruja
   ```

3. **Check impact of changes:**
   ```bash
   sruja impact UserAPI -r . --depth 2
   ```

4. **Generate context for your team:**
   ```bash
   sruja ai-context -r . -o team-context.md
   ```

## Learning Outcomes

- ✅ Track ownership of components across repositories
- ✅ Define and expose contracts between services
- ✅ Set SLA requirements for components
- ✅ Use `sruja drift` to validate architecture changes

## Quiz: Test Your Understanding

### Q1: What metadata field should every component have for tracking ownership?

A) `version`
B) `owner`
C) `status`
D) `priority`

### Q2: What does the `contract` block in Sruja represent?

A) A legal agreement
B) An exposed API interface definition between services
C) A database schema
D) A CI/CD pipeline

### Q3: Which command helps detect architectural drift between code and architecture?

A) `sruja lint`
B) `sruja drift`
C) `sruja validate`
D) `sruja check`

## Module Complete!

You've completed Cross-Repo Relationships. You now understand:
- ✅ Canonical ID format and usage
- ✅ Importing and referencing external bundles
- ✅ Tracking ownership across repos
- ✅ Exposing and consuming contracts

Next, Module 3 covers conflict resolution.
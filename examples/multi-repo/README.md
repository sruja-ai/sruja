# Multi-Repository Architecture with Sruja

This guide demonstrates how to use Sruja in a **multi-repo microservices environment** where each service is maintained in its own repository.

## Overview

In a microservices architecture, your system is composed of multiple independent services, each living in its own repository. Sruja supports this through **external system references**, allowing you to:

- Define each service's architecture in its own repository
- Reference other services without duplicating their architecture
- Validate each repository independently
- Maintain clear bounded contexts

## Pattern: Distributed Architecture Files

### Repository Structure

```
github.com/org/
├── user-service/              # Repo 1
│   ├── architecture.sruja     # User Service architecture
│   ├── .cursorrules           # AI integration (Cursor)
│   ├── .copilot-instructions.md  # AI integration (Copilot)
│   └── src/
│
├── order-service/             # Repo 2
│   ├── architecture.sruja     # Order Service architecture
│   ├── .cursorrules
│   ├── .copilot-instructions.md
│   └── src/
│
└── payment-service/           # Repo 3
    ├── architecture.sruja     # Payment Service architecture
    ├── .cursorrules
    ├── .copilot-instructions.md
    └── src/
```

### Each Service Defines

1. **Its own components** (containers, datastores, etc.)
2. **External systems** (references to other services)
3. **Relationships** to both internal and external components

## AI Generation Workflow

### Step 1: Set Up AI Integration

In each service repository, generate AI integration files:

```bash
cd user-service/
sruja generate ai-files
```

This creates:
- `.cursorrules` - For Cursor IDE
- `.copilot-instructions.md` - For GitHub Copilot
- `.architecture-skill.md` - Links to full architecture skill

### Step 2: Generate Architecture with AI

**For Cursor:**
```
Generate architecture.sruja for the User Service in this repository.

THIS REPOSITORY contains:
- User Service: User management and authentication (Node.js, PostgreSQL, Redis)

EXTERNAL REPOSITORIES (declare as external systems):
- Order Service: Order processing service
- Payment Service: Payment processing service
- Notification Service: Email and push notifications

Requirements:
1. Define all components in THIS repository with technology labels
2. Declare external services using `external system`
3. Add relationships to external systems
4. Run `sruja lint architecture.sruja` to validate
```

**For GitHub Copilot:**
Same prompt, Copilot will use the `.copilot-instructions.md` file for context.

### Step 3: Validate

After AI generates the architecture:

```bash
sruja lint architecture.sruja
```

Fix any errors reported by the linter.

### Step 4: Export Documentation

Generate human-readable documentation:

```bash
sruja export markdown architecture.sruja > docs/architecture.md
sruja export mermaid architecture.sruja > docs/diagram.mmd
```

## Example: External System References

### In `user-service/architecture.sruja`

```sruja
// Define THIS service's architecture
system "User Service" {
  api = container "User API" {
    technology "Node.js"
    description "REST API for user management"
  }
  
  db = datastore "User Database" {
    technology "PostgreSQL"
    description "User data storage"
  }
  
  api -> db "SQL queries"
}

// Declare EXTERNAL services (defined in other repos)
order_service = external_system "Order Service" {
  description "Order processing service (separate repo: github.com/org/order-service)"
}

payment_service = external_system "Payment Service" {
  description "Payment processing service (separate repo: github.com/org/payment-service)"
}

// Reference external systems in relationships
user_service.api -> order_service.api "REST API"
user_service.api -> payment_service.api "gRPC"
```

### In `order-service/architecture.sruja`

```sruja
// Define THIS service's architecture
system "Order Service" {
  api = container "Order API" {
    technology "Python"
    description "REST API for order management"
  }
  
  db = datastore "Order Database" {
    technology "PostgreSQL"
    description "Order data storage"
  }
  
  api -> db "SQL queries"
}

// Declare EXTERNAL services
user_service = external_system "User Service" {
  description "User management service (separate repo)"
}

payment_service = external_system "Payment Service" {
  description "Payment processing service (separate repo)"
}

// Reference external systems
order_service.api -> user_service.api "REST API - validate user"
order_service.api -> payment_service.api "gRPC - process payment"
```

## Validation in CI/CD

### GitHub Actions Example

Create `.github/workflows/sruja.yml` in each repository:

```yaml
name: Validate Architecture

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  paths:
    - '**/*.sruja'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install Sruja CLI
        run: cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked
      
      - name: Validate architecture
        run: sruja lint architecture.sruja
```

## Getting a Complete System View

### Option 1: Central Aggregator (Optional)

Create a separate "architecture" repository that imports all services:

```sruja
// In architecture-repo/master.sruja
import "github.com/org/user-service/blob/main/architecture.sruja"
import "github.com/org/order-service/blob/main/architecture.sruja"
import "github.com/org/payment-service/blob/main/architecture.sruja"

// This file now contains all services and can be validated together
```

Note: Import mechanism is planned but not yet implemented. For now, manually copy architecture files to a central location if needed.

### Option 2: Documentation Aggregation

Each service exports its architecture to Markdown, and a central documentation site aggregates them:

```bash
# In each service repo
sruja export markdown architecture.sruja > docs/architecture.md

# Central docs site can pull these from each repo
```

### Option 3: Architecture Registry (Future)

Publish architectures to a central registry:

```bash
# In each service repo
sruja publish --registry architecture.company.com

# In central repo
sruja import user-service@v1.2.0
sruja import order-service@v2.0.0
```

Note: Registry functionality is planned but not yet implemented.

## Best Practices

### 1. Consistent Naming

Use consistent names across repos:
- **Service names**: "User Service", "Order Service" (not "UserService", "order-svc")
- **Component IDs**: `user_api`, `order_api` (snake_case)
- **External references**: Match the actual service name

### 2. Descriptive Relationships

Be specific about how services communicate:
```sruja
// Good
user_service.api -> external:"Order Service".api "REST API - validate user for order creation"

// Too vague
user_service.api -> external:"Order Service".api "calls"
```

### 3. Include Location

Document where external systems live:
```sruja
order_service = external_system "Order Service" {
  description "Order processing service"
  location "github.com/org/order-service"
}
```

### 4. Use Tags

Tag services for categorization:
```sruja
architecture "User Service" {
  tags = ["microservice", "authentication", "user-management"]
}
```

### 5. Define Bounded Contexts

Clearly state what each service is responsible for:
```sruja
system "User Service" {
  description "User management bounded context: authentication, profiles, preferences"
}
```

### 6. Document Communication Patterns

Use views to show different interaction patterns:
```sruja
view "Synchronous Dependencies" {
  includes = [
    "user_service.api",
    "external:Order Service",
    "external:Payment Service"
  ]
}

view "Event-Driven Flow" {
  includes = [
    "order_service.queue",
    "external:Notification Service"
  ]
}
```

## Example Files in This Directory

- `user-service.sruja` - User management and authentication service
- `order-service.sruja` - Order processing with event-driven architecture
- `payment-service.sruja` - Payment processing with third-party integrations

Each file demonstrates:
- Internal architecture (containers, datastores)
- External system references
- Different communication protocols (REST, gRPC, AMQP)
- Views for different flows
- Tags and metadata

## Validation Rules

The linter validates:
- ✅ All component references exist (internal or external)
- ✅ Every container has a `technology` field
- ✅ Every component has a `description`
- ✅ No orphan components (all have relationships)
- ✅ No circular dependencies between systems
- ✅ Valid relationship syntax

## Common Patterns

### API Gateway Pattern

```sruja
gateway = container "API Gateway" {
  technology "Kong"
  description "API gateway, authentication, rate limiting"
}

user -> gateway "HTTPS"
gateway -> user_service.api "REST"
gateway -> order_service.api "REST"
```

### Event-Driven Pattern

```sruja
message_queue = datastore "Event Bus" {
  technology "RabbitMQ"
  description "Async event bus"
}

order_service.api -> message_queue "publishes OrderCreated event"
message_queue -> inventory_service.worker "consumes OrderCreated events"
```

### Database per Service

```sruja
system "User Service" {
  api = container "User API" { technology "Node.js" }
  db = datastore "User DB" { technology "PostgreSQL" }
  api -> db "SQL"
}

system "Order Service" {
  api = container "Order API" { technology "Python" }
  db = datastore "Order DB" { technology "MySQL" }
  api -> db "SQL"
}
```

## Troubleshooting

### Error: Undefined reference 'order_service'

**Solution**: Declare it as external system:
```sruja
order_service = external_system "Order Service" {
  description "Order processing service"
  metadata {
    location "github.com/org/order-service"
    tags ["external", "microservice"]
  }
}
```

### Error: Missing 'technology' on container

**Solution**: Add technology field:
```sruja
container "API" {
  technology "Node.js"  // Add this
  description "API server"
}
```

### How do I reference a specific container in an external system?

**Solution**: Use the external keyword:
```sruja
user_service.api -> external:"Order Service".api "REST API"
```

## Next Steps

1. **Set up AI integration**: `sruja generate ai-files`
2. **Generate architecture**: Use AI with the prompts above
3. **Validate**: `sruja lint architecture.sruja`
4. **Export docs**: `sruja export markdown architecture.sruja`
5. **Commit**: Add `.sruja` files and AI integration files to git
6. **CI/CD**: Add validation to your pipeline
7. **Iterate**: Update architecture as your system evolves

## Related Documentation

- [Language Specification](../../docs/LANGUAGE_SPECIFICATION.md)
- [AI Editor Integration](../../docs/AI_EDITOR_INTEGRATION.md)
- [Using Sruja in Your Project](../../docs/USING_SRUJA_IN_YOUR_PROJECT.md)
- [Examples Directory](../) - More architecture examples
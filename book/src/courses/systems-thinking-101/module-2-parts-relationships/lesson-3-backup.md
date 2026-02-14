# Lesson 3: Defining Relationships

## Learning Goals

- Understand how parts connect
- Learn Sruja relationship syntax
- Write clear, meaningful relationship labels
- Model different types of interactions

## What Are Relationships?

Relationships describe how parts interact. They show:

- Communication between components
- Data flow
- Dependencies
- User actions

## Basic Syntax

```sruja
From -> To "Label"
```

- `From`: Source element
- `To`: Destination element
- `"Label"`: What the relationship represents (required)

## Examples

### Person to System

```sruja
Customer -> Shop "Uses"
Administrator -> Shop "Manages"
```

### Container to Container

```sruja
Shop.WebApp -> Shop.API "Makes API calls"
Shop.API -> Shop.Database "Reads and writes"
```

### Component to Component

```sruja
Shop.API.ProductService -> Shop.API.CartService "Gets product details"
Shop.API.CartService -> Shop.API.OrderService "Creates order"
```

## Relationship Labels

Labels should be **clear, concise, and meaningful**.

### Good Labels

```sruja
Customer -> Shop.WebApp "Browses products"
Shop.API -> Shop.Database "Queries data"
Shop.API -> PaymentGateway "Processes payment"
```

### Bad Labels

```sruja
Customer -> Shop.WebApp "Uses"  // Too generic
Shop.API -> Shop.Database "Connects"  // Doesn't describe what happens
Shop.API -> PaymentGateway "Integration"  // Technical, not behavioral
```

### Label Guidelines

- **Use present tense verbs**: "uses", "reads", "calls"
- **Be specific**: "Processes payment" vs "uses"
- **Show direction**: Clear who initiates the action
- **Keep it short**: 2-5 words is ideal

## Relationship Patterns

### Pattern 1: User Interactions

```sruja
User -> WebApp "Logs in"
User -> WebApp "Views products"
User -> WebApp "Adds to cart"
User -> WebApp "Checks out"
```

### Pattern 2: Service Communication

```sruja
WebApp -> API "Sends requests"
API -> Database "Persists data"
API -> Cache "Reads cache"
Cache -> API "Returns cached data"
```

### Pattern 3: External Dependencies

```sruja
API -> PaymentGateway "Process payment"
API -> EmailService "Send notifications"
API -> AnalyticsService "Track events"
```

## Nested Element References

Use dot notation to reference nested elements:

```sruja
// Direct child reference
Customer -> Shop.WebApp "Uses"

// Nested component reference
Shop.API.ProductService -> Shop.API.CartService "Get product info"

// Cross-system reference
Shop.API -> PaymentGateway.ChargeService "Process payment"
```

## Relationship Tags

Add tags to categorize relationships:

```sruja
From -> To "Label" [tag1, tag2]

// Example
Shop.API -> PaymentGateway "Process payment" [critical, external]
Shop.WebApp -> Shop.API "API call" [http]
```

### Common Tags

```sruja
// Protocol
[http], [grpc], [websocket]

// Importance
[critical], [optional], [best-effort]

// Data type
[synchronous], [asynchronous], [streaming]

// Security
[encrypted], [authenticated], [public]

// Scope
[internal], [external], [partner]
```

## Multiple Relationships

Elements can have multiple relationships:

```sruja
// WebApp has multiple relationships
Customer -> Shop.WebApp "Browses"
Shop.WebApp -> Shop.API "Queries products"
Shop.WebApp -> Shop.Cache "Reads cache"
Shop.WebApp -> Customer "Displays products"

// API has multiple relationships
Shop.WebApp -> Shop.API "Sends request"
Shop.API -> Shop.Database "Persists order"
Shop.API -> PaymentGateway "Process payment"
Shop.API -> Shop.WebApp "Returns response"
```

## Relationship Direction

### One-Way

```sruja
Customer -> Shop.WebApp "Uses"
```

### Two-Way (Two separate relationships)

```sruja
User -> App.API "Submits data"
App.API -> User "Returns result"
```

### Feedback Loop (Cycle)

```sruja
User -> App.WebApp "Submits form"
App.WebApp -> App.API "Validates"
App.API -> App.WebApp "Returns errors"
App.WebApp -> User "Shows errors"
// User resubmits (loop completes)
```

## Relationships in Views

Relationships are automatically included in views:

```sruja
view index {
  include *
}

view container_view of Shop {
  include Shop.*
}
```

## Complete Example

```sruja
import { * } from 'sruja.ai/stdlib'

// Elements
Customer = person "Customer"
Admin = person "Administrator"

Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "Database"
}

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Relationships
Customer -> Shop.WebApp "Browses products" [public]
Customer -> Shop.WebApp "Adds to cart" [authenticated]
Customer -> Shop.WebApp "Checks out" [authenticated]

Shop.WebApp -> Shop.API "Sends requests" [http, encrypted]
Shop.API -> Shop.DB "Persists data" [critical]
Shop.API -> Shop.DB "Queries data" [cached]

Shop.API -> PaymentGateway "Process payment" [critical, external]

Admin -> Shop.WebApp "Manages products" [authenticated]
Admin -> Shop.WebApp "Views reports" [authenticated]

view index {
  include *
}
```

## Exercise

Add relationships to this architecture:

```sruja
import { * } from 'sruja.ai/stdlib'

Reader = person "Reader"
Writer = person "Writer"

Blog = system "Blog Platform" {
  Frontend = container "Web App"
  Backend = container "API"
  Database = database "PostgreSQL"
}

EmailService = system "Email Service" {
  tags ["external"]
}
```

**Define relationships for:**

- Reader interactions
- Writer interactions
- Backend-Database communication
- Backend-EmailService integration

## Key Takeaways

1. **Relationships show interactions**: How parts communicate
2. **Labels must be meaningful**: Clear, specific verbs
3. **Use dot notation**: Reference nested elements
4. **Add tags**: Categorize relationships
5. **Multiple relationships**: Elements can have many connections

## Next Lesson

In [Lesson 4](lesson-4.md), you'll learn how to organize parts using hierarchy and nesting.

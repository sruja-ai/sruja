# Lesson 3: Modeling Cycles

## Learning Goals

- Learn how to create valid cycles in Sruja
- Model feedback loops explicitly
- Differentiate cycles from circular dependencies

## Cycles in Sruja

Unlike circular dependencies (bad), feedback loops (good) are valid cycles in Sruja.

### Basic Cycle Syntax

```sruja
// partial
// Valid feedback loop
User -> App.WebApp "Submits form"
App.WebApp -> App.API "Validates"
App.API -> App.WebApp "Returns result"
App.WebApp -> User "Shows feedback"

// User resubmits (cycle completes)
```

## Modeling Feedback Loops

### Example 1: User Feedback Loop

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

User = person "User"

App = system "Application" {
  WebApp = container "Web Application"
  API = container "API Service"
}

// User feedback cycle
UserFeedback = scenario "User Form Feedback" {
  User -> App.WebApp "Submit form"
  App.WebApp -> App.API "Validate input"
  App.API -> App.WebApp "Return validation result"

  // Error path (loop)
  if has_errors {
    App.WebApp -> User "Show errors"
    User -> App.WebApp "Correct and resubmit"
  }

  // Success path (end cycle)
  if no_errors {
    App.WebApp -> Database "Save data"
    App.WebApp -> User "Show success"
  }
}
```

### Example 2: System Self-Regulation

```sruja
// partial
AutoScaling = system "Auto-Scaling System" {
  Monitor = container "Monitoring Service"
  Scaling = container "Scaling Service"
}

App = system "Application" {
  API = container "API Service"
}

// Auto-scaling feedback loop
ScalingLoop = scenario "Auto-Scaling Feedback" {
  App.API -> AutoScaling.Monitor "Reports load"

  // Scale up if load is high
  if load_high {
    AutoScaling.Monitor -> AutoScaling.Scaling "Trigger scale up"
    AutoScaling.Scaling -> App.API "Add instances"
    App.API -> AutoScaling.Monitor "Reports new load"
    // Loop continues until load normalizes
  }

  // Scale down if load is low
  if load_low {
    AutoScaling.Monitor -> AutoScaling.Scaling "Trigger scale down"
    AutoScaling.Scaling -> App.API "Remove instances"
    App.API -> AutoScaling.Monitor "Reports new load"
    // Loop continues until load normalizes
  }
}
```

### Example 3: Inventory Management

```sruja
// partial
Admin = person "Administrator"

Shop = system "Shop" {
  API = container "API Service"
  Inventory = database "Inventory Database"
}

// Inventory feedback loop
InventoryLoop = scenario "Inventory Feedback" {
  Shop.API -> Shop.Inventory "Update stock"

  // Low stock alert
  if stock_low {
    Shop.Inventory -> Shop.API "Notify low stock"
    Shop.API -> Admin "Send restock alert"
    Admin -> Shop.API "Restock inventory"
    Shop.API -> Shop.Inventory "Update stock"
    // Inventory updates, loop may repeat
  }

  // Normal stock (no action needed)
  if stock_normal {
    Shop.Inventory -> Shop.API "Stock OK"
  }
}
```

### Example 4: Learning System

```sruja
// partial
User = person "User"

MLSystem = system "ML Recommendation System" {
  API = container "Recommendation API"
  Model = database "ML Model"
  Training = container "Training Pipeline"
}

// Learning feedback loop
LearningLoop = scenario "ML Learning Cycle" {
  User -> MLSystem.API "Request recommendations"
  MLSystem.API -> MLSystem.Model "Get predictions"
  MLSystem.Model -> MLSystem.API "Return recommendations"
  MLSystem.API -> User "Show recommendations"

  // User feedback
  User -> MLSystem.API "Rate recommendations"

  // Model update
  MLSystem.API -> MLSystem.Training "Add training data"
  MLSystem.Training -> MLSystem.Model "Update model"

  // Improved recommendations next time
  MLSystem.Model -> MLSystem.API "Better predictions"
  MLSystem.API -> User "Show improved recommendations"
}
```

## Explicit vs Implicit Cycles

### Explicit Cycle (Clear Feedback)

```sruja
// partial
// Shows the complete feedback path
User -> App "Submit data"
App -> User "Show result"
User -> App "Adjust and resubmit"

// Clearly shows learning/adaptation
```

### Implicit Cycle (Inferred)

```sruja
// partial
// Relationships imply the cycle exists
User -> App "Uses"
App -> User "Responds"

// Cycle is there but not explicitly modeled
```

**Recommendation**: Use explicit cycles for important feedback mechanisms.

## Valid Cycles vs Circular Dependencies

### Circular Dependency (Bad)

```sruja
// partial
// Static compile-time dependency
ModuleA -> ModuleB "Imports"
ModuleB -> ModuleA "Imports"

// Problems:
// - Impossible to initialize
// - Tight coupling
// - No clear purpose
```

### Feedback Loop (Good)

```sruja
// partial
// Dynamic runtime feedback
User -> App "Submits data"
App -> User "Shows result"

// Benefits:
// - Enables adaptation
// - Clear purpose
// - Loose coupling (eventual consistency)
```

## Feedback Loop Patterns

### Pattern 1: Immediate Feedback

```sruja
// partial
InstantFeedback = scenario "Form Validation" {
  User -> WebApp "Type in field"
  WebApp -> API "Validate"
  API -> WebApp "Result (instant)"
  WebApp -> User "Show error/success"
}
```

### Pattern 2: Delayed Feedback

```sruja
// partial
DelayedFeedback = scenario "Performance Monitoring" {
  App -> Monitoring "Log metrics"

  // Time passes

  Monitoring -> App "Send alert (after threshold)"
  App -> Admin "Notify"
  Admin -> App "Adjust configuration"
  App -> Monitoring "Log new metrics"
}
```

### Pattern 3: Aggregated Feedback

```sruja
// partial
AggregatedFeedback = scenario "A/B Testing" {
  Users -> App "Use feature A"

  // Aggregate many interactions
  App -> Analytics "Log events"
  Analytics -> Dashboard "Show aggregated results"

  Team -> App "Make decision based on data"
  App -> Users "Roll out winner to all"
}
```

## Feedback Loop Controls

### Prevent Runaway Behavior

```sruja
// partial
AutoScaling = container "Auto-Scaling Service" {
  scale {
    min 2
    max 10
    metric "cpu > 80%"
    cooldown "5 minutes"  // Prevent rapid scaling
  }
}
```

### Circuit Breaker Pattern

```sruja
// partial
CircuitBreaker = scenario "Circuit Breaker Feedback" {
  App -> Service "Make request"

  // If failures exceed threshold, open circuit
  if failures > threshold {
    App -> Fallback "Use fallback"
    Fallback -> App "Return cached data"

    // After cooldown, try again
    if cooldown_elapsed {
      App -> Service "Make request"
      Service -> App "Success (close circuit)"
    }
  }
}
```

### Rate Limiting

```sruja
// partial
RateLimited = scenario "Rate Limited Requests" {
  User -> API "Send request"

  // Check rate limit
  API -> RateLimiter "Check limit"

  if under_limit {
    RateLimiter -> API "Allow"
    API -> User "Process request"
  }

  if over_limit {
    RateLimiter -> API "Throttle"
    API -> User "Rate limit exceeded"

    // User waits and retries (feedback loop)
    User -> API "Retry after delay"
  }
}
```

## Documenting Feedback Loops

### Add Metadata

```sruja
// partial
AutoScaling = system "Auto-Scaling" {
  metadata {
    feedback_loop {
      type "negative_balancing"
      purpose "Maintain target CPU usage"
      target "70% CPU"
      controls "min/max instance limits"
      monitoring "CPU, latency, error rate"
    }
  }
}
```

## Complete Example: E-Commerce Feedback Loops

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"
Admin = person "Administrator"

Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "Database"
  Cache = database "Redis Cache"
}

// User feedback loop (interactive)
UserFeedback = scenario "Checkout Feedback" {
  Customer -> Shop.WebApp "Submit order"
  Shop.WebApp -> Shop.API "Process order"

  // Payment feedback
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment result"

  if payment_failed {
    Shop.API -> Shop.WebApp "Return error"
    Shop.WebApp -> Customer "Show error, retry"
    Customer -> Shop.WebApp "Try again"  // Loop
  }

  if payment_success {
    Shop.API -> Shop.Database "Save order"
    Shop.API -> Shop.WebApp "Success"
    Shop.WebApp -> Customer "Show confirmation"
  }
}

// Inventory feedback loop (self-regulating)
InventoryFeedback = scenario "Inventory Feedback" {
  Shop.API -> Shop.Database "Update inventory"

  if stock_low {
    Shop.Database -> Shop.API "Notify low stock"
    Shop.API -> Admin "Send alert"
    Admin -> Shop.API "Restock"
    Shop.API -> Shop.Database "Update inventory"
  }
}

// Cache feedback loop (learning)
CacheFeedback = scenario "Cache Learning" {
  Shop.API -> Shop.Cache "Query cache"

  if cache_hit {
    Shop.Cache -> Shop.API "Return data (fast)"
  }

  if cache_miss {
    Shop.API -> Shop.Database "Query database"
    Shop.Database -> Shop.API "Return data"
    Shop.API -> Shop.Cache "Store in cache"
    // Next request will be a cache hit
  }
}

view index {
  include *
}
```

## Exercise

Model feedback loops for:

1. **Chat application**: User types, app shows "typing indicator", receiver sees it, receiver types, sender sees "typing indicator"...

2. **Review system**: User rates product, system updates average rating, displays to next users...

3. **Load balancing**: Server gets overloaded, balancer sends traffic to other servers, load redistributes...

## Key Takeaways

1. **Cycles are valid in Sruja**: Feedback loops are not errors
2. **Explicit cycles**: Model important feedback mechanisms
3. **Differentiate**: Circular dependencies (bad) vs feedback loops (good)
4. **Add controls**: Prevent runaway behavior with limits
5. **Document clearly**: Use metadata to explain feedback loops

## Module 5 Complete

You've completed Feedback Loops! You now understand:

- What feedback loops are and why they matter
- Types of feedback loops (positive, negative, delayed)
- How to model cycles in Sruja

**Next**: Learn about [Module 6: Context](../module-6-context/module-overview.md).

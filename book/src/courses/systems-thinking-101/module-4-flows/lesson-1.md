# Lesson 1: Understanding Flows

## Learning Goals

- Understand what flows are in systems thinking
- Learn when to use flows vs. static relationships
- Recognize different types of flows

## What Are Flows?

Flows show how information, data, and actions move through a system. Unlike static relationships (which show connections), flows show **sequences** and **transformations**.

### Static Relationship vs. Flow

```sruja
// Static relationship: Shows connection
Customer -> Shop.WebApp "Uses"

// Flow: Shows sequence
CheckoutFlow = scenario "Checkout" {
  Customer -> Shop.WebApp "Submits order"
  Shop.WebApp -> Shop.API "Sends order data"
  Shop.API -> Shop.Database "Saves order"
}
```

## Why Flows Matter

### 1. Data Lineage

Where does data come from and where does it go?

```sruja
DataFlow = flow "Order Analytics" {
  Customer -> Shop.WebApp "Order data"
  Shop.WebApp -> Shop.API "API request"
  Shop.API -> Shop.Database "Persist"
  Shop.Database -> Analytics "Extract"
  Analytics -> Dashboard "Visualize"
}
```

### 2. Process Understanding

What's the sequence of actions?

```sruja
OrderProcess = scenario "Order Processing" {
  Customer -> Shop.WebApp "Submits order"
  Shop.WebApp -> Shop.API "Process order"
  Shop.API -> PaymentGateway "Charge payment"
  Shop.API -> InventoryService "Reserve items"
  Shop.API -> EmailService "Send confirmation"
}
```

### 3. Bottleneck Identification

Where can things slow down?

```sruja
UploadFlow = scenario "File Upload" {
  User -> Frontend "Upload file"
  Frontend -> API "Send file data"
  API -> Storage "Store file"
  Storage -> Processing "Process file"  // Potential bottleneck
  Processing -> Notification "Notify user"
}
```

### 4. Error Paths

What happens when things fail?

```sruja
OrderFlow = scenario "Order Processing" {
  Customer -> Shop.API "Submit order"
  Shop.API -> PaymentGateway "Charge payment"
  PaymentGateway -> Shop.API "Payment result"

  // Success path
  Shop.API -> Shop.Database "Save order"
  Shop.API -> EmailService "Send confirmation"

  // Failure path
  Shop.API -> Shop.WebApp "Return error"
  Shop.WebApp -> Customer "Display error"
}
```

## Types of Flows

### 1. Data Flow (DFD Style)

Shows how data moves through the system:

```sruja
OrderFlow = flow "Order Data Flow" {
  Customer -> Shop.WebApp "Order details"
  Shop.WebApp -> Shop.API "Order JSON"
  Shop.API -> Shop.Database "Order record"
  Shop.Database -> Analytics "Order event"
}
```

**Use when**: Modeling data lineage, ETL processes, analytics pipelines

### 2. User Journey / Scenario (BDD Style)

Shows user interactions and system responses:

```sruja
Checkout = story "User Checkout Flow" {
  Customer -> Shop.WebApp "Clicks checkout"
  Shop.WebApp -> Shop.API "Validate cart"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment result"
  Shop.API -> Shop.WebApp "Order confirmation"
  Shop.WebApp -> Customer "Show success message"
}
```

**Use when**: Modeling user stories, test scenarios, requirements

### 3. Control Flow

Shows decision points and branches:

```sruja
ApprovalFlow = scenario "Order Approval" {
  Order -> ApprovalService "Submit for approval"

  // Branch 1: Auto-approved
  ApprovalService -> Database "Save order"

  // Branch 2: Manual review
  ApprovalService -> Manager "Request approval"
  Manager -> ApprovalService "Approve/Reject"
  ApprovalService -> Database "Save order"
}
```

**Use when**: Modeling business logic, workflows

### 4. Event Flow

Shows events and their propagation:

```sruja
EventFlow = flow "Order Event Flow" {
  API -> EventBus "Publish order created"
  EventBus -> NotificationService "Consume event"
  EventBus -> AnalyticsService "Consume event"
  EventBus -> EmailService "Consume event"
}
```

**Use when**: Event-driven architectures, pub/sub patterns

## Flow Characteristics

### Linear Flow

```sruja
LinearFlow = scenario "Simple Process" {
  Step1 -> Step2
  Step2 -> Step3
  Step3 -> Step4
}
```

### Branching Flow

```sruja
BranchingFlow = scenario "With Conditions" {
  Step1 -> Step2

  // Branch A
  Step2 -> Step3A
  Step3A -> Step4

  // Branch B
  Step2 -> Step3B
  Step3B -> Step4
}
```

### Converging Flow

```sruja
ConvergingFlow = scenario "Parallel then Merge" {
  Step1 -> Step2A
  Step1 -> Step2B
  Step2A -> Step3
  Step2B -> Step3
  Step3 -> Step4
}
```

## Flows in Sruja

### Using `scenario`

```sruja
MyScenario = scenario "Scenario Title" {
  Step1 -> Step2 "Action"
  Step2 -> Step3 "Action"
}
```

### Using `story` (alias)

```sruja
MyStory = story "Story Title" {
  Step1 -> Step2 "Action"
  Step2 -> Step3 "Action"
}
```

### Using `flow`

```sruja
MyFlow = flow "Flow Title" {
  Step1 -> Step2 "Data"
  Step2 -> Step3 "Data"
}
```

## When to Use Flows

### Use Flows When

- You need to show sequence (order matters)
- Modeling data lineage or transformation
- Documenting user journeys
- Understanding process steps
- Identifying bottlenecks

### Don't Use Flows When

- Showing general connections (use static relationships)
- High-level overview (too much detail)
- Simple system (relationships suffice)

## Flow Anti-Patterns

### Anti-Pattern 1: Too Detailed

```sruja
// Bad: Too much detail
Flow = scenario "Login" {
  User -> UI "Click login button"
  UI -> API "HTTP POST /login"
  API -> Database "SELECT * FROM users"
  Database -> API "User record"
  API -> Auth "Verify password"
  Auth -> API "Result"
  API -> UI "Response"
  UI -> User "Show dashboard"
}
```

**Solution**: Group related steps.

### Anti-Pattern 2: Too Abstract

```sruja
// Bad: Not useful
Flow = scenario "Process" {
  Start -> End "Process completes"
}
```

**Solution**: Add meaningful intermediate steps.

### Anti-Pattern 3: Mixing Flows with Static Relationships

```sruja
// Bad: Confusing to read
Customer -> Shop.WebApp "Uses"
OrderFlow = scenario "Checkout" {
  Customer -> Shop.WebApp "Submits order"
}
```

**Solution**: Keep flows separate from static architecture.

## Exercise

Identify the type of flow for each scenario:

1. "User clicks 'Buy Now', sees payment form, enters card details, sees success page"
2. "Order data is sent to API, saved to database, extracted to analytics warehouse"
3. "Order is created, event is published, multiple services process the event"
4. "Order is submitted, if >$100 requires approval, otherwise auto-approved"

## Key Takeaways

1. **Flows show sequences**: How things happen, not just connections
2. **Multiple flow types**: Data flows, user journeys, control flows, event flows
3. **Right level of detail**: Not too detailed, not too abstract
4. **Use appropriately**: When sequence matters
5. **Separate from static**: Flows complement, don't replace, relationships

## Next Lesson

In [Lesson 2](lesson-2.md), you'll learn how to create data flow diagrams.

# Lesson 3: User Journeys

## Learning Goals

- Model user journeys and behavioral scenarios
- Use BDD-style scenarios for requirements
- Document test cases with flows

## What Are User Journeys?

User journeys (or scenarios) show how users interact with a system to achieve a goal. They're behavioral flows that capture:

- User actions
- System responses
- Success and failure paths
- Decision points

## User Journey Elements in Sruja

### `scenario` for Behavioral Flows

```sruja
CheckoutScenario = scenario "User Checkout" {
  Customer -> Shop.WebApp "Clicks checkout"
  Shop.WebApp -> Shop.API "Validate cart"
  Shop.API -> PaymentGateway "Process payment"
  Shop.API -> EmailService "Send confirmation"
  Shop.WebApp -> Customer "Show success page"
}
```

### `story` (alias)

```sruja
CheckoutStory = story "As a customer, I want to checkout" {
  Customer -> Shop.WebApp "Clicks checkout"
  Shop.WebApp -> Shop.API "Validate cart"
  Shop.API -> PaymentGateway "Process payment"
  Shop.WebApp -> Customer "Show success page"
}
```

## BDD (Behavior-Driven Development) Style

### Given-When-Then Pattern

```sruja
// GIVEN: Customer has items in cart
// WHEN: Customer clicks checkout
// THEN: Payment is processed and confirmation shown

CheckoutScenario = scenario "Customer Checkout" {
  Customer -> Shop.WebApp "Clicks checkout"
  Shop.WebApp -> Shop.API "Validate cart"
  Shop.API -> PaymentGateway "Process payment"
  Shop.API -> Shop.WebApp "Order confirmation"
  Shop.WebApp -> Customer "Show success page"
}
```

## User Journey Patterns

### Pattern 1: Happy Path

```sruja
HappyPath = scenario "Successful User Registration" {
  User -> WebApp "Opens registration form"
  WebApp -> API "Submit registration"
  API -> Database "Create user"
  API -> EmailService "Send welcome email"
  API -> WebApp "Registration success"
  WebApp -> User "Show welcome page"
}
```

### Pattern 2: Error Path

```sruja
ErrorPath = scenario "Registration with Duplicate Email" {
  User -> WebApp "Submit registration"
  WebApp -> API "Submit registration"
  API -> Database "Check email exists"
  Database -> API "Email already exists"
  API -> WebApp "Return error"
  WebApp -> User "Show error: Email already registered"
}
```

### Pattern 3: Branching Path

```sruja
BranchingPath = scenario "Order Approval" {
  Manager -> WebApp "Submit for approval"

  // Branch 1: Auto-approved for < $100
  if value < 100 {
    WebApp -> API "Auto-approve"
    API -> Database "Save approved order"
  }

  // Branch 2: Manual review for > $100
  if value > 100 {
    WebApp -> API "Request manual approval"
    API -> Approver "Send approval request"
    Approver -> API "Approve order"
    API -> Database "Save approved order"
  }

  API -> WebApp "Approval result"
  WebApp -> Manager "Show confirmation"
}
```

### Pattern 4: Retry Path

```sruja
RetryPath = scenario "Payment with Retry" {
  Customer -> WebApp "Submit order"
  WebApp -> API "Process order"

  // First attempt fails
  API -> PaymentGateway "Process payment"
  PaymentGateway -> API "Payment failed: timeout"

  // Retry
  API -> PaymentGateway "Retry payment"
  PaymentGateway -> API "Payment success"

  API -> Database "Save order"
  API -> WebApp "Order confirmed"
  WebApp -> Customer "Show confirmation"
}
```

## User Journey Examples

### Example 1: E-Commerce

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "Database"
}

PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"

// Complete user journey
CheckoutJourney = scenario "Complete Checkout" {
  Customer -> Shop.WebApp "Add item to cart"
  Customer -> Shop.WebApp "Click checkout"
  Shop.WebApp -> Shop.API "Validate cart"
  Shop.API -> Shop.Database "Check inventory"
  Shop.Database -> Shop.API "Inventory available"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment success"
  Shop.API -> Shop.Database "Save order"
  Shop.API -> EmailService "Send confirmation"
  Shop.WebApp -> Customer "Show order confirmation"
}

view index {
  include *
}
```

### Example 2: User Authentication

```sruja
AuthJourney = scenario "User Login" {
  User -> WebApp "Enter credentials"
  WebApp -> API "Submit login"
  API -> Database "Verify credentials"
  Database -> API "Valid credentials"
  API -> AuthService "Generate JWT token"
  AuthService -> API "Token"
  API -> WebApp "Return token and user data"
  WebApp -> User "Show dashboard"
}
```

### Example 3: File Upload

```sruja
UploadJourney = scenario "File Upload" {
  User -> WebApp "Select file and click upload"
  WebApp -> API "Upload file data"
  API -> Storage "Store file"
  Storage -> API "File URL"
  API -> ProcessingService "Process file"
  ProcessingService -> API "Processing complete"
  API -> WebApp "Upload success"
  WebApp -> User "Show file preview"
}
```

## Complex User Journey

```sruja
ComplexJourney = scenario "Complete Order Workflow" {
  Customer -> Shop.WebApp "Browse products"
  Shop.WebApp -> Shop.API "Get products"
  Shop.API -> Shop.Database "Query products"
  Shop.Database -> Shop.API "Product data"
  Shop.API -> Shop.WebApp "Product list"
  Shop.WebApp -> Customer "Display products"

  Customer -> Shop.WebApp "Add to cart"
  Shop.WebApp -> Shop.API "Add to cart"
  Shop.API -> Shop.Database "Save cart item"

  Customer -> Shop.WebApp "Checkout"
  Shop.WebApp -> Shop.API "Create order"
  Shop.API -> Shop.Database "Save order"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment result"

  // Success path
  if payment_success {
    Shop.API -> EmailService "Send confirmation"
    Shop.API -> InventoryService "Reserve items"
    Shop.API -> NotificationService "Notify warehouse"
    Shop.WebApp -> Customer "Order confirmation"
  }

  // Failure path
  if payment_failed {
    Shop.API -> Shop.WebApp "Payment error"
    Shop.WebApp -> Customer "Show error message"
  }
}
```

## Testing with Scenarios

### Acceptance Criteria

```sruja
// As a customer, I want to checkout so that I can purchase products

AcceptanceScenario = scenario "Checkout Acceptance Criteria" {
  // AC1: Customer can checkout with valid payment
  Customer -> Shop.WebApp "Checkout with valid payment"
  Shop.WebApp -> Shop.API "Process order"
  Shop.API -> PaymentGateway "Charge card"
  PaymentGateway -> Shop.API "Success"
  Shop.API -> Shop.WebApp "Order confirmed"
  Shop.WebApp -> Customer "Show confirmation page"

  // AC2: Customer sees error with invalid payment
  Customer -> Shop.WebApp "Checkout with invalid card"
  Shop.WebApp -> Shop.API "Process order"
  Shop.API -> PaymentGateway "Charge card"
  PaymentGateway -> Shop.API "Declined"
  Shop.API -> Shop.WebApp "Payment failed"
  Shop.WebApp -> Customer "Show error message"

  // AC3: Customer receives email confirmation
  Shop.API -> EmailService "Send confirmation"
  Customer -> EmailService "Receive confirmation email"
}
```

## Documenting Edge Cases

### Edge Case Scenarios

```sruja
EdgeCase1 = scenario "Checkout with Expired Card" {
  Customer -> Shop.WebApp "Checkout"
  Shop.WebApp -> Shop.API "Process payment"
  Shop.API -> PaymentGateway "Charge card"
  PaymentGateway -> Shop.API "Card expired"
  Shop.API -> Shop.WebApp "Error: Card expired"
  Shop.WebApp -> Customer "Show error and prompt new card"
}

EdgeCase2 = scenario "Checkout with Insufficient Inventory" {
  Customer -> Shop.WebApp "Checkout"
  Shop.WebApp -> Shop.API "Process order"
  Shop.API -> Shop.Database "Check inventory"
  Shop.Database -> Shop.API "Insufficient stock"
  Shop.API -> Shop.WebApp "Error: Not enough stock"
  Shop.WebApp -> Customer "Show error and suggest alternatives"
}
```

## Exercise

Create user journeys for:

1. **Happy Path**: User registers successfully
2. **Error Path**: User tries to register with existing email
3. **Branching**: User submits order that requires approval if > $1000
4. **Retry**: Payment fails twice, succeeds on third attempt

## Key Takeaways

1. **Use `scenario`/`story`**: For behavioral flows
2. **Model happy and error paths**: Document all outcomes
3. **Include user actions**: From user perspective
4. **Document edge cases**: Unexpected scenarios
5. **Use for testing**: Scenarios make good test cases

## Module 4 Complete

You've completed Flows! You now understand:

- What flows are and when to use them
- Data Flow Diagrams (DFDs)
- User journeys and behavioral scenarios

**Next**: Learn about [Module 5: Feedback Loops](../module-5-feedback-loops/module-overview.md).

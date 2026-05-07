---
title: "Lesson 3: Advanced Scenarios"
weight: 3
summary: "The diagram looked perfect. Then users started clicking buttons."
time: "11 min"
---

# The Happy Path Lie: Why Scenarios Reveal What Diagrams Hide

The architecture diagram was beautiful. Clean boxes, clear connections, everything made sense. The new engineer studied it for an hour and said, "I understand how the system works."

Then they tried to trace a user logging in.

"Wait, does the web app call the auth service first, or does it check the local cache?" "Does the auth service validate against the database or an external provider?" "What happens if the external provider is down?" "Where does the refresh token come from?"

The diagram showed what connected to what. It didn't show what happened when.

This is the gap scenarios fill. Static architecture diagrams show structure. Scenarios show behavior. They show the sequence, the timing, the decision points, the failure paths. They show how your system actually works when users start clicking buttons.

This lesson is about using scenarios to model runtime behavior. You'll learn when scenarios help (and when they're just documentation overhead), how to model user journeys and technical sequences, and the practical patterns that make scenarios useful instead of busywork.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand why static diagrams aren't enough for complex behavior
- Recognize when scenarios add value vs. when they're overkill
- Model user journeys with the `story` keyword
- Model technical sequences with the `scenario` keyword
- Add metadata (latency, protocols) to scenario steps
- Avoid common scenario mistakes

## Why Scenarios Matter: The Static Diagram Problem

Here's a truth that took me years to learn: **Architecture diagrams show potential connections, not actual behavior.**

When you look at a diagram with 20 boxes and 30 arrows, you know what *can* connect to what. But you don't know:
- What happens first?
- What happens in parallel?
- What happens if something fails?
- How long does each step take?
- Which path is the happy path vs. error handling?

**The problem:** New team members stare at diagrams and nod, but they don't actually understand the system. They understand the components, but not how they interact.

**The solution:** Scenarios that show specific sequences. Not every possible interaction—just the important ones.

### A Real Example: The Checkout Confusion

I worked with a team that had a beautiful e-commerce architecture diagram. But every time someone new joined, they'd ask the same questions:

1. "When does inventory get reserved?"
2. "What if payment fails after inventory is reserved?"
3. "Does the user get an email before or after the order is confirmed?"
4. "What if the email service is down?"

The diagram didn't answer these questions. It showed that Cart, Payment, Inventory, and Email services existed and were connected. It didn't show the sequence.

**The fix:** We wrote out three scenarios:
- Happy path: Successful checkout
- Payment failure: Rollback inventory, notify user
- Email failure: Order still completes, retry email later

Suddenly, new engineers could understand the system in 10 minutes instead of 2 weeks. The scenarios captured what the diagram couldn't: behavior over time.

## When Scenarios Help (And When They're Overkill)

After years of using scenarios, I've learned they're not always the answer.

**Scenarios help when:**

1. **Onboarding new engineers** - Show them the key flows, not just the components
2. **Debugging complex interactions** - Trace the actual sequence when something breaks
3. **Designing new features** - Think through the happy path and error cases before coding
4. **Communicating with non-technical stakeholders** - User journeys make sense to product managers
5. **Documenting edge cases** - "What happens if X fails?" is a scenario

**Scenarios are overkill when:**

1. **Simple CRUD operations** - "User creates a record" doesn't need a scenario
2. **Obvious flows** - If the sequence is clear from the diagram, don't document it twice
3. **Exhaustive documentation** - You don't need scenarios for every possible path
4. **Implementation details** - Scenarios should show behavior, not code

The key is to model the *important* flows, not all flows. A system with 50 scenarios is as confusing as a system with none.

## Two Types of Scenarios

Sruja supports two types of scenarios, each for a different audience.

### Type 1: User Flows (Stories)

**Audience:** Product managers, designers, stakeholders
**Focus:** Value delivered to the user
**Keyword:** `story` (alias for `scenario`)

User flows show what the user experiences, not technical details. They answer: "What does the user do, and what do they get?"

**Example: Buying a Ticket**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

User = person "Customer"

Ticketing = system "Ticketing System" {
    WebApp = container "Web Application" {
        technology "React"
    }
    PaymentService = container "Payment Service" {
        technology "Rust"
    }
    TicketDB = database "Ticket Database" {
        technology "PostgreSQL"
    }

    WebApp -> PaymentService "Processes payment"
    PaymentService -> TicketDB "Stores transaction"
}

// User-focused story: What does the user experience?
BuyTicket = story "User purchases a ticket" {
    User -> Ticketing.WebApp "Selects ticket"
    Ticketing.WebApp -> Ticketing.PaymentService "Process payment" {
        latency "500ms"
        protocol "HTTPS"
    }
    Ticketing.PaymentService -> User "Sends receipt"
}

view index {
    include *
}
```

**Notice:**
- Focus on user actions: "Selects ticket", "Process payment"
- Includes user-facing metadata: latency (how long does it take?)
- Skips technical details: What database? What API format?
- Tells a story: User does X, system responds with Y

**When to use stories:** When you're communicating with non-engineers or documenting user-facing behavior.

### Type 2: Technical Sequences

**Audience:** Engineers, architects
**Focus:** Technical implementation details
**Keyword:** `scenario`

Technical sequences show how containers and components interact. They answer: "What calls what, in what order, with what data?"

**Example: Authentication Flow**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

User = person "End User"

AuthSystem = system "Authentication System" {
    WebApp = container "Web Application" {
        technology "React"
    }
    AuthServer = container "Auth Server" {
        technology "Node.js, OAuth2"
    }
    Database = database "User Database" {
        technology "PostgreSQL"
    }

    WebApp -> AuthServer "Validates tokens"
    AuthServer -> Database "Queries user data"
}

// Technical sequence: How does it actually work?
AuthFlow = scenario "Authentication" {
    User -> AuthSystem.WebApp "Provides credentials"
    AuthSystem.WebApp -> AuthSystem.AuthServer "Validates token"
    AuthSystem.AuthServer -> AuthSystem.Database "Looks up user"
    AuthSystem.Database -> AuthSystem.AuthServer "Returns user data"
    AuthSystem.AuthServer -> AuthSystem.WebApp "Confirms token valid"
    AuthSystem.WebApp -> User "Shows login success"
}

view index {
    include *
}
```

**Notice:**
- Focus on technical steps: "Validates token", "Looks up user"
- Shows the full round-trip: Request → validation → database → response
- More detailed than user stories
- Useful for debugging and onboarding engineers

**When to use technical sequences:** When you're debugging, designing implementation, or helping engineers understand the system.

## Adding Metadata to Scenarios

Scenarios become more valuable when you add metadata. Sruja supports properties on each step:

```sruja
// partial
PaymentFlow = scenario "Payment Processing" {
    Customer -> ECommerce.Cart "Initiates checkout" {
        latency "50ms"
    }
    ECommerce.Cart -> Inventory.InventoryService "Reserves items" {
        latency "200ms"
        protocol "gRPC"
    }
    Inventory.InventoryService -> ECommerce.Cart "Confirms reserved" {
        latency "100ms"
    }
    ECommerce.Cart -> ECommerce.Payment "Charges payment" {
        latency "500ms - 2000ms"
        protocol "HTTPS"
        retry "3x with exponential backoff"
    }
    ECommerce.Payment -> Customer "Sends confirmation" {
        async true
    }
}
```

**Why metadata matters:**
- **Latency** helps identify bottlenecks (total = sum of steps)
- **Protocol** shows how services communicate (HTTP vs gRPC vs async)
- **Retry logic** shows resilience patterns
- **Async flags** show what happens in background

This metadata turns scenarios from documentation into design tools. You can see: "Wait, this flow takes 2+ seconds? Where's the time going?"

## Real-World Scenario Patterns

After years of writing scenarios, I've found a few patterns that work well.

### Pattern 1: Happy Path + Edge Cases

Don't try to model every possible path. Model the happy path and the important edge cases:

```sruja
// partial
// Happy path
CheckoutSuccess = story "Successful checkout" {
    Customer -> Cart "Initiates checkout"
    Cart -> Payment "Processes payment"
    Payment -> Customer "Shows success"
}

// Edge case: Payment failure
CheckoutPaymentFailed = story "Checkout - payment fails" {
    Customer -> Cart "Initiates checkout"
    Cart -> Payment "Processes payment"
    Payment -> Cart "Returns error: insufficient funds"
    Cart -> Customer "Shows payment error"
    Cart -> Inventory "Releases reserved items"
}

// Edge case: Inventory unavailable
CheckoutNoInventory = story "Checkout - no inventory" {
    Customer -> Cart "Initiates checkout"
    Cart -> Inventory "Checks availability"
    Inventory -> Cart "Returns error: out of stock"
    Cart -> Customer "Shows out of stock message"
}
```

Three scenarios cover 95% of checkout behavior. You don't need 20 scenarios for every edge case.

### Pattern 2: User Journey Mapping

For complex user journeys, break them into phases:

```sruja
// partial
// Phase 1: Discovery
OnboardingDiscovery = story "User discovers features" {
    User -> App "Opens app"
    App -> User "Shows feature highlights"
    User -> App "Explores feature X"
}

// Phase 2: Activation
OnboardingActivation = story "User tries core feature" {
    User -> App "Uses feature X for first time"
    App -> User "Shows success"
    App -> Analytics "Tracks activation"
}

// Phase 3: Retention
OnboardingRetention = story "User returns" {
    User -> App "Opens app (day 2+)"
    App -> User "Shows personalized content"
    App -> Notifications "Sends reminder (if inactive)"
}
```

This helps product teams understand user behavior at each stage.

### Pattern 3: Failure Scenarios

Model what happens when things break:

```sruja
// partial
// Service failure
PaymentServiceDown = scenario "Payment service unavailable" {
    Customer -> Cart "Initiates checkout"
    Cart -> Payment "Processes payment"
    Payment -> Cart "Returns error: service unavailable"
    Cart -> Queue "Queues order for retry"
    Cart -> Customer "Shows 'we'll process shortly'"
}

// Recovery
PaymentServiceRecovery = scenario "Payment service recovers" {
    Queue -> Payment "Retries payment"
    Payment -> Queue "Confirms success"
    Queue -> Customer "Sends delayed confirmation"
}
```

Failure scenarios help you design resilient systems.

## Common Scenario Mistakes

I've seen teams misuse scenarios in predictable ways.

**Mistake 1: Scenarios for Everything**

"We need scenarios for every user action!"

**Reality:** You end up with 100 scenarios no one reads. Focus on the important flows. Simple CRUD doesn't need scenarios.

**Mistake 2: Too Much Detail**

"Let's add every API call and database query!"

**Reality:** Scenarios become implementation documentation. Keep them at the right level of abstraction. Technical sequences show component interactions, not code execution.

**Mistake 3: No Failure Paths**

"We only model the happy path."

**Reality:** Users don't follow the happy path. They click wrong buttons, enter invalid data, and encounter errors. Model the important failure cases.

**Mistake 4: Outdated Scenarios**

"We wrote scenarios once and never updated them."

**Reality:** Scenarios that don't match reality are worse than no scenarios. Either keep them updated or don't write them.

**Mistake 5: Mixing Audiences**

"We'll write one scenario for everyone."

**Reality:** Engineers need technical sequences. Product managers need user stories. Different audiences need different levels of detail.

## A Complete Example: E-Commerce Checkout

Let me show you a complete example with all the patterns:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

ECommerce = system "E-Commerce System" {
    Cart = container "Shopping Cart" {
        technology "React"
        description "Manages shopping cart state"
    }
    Payment = container "Payment Service" {
        technology "Rust"
        description "Processes payments via Stripe"
    }
    OrderDB = database "Order Database" {
        technology "PostgreSQL"
    }
}

Inventory = system "Inventory System" {
    InventoryService = container "Inventory Service" {
        technology "Java"
        description "Manages stock levels and reservations"
    }
    InventoryDB = database "Inventory Database" {
        technology "PostgreSQL"
    }
    InventoryService -> InventoryDB "Reads/Writes"
}

// Static relationships
Customer -> ECommerce.Cart "Adds items"
ECommerce.Cart -> Inventory.InventoryService "Checks availability"
ECommerce.Cart -> ECommerce.Payment "Processes payment"

// Scenario 1: Happy path (user story)
CheckoutSuccess = story "Successful checkout" {
    description "The ideal checkout flow with payment and confirmation"
    
    Customer -> ECommerce.Cart "Initiates checkout"
    ECommerce.Cart -> Inventory.InventoryService "Reserves items" {
        latency "200ms"
        protocol "gRPC"
    }
    Inventory.InventoryService -> ECommerce.Cart "Confirms reserved"
    ECommerce.Cart -> ECommerce.Payment "Charges payment" {
        latency "500ms - 2000ms"
        protocol "HTTPS"
    }
    ECommerce.Payment -> Customer "Sends confirmation email" {
        async true
    }
}

// Scenario 2: Technical sequence (detailed)
CheckoutTechnical = scenario "Checkout - technical sequence" {
    Customer -> ECommerce.Cart "Initiates checkout"
    ECommerce.Cart -> ECommerce.OrderDB "Creates order record" {
        latency "50ms"
    }
    ECommerce.Cart -> Inventory.InventoryService "Reserves items"
    Inventory.InventoryService -> Inventory.InventoryDB "Updates stock"
    ECommerce.Cart -> ECommerce.Payment "Processes payment"
    ECommerce.Payment -> ECommerce.OrderDB "Updates order status"
    ECommerce.Cart -> Customer "Shows success"
}

view index {
    title "E-Commerce System"
    include *
}

view checkout {
    title "Checkout Flow"
    include Customer ECommerce.Cart ECommerce.Payment
    include Inventory.InventoryService
    exclude ECommerce.OrderDB Inventory.InventoryDB
}
```

Notice:
- Two scenarios for different audiences (story vs technical)
- Metadata on important steps (latency, protocol, async)
- Multiple views for different perspectives
- Description on complex scenarios

## Syntax Options: Simple vs. Formal

Sruja supports both simple and formal syntax for scenarios.

### Simple Syntax

Quick and lightweight:

```sruja
// partial
LoginFailure = scenario "Login Failure" {
    User -> AuthSystem.WebApp "Enters wrong password"
    AuthSystem.WebApp -> User "Shows error message"
}
```

**Use when:** Quick sketches, simple flows, internal documentation.

### Formal Syntax

More structure for important scenarios:

```sruja
// partial
CheckoutProcess = scenario "Checkout Process" {
    description "The complete checkout flow including payment and inventory check"
    
    Customer -> ECommerce.Cart "Initiates checkout"
    ECommerce.Cart -> Inventory.InventoryService "Reserves items"
    Inventory.InventoryService -> ECommerce.Cart "Confirms reserved"
    ECommerce.Cart -> ECommerce.Payment "Charges payment"
    ECommerce.Payment -> Customer "Sends confirmation"
}
```

**Use when:** External documentation, complex flows, important scenarios.

## What to Remember

**Static diagrams show structure, scenarios show behavior.** Diagrams answer "what connects to what?" Scenarios answer "what happens when?" You need both.

**Scenarios are for important flows, not all flows.** Model the 20% of flows that matter (happy path + key edge cases). Don't try to document every possible interaction.

**Two types for two audiences:** User stories (product focus) for non-engineers, technical sequences (implementation focus) for engineers. Use the right type for your audience.

**Add metadata to make scenarios useful:** Latency, protocols, retry logic, async flags—these turn scenarios from documentation into design tools.

**Model failure paths, not just happy paths:** Users don't follow the happy path. Services fail. Networks timeout. Model what happens when things break.

**Keep scenarios updated or delete them:** Outdated scenarios are misleading. If you can't maintain them, don't write them.

**Use views to show different perspectives:** A checkout flow looks different to a product manager vs. an engineer. Use Sruja's views to show the right level of detail to each audience.

Scenarios aren't busywork—they're how you communicate behavior that diagrams can't capture. Use them when they add value, skip them when they don't.

## What's Next

Now that you understand scenarios, [Lesson 4](lesson-4.md) covers a different dimension: modeling requirements and constraints. You'll learn how to capture non-functional requirements (performance, scalability, compliance) in Sruja and link them to the components they affect.

This completes the core modeling concepts. The remaining lessons cover specific patterns and production concerns.

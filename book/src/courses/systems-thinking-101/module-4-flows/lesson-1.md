---
title: "Lesson 1: Understanding Flows"
weight: 1
summary: "What flows are and when to use them"
time: "5 min"
---

# Seeing Movement: Understanding Flows

Ever watch water flow down a stream? You can see the path it takes, where it speeds up, where it slows down, where it gets stuck. Static diagrams show you the rocks and the banks, but they don't show you how the water actually moves through the system.

That's what flows are for in architecture—they show you how information, data, and actions move through your system over time. Static relationships tell you what's connected. Flows tell you what happens.

In this lesson, you'll learn to model flows effectively. You'll discover different types of flows, when to use them, and how they reveal bottlenecks, errors, and opportunities that static diagrams miss.

Let's start by understanding what flows actually are and why they matter.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand what flows are and how they differ from static relationships
- Recognize different types of flows (data, user journeys, control, event)
- Know when to use flows versus static diagrams
- Model flows at the right level of detail
- Identify common pitfalls and avoid them

## What Are Flows, Really?

At its simplest level, a flow shows **sequence**—how something moves from point A to point B to point C. Unlike static relationships that just say "A is connected to B," flows tell you "A talks to B, then B talks to C, and here's what happens at each step."

### Static Relationship vs. Flow

Let me show you the difference:

```sruja
// Static relationship: Just shows connection
Customer -> Shop.WebApp "Uses"

// Flow: Shows the sequence
CheckoutFlow = scenario "Customer Checkout" {
  Customer -> Shop.WebApp "Submits order"
  Shop.WebApp -> Shop.API "Sends order data"
  Shop.API -> Shop.Database "Saves order"
  Shop.API -> PaymentGateway "Processes payment"
  PaymentGateway -> Shop.API "Returns result"
  Shop.API -> Shop.WebApp "Confirmation"
  Shop.WebApp -> Customer "Show success page"
}
```

The static relationship just tells you "customer uses the web app." The flow tells you the complete sequence—what the customer does, what the web app does, what the API does, in what order, and what happens at each step.

### Why This Matters

I once worked on a system where we had perfect static diagrams showing all the components. But nobody understood the actual order processing flow. When we debugged issues, we'd spend hours tracing through code because the diagrams didn't show sequence.

We added flows, and suddenly everything became clear. Developers could see the complete path from customer action to database storage. Product managers could see exactly what users experienced. Everyone had the same mental model of how things moved through the system.

## Why Flows Matter: The Real Benefits

After years of modeling systems, I've found flows reveal things static relationships never can. Let me show you what flows actually surface.

### 1. Data Lineage: Where Does Data Come From?

Flows show you the complete path data takes through your system—where it starts, how it's transformed, and where it ends up.

```sruja
OrderAnalyticsFlow = flow "Order Data Lineage" {
  // Where data starts
  Customer -> Shop.WebApp "Order details"
  
  // How data flows through system
  Shop.WebApp -> Shop.API "Order JSON payload"
  Shop.API -> Shop.Database "Persist order record"
  
  // Where data goes for analytics
  Shop.Database -> Analytics.Extractor "Extract order events"
  Analytics.Extractor -> Analytics.Processor "Enrich with user data"
  Analytics.Processor -> Analytics.Warehouse "Store aggregated metrics"
  
  // Where data is ultimately used
  Analytics.Warehouse -> Dashboard.Query "Fetch metrics"
  Dashboard.Query -> BusinessUser "Display analytics"
}
```

This flow tells the complete story: customer creates an order, order flows through the app and API, gets persisted to the database, gets extracted for analytics, gets enriched and aggregated, stored in the data warehouse, and ultimately shows up on a business dashboard.

Without this flow, would you know the order data goes to a data warehouse? Would you know there's an enrichment step? Would you know business users depend on this data? Probably not.

### 2. Process Understanding: What's the Sequence?

Flows show you the exact sequence of actions that happen when something occurs. This is crucial for understanding how your system actually works.

```sruja
OrderProcessFlow = scenario "Order Processing Sequence" {
  // Customer action
  Customer -> Shop.WebApp "Submits order"
  
  // API processing
  Shop.WebApp -> Shop.API "Validates cart"
  Shop.API -> Shop.Database "Checks inventory"
  Shop.Database -> Shop.API "Inventory available"
  
  // External integration
  Shop.API -> PaymentGateway "Charges payment"
  PaymentGateway -> Shop.API "Payment successful"
  
  // Order finalization
  Shop.API -> Shop.Database "Saves order"
  Shop.API -> InventoryService "Reserves items"
  Shop.API -> EmailService "Sends confirmation"
}
```

This flow reveals the complete sequence of what happens when an order is submitted. You can see exactly what the API does (validate, check inventory, charge, save, reserve, email). You can see the dependencies between steps (inventory must be available before charging). You can see the external integrations.

### 3. Bottleneck Identification: Where Can Things Slow Down?

Flows make bottlenecks obvious. You can see which steps might become slow, where queues might form, and where performance issues will surface first.

```sruja
FileUploadFlow = scenario "File Upload" {
  // Fast steps
  User -> Frontend "Selects file and clicks upload"
  Frontend -> API "Sends file data"
  API -> Storage "Stores file" [fast]
  
  // Potential bottleneck
  Storage -> ProcessingService "Processes file" [slow, cpu-intensive]
  
  // Continues if processing succeeds
  ProcessingService -> Notification "Sends completion notification"
  Notification -> User "Receives notification"
}
```

Look at this flow. Storage is fast. Processing service is marked as "slow" and "CPU-intensive." This tells you immediately: **the processing service is the bottleneck**. If users complain about slow uploads, you know exactly where to look.

I've used this pattern countless times. A team would spend weeks optimizing the "fast" parts of the system while ignoring the actual bottleneck. Flows make bottlenecks obvious.

### 4. Error Paths: What Happens When Things Fail?

Static relationships show you the happy path. Flows let you model what happens when things go wrong.

```sruja
OrderWithErrorsFlow = scenario "Order Processing with Error Handling" {
  Customer -> Shop.API "Submits order"
  Shop.API -> PaymentGateway "Charges payment"
  
  // Success path
  PaymentGateway -> Shop.API "Payment successful"
  Shop.API -> Shop.Database "Saves order"
  Shop.API -> EmailService "Sends confirmation"
  
  // Failure path
  PaymentGateway -> Shop.API "Payment declined"
  Shop.API -> Shop.WebApp "Returns error"
  Shop.WebApp -> Customer "Shows error message: Payment declined"
}
```

This flow models both the success path and the failure path. When the payment gateway returns a decline, the API returns an error to the web app, which displays it to the customer.

Modeling error paths is crucial. I once worked on a system where we'd only modeled happy paths. When failures occurred, we had no clear strategy for what should happen. Chaos ensued. Model both paths upfront.

## Types of Flows You'll Use

After modeling systems for years, I've found there are really four main types of flows you'll encounter. Understanding which type you're modeling helps you get the details right.

### 1. Data Flow (DFD Style)

Data flows show how data moves and transforms as it passes through your system. Think of them like a pipeline—data goes in one end, gets transformed, and comes out the other.

```sruja
OrderDataFlow = flow "Order Data Pipeline" {
  // Data originates from customer
  Customer -> Shop.WebApp "Order details"
  
  // Data flows through API as JSON
  Shop.WebApp -> Shop.API "Order JSON payload"
  
  // Data gets persisted as record
  Shop.API -> Shop.Database "Order record"
  
  // Data extracted as event
  Shop.Database -> Analytics.EventStream "Order event"
  
  // Data aggregated for reporting
  Analytics.EventStream -> Analytics.Aggregator "Daily aggregation"
  Analytics.Aggregator -> Analytics.Warehouse "Stored metrics"
}
```

**Use data flows when:**
- Modeling data lineage (where data comes from and goes)
- Documenting ETL processes
- Designing analytics pipelines
- Understanding data transformations

**Characteristics:**
- Focus on data, not actions
- Show how data changes shape/form
- Include storage and processing steps

### 2. User Journey / Scenario (BDD Style)

User journeys show how a person interacts with your system to achieve a goal. These are behavioral flows from the user's perspective.

```sruja
CheckoutJourney = scenario "Customer Checkout Experience" {
  // User actions
  Customer -> Shop.WebApp "Clicks checkout button"
  Customer -> Shop.WebApp "Enters shipping address"
  Customer -> Shop.WebApp "Enters payment details"
  Customer -> Shop.WebApp "Clicks 'Place Order'"
  
  // System responses
  Shop.WebApp -> Shop.API "Validates cart"
  Shop.API -> PaymentGateway "Processes payment"
  PaymentGateway -> Shop.API "Payment successful"
  Shop.API -> Shop.Database "Saves order"
  
  // Final user experience
  Shop.WebApp -> Customer "Shows order confirmation page"
  Shop.API -> EmailService "Sends confirmation email"
  EmailService -> Customer "Receives confirmation email"
}
```

**Use user journeys when:**
- Modeling user stories and requirements
- Designing test scenarios
- Understanding customer experience
- Documenting acceptance criteria

**Characteristics:**
- User's perspective, not system's
- Include both user actions and system responses
- Show the complete experience from start to finish

### 3. Control Flow

Control flows show decision points and branching logic. They model the "if this, then that" parts of your system.

```sruja
ApprovalFlow = scenario "Order Approval Workflow" {
  Order -> ApprovalService "Submits for approval"
  
  // Branch 1: Auto-approved (low value)
  ApprovalService -> Database "Saves as auto-approved" [if value < $100]
  
  // Branch 2: Manual review (high value)
  ApprovalService -> Manager "Sends approval request" [if value >= $100]
  Manager -> ApprovalService "Approves order"
  ApprovalService -> Database "Saves as approved"
  
  // Both paths converge
  Database -> EmailService "Sends order confirmation"
}
```

**Use control flows when:**
- Modeling business logic and rules
- Documenting decision trees
- Understanding conditional paths
- Designing workflow systems

**Characteristics:**
- Show decision points (if/else logic)
- Include multiple branches that may converge
- Document conditions for each branch

### 4. Event Flow

Event flows show how events propagate through an event-driven system. They model pub/sub patterns and event sourcing architectures.

```sruja
OrderEventFlow = flow "Order Event Propagation" {
  // Event published
  OrderAPI -> EventBus "Publishes OrderCreated event"
  
  // Multiple consumers process same event
  EventBus -> NotificationService "Consumes event (sends confirmation)"
  EventBus -> AnalyticsService "Consumes event (tracks metrics)"
  EventBus -> InventoryService "Consumes event (reserves items)"
  EventBus -> EmailService "Consumes event (sends marketing email)"
}
```

**Use event flows when:**
- Modeling event-driven architectures
- Designing pub/sub systems
- Understanding event propagation
- Documenting event sourcing patterns

**Characteristics:**
- One event, multiple consumers
- Asynchronous processing
- Eventual consistency (events processed at different times)

## Flow Patterns You'll See

Beyond types, there are structural patterns for how flows behave. Understanding these helps you recognize and model common scenarios.

### Linear Flow

The simplest pattern—things happen one after another in a straight line.

```sruja
LinearFlow = scenario "Simple Three-Step Process" {
  Step1 -> Step2 "Process A"
  Step2 -> Step3 "Process B"
  Step3 -> End "Process C"
}
```

**Characteristics:**
- Easy to understand
- No parallel processing
- Single point of failure
- Common in simple workflows

### Branching Flow

One step branches into multiple possible paths. This is where your control flow logic lives.

```sruja
BranchingFlow = scenario "Conditional Processing" {
  Start -> DecisionPoint "Initial processing"
  
  // Branch A: High priority
  DecisionPoint -> FastProcessor "Process immediately" [if priority = high]
  FastProcessor -> End "Complete"
  
  // Branch B: Low priority
  DecisionPoint -> Queue "Add to queue" [if priority = low]
  Queue -> SlowProcessor "Process when available"
  SlowProcessor -> End "Complete"
}
```

**Characteristics:**
- Multiple possible paths
- Based on conditions
- Paths may have different characteristics
- Common in workflows with approvals

### Converging Flow

Multiple paths start separately but eventually come back together.

```sruja
ConvergingFlow = scenario "Parallel then Merge" {
  // Parallel paths
  Start -> PathA "Process A"
  Start -> PathB "Process B"
  
  // Both paths converge
  PathA -> MergePoint "Contributes data"
  PathB -> MergePoint "Contributes data"
  
  // Single continuation
  MergePoint -> End "Combine and complete"
}
```

**Characteristics:**
- Parallel processing possible
- Multiple contributions to single result
- Synchronization point at convergence
- Common in gather-aggregate patterns

### Looping Flow

A step repeats multiple times until a condition is met.

```sruja
RetryingFlow = scenario "Payment with Retry Logic" {
  API -> PaymentGateway "Process payment"
  
  // First attempt fails
  PaymentGateway -> API "Payment failed: timeout"
  
  // Loop: retry up to 3 times
  API -> PaymentGateway "Retry payment" [attempt 2]
  PaymentGateway -> API "Payment failed: timeout"
  
  API -> PaymentGateway "Retry payment" [attempt 3]
  PaymentGateway -> API "Payment successful"
  
  // Exit loop
  API -> Database "Save order"
}
```

**Characteristics:**
- Repeated execution
- Exit condition required
- Common in retry logic and polling

## Creating Flows in Sruja

Sruja gives you three keywords for flows, and they're essentially interchangeable. Use whichever makes semantic sense for what you're modeling.

### Using `scenario`

Use `scenario` for behavioral flows, especially user journeys and BDD-style scenarios.

```sruja
MyScenario = scenario "User Logs In" {
  User -> WebApp "Enters credentials"
  WebApp -> API "Submits login"
  API -> Database "Verifies user"
  Database -> API "User found"
  API -> WebApp "Returns session token"
  WebApp -> User "Shows dashboard"
}
```

### Using `story`

Use `story` as an alias for `scenario`. It's the same thing, just a different keyword that some teams prefer for user stories.

```sruja
MyStory = story "As a customer, I want to purchase products" {
  Customer -> WebApp "Browses products"
  Customer -> WebApp "Adds to cart"
  Customer -> WebApp "Checks out"
  WebApp -> API "Processes order"
  API -> Customer "Shows confirmation"
}
```

### Using `flow`

Use `flow` for data-oriented flows, especially DFD-style data flows and event flows.

```sruja
MyFlow = flow "Data Processing Pipeline" {
  Source -> Ingestion "Raw data"
  Ingestion -> Processing "Transformed data"
  Processing -> Storage "Stored data"
  Storage -> Analytics "Query results"
}
```

## When to Use Flows (And When Not To)

Flows are powerful, but they're not always the right tool. Here's how I decide.

### Use Flows When

- **Sequence matters** — You need to show the order in which things happen
- **Modeling data lineage** — You need to track where data comes from and goes
- **Documenting user journeys** — You need to understand the user experience
- **Understanding process steps** — You need to see the complete workflow
- **Identifying bottlenecks** — You need to find where things slow down
- **Modeling error paths** — You need to show what happens when things fail

### Don't Use Flows When

- **Showing general connections** — Static relationships are better for showing overall architecture
- **High-level overview** — Flows have too much detail for executive diagrams
- **Simple systems** — If everything connects to everything, a flow doesn't add clarity
- **Static structure** — If you're showing what components exist, not what they do, use static relationships

## Pitfalls to Avoid (I've Made All of These)

Let me share some mistakes I've made and seen others make. Hopefully, you can avoid them.

### Mistake 1: Too Much Detail

```sruja
// Bad: Way too detailed
LoginFlow = scenario "User Login (Too Detailed)" {
  User -> UI "Clicks login button"
  UI -> API "HTTP POST /login"
  API -> Database "SELECT * FROM users WHERE email = ?"
  Database -> API "Returns user record"
  API -> AuthService "Verifies password hash"
  AuthService -> API "Password matches"
  API -> TokenService "Generates JWT token"
  TokenService -> API "Returns token"
  API -> UI "JSON response with token"
  UI -> User "Shows dashboard"
}
```

This is ridiculous. We're showing database queries, HTTP methods, password hashing. This is implementation detail, not architecture. Nobody reading a diagram needs this level of detail.

**Better approach:** Group related steps together.
```sruja
// Good: Right level of detail
LoginFlow = scenario "User Login" {
  User -> UI "Submits credentials"
  UI -> API "Authenticates user"
  API -> Database "Verifies credentials"
  Database -> API "User found"
  API -> TokenService "Creates session"
  API -> UI "Returns token"
  UI -> User "Shows dashboard"
}
```

### Mistake 2: Too Abstract

```sruja
// Bad: Not useful
ProcessFlow = scenario "Data Processing (Too Abstract)" {
  Start -> End "Data gets processed"
}
```

This tells you nothing. What kind of data? How is it processed? What happens in between? Useless.

**Better approach:** Add meaningful intermediate steps that explain what's actually happening.
```sruja
// Good: Meaningful steps
ProcessFlow = scenario "Order Processing" {
  Order -> API "Submits order"
  API -> Payment "Processes payment"
  Payment -> Database "Saves order"
  Database -> Email "Sends confirmation"
}
```

### Mistake 3: Mixing Flows with Static Relationships

```sruja
// Bad: Confusing mix
Customer -> Shop.WebApp "Uses"  // Static relationship

CheckoutFlow = scenario "Checkout" {
  Customer -> Shop.WebApp "Submits order"  // Flow
  Shop.WebApp -> Shop.API "Sends data"
}
```

This is confusing. You have both a static relationship and a flow between the same elements. Readers won't know which to look at or what the difference is.

**Better approach:** Keep flows and static relationships separate. Use flows for sequences, static relationships for overall structure.
```sruja
// Static architecture view
view architecture {
  title "System Architecture"
  include *
}

// Flow view
view checkout_flow {
  title "Checkout Sequence"
  CheckoutFlow = scenario "Checkout" {
    Customer -> Shop.WebApp "Submits order"
    Shop.WebApp -> Shop.API "Sends data"
  }
}
```

### Mistake 4: One Path Assumes Success

```sruja
// Bad: Only shows happy path
OrderFlow = scenario "Order Processing" {
  Customer -> Shop.API "Submits order"
  Shop.API -> PaymentGateway "Charges payment"
  PaymentGateway -> Shop.API "Success"
  Shop.API -> Database "Saves order"
}
```

This only works if everything goes perfectly. But what happens if payment fails? What if the database is down? What if the API times out?

**Better approach:** Model both success and failure paths.
```sruja
// Good: Shows both paths
OrderFlow = scenario "Order Processing with Errors" {
  Customer -> Shop.API "Submits order"
  Shop.API -> PaymentGateway "Charges payment"
  
  // Success path
  PaymentGateway -> Shop.API "Payment successful"
  Shop.API -> Database "Saves order"
  Shop.API -> Email "Sends confirmation"
  
  // Failure path
  PaymentGateway -> Shop.API "Payment declined"
  Shop.API -> Shop.WebApp "Returns error"
  Shop.WebApp -> Customer "Shows error message"
}
```

## What to Remember

Flows reveal what static relationships cannot. They show sequence, transformation, bottlenecks, and error paths. When you create flows:

- **Show the sequence** — Not just what's connected, but in what order
- **Model both paths** — Success and failure, not just happy paths
- **Use the right type** — Data flow for lineage, user journey for experience, control flow for logic
- **Right level of detail** — Not too deep (implementation), not too shallow (useless)
- **Label meaningfully** — Describe what's actually happening, not generic actions
- **Separate from static** — Flows complement, don't replace, static architecture

If you take away one thing, let it be this: **flows tell the story of how things move through your system over time**. Static relationships show the stage. Flows show the performance.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're documenting an order processing system. Which type of flow is most appropriate for modeling the complete customer experience from browsing products to receiving an order confirmation?

> "A customer browses products, adds items to cart, proceeds to checkout, enters payment details, and places the order. If payment succeeds, they see a confirmation page and receive a confirmation email. If payment fails, they see an error message."

**A)** Data Flow
**B)** User Journey / Scenario
**C)** Control Flow
**D)** Event Flow

<details>
<summary>Click to see the answer</summary>

**Answer: B) User Journey / Scenario**

Let's analyze each option:

**A)** Incorrect. Data flows focus on how data moves and transforms through the system. While data is involved in this scenario, the focus is on the **customer's experience** and actions, not on data transformations. A data flow would show order data flowing through the API, database, and analytics systems—not the customer's actions.

**B)** Correct! A user journey (or scenario) is the right choice here because:
- The scenario describes **user actions** (browses, adds to cart, proceeds to checkout, enters payment, places order)
- It shows **system responses** to those actions (confirmation page, error message, confirmation email)
- It includes **both success and failure paths** from the user's perspective
- It captures the **complete customer experience** from start to finish

User journeys are behavioral flows from the user's perspective. They're perfect for modeling user stories, requirements, and customer experiences.

**C)** Incorrect. Control flows show decision points and branching logic (if/else). While this scenario has branching (success vs. failure), the branching is based on external payment gateway response, not internal business logic rules. Control flows are better for modeling things like "if order value > $100, require approval" or "if user is VIP, skip verification."

**D)** Incorrect. Event flows show how events propagate through event-driven systems (pub/sub patterns). This scenario doesn't describe an event-driven architecture—it describes a synchronous request/response flow where the customer waits for a response. Event flows would show things like "order created event published → multiple services consume event → eventual consistency."

**Key insight:** Choose flow types based on what you're modeling. Showing the customer's experience and actions? Use a user journey. Tracking data lineage and transformations? Use a data flow. Modeling business logic decisions? Use a control flow. Designing event-driven architecture? Use an event flow.

</details>

---

### Question 2

You're creating a flow to model how order data moves through your system from creation to analytics. The data originates from the customer, gets stored in the transactional database, gets extracted daily by an ETL job, gets transformed and aggregated, and ends up in the data warehouse for reporting. Which flow type is most appropriate?

**A)** User Journey / Scenario
**B)** Control Flow
**C)** Data Flow
**D)** Event Flow

<details>
<summary>Click to see the answer</summary>

**Answer: C) Data Flow**

Let's analyze each option:

**A)** Incorrect. A user journey focuses on the user's perspective and actions. This scenario is about **data lineage and transformations**, not about a user's experience. The customer creates the order, but the rest of the flow (ETL, transformation, aggregation, data warehouse) happens automatically without user involvement. A user journey wouldn't capture these data processing steps effectively.

**B)** Incorrect. Control flows show decision points and branching logic (if/else). This scenario describes a **pipeline** where data flows through sequential steps, not a decision tree with branches. There's no conditional logic—every order follows the same path through ETL, transformation, aggregation, and the data warehouse.

**C)** Correct! A data flow is the right choice here because:
- The scenario describes **data lineage** — where data starts (customer), where it goes (database, ETL, warehouse), and where it ends (reporting)
- It shows **data transformations** — order record → ETL extraction → transformation → aggregation → warehouse data
- It models a **pipeline** — sequential steps that process data
- It's focused on **data movement**, not user actions or business logic

Data flows (DFD-style) are perfect for showing how data moves through a system, including where it originates, how it's stored, how it's transformed, and where it ultimately goes.

**D)** Incorrect. An event flow would show how an event (like "OrderCreated") propagates through an event-driven system with multiple consumers. While event-driven systems can use data flows internally, this scenario describes an **ETL pipeline** with scheduled batch processing, not real-time event propagation. The ETL job pulls data daily, transforms it, and pushes it to the warehouse—this is classic data flow, not event flow.

**Key insight:** Data flows are all about lineage and transformation. If you're modeling where data comes from, how it changes shape, and where it ends up, use a data flow. Think of it like tracing the path data takes through a pipeline.

</details>

---

## What's Next?

Now you understand what flows are and why they matter. You know the different types of flows (data, user journey, control, event) and when to use each one.

But we've only talked about **what** flows are. We haven't talked about **how** to create specific types of flows in practice.

In the next lesson, you'll learn about **Data Flow Diagrams**—how to model DFD-style data flows that show lineage, transformations, and analytics pipelines. You'll discover how to document where data comes from, how it changes, and where it ultimately ends up.

See you there!
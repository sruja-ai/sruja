---
title: "Lesson 6: Advanced DSL Features"
weight: 6
summary: "Master views, scenarios, flows, and element kinds to create production-ready models."
---

# Lesson 6: Advanced DSL Features

Six months into a project, a new architect joined our team. During onboarding, she asked a simple question: "Why did we choose PostgreSQL instead of MongoDB?"

I stared at her. Then I stared at the architecture diagrams. Then I dug through old Slack channels, Jira tickets, and Google Docs.

Two hours later, I found a comment from eight months ago: "PostgreSQL because we need ACID transactions for payments."

That was it. No context about what alternatives we considered. No explanation of the trade-offs. No record of who made the decision or why.

We had beautiful architecture diagrams. We had detailed API specs. We had comprehensive test coverage. But we had **no memory of our decisions**.

That's when I learned: **architecture without context is just pretty pictures**. You need to capture not just WHAT you built, but WHY you built it that way.

This lesson covers the advanced Sruja features that transform your diagrams from "nice drawings" into "living documentation" that tells the full story.

## The Five Pillars of Complete Architecture

A production-ready architecture model needs five things:

1. **Kinds & Types** - Define your vocabulary (what elements exist)
2. **Views** - Show different perspectives (who needs to see what)
3. **Scenarios** - Model user behavior (how people use it)
4. **Flows** - Model data movement (how data flows)
5. **Decisions** - Document why (requirements, ADRs, policies)

Let me walk you through each one.

---

## 1. Kinds and Types: Define Your Vocabulary

### The Naming Chaos Story

I once joined a project where everyone named things differently:

- "Database" vs "DB" vs "DataStore" vs "Persistence"
- "Service" vs "API" vs "Backend" vs "Server"
- "Queue" vs "MessageBus" vs "EventStream" vs "Broker"

The architecture had 47 elements. It felt like 47 different architects had named them.

When I tried to write validation rules like "all databases must be encrypted," I couldn't. Because "databases" were hidden among "DataStores," "DBs," and "Persistence Layers."

**The fix:** Define your vocabulary upfront. Decide what you'll call things, then stick to it.

### How Sruja Helps

Sruja's `stdlib` provides standard element kinds:

```sruja
import { * } from 'sruja.ai/stdlib'

// Now you have consistent vocabulary:
// - person, system, container, component
// - datastore, database, queue
// - flow, scenario, requirement, adr, policy
```

This isn't just about naming. It enables:

**1. Early Validation**

```sruja
// partial
// This will fail validation - "datasource" isn't a valid kind
MyDB = datasource "Database"  // ❌ Typo caught immediately

// Correct:
MyDB = datastore "Database"   // ✅ Validates successfully
```

**2. Better Tooling**
- Autocomplete knows what kinds exist
- Refactoring works across all instances
- IDE can suggest valid relationships

**3. Self-Documenting Models**
```sruja
// partial
// Anyone reading this knows these are the valid element types
// No guessing, no inconsistencies
Customer = person "Customer"
App = system "Application" {
  API = container "API"
  DB = datastore "Database"
}
```

### Best Practice

Import `stdlib` at the top of every model. This makes your vocabulary explicit and consistent.

---

## 2. Multiple Views: One Model, Many Perspectives

We covered this extensively in Lesson 4, so I'll keep this brief.

**The key insight:** Different audiences need different views. Executives need business context. Architects need service boundaries. Developers need implementation details.

**The mistake to avoid:** Creating separate models for each audience. Instead, create multiple `view` blocks from one model.

```sruja
// partial
// Define once...
Customer = person "Customer"
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = datastore "Database"
}

// ...show many ways
view executive {
  title "Executive Overview"
  include Customer
  include ECommerce
  // Hide technical details
  exclude ECommerce.WebApp ECommerce.API ECommerce.DB
}

view developer {
  title "Developer View"
  include ECommerce.WebApp ECommerce.API ECommerce.DB
  // Hide business context
  exclude Customer
}
```

**Quick tip:** Create views by:
- **Audience** (executive, architect, developer, product)
- **Concern** (security, data flow, performance)
- **Feature** (checkout, user management, analytics)

For detailed guidance, revisit Lesson 4.

---

## 3. Scenarios: Modeling User Behavior

### The Checkout Confusion Story

We were building an e-commerce platform. The architects drew beautiful boxes: Web App, API Service, Database, Payment Gateway. All connected with arrows.

Then product asked: "Can you walk us through the checkout flow?"

We stared at the diagram. There was an arrow from API to Database, and another from API to Payment Gateway. But in what order? What happened if payment failed? Did we reserve inventory before or after payment?

The diagram showed **structure** but not **behavior**.

That's when I learned: static diagrams need scenarios to show how the system actually works.

### What Scenarios Model

Scenarios capture **behavioral flows** - sequences of actions that happen when users interact with your system:

- User journeys (signup, checkout, search)
- Business processes (order fulfillment, refund processing)
- Error paths (payment failure, timeout handling)
- Use cases (admin creates user, customer updates profile)

### Real-World Example: E-Commerce Checkout

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

ECommerce = system "E-Commerce System" {
  WebApp = container "Web Application"
  API = container "API Service"
  OrderDB = datastore "Order Database"
}

Inventory = system "Inventory System" {
  InventoryService = container "Inventory Service"
}

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Happy path: User completes checkout successfully
CheckoutSuccess = scenario "Successful Checkout" {
  Customer -> ECommerce.WebApp "Adds items to cart"
  ECommerce.WebApp -> ECommerce.API "Submits checkout"
  ECommerce.API -> Inventory.InventoryService "Reserves stock"
  Inventory.InventoryService -> ECommerce.API "Confirms availability"
  ECommerce.API -> PaymentGateway "Processes payment"
  PaymentGateway -> ECommerce.API "Confirms payment"
  ECommerce.API -> ECommerce.OrderDB "Saves order"
  ECommerce.API -> ECommerce.WebApp "Returns confirmation"
  ECommerce.WebApp -> Customer "Shows order confirmation"
}

// Error path: Payment fails
CheckoutPaymentFailure = scenario "Payment Failure" {
  Customer -> ECommerce.WebApp "Attempts checkout"
  ECommerce.WebApp -> ECommerce.API "Submits checkout"
  ECommerce.API -> Inventory.InventoryService "Reserves stock"
  Inventory.InventoryService -> ECommerce.API "Confirms availability"
  ECommerce.API -> PaymentGateway "Processes payment"
  PaymentGateway -> ECommerce.API "Declines payment"
  ECommerce.API -> Inventory.InventoryService "Releases stock"
  ECommerce.API -> ECommerce.WebApp "Returns error"
  ECommerce.WebApp -> Customer "Shows payment error"
}

// Error path: Out of stock
CheckoutStockFailure = scenario "Out of Stock" {
  Customer -> ECommerce.WebApp "Attempts checkout"
  ECommerce.WebApp -> ECommerce.API "Submits checkout"
  ECommerce.API -> Inventory.InventoryService "Checks stock"
  Inventory.InventoryService -> ECommerce.API "Out of stock"
  ECommerce.API -> ECommerce.WebApp "Returns error"
  ECommerce.WebApp -> Customer "Shows out of stock message"
}

view index {
  include *
}
```

### Real-World Case Studies

**Amazon: Scenario-Driven Development**

Amazon uses scenarios extensively. Before writing any code, they write the "press release" - a narrative of the customer experience. This becomes scenarios that guide implementation:

1. Customer searches for product
2. Customer reads reviews
3. Customer adds to cart
4. Customer checks out
5. Customer receives order confirmation
6. Customer receives shipping notification
7. Customer receives package
8. Customer writes review

Each step has success and failure scenarios.

**Netflix: Error Scenarios First**

Netflix practices "Chaos Engineering" - they intentionally cause failures. Their scenarios focus heavily on error paths:

- What happens if the recommendation service is down?
- What happens if the CDN fails?
- What happens if the payment processor times out?

By modeling these scenarios upfront, they build resilient systems.

### When to Use Scenarios

**Use scenarios when:**
- Documenting user journeys
- Planning features with product managers
- Designing error handling
- Writing integration tests
- Onboarding new developers

**Skip scenarios when:**
- Simple CRUD operations (create, read, update, delete)
- Background processes with no user interaction
- Early exploration (add them later when design stabilizes)

---

## 4. Flows: Modeling Data Movement

### The Data Pipeline Mystery

We had a analytics dashboard showing "real-time" metrics. But the numbers were always 2 hours behind.

I traced the data flow:
1. Events logged to application database
2. Batch job every 2 hours extracts events
3. Batch job transforms and aggregates
4. Batch job loads into analytics warehouse
5. Dashboard queries warehouse

The architecture diagram showed: App → Database → Dashboard.

It didn't show the 2-hour batch pipeline, the transformation logic, or the intermediate staging tables.

**The diagram showed services, not data movement.**

That's when I learned: data-intensive systems need **flows** to show how data actually moves.

### What Flows Model

Flows capture **data-oriented processes** - how data moves, transforms, and gets stored:

- ETL pipelines (extract, transform, load)
- Streaming data (events, logs, metrics)
- Batch processing (daily aggregations, monthly reports)
- Data synchronization (between systems)

### Real-World Example: Analytics Pipeline

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Analytics = system "Analytics Platform" {
  IngestionService = container "Data Ingestion"
  ProcessingService = container "Data Processing"
  QueryService = container "Query Service"
  EventStream = queue "Event Stream"
  RawDataDB = datastore "Raw Data Store"
  ProcessedDataDB = datastore "Processed Data Warehouse"
}

// Real-time streaming flow
EventStreaming = flow "Real-time Event Streaming" {
  Analytics.IngestionService -> Analytics.EventStream "Publishes events"
  Analytics.EventStream -> Analytics.ProcessingService "Streams events"
  Analytics.ProcessingService -> Analytics.RawDataDB "Stores raw data"
  Analytics.ProcessingService -> Analytics.ProcessedDataDB "Aggregates in real-time"
  Analytics.QueryService -> Analytics.ProcessedDataDB "Queries analytics"
}

// Batch processing flow
DailyBatchProcessing = flow "Daily Batch Aggregation" {
  Analytics.RawDataDB -> Analytics.ProcessingService "Extracts daily data"
  Analytics.ProcessingService -> Analytics.ProcessingService "Transforms and aggregates"
  Analytics.ProcessingService -> Analytics.ProcessedDataDB "Loads aggregated data"
}

view index {
  include *
}
```

### Real-World Case Studies

**Uber: Real-time Data Flows**

Uber processes millions of events per second. Their flows show:

1. Driver location updates → Kafka → Real-time processing → Matching service
2. Ride requests → Pricing service → Surge calculation → User notification
3. Trip completion → Billing → Receipt generation → Email service

Each flow has different latency requirements (real-time vs. batch).

**Spotify: Batch + Streaming**

Spotify uses a "Lambda Architecture" with two flows:

1. **Speed layer** (real-time): Events → Streaming → Fast approximate results
2. **Batch layer** (daily): Events → Hadoop → Accurate complete results

Both flows serve queries, with different trade-offs.

### Scenario vs. Flow: When to Use Which

This is the question I get most often. Here's the framework:

**Use SCENARIOS for:**
- ✅ User actions and behavior
- ✅ Business processes
- ✅ Request/response interactions
- ✅ Sequential user journeys
- ❌ Data pipelines

**Use FLOWS for:**
- ✅ Data movement and transformation
- ✅ ETL processes
- ✅ Event streaming
- ✅ Batch processing
- ❌ User interactions

**Quick test:** 
- Does a person initiate this? → Scenario
- Does data move between systems automatically? → Flow

**Example:**

```sruja
// partial
// User behavior = SCENARIO
UserCheckout = scenario "User Checks Out" {
  Customer -> WebApp "Clicks checkout"
  WebApp -> API "Processes payment"
}

// Data movement = FLOW
PaymentReconciliation = flow "Daily Payment Reconciliation" {
  TransactionDB -> ReconciliationService "Extracts transactions"
  ReconciliationService -> PaymentGateway "Compares with gateway records"
  ReconciliationService -> ReportingDB "Stores discrepancies"
}
```

---

## 5. Requirements, ADRs, and Policies: Documenting Decisions

### The "Why Did We Do This?" Problem

Remember my story at the beginning? The one about reverse-engineering why we chose PostgreSQL?

That's the problem ADRs (Architecture Decision Records) solve.

### Requirements: What We Need

Requirements capture **what the system must do**:

```sruja
import { * } from 'sruja.ai/stdlib'

// Functional requirements
R1 = requirement "Must handle 10k concurrent users" {
  tags ["functional"]
}

// Performance requirements
R2 = requirement "API response < 200ms p95" {
  tags ["performance"]
}

// Scalability requirements
R3 = requirement "Scale to 1M users" {
  tags ["scalability"]
}

// Security requirements
R4 = requirement "All PII encrypted at rest" {
  tags ["security"]
}
```

**Real-world example:** When Airbnb designed their booking system, they had requirements like:
- Must handle 1M+ bookings per day
- Must prevent double-booking
- Must support instant book and request-to-book
- Must handle timezone complexity

These requirements drove every architectural decision.

### ADRs: Why We Decided

ADRs capture **why we made specific architecture decisions**:

```sruja
ADR001 = adr "Use microservices for independent scaling" {
  status "Accepted"
  context "Need to scale order processing independently from inventory"
  decision "Split into OrderService and InventoryService"
  consequences "Better scalability, increased network complexity"
}

ADR002 = adr "Use PostgreSQL for strong consistency" {
  status "Accepted"
  context "Need ACID transactions for financial data"
  decision "Use PostgreSQL instead of NoSQL"
  consequences "Strong consistency, SQL complexity"
}
```

**ADR Structure:**
1. **Context** - What's the situation? What problem are we solving?
2. **Decision** - What did we decide to do?
3. **Consequences** - What are the trade-offs? What becomes easier/harder?
4. **Status** - Proposed, Accepted, Deprecated, Superseded?

**Real-world case study:** Netflix's ADR for "Use Chaos Engineering"

- **Context:** Need to ensure resilience in distributed systems
- **Decision:** Intentionally inject failures in production
- **Consequences:** Higher confidence in resilience, requires cultural shift, potential customer impact during experiments
- **Status:** Accepted

### Policies: How We Enforce

Policies capture **rules that must be followed**:

```sruja
SecurityPolicy = policy "All databases must be encrypted" {
  category "security"
  enforcement "required"
  description "Compliance requirement for PII data"
}

ScalingPolicy = policy "Services must implement health checks" {
  category "operations"
  enforcement "required"
  description "Required for auto-scaling"
}
```

**Real-world example:** Amazon's "Two-Pizza Team" policy:
- Each service owned by a team that can be fed by two pizzas (6-10 people)
- Enforcement: Service ownership must be documented
- Consequence: Services that grow too large must be split

### Putting It All Together

Here's how requirements, ADRs, and architecture integrate:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

// REQUIREMENTS: What we need
R1 = requirement "Handle 10k concurrent users" { tags ["functional"] }
R2 = requirement "API response < 200ms p95" { tags ["performance"] }
R3 = requirement "Scale to 1M users" { tags ["scalability"] }
R4 = requirement "All PII encrypted at rest" { tags ["security"] }

// ADRs: Why we decided
ADR001 = adr "Use microservices for independent scaling" {
  status "Accepted"
  context "Need to scale order processing independently from inventory"
  decision "Split into OrderService and InventoryService"
  consequences "Better scalability, increased network complexity"
}

ADR002 = adr "Use PostgreSQL for strong consistency" {
  status "Accepted"
  context "Need ACID transactions for financial data"
  decision "Use PostgreSQL instead of NoSQL"
  consequences "Strong consistency, SQL complexity"
}

// ARCHITECTURE: What we built
ECommerce = system "E-Commerce Platform" {
  API = container "API Service" {
    technology "Rust"
    description "Satisfies R1, R2, R3"
    
    slo {
      availability {
        target "99.99%"
        window "30 days"
      }
      latency {
        p95 "200ms"
        p99 "500ms"
      }
    }
  }
  
  OrderDB = datastore "Order Database" {
    technology "PostgreSQL"
    description "Satisfies R4 - encrypted at rest"
  }
}

// POLICIES: What we enforce
SecurityPolicy = policy "All databases must be encrypted" {
  category "security"
  enforcement "required"
  description "Compliance requirement for PII data"
}

view index {
  include *
}
```

**The power:** When someone asks "Why did we use PostgreSQL?", you point to ADR002. When they ask "What requirement does this satisfy?", you point to R4. When they ask "Is this encrypted?", you point to SecurityPolicy.

---

## Common Mistakes I See All the Time

### Mistake #1: Inconsistent Element Types

**What happens:** One diagram has "Database", another has "DataStore", another has "DB".

**Why it fails:** 
- Validation rules can't be written
- Tooling breaks
- Confusion about what's what

**The fix:** Import `stdlib` and use consistent kinds everywhere.

### Mistake #2: Confusing Scenarios with Flows

**What happens:** You model "user checkout" as a flow, or "data pipeline" as a scenario.

**Why it fails:**
- Wrong mental model
- Confusing to readers
- Doesn't match how systems work

**The fix:** 
- User behavior = Scenario
- Data movement = Flow

### Mistake #3: No ADRs for Major Decisions

**What happens:** You make important architecture decisions but don't document why.

**Why it fails:**
- New team members don't understand context
- Can't evaluate if decision is still valid
- Repeat same discussions

**The fix:** Write an ADR for every significant architecture decision. If someone might ask "why did we do this?" in 6 months, write an ADR.

### Mistake #4: Requirements Not Linked to Architecture

**What happens:** Requirements live in a separate document, architecture in another.

**Why it fails:**
- Can't trace requirements to implementation
- Don't know if requirements are satisfied
- Changes to requirements don't trigger architecture review

**The fix:** Link requirements to architecture elements in the same model.

### Mistake #5: Too Many Views (or Too Few)

**What happens:** 
- Too many: 20 views, each slightly different, maintenance nightmare
- Too few: One giant view showing everything

**Why it fails:**
- Information overload or information scattered
- Hard to maintain

**The fix:** 
- Start with 3 views: executive, architect, developer
- Add more only when you have a specific audience in mind

### Mistake #6: Scenarios That Are Too Detailed

**What happens:** Scenarios show every database query, every cache check, every validation.

**Why it fails:**
- Too much noise
- Hard to see the user journey
- Becomes implementation documentation

**The fix:** Scenarios should show the **user's perspective**, not implementation details. Keep them at the business level.

---

## The COMPLETE Framework

When building production-ready architecture models, use this framework:

**C - Kinds & Components**
- Define element types upfront
- Use consistent vocabulary
- Import stdlib

**O - Organize with Views**
- Create views for different audiences
- Start with 3 (executive, architect, developer)
- Add more as needed

**M - Model Behavior with Scenarios**
- Document user journeys
- Include error paths
- Keep at business level

**P - Policies and ADRs**
- Document decisions with ADRs
- Link requirements to architecture
- Define enforcement policies

**L - Link Data Flows**
- Use flows for data pipelines
- Show ETL and streaming
- Separate from user scenarios

**E - Evaluate and Iterate**
- Review with team regularly
- Update when decisions change
- Keep single source of truth

---

## What to Remember

1. **Kinds define vocabulary** - Import stdlib, use consistent element types
2. **Views provide perspective** - One model, multiple views for different audiences
3. **Scenarios model behavior** - User journeys, business processes, error paths
4. **Flows model data** - ETL, streaming, batch processing
5. **ADRs capture why** - Document context, decision, and consequences
6. **Requirements link to architecture** - Trace from need to implementation
7. **Policies enforce rules** - Security, compliance, operational standards
8. **Scenarios ≠ Flows** - User behavior vs. data movement
9. **Keep scenarios high-level** - Business perspective, not implementation
10. **Write ADRs for major decisions** - If someone might ask "why?", document it

---

## When to Skip These Features

You don't always need everything. Here's when to simplify:

**Skip advanced features when:**
- **Prototyping** - Focus on structure, add details later
- **Simple systems** - Single service + database doesn't need scenarios
- **Early exploration** - Get the design right first, document later
- **Time pressure** - Better to have good diagrams than perfect documentation

**Always include:**
- Consistent element kinds (minimal effort, big payoff)
- At least 2 views (executive + developer)
- ADRs for major decisions (you'll thank yourself later)

---

**Next up:** Module 4 brings it all together with production readiness - how to make your architecture real, maintainable, and valuable.

👉 **[Module 4: Production Readiness](../module-4-production-readiness/lesson-1)** - Learn how to make your architecture production-ready.

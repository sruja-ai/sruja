---
title: "Lesson 1: Microservices Architecture"
weight: 1
summary: "The architecture that solved Netflix's scaling problem—and created new ones"
time: "12 min"
---

# The Distributed Monolith: When Microservices Go Wrong

It was 2 AM when the call came in. The payment service was down, which meant the entire e-commerce platform was down. No one could buy anything.

I pulled up the logs and traced the failure: Payment Service → Inventory Service → Notification Service → Email Service. The email service had crashed, which cascaded back through the entire chain, which brought down payments.

We had "microservices." We had separate services for everything. But what we really had was a distributed monolith—services that couldn't function independently, spread across multiple servers instead of one.

The outage lasted six hours. The post-mortem revealed the real problem: we'd adopted microservices without understanding what makes them work. We'd split by technical layers (web, API, database) instead of business capabilities. Every service depended on every other service. We'd gained all the complexity of distributed systems with none of the benefits.

This lesson is about avoiding that mistake. You'll learn when microservices actually help (hint: not always), how to draw service boundaries that make sense, and the real trade-offs that most tutorials skip over.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand what microservices really are (beyond the buzzword)
- Recognize when microservices help and when they hurt
- Draw service boundaries based on business capabilities, not technical layers
- Model microservices architecture in Sruja with multiple views
- Avoid the distributed monolith anti-pattern

## What Are Microservices, Really?

Let's cut through the hype. Microservices are small, independent services that:
- **Own their own data** (no shared databases)
- **Communicate through APIs** (not direct database calls)
- **Can be deployed independently** (without coordinated releases)
- **Scale independently** (add more instances of just the busy service)
- **Fail independently** (one service crashing doesn't bring down everything)

The key word is **independently**. If your services can't operate independently, they're not microservices—they're just a distributed monolith with extra steps.

Here's the uncomfortable truth that took me years to learn: **Microservices are not an architecture. They're a trade-off.**

You gain:
- Independent scaling (scale just what needs scaling)
- Technology diversity (different services can use different stacks)
- Fault isolation (one service failure doesn't cascade)
- Team autonomy (different teams own different services)

You pay with:
- Distributed system complexity (network calls instead of function calls)
- Data consistency challenges (no more ACID transactions across services)
- Operational overhead (monitoring, logging, tracing across services)
- Cognitive load (understanding the whole system is harder)

## The Monolith: Not Evil, Just Misunderstood

Monoliths get a bad reputation, but let me share a secret: **most successful companies started with a monolith.**

Amazon started with a monolith. Netflix started with a monolith. Uber started with a monolith. Facebook, Twitter, Airbnb—they all started with a single application that did everything.

There's a good reason: **Monoliths are easier to build, deploy, and debug.**

When you're a startup with three engineers trying to find product-market fit, you don't need microservices. You need to ship features fast. You need to iterate quickly. You need to change your mind without refactoring 50 services.

**Monoliths work well when:**
- Your team is small (< 20 engineers)
- Your domain is not well-understood yet (you're still learning)
- Your traffic is predictable and moderate
- Your deployment velocity is more important than independent scaling
- You want to move fast and break things in one place

**Monoliths break down when:**
- Different parts of the system need different scaling
- Teams are stepping on each other's toes (coordinated deployments take forever)
- One part of the system is unstable and brings down everything
- You need different technologies for different parts (e.g., Python for ML, Go for high-throughput APIs)

The monolith isn't wrong. It's just a tool with specific use cases. Don't adopt microservices because you think monoliths are "bad." Adopt them because you have specific problems they solve.

## The Microservices Promise (And Hidden Costs)

Let me share three real stories about microservices—the good, the bad, and the ugly.

### The Good: Amazon's Two-Pizza Teams

In 2002, Amazon's CEO Jeff Bezos issued a famous memo (often called the "API Mandate") that fundamentally changed how Amazon operated:

1. All teams will expose their data and functionality through service interfaces
2. Teams must communicate with each other through these interfaces
3. No other form of interprocess communication is allowed
4. It doesn't matter what technology you use
5. All service interfaces must be designed to be externalizable

This wasn't about technology—it was about organizational structure. By forcing teams to communicate through APIs, Amazon created independent teams that could move fast without coordinating with each other. The famous "two-pizza team" rule (a team should be small enough to be fed by two pizzas) became possible because services provided clear boundaries.

**The result:** Amazon went from a slow-moving retailer to a platform that could launch new services rapidly. Each team owned their service end-to-end. No coordination required.

**The key insight:** Microservices enabled organizational scaling, not just technical scaling.

### The Bad: The Startup That Microserviced Too Early

A startup I advised had five engineers and decided to build their MVP as microservices from day one. They'd read about Netflix and Amazon and wanted to "do it right from the start."

Six months later, they'd built eight services for a product that barely had users. Every feature change required coordinating across multiple services. Every deployment was a multi-service rollout. Debugging meant tracing logs across five different systems.

They spent more time managing services than building features. When they finally simplified to a monolith, their development velocity doubled.

**The lesson:** You don't start with microservices. You evolve into them when you need them.

### The Ugly: The Distributed Monolith Disaster

I worked with a company that had 50 "microservices" but they were all tightly coupled:
- Services called each other synchronously in chains
- They shared a single database (just different schemas)
- A deploy of any service required deploying all services
- When one service crashed, everything crashed

They had all the costs of microservices with none of the benefits. It was a monolith spread across 50 codebases.

**The fix:** We consolidated into 8 services based on business capabilities, gave each its own database, and made them truly independent. The system became simpler and more reliable.

**The lesson:** More services ≠ better architecture. The goal is independence, not count.

## When to Use Microservices (And When NOT To)

After years of watching teams struggle with this decision, I've developed a simple framework.

**Use microservices when:**

1. **You have clear business boundaries** (e.g., Payments, Shipping, User Management are obviously separate)
2. **Different parts need different scaling** (e.g., Search service needs 100x more instances than User Profile service)
3. **Multiple teams need independent deployment** (Team A shouldn't wait for Team B's deployment)
4. **Different technology needs** (e.g., Python for ML, Go for APIs, Node.js for web)
5. **Fault isolation is critical** (e.g., Recommendations can fail without breaking Checkout)

**Don't use microservices when:**

1. **You're still exploring the domain** (You don't know the boundaries yet)
2. **Your team is small** (< 10-20 engineers)
3. **You don't have operational maturity** (Monitoring, logging, tracing, deployment automation)
4. **Simple deployment matters more than independent scaling**
5. **You're doing it because "that's what Netflix does"**

The biggest mistake I see? Teams adopting microservices because they think it's "modern" or "best practice." Architecture should solve specific problems, not follow trends.

## The Hardest Problem: Drawing the Lines

Let's say you've decided to use microservices. Now comes the hard part: **Where do you draw the boundaries?**

This is where most teams fail. They split by technical layers:
- Frontend Service
- Backend Service
- Database Service
- Cache Service

This is wrong. These aren't business capabilities—they're technical implementation details. You end up with services that can't function independently.

**The right approach: Split by business capability.**

A business capability is something the business does that generates value. Ask: "What does this business actually do?"

For an e-commerce platform:
- **Catalog** - Manage products and inventory
- **Orders** - Process customer orders
- **Payments** - Handle payment processing
- **Shipping** - Coordinate delivery
- **Customers** - Manage user accounts and profiles

Each of these can be an independent service that owns its data and logic. If Payments crashes, Orders can still be taken (just processed later). If Shipping is slow, it doesn't affect browsing the catalog.

## A Framework for Boundary Decisions

Here's the process I use when helping teams define service boundaries:

### Step 1: Identify Business Capabilities

List what the business actually does, not technical components:
- What are the main things users do?
- What different departments manage?
- What would exist even if technology was different?

### Step 2: Look for Natural Seams

Where does data naturally separate?
- Does Order data need to know about Shipping details?
- Can Customers exist without Orders?
- What data can be eventually consistent vs. strongly consistent?

### Step 3: Consider Team Structure

Who will own what?
- Can a single team own this service end-to-end?
- Does splitting this create coordination overhead?
- Does the org structure support these boundaries?

### Step 4: Start Bigger, Split Later

Begin with larger services and split as needed:
- Start with "Order Management" not "Order Placement" + "Order Tracking"
- Split only when you feel the pain of not splitting
- It's easier to split than to merge

### Step 5: Question Every Boundary

For each proposed service, ask:
- Can this deploy independently?
- Can this scale independently?
- Does this own its data?
- Would I need to coordinate with other teams to change this?

If the answer is no, reconsider the boundary.

## Real-World Stories: How Amazon and Netflix Did It

### Amazon: The Service-Oriented Evolution

Amazon's journey to microservices wasn't a rewrite—it was an evolution:

**2002:** Bezos's API Mandate. All communication through APIs.
**2003-2005:** Gradual extraction of services from the monolith.
**2006:** Launch of AWS (infrastructure became a product).
**Today:** Hundreds of services, but each is owned by a specific team.

The key: They evolved gradually. They didn't rewrite everything at once.

### Netflix: The Chaos Engineering Pioneer

Netflix's microservices journey was driven by a specific problem:

**2008:** Single datacenter outage took down everything.
**2009:** Decision to move to AWS and microservices.
**2010-2011:** Massive migration, created Chaos Monkey to test resilience.
**Result:** 99.99%+ uptime, ability to survive AWS region failures.

The key: They had a specific problem (single point of failure) and microservices solved it.

**What both companies share:** They started with monoliths and evolved to microservices based on specific needs, not trends.

## Modeling Microservices in Sruja

Now that you understand the concepts, let's see how to model microservices in Sruja. The key is modeling each service as an independent unit with its own data.

### Basic Example: E-Commerce Platform

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// Customer interacts with the system
Customer = person "Customer"

// Order Service: Handles order placement and tracking
OrderSystem = system "Order Management" {
    OrderService = container "Order Service" {
        technology "Rust"
        description "Handles order placement and tracking."
    }
    OrderDB = database "Order Database" {
        technology "PostgreSQL"
        description "Owns all order data. No other service accesses this."
    }
    OrderService -> OrderDB "Reads/Writes"
}

// Inventory Service: Tracks stock levels
InventorySystem = system "Inventory Management" {
    InventoryService = container "Inventory Service" {
        technology "Java"
        description "Tracks stock levels and reservations."
    }
    InventoryDB = database "Inventory Database" {
        technology "PostgreSQL"
        description "Owns all inventory data. Independent from orders."
    }
    InventoryService -> InventoryDB "Reads/Writes"
}

// Inter-service communication through APIs
Customer -> OrderSystem.OrderService "Places order"
OrderSystem.OrderService -> InventorySystem.InventoryService "Reserves stock (async)"

// Requirements drive architecture
requirement R1 functional "Must handle 10k orders/day"
requirement R2 performance "Order placement < 500ms"
requirement R3 scalability "Scale order processing independently from inventory"

// Document architecture decisions
adr ADR001 "Split into microservices" {
    status "Accepted"
    context "Need independent scaling for order vs inventory"
    decision "Separate OrderSystem and InventorySystem"
    consequences "Better scalability, but added network latency and operational complexity"
}
```

Notice how each system owns its database. This is critical—shared databases defeat the purpose of microservices.

### Multiple Views for Different Audiences

One of the most powerful features of Sruja is the ability to create different views for different audiences:

```sruja
// partial
// System Overview: Shows the big picture
view index {
    title "System Overview"
    include *
}

// Developer View: Focus on services and APIs
view developer {
    title "Developer View - Service Architecture"
    include OrderSystem.OrderService OrderSystem.OrderDB
    include InventorySystem.InventoryService InventorySystem.InventoryDB
    exclude Customer
}

// Product View: Focus on user experience
view product {
    title "Product View - User Journey"
    include Customer
    include OrderSystem
    exclude InventorySystem.InventoryDB
}

// Data Flow View: Show data dependencies
view dataflow {
    title "Data Flow View"
    include OrderSystem.OrderService OrderSystem.OrderDB
    include InventorySystem.InventoryService InventorySystem.InventoryDB
    exclude Customer
}
```

**Why multiple views matter:**
- **Different audiences** need different levels of detail
- **Reduced complexity** - each view shows only what's relevant
- **Better communication** - stakeholders get diagrams they understand
- **Living documentation** - views stay in sync with the architecture

## Common Microservices Mistakes

After years of working with microservices, I've seen these patterns repeat:

**Mistake 1: Shared Database**
Services share a database "for convenience." Result: You can't change the database schema without breaking multiple services. Each service should own its data.

**Mistake 2: Synchronous Everything**
Every service calls every other service synchronously. Result: One slow service brings down everything. Use async communication where possible.

**Mistake 3: Too Many Services**
50 services for a simple application. Result: Complexity explosion. Start with fewer, larger services and split when needed.

**Mistake 4: Distributed Monolith**
Services that can't function independently. Result: All the costs, none of the benefits. If a service can't deploy without others, it's not a microservice.

**Mistake 5: Early Optimization**
Starting with microservices "because we'll need them." Result: Wasted time and complexity you don't need. Build what you need, when you need it.

## What to Remember

**Microservices are a trade-off, not a best practice.** You gain independence but pay in complexity. Don't adopt them because they're trendy—adopt them because you have specific problems they solve.

**Start with a monolith.** Amazon, Netflix, Uber, Google—they all started with monoliths. Evolve to microservices when you feel the pain of not having them.

**Split by business capability, not technical layer.** Services should represent business domains (Orders, Payments, Shipping), not technical components (Database, Cache, API).

**Each service owns its data.** Shared databases create coupling. If two services share a database, they're not independent.

**Multiple views serve multiple audiences.** Developers need technical details. Product managers need user flows. Executives need system overview. Sruja's views let you show the right level to each audience.

**The goal is independence, not service count.** More services ≠ better architecture. The question is: Can this service deploy, scale, and fail independently?

Microservices aren't wrong. But using them without understanding the trade-offs is.

## What's Next

Now that you understand microservices architecture, [Lesson 2](lesson-2.md) covers the API Gateway pattern—the traffic controller that manages communication between your services and the outside world. You'll learn when you need one, when you don't, and how to avoid creating a new bottleneck.

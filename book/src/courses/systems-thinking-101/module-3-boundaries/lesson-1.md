---
title: "Lesson 1: Understanding Boundaries"
weight: 1
summary: "What boundaries are and why they matter in software architecture"
time: "5 min"
---

# Drawing the Line: Understanding Boundaries

Ever played a game of tag as a kid? There was always that safe zone—home base—where you couldn't be tagged. Everything outside that zone was fair game, everything inside was safe. Boundaries in software architecture work the same way: they define what's safe inside your system and what's risky outside it.

In this lesson, you'll learn to draw those lines clearly. You'll discover why boundaries matter, what types of boundaries you'll encounter, and how to define them effectively in your architectures.

Let's start by understanding what boundaries actually are.

## Learning Goals

By the end of this lesson, you'll be able to:

- Define what boundaries are in the context of software architecture
- Recognize the different types of boundaries you'll encounter
- Understand why boundaries matter for ownership, risk, and clarity
- Apply boundaries effectively in your Sruja models

## What Are Boundaries, Really?

At its simplest level, a boundary is a line that separates what's **inside** your system from what's **outside**. But this line isn't arbitrary—it represents something meaningful:

- **Inside**: What you build, own, maintain, and control
- **Outside**: The environment, external dependencies, things you rely on but don't control

Think of it like your house:

```
┌─────────────────────────────────────┐
│          OUTSIDE                    │
│  (The neighborhood, other houses)  │
│                                     │
│    ┌───────────────────────────┐    │
│    │       YOUR HOUSE          │    │
│    │   (Your safe space)       │    │
│    │                           │    │
│    │  [Your rooms, stuff]      │    │
│    │                           │    │
│    └───────────────────────────┘    │
│             ↑                         │
│     Front door = Boundary Line       │
└─────────────────────────────────────┘
```

You control everything inside your house. You don't control the neighbor's house down the street. The front door is your boundary—it's clear where your space ends and public space begins.

In software, boundaries work the same way. They tell everyone what you're responsible for and what you're not.

## Why Boundaries Matter (The Real Reasons)

I've seen too many projects suffer from unclear boundaries. Teams argue about who owns what. Outages cascade because nobody planned for external failures. Security gaps emerge because nobody thought about what crosses the boundary.

Let me show you why boundaries matter in practice.

### 1. Clear Ownership: Who's Responsible?

This is the most common reason I see teams argue. When boundaries are unclear, nobody knows who fixes what.

```sruja
// partial
// Inside: Your team owns this
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API Service"
}

// Outside: Another team/vendor owns this
PaymentGateway = system "Payment Gateway"
```

When something breaks with payments, who fixes it? If the boundary is clear, everyone knows: **your team** fixes anything inside Shop, **the payment vendor** fixes anything in PaymentGateway. No confusion, no finger-pointing.

### 2. Risk Management: What Could Go Wrong?

Every time you cross a boundary, you're introducing risk. You're depending on something outside your control.

```sruja
// partial
// External dependency = external risk
Shop.API -> PaymentGateway "Process payment"

// If PaymentGateway is down, what happens?
// - Can customers still check out?
// - Do we have a backup?
// - What's our SLA with them?
```

When I model systems, I always ask: "What happens if this external thing breaks?" If I don't have a good answer, that's a risk I need to document and plan for.

### 3. Testing Scope: What Do We Test?

Boundaries tell you what kind of testing you need.

```sruja
// partial
// Internal: Unit tests are sufficient
Shop.WebApp -> Shop.API

// External: Integration tests are required
Shop.API -> PaymentGateway
```

You can unit test internal interactions all day. But when you cross a boundary? You need integration tests. You need to handle network failures. You need to test timeout scenarios. Boundaries tell you where testing gets more complex.

### 4. Security: What Needs Protection?

Security controls should be strongest at your boundaries. That's where attacks happen.

```sruja
// partial
// Inside boundary: Apply internal controls
Shop.WebApp -> Shop.API

// Crossing boundary: Validate everything
Shop.API -> PaymentGateway "Process payment" [authenticated, encrypted]
```

I learned this the hard way early in my career when I didn't properly validate data crossing a boundary and ended up with a security vulnerability. Lesson learned: **boundaries are where security matters most**.

## Types of Boundaries You'll Encounter

After years of modeling systems, I've noticed there are really five main types of boundaries you'll run into. Understanding which type you're dealing with helps you make the right decisions.

### 1. System Boundary: Your App vs. The World

This is the most common boundary—your main application versus everything else.

```sruja
// partial
// Inside: Your system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Outside: Everything else
Customer = person "Customer"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
```

The system boundary defines your overall scope. Everything inside is yours. Everything outside is not.

### 2. Team Boundary: What Your Team Owns

In larger organizations, different teams own different systems. These team boundaries matter for communication and coordination.

```sruja
// partial
// Your team's system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Another team's system
Analytics = system "Analytics Platform" {
  metadata {
    tags ["internal", "data-team"]
    owner "Data Team"
  }
}
```

When I see team boundaries, I always add metadata about who owns what. This prevents confusion when issues arise and everyone's trying to figure out who to call.

### 3. Organizational Boundary: Inside vs. Outside Your Company

Sometimes boundaries exist at the organizational level—your company versus external vendors.

```sruja
// Your company's system
Shop = system "Shop"

// External vendor
Stripe = system "Stripe" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe Inc."
  }
}
```

Vendor relationships are fundamentally different from internal relationships. Different SLAs, different support channels, different everything. Mark these boundaries clearly so everyone knows the difference.

### 4. Deployment Boundary: What Deploys Together

Sometimes the same team owns systems, but they deploy independently. That's a deployment boundary.

```sruja
// partial
// Same deployment
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Separate deployment
Database = system "Database Cluster" {
  metadata {
    tags ["internal", "dba-team"]
    deployment "Managed Service"
  }
}
```

Deployment boundaries tell you about failure domains. If the API and database deploy independently, they can fail independently. That's important to understand.

### 5. Trust Boundary: Security Zones

This is one of the most important boundaries from a security perspective. It defines what's trusted versus untrusted.

```sruja
// partial
// Trusted: Internal network, internal users
InternalAPI = container "Internal API"

// Untrusted: Public internet, anyone
PublicAPI = container "Public API"
```

I always pay special attention to trust boundaries. This is where you need authentication, authorization, encryption, and all the other security controls. Cross a trust boundary without proper security? That's how breaches happen.

## Real-World Examples

Let me show you some examples of boundaries in action.

### Example 1: A Typical E-Commerce Platform

```sruja
// partial
// ┌──────────── EXTERNAL WORLD ────────────┐
// │                                        │
// │   Customer (person)                    │
// │   Payment Gateway (system)             │
// │   Email Service (system)               │
// │   Analytics (system)                   │
// │                                        │
// │   ┌────── YOUR SYSTEM ──────────┐     │
// │   │ Shop (system)              │     │
// │   │   WebApp (container)       │     │
// │   │   API (container)          │     │
// │   │   Database (database)      │     │
// │   └───────────────────────────┘     │
// │                                        │
// └────────────────────────────────────────┘

// People are always outside
Customer -> Shop.WebApp "Browses"

// Everything inside your boundary
Shop.WebApp -> Shop.API "Makes requests"
Shop.API -> Shop.Database "Persists orders"

// Crossing your boundary to external systems
Shop.API -> PaymentGateway "Process payment"
Shop.API -> EmailService "Send confirmation"
Shop.API -> Analytics "Track events"
```

See how clear the boundary is? Everything inside "Your System" is yours. Everything else is external. Anyone looking at this diagram immediately understands the scope and dependencies.

### Example 2: Microservices with Internal Boundaries

```sruja
// partial
// Even within an organization, you can have boundaries

OrderService = system "Order Service" {
  metadata {
    tags ["internal", "orders-team"]
    owner "Orders Team"
  }
}

PaymentService = system "Payment Service" {
  metadata {
    tags ["internal", "payments-team"]
    owner "Payments Team"
  }
}

InventoryService = system "Inventory Service" {
  metadata {
    tags ["internal", "inventory-team"]
    owner "Inventory Team"
  }
}

// Cross-team boundaries
OrderService -> PaymentService "Request payment"
PaymentService -> OrderService "Payment result"
OrderService -> InventoryService "Reserve items"
```

Each service is a bounded context owned by a different team. Even though they're all "internal" to the company, the team boundaries are real and matter for coordination.

## Pitfalls to Avoid (I've Made These)

Let me share some boundary mistakes I've made or seen others make. Hopefully, you can avoid them.

### Mistake 1: No Clear Boundary

```sruja
// partial
// Bad: Everything looks the same
Customer = person "Customer"
Shop = system "Shop"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"

// Without tags, what's external? What's internal?
```

I've seen diagrams like this where everything looks internal. Nobody knows what the team controls versus what they depend on. It's confusing and risky.

**Fix**: Use metadata tags to mark external systems clearly:
```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
  }
}
```

### Mistake 2: Everything Marked External

```sruja
// partial
// Bad: Everything marked external, no ownership
Shop = system "Shop" {
  tags ["external"]
}
WebApp = container "Web App" {
  tags ["external"]
}
```

This is the opposite problem. If everything is external, who owns anything? Who's responsible? The diagram provides no clarity about ownership.

**Fix**: Only mark truly external systems:
```sruja
// partial
Shop = system "Shop" {
  // No tags = internal
  WebApp = container "Web App"
}

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]  // Only mark what's really external
  }
}
```

### Mistake 3: Too Many Fragmented Boundaries

```sruja
// partial
// Bad: Overly fragmented, hard to understand
System1 = system "System 1"
System2 = system "System 2"
System3 = system "System 3"
System4 = system "System 4"
System5 = system "System 5"
// ... and so on
```

I've seen teams create a separate system boundary for every tiny piece of functionality. The diagram becomes a mess of boxes with no clear groupings. Nobody can understand the big picture.

**Fix**: Group related functionality into coherent systems:
```sruja
// partial
Shop = system "Shop" {
  // Group all shop-related containers
  WebApp = container "Web App"
  API = container "API"
  Cache = queue "Cache"
}

Analytics = system "Analytics" {
  // Group all analytics-related containers
  Collector = container "Event Collector"
  Processor = container "Event Processor"
}
```

## Defining Boundaries in Sruja

Sruja gives you the tools to define boundaries clearly. Here's how I use them.

### Use `system` for Your Main Boundary

Your main application is a system. Everything you own goes inside it.

```sruja
// partial
Shop = system "Shop" {
  // Everything you control goes here
  WebApp = container "Web App"
  API = container "API"
  Database = database "Database"
}
```

### Use Metadata to Mark External Systems

External systems get metadata tags so everyone knows they're outside your boundary.

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe"
    sla "99.9% uptime"
  }
}
```

I always add context to external systems: who owns it, what the SLA is, any relevant compliance requirements. This helps people understand the risk and dependency.

### Use `person` for External Actors

Remember: people are always outside your system boundary.

```sruja
// partial
// Users are external to your system
Customer = person "Customer"
Administrator = person "Administrator"
Support = person "Customer Support"

// Your system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// People interact with internal components
Customer -> Shop.WebApp "Browses products"
Administrator -> Shop.WebApp "Manages inventory"
Support -> Shop.WebApp "Monitors health"
```

People are the actors who use your system. They're not "inside" your system—they interact with it from the outside. Always model them as separate entities.

## What to Remember

Boundaries are about clarity—clarity of ownership, clarity of risk, clarity of responsibility. When you draw boundaries:

- **Be intentional**: Every boundary should represent something meaningful (ownership, trust, deployment)
- **Mark external systems clearly**: Use metadata tags so everyone knows what's outside your control
- **Document crossings**: Every relationship that crosses a boundary is an integration point that needs attention
- **Avoid fragmentation**: Group related functionality into coherent systems
- **Think about risk**: Every boundary crossing introduces external risk—plan for it

If you take away one thing, let it be this: **clear boundaries prevent confusion, reduce risk, and make your architectures more maintainable**. Everything else flows from that principle.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're modeling a healthcare platform with these requirements:

> "A hospital scheduling system allows patients to book appointments, doctors to manage their schedules, and administrators to oversee operations. The system integrates with an external insurance API for coverage verification and sends SMS notifications through Twilio. Patient data is stored in the hospital's database."

Which parts should be modeled as **outside the system boundary** (external)?

**A)** Patients, Doctors, Administrators, Insurance API, Twilio
**B)** Insurance API, Twilio, Hospital Database
**C)** Patients, Doctors, Administrators
**D)** Insurance API, Twilio, Hospital Scheduling System

<details>
<summary>Click to see the answer</summary>

**Answer: A) Patients, Doctors, Administrators, Insurance API, Twilio**

Let's break this down:

- **Patients, Doctors, Administrators** — These are people (actors), and people are always outside the system boundary. They interact with the system but aren't part of it.

- **Insurance API** — This is an external service the system depends on. The hospital doesn't control it. It's outside the boundary.

- **Twilio** — This is a third-party SMS service. External vendor, outside the boundary.

- **Hospital Database** — This is inside the boundary. The hospital owns and maintains it.

- **Hospital Scheduling System** — This is the main system being built. It's the boundary itself, containing all the internal components.

**Why not the other options?**

- **B)** Incorrect. The Hospital Database is internal—it's owned and controlled by the hospital. It should be inside the boundary.

- **C)** Incorrect. This only includes people but misses the external services (Insurance API, Twilio) that the system depends on.

- **D)** Incorrect. This includes the main system itself as "external," which doesn't make sense. The system defines the boundary—it's not outside itself.

**Key insight:** People are always external. Third-party services are external. Systems you build and own are internal. Systems you depend on but don't control are external.

</details>

---

### Question 2

You're reviewing an architecture diagram and notice this structure:

```sruja
// partial
Shop = system "Shop"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
AnalyticsService = system "Analytics Service"

Shop.WebApp -> Shop.API "Requests"
Shop.API -> PaymentGateway "Process payment"
Shop.API -> EmailService "Send email"
Shop.API -> AnalyticsService "Track events"
```

What's the main problem with this diagram from a boundaries perspective?

**A)** The Shop system doesn't have containers
**B)** External systems aren't marked as external
**C)** There are too many external dependencies
**D)** The relationships are too generic

<details>
<summary>Click to see the answer</summary>

**Answer: B) External systems aren't marked as external**

The problem is that **all four systems look identical**. There's no way to tell that Shop is the system being built while PaymentGateway, EmailService, and AnalyticsService are external dependencies.

**What should it look like?**

```sruja
// partial
// Your system (no tags = internal)
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API Service"
}

// External systems (marked clearly)
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "vendor"]
    owner "SendGrid"
  }
}

AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "vendor"]
    owner "Google"
  }
}
```

**Why the other options are wrong:**

- **A)** Incorrect. While Shop does have containers in the corrected version, the main boundaries problem isn't about whether containers exist—it's about marking what's external versus internal.

- **C)** Incorrect. Having multiple external dependencies is normal. The problem isn't the number of dependencies—it's that they aren't marked as external, so nobody knows which systems are under your control and which aren't.

- **D)** Incorrect. The relationship labels are descriptive enough ("Process payment", "Send email", "Track events"). The issue is boundaries, not relationship quality.

**Key insight:** Always use metadata tags to mark external systems. This makes boundaries immediately visible to anyone reading your diagram. It prevents confusion about ownership and responsibility.

</details>

---

## What's Next?

Now you understand what boundaries are and why they matter. You know how to identify different types of boundaries and mark them clearly in your diagrams.

But there's a question we haven't answered: **how do you actually mark components as internal or external in Sruja?** How do you use metadata effectively to document ownership and dependencies?

In the next lesson, you'll learn exactly that. We'll cover how to annotate boundary elements, model team and organizational boundaries, and create diagrams that make it instantly clear what's inside versus what's outside.


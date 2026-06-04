---
title: "Lesson 4: Parts & Relationships"
weight: 4
summary: "What happens when you only see parts, not how they connect?"
time: "2 minutes"
---

# Lesson 4: Parts & Relationships

## Learning Goals

By the end of this lesson, you'll be able to:
- Distinguish between parts (what exists) and relationships (how they interact)
- Use the C4 model hierarchy to identify components at the right level
- Label relationships with protocols and actions for clarity
- Avoid common mistakes when modeling parts and relationships

## Understanding Parts & Relationships

Think about the last system diagram you looked at or created. What stood out to you? Was it the individual boxes—API, database, cache? Or was it the lines connecting them?

## Understanding Parts vs. Relationships

Let's dive deeper into this distinction because it's foundational for everything you'll model in Sruja.

### Parts: The Building Blocks

**Parts** are the distinct components that make up your system. Think of them as nouns—things that exist.

In Sruja, these are:
- **People:** Users, stakeholders, administrators
- **Systems:** Software applications, external services
- **Containers:** Applications, databases, services
- **Components:** Modules, classes, libraries (when you need more detail)

Parts tell you *what exists* in your system.

### Relationships: The Connections

**Relationships** are how those parts connect and interact with each other. Think of them as verbs—what happens.

In Sruja, you show relationships with arrows and labels:
```sruja
// partial
Customer -> WebApp "Browses"
WebApp -> API "Fetches data"
API -> Database "Queries"
```

Relationships tell you *how things work*.

### Why This Distinction Matters

Here's a real example of why this matters:

Imagine you see this diagram:
```sruja
// partial
API -> Database "Uses"
```

What does that tell you? Not much. "Uses" could mean anything—reads, writes, queries, updates, anything.

Now imagine you see this:
```sruja
// partial
API -> Database "PostgreSQL/Reads"
```

What does that tell you? A lot more! You know:
- The database technology (PostgreSQL)
- The operation type (Reads, not Writes)
- The action direction (API queries database)

This small difference makes your diagrams **actionable** instead of just informative.

## The C4 Model: A Hierarchy for Your Diagrams

The C4 model gives you a structured way to think about which level of detail is appropriate for your audience.

### The Four Levels

```
Level 1: Person (Users, stakeholders)
    ↓
Level 2: System (Software systems)
    ↓
Level 3: Container (Applications, databases)
    ↓
Level 4: Component (Modules, classes)
```

### When to Use Each Level

Think about who you're talking to—that determines your starting point:

**Talking to executives or business stakeholders?** Start at Level 1 or 2. They care about:
- Which systems exist?
- How do they connect to business value?
- What are the major dependencies?

**Talking to developers or architects?** Start at Level 2 or 3. They care about:
- What containers and services exist?
- How do APIs communicate?
- What's the tech stack?

**Talking to implementation teams?** Drill down to Level 4. They care about:
- What modules and classes make up a service?
- How do components interact internally?
- What's the internal architecture?

### The Golden Rule

You don't have to show every level in every diagram. Match your detail level to your audience. More detail isn't always better—sometimes it just creates clutter.

The C4 model helps you make that decision intentionally.

## Identifying Parts: A Practical Approach

Let's walk through how to actually identify parts for a real system. We'll use an e-commerce example since it's relatable.

### Step 1: Start with People

Who uses or interacts with your system?

For an e-commerce platform:
- Customers browse and purchase
- Administrators manage inventory
- Support teams handle issues
- Product managers track metrics

These are your people (Level 1 of C4).

### Step 2: Identify Systems

What software systems are involved?

- Your e-commerce platform (the main system)
- External payment gateway
- Email service for notifications
- Analytics platform for tracking

These are your systems (Level 2 of C4).

### Step 3: Break Down Systems into Containers

What makes up each system?

For the e-commerce platform:
- Web application (frontend)
- API service (backend)
- Database (data storage)
- Cache (performance)

These are your containers (Level 3 of C4).

### Step 4: Decide If You Need Components

Do you need to show internal detail of a container?

Most of the time, you won't for architecture diagrams. Components (Level 4) are for detailed technical discussions or when you need to show how a service is built internally.

When in doubt? Start with Person → System → Container. You can always drill down to Component level if needed.

### What This Looks Like in Sruja

```sruja
// partial
// Step 1: People
Customer = person "Customer"
Admin = person "Administrator"
Support = person "Support Agent"

// Step 2: Systems
ECommerce = system "E-Commerce Platform"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
Analytics = system "Analytics Platform"

// Step 3: Containers
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
  Cache = queue "Redis Cache"
}
```

Notice how this builds hierarchically—Person → System → Container.

## Modeling Relationships: Labels Matter

You've identified your parts—now how do they interact? This is where relationships come in.

### The Power of Specific Labels

Here's the key insight: The specificity of your relationship labels determines how useful your diagrams are.

Let's compare:

**Vague labels:**
```sruja
// partial
Customer -> WebApp "Uses"
WebApp -> API "Connects to"
API -> Database "Uses"
```

This tells you something is happening, but not what. "Uses" could mean anything.

**Specific labels:**
```sruja
// partial
Customer -> WebApp "HTTPS/REST (Browses)"
WebApp -> API "HTTP/JSON (Submits data)"
API -> Database "PostgreSQL/Reads"
```

This tells you:
- The protocols (HTTPS, HTTP, PostgreSQL)
- The data formats (REST, JSON)
- The actions (Browses, Submits, Reads)

Suddenly, your diagram is actionable. A developer can look at this and immediately understand what's actually happening—they don't have to guess or ask questions.

### Relationship Types

Relationships can be different types depending on what they represent:

**Synchronous relationships:** The sender waits for a response (HTTP requests, database queries)

**Asynchronous relationships:** The sender doesn't wait (message queues, event buses)

**One-way:** Data flows only one direction (analytics events)

**Request-response:** Two-way communication (API calls)

The type of relationship affects how you design your system. Synchronous relationships are simpler but create coupling. Asynchronous relationships are more complex but enable better scalability.

### Example in Sruja

```sruja
// partial
Customer = person "Customer"
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
}

// Clear, labeled relationships
Customer -> ECommerce.WebApp "HTTPS/REST (Browses)"
ECommerce.WebApp -> ECommerce.API "HTTP/JSON (Submits order)"
ECommerce.API -> ECommerce.DB "PostgreSQL/Reads/Writes"
```

Notice how each label tells a story about what's happening.

## Practical Tips for Better Diagrams

After working with many teams, I've noticed patterns that consistently create clearer, more useful diagrams. Here are some practical tips.

### Be Specific with Relationships

Instead of writing `API → DB "uses"`, try something like `API → DB "PostgreSQL/Reads"`. 

The difference might seem small, but it's huge in practice. Specific labels mean developers can look at your diagram and immediately understand what's actually happening—they don't have to guess or ask questions.

### Mark External Systems Clearly

When you depend on something outside your control, make that visible. Use tags, colors, or annotations in Sruja:

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}
```

This isn't just for documentation—it helps everyone understand risk and plan accordingly. If your system depends on a payment gateway and that gateway goes down, everyone needs to know that's a single point of failure.

### Match Detail Level to Audience

The right level of detail depends on who you're talking to:

- **Executives:** Show systems and major dependencies. They don't care about individual containers.
- **Architects:** Show containers and relationships between systems. They care about architecture patterns.
- **Developers:** Show detailed relationships with specific protocols. They care about implementation details.

Don't try to put everything in one diagram. Create different diagrams for different audiences.

### Avoid Over-Crowding

More components and relationships isn't always better. If a diagram has too many arrows crossing each other, no one can understand it.

Aim for clarity over comprehensiveness. If you need to show everything complex, break it into multiple diagrams.

### Use Consistent Naming

It sounds obvious, but I see this all the time: "API Service" in one place, "API" in another, "Backend API" somewhere else.

Pick a convention and stick to it. Small inconsistencies add up and create confusion over time.

## What to Remember

The distinction between parts and relationships might seem subtle, but it's crucial.

**Parts define structure.** They tell you what exists in your system—users, systems, containers, components.

**Relationships define behavior.** They tell you how things work together—data flows, API calls, event streams.

A simple rule of thumb: If you can't describe how two components interact, you don't really understand the system yet.

When you model in Sruja, focus on getting both right: identify the parts accurately, then label their relationships with specific protocols and actions.

That's when diagrams go from informative to actionable—and that's when they become truly useful.

## Check Your Understanding

Let's see if this is clicking.

### Quick Check

**1. You're creating a diagram for your CTO. They want to understand your system's architecture and major dependencies. Which levels of C4 model should you focus on?**

[ ] A. Just Component level (modules, classes)
[ ] B. Container level (apps, databases, APIs)
[ ] C. System and Person level (software systems and stakeholders)
[ ] D. All four levels—include everything

**2. You see this relationship in a diagram: `API -> Database "Uses"`. What's the problem with this label?**

[ ] A. It's too specific—should be more general
[ ] B. It's too vague—"Uses" could mean reads, writes, queries, or anything
[ ] C. It should include the technology name like "PostgreSQL/Uses"
[ ] D. It should be in all caps like "USES"

---

### Answers & Discussion

**1. C. System and Person level** – For executives or CTOs, you want to focus on the big picture: which systems exist, who uses them, and how they connect to business value. Container and Component level details would be unnecessary clutter for this audience.

**2. B. It's too vague—"Uses" could mean reads, writes, queries, or anything** – The label doesn't tell you what's actually happening. Does the API read from the database? Write to it? Run queries against it? A better label would be specific like "PostgreSQL/Reads" or "PostgreSQL/Reads/Writes" to make the diagram actionable.

## What's Next

Now that you understand how to identify parts and model their relationships, let's explore the concept of **Boundaries**—what's inside your system versus what's outside.

This will help you clearly define scope, understand dependencies, and design systems with clear ownership boundaries.

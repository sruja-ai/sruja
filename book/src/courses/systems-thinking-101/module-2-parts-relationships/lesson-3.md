title: "Lesson 3: Connecting the Dots: Defining Relationships"
weight: 3
summary: "Modeling interactions between system parts"
time: "5 min"
---

# Connecting the Dots: Defining Relationships

Imagine looking at a map with cities marked but no roads connecting them. You'd know where things are, but you'd have no idea how to get from one place to another. Relationships in software architecture are those roads—they show how your parts connect, communicate, and depend on each other.

In this lesson, you'll learn to create meaningful connections between your elements. These connections transform a collection of isolated parts into a coherent system that tells a story.

## Learning Goals

By the end of this lesson, you'll be able to:

- Write clear, meaningful relationship labels
- Model different types of interactions between parts
- Use tags to categorize and enrich your relationships
- Handle complex scenarios like nested references and multiple connections
- Create relationships that make your diagrams tell a story

## What Are Relationships, Really?

In the simplest terms, relationships describe how parts interact. They show:

- **Communication**: How components talk to each other
- **Data flow**: Where information moves through the system
- **Dependencies**: What happens if one part goes down
- **User actions**: What people actually do with the system

But here's the thing I've learned over the years: relationships aren't just technical—they're about telling the story of your system. A well-crafted relationship doesn't just say "connects"—it explains **why** and **how** things connect.

## The Basic Syntax

Sruja keeps it simple:

```sruja
// partial
From -> To "Label"
```

- `From`: Who initiates the interaction?
- `To`: Who receives it?
- `"Label"`: What's happening? (This is the most important part!)

Here are some examples:

```sruja
// partial
// Person to System
Customer -> Shop "Browses products"
Administrator -> Shop "Manages inventory"

// Container to Container
Shop.WebApp -> Shop.API "Makes API calls"
Shop.API -> Shop.Database "Reads and writes"

// Component to Component
Shop.API.ProductService -> Shop.API.CartService "Gets product details"
```

Notice how each label uses **present tense verbs** and **specific actions**. "Browses products" is way more informative than "uses."

## Writing Labels That Tell a Story

The difference between a good architecture diagram and a great one often comes down to relationship labels. Let me show you what I mean.

### Good Labels Tell a Story

```sruja
// partial
Customer -> Shop.WebApp "Browses products"
Shop.API -> Shop.Database "Queries data"
Shop.API -> PaymentGateway "Processes payment"
```

Each label tells you something meaningful:
- The customer is **browsing** (shopping behavior)
- The API is **querying** (data retrieval)
- The payment gateway is **processing** (transaction handling)

### Bad Labels Are Mysterious

```sruja
// partial
Customer -> Shop.WebApp "Uses"  // Too generic—what do they do?
Shop.API -> Shop.Database "Connects"  // Doesn't describe the interaction
Shop.API -> PaymentGateway "Integration"  // Technical term, not behavioral
```

These labels leave you with questions. What does "uses" mean? What kind of connection? What's being integrated?

### My Label-Writing Process

When I'm writing relationship labels, I ask myself three questions:

1. **What's the action?** (Browse, query, process, send, receive)
2. **What's the context?** (Products, data, payment, notifications)
3. **Is it specific enough?** (Add details if it helps understanding)

Then I combine them: `Action + Context = "Browses products"`

## Relationship Patterns You'll See Everywhere

After modeling hundreds of systems, I've noticed patterns that repeat constantly. Recognizing these patterns will make you faster and your diagrams more consistent.

### Pattern 1: User Interactions

This pattern shows what people actually do with your system:

```sruja
// partial
Customer -> Shop.WebApp "Logs in"
Customer -> Shop.WebApp "Views products"
Customer -> Shop.WebApp "Adds to cart"
Customer -> Shop.WebApp "Checks out"
```

I model each user action as a separate relationship. This makes it clear what functionality the system needs to support.

### Pattern 2: Service Communication

This pattern shows how your backend services talk to each other:

```sruja
// partial
WebApp -> API "Sends requests"
API -> Database "Persists data"
API -> Cache "Reads cache"
Cache -> API "Returns cached data"
```

Notice the bidirectional communication? The API writes to the cache, but also reads from it. This shows the caching strategy clearly.

### Pattern 3: External Dependencies

This pattern shows systems you depend on outside your control:

```sruja
// partial
API -> PaymentGateway "Process payment"
API -> EmailService "Send notifications"
API -> AnalyticsService "Track events"
```

I always mark these systems as `external` so anyone looking at the diagram immediately understands which dependencies are within your control and which aren't.

## Referencing Nested Elements

One of Sruja's powerful features is dot notation for nested elements. Let me show you how it works.

### Direct Child Reference

```sruja
// partial
Customer -> Shop.WebApp "Uses"
```

This is straightforward—you're referencing a direct child.

### Nested Component Reference

```sruja
// partial
Shop.API.ProductService -> Shop.API.CartService "Get product info"
```

Here, both products and carts live inside the API container. The dot notation makes it clear where each component lives.

### Cross-System Reference

```sruja
// partial
Shop.API -> PaymentGateway.ChargeService "Process payment"
```

This shows that you're calling a specific service within an external system. This level of detail can be crucial when debugging integration issues.

## Using Tags to Add Meaning

Tags are optional but incredibly useful for adding context to your relationships. Think of them as annotations that help people understand what they're looking at.

### Common Tags I Use

```sruja
// partial
// Protocol information
Shop.WebApp -> Shop.API "Sends requests" [http]
Shop.API -> Shop.Database "Queries data" [sql]

// Importance
Shop.API -> PaymentGateway "Process payment" [critical]
Shop.API -> EmailService "Send notifications" [optional]

// Data flow
Shop.WebApp -> Shop.API "Sends requests" [synchronous]
Shop.API -> EmailService "Send notifications" [asynchronous]

// Security
Shop.WebApp -> Shop.API "Sends requests" [authenticated]
Customer -> Shop.WebApp "Browses" [public]
```

I use tags when the detail matters for understanding the system. For example, marking something as `critical` tells people it's a path to watch closely during outages. Marking something as `external` reminds everyone it's outside your control.

## Handling Multiple Relationships

Elements rarely have just one relationship—they're connected to many things. That's normal and expected.

### A Typical Container with Multiple Connections

```sruja
// partial
// WebApp connects to several things
Customer -> Shop.WebApp "Browses"
Shop.WebApp -> Shop.API "Queries products"
Shop.WebApp -> Shop.Cache "Reads cache"
Shop.WebApp -> Customer "Displays products"

// API is even more connected
Shop.WebApp -> Shop.API "Sends request"
Shop.API -> Shop.Database "Persists order"
Shop.API -> PaymentGateway "Process payment"
Shop.API -> Shop.WebApp "Returns response"
Shop.API -> Shop.EmailService "Send confirmation"
```

This might look overwhelming at first, but it's actually telling a clear story. The WebApp is the interface for customers. The API is the orchestrator that talks to databases, payment systems, and email services.

### When Does Too Many Become Too Many?

If an element has more than 10-12 relationships, I ask myself: "Is this element doing too much?"

Maybe it's time to split it into smaller, more focused parts. A single service that talks to 15 different systems is probably violating the single responsibility principle.

## Understanding Relationship Direction

Direction matters. It tells you who initiates the interaction and who responds.

### One-Way Relationships

```sruja
// partial
Customer -> Shop.WebApp "Browses"
```

The customer acts, the web app responds. Simple.

### Two-Way Relationships (Separate Connections)

```sruja
// partial
User -> App.API "Submits data"
App.API -> User "Returns result"
```

Here, the user sends data, and the API sends data back. They're two separate relationships because each one tells its own story.

### Feedback Loops (Cycles)

```sruja
// partial
User -> App.WebApp "Submits form"
App.WebApp -> App.API "Validates"
App.API -> App.WebApp "Returns errors"
App.WebApp -> User "Shows errors"
// User resubmits (loop completes)
```

This creates a feedback loop: submit → validate → error → show → resubmit. Feedback loops are everywhere in real systems. The thermostat in your house is one. The login flow on a website is another.

## Bringing It All Together

Let me show you a complete example that uses everything we've covered:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// People
Customer = person "Customer"
Admin = person "Administrator"

// Systems
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

// Customer interactions
Customer -> Shop.WebApp "Browses products" [public]
Customer -> Shop.WebApp "Adds to cart" [authenticated]
Customer -> Shop.WebApp "Checks out" [authenticated]

// Internal communication
Shop.WebApp -> Shop.API "Sends requests" [http, encrypted]
Shop.API -> Shop.DB "Persists data" [critical]
Shop.API -> Shop.DB "Queries data" [cached]

// External dependencies
Shop.API -> PaymentGateway "Process payment" [critical, external]

// Admin interactions
Admin -> Shop.WebApp "Manages products" [authenticated]
Admin -> Shop.WebApp "Views reports" [authenticated]

view index {
  include *
}
```

This diagram tells a complete story. You can see who uses the system, how parts communicate internally, and what external dependencies exist. The tags add crucial context about security, importance, and whether systems are under your control.

## What to Remember

Relationships are what transform a collection of parts into a living, breathing system. When you write relationships:

- **Be specific**: "Browses products" not "uses"
- **Use present tense**: "Processes" not "will process"
- **Think about the story**: What's really happening here?
- **Add context with tags**: When details matter
- **Don't overthink it**: Most relationships are simple

If you take away one thing, let it be this: **a good relationship label tells a story about how your system actually works**. Everything else flows from that principle.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding:

### Question 1

You're modeling relationships for a blog platform. Which relationship label is the most informative?

**A)** `Reader -> Blog.Frontend "Uses"`
**B)** `Reader -> Blog.Frontend "Reads posts"`
**C)** `Reader -> Blog.Frontend "Frontend connection"`
**D)** `Reader -> Blog.Frontend "HTTP request"`

<details>
<summary>Click to see the answer</summary>

**Answer: B) Reader -> Blog.Frontend "Reads posts"**

Let's compare the options:

- **A)** "Uses" is too generic. What are they using it for? Reading? Writing? Browsing?
- **B)** "Reads posts" is specific and informative. It tells you exactly what the reader is doing.
- **C)** "Frontend connection" is technical jargon that doesn't describe the user action.
- **D)** "HTTP request" describes the protocol, not the business action. This is an implementation detail.

The best labels describe **what's happening** in business terms, not technical terms. "Reads posts" tells you the user behavior. "HTTP request" tells you nothing about the user's intent.

</details>

---

### Question 2

When should you add tags to your relationships?

**A)** Always—tags provide useful context
**B)** Never—tags make diagrams too complex
**C)** When the tag adds meaningful context that helps people understand the system
**D)** Only for external dependencies

<details>
<summary>Click to see the answer</summary>

**Answer: C) When the tag adds meaningful context that helps people understand the system**

Tags are **optional**—use them when they add value:

**Good use cases for tags:**

- `[critical]` — Highlights important paths that deserve attention during outages
- `[external]` — Marks dependencies outside your control
- `[synchronous]` vs `[asynchronous]` — Shows communication patterns that affect reliability
- `[authenticated]` vs `[public]` — Indicates security requirements

**Bad use cases for tags:**

- Tags that are obvious (like `[http]` when everything uses HTTP)
- Tags that don't add clarity
- Tags just for the sake of adding tags

The golden rule: **only add tags when they help your audience understand something important about the relationship**.

</details>

---

## What's Next?

Now you know how to define parts and connect them with relationships. You can create diagrams that tell stories about how systems work.

But there's one more thing: **how do you organize all these parts**? That's what the next lesson is about. You'll learn about hierarchy and nesting—how to structure your systems in a way that's clear, consistent, and scalable.

See you there!

---
title: "Lesson 4: Architectural Perspectives"
weight: 4
summary: "Understanding context, containers, and components without special DSL keywords."
---

# Lesson 4: Architectural Perspectives

I'll never forget the board meeting where I learned this lesson the hard way.

The VP of Engineering asked me to present our architecture to the executive team. I spent hours crafting this beautiful, comprehensive diagram showing every service, database, queue, and API endpoint. It was a masterpiece of technical completeness.

I proudly projected it on the screen.

The CEO stared at it for ten seconds, then asked: "So... where's the revenue coming from?"

The diagram had 47 boxes. None of them mentioned customers, payments, or business value. I'd shown them the engine when they wanted to see the car.

That's when I realized: **one diagram cannot serve all audiences**. You need different maps for different travelers.

## The Google Maps Principle

Think about Google Maps. Same underlying data, but multiple views:

- **Satellite view** - See everything from above (executives)
- **Traffic view** - See flow and congestion (architects)
- **Transit view** - See routes and connections (developers)
- **Street view** - See ground-level details (implementers)

You don't create four different maps. You create **one model with multiple perspectives**.

That's exactly what architectural views do for your system.

## One Model, Multiple Perspectives

Here's the beautiful thing about Sruja: you define your architecture **once**, then create different views for different audiences. No duplication. No inconsistency. No "wait, which version is current?"

Let me show you how this works with a real example:

```sruja
import { * } from 'sruja.ai/stdlib'

// Define people who interact with the system
Customer = person "Customer"
Admin = person "Administrator"

// Define the main system
Shop = system "E-Commerce Shop" {
  WebApp = container "Web Application" {
    technology "React"
    CartComponent = component "Shopping Cart"
    ProductComponent = component "Product Catalog"
  }
  
  API = container "API Service" {
    technology "Rust"
    OrderController = component "Order Controller"
    PaymentController = component "Payment Controller"
  }
  
  DB = database "Database" {
    technology "PostgreSQL"
  }
  
  Cache = database "Cache" {
    technology "Redis"
  }
}

// Define external systems
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Define relationships
Customer -> Shop.WebApp "Browses products"
Admin -> Shop.WebApp "Manages inventory"
Shop.WebApp -> Shop.API "Fetches data"
Shop.API -> Shop.DB "Reads/Writes"
Shop.API -> Shop.Cache "Caches queries"
Shop.API -> PaymentGateway "Processes payments"

// EXECUTIVE VIEW: Business context
view executive {
  title "Executive Overview"
  include Customer
  include Admin
  include Shop
  include PaymentGateway
  // Hide technical details
  exclude Shop.WebApp
  exclude Shop.API
  exclude Shop.DB
  exclude Shop.Cache
}

// ARCHITECT VIEW: Service boundaries
view architect {
  title "Architectural View"
  include Shop Shop.WebApp Shop.API Shop.DB Shop.Cache
  include PaymentGateway
  // Hide people and implementation details
  exclude Customer Admin
  exclude Shop.CartComponent Shop.ProductComponent
  exclude Shop.OrderController Shop.PaymentController
}

// DEVELOPER VIEW: Implementation details
view developer {
  title "Developer View"
  include Shop.WebApp Shop.WebApp.CartComponent Shop.WebApp.ProductComponent
  include Shop.API Shop.API.OrderController Shop.API.PaymentController
  include Shop.DB Shop.Cache
  // Hide external concerns
  exclude Customer Admin PaymentGateway
}

// DATA FLOW VIEW: Data dependencies
view dataflow {
  title "Data Flow View"
  include Shop.API Shop.DB Shop.Cache
  // Hide everything else
  exclude Customer Admin Shop.WebApp PaymentGateway
}

// COMPLETE VIEW: Everything
view index {
  title "Complete System View"
  include *
}
```

One model. Five perspectives. Zero duplication.

## Real-World Case Studies

### Netflix: Views for Scale

Netflix operates one of the world's most complex architectures. They use different views for:

1. **Board presentations** - Show content delivery to customers, geographic reach, no mention of microservices
2. **Architecture reviews** - Show the 700+ microservices, their interactions, data flow
3. **Incident response** - Show specific service dependencies, failure domains, circuit breakers
4. **Developer onboarding** - Show just the relevant service group with its databases and queues

Imagine if they tried to put all of this on one diagram. It would be unusable.

### Amazon: The "Working Backwards" Views

Amazon takes this further. They start with the customer-facing view (press release style), then work backwards to technical architecture:

1. **Customer view** - What does the customer experience?
2. **Business view** - What business capabilities do we need?
3. **Architecture view** - What services support those capabilities?
4. **Implementation view** - How do we build those services?

Each view serves a specific audience and decision-making process.

### Stripe: Developer-Friendly Views

Stripe's famous for their developer experience. Their architecture documentation has:

- **Product view** - For potential customers evaluating Stripe
- **Integration view** - For developers implementing payments
- **Architecture view** - For security teams reviewing compliance
- **Operational view** - For DevOps monitoring systems

Same system. Different lenses.

## When to Create Views (Framework)

Here's the framework I use to decide which views to create:

### Always Create These Views:

**1. Context View (Executive)**
- **Audience:** Executives, stakeholders, non-technical team members
- **Shows:** Systems, external dependencies, users
- **Hides:** Technical implementation details
- **Use when:** Explaining business value, system scope, external integrations

**2. Container View (Architect)**
- **Audience:** Architects, tech leads, senior engineers
- **Shows:** Deployable units, inter-service communication, tech stack
- **Hides:** Internal code structure, business context
- **Use when:** Planning deployments, discussing scalability, reviewing tech choices

**3. Component View (Developer)**
- **Audience:** Developers implementing features
- **Shows:** Internal structure, code organization, responsibilities
- **Hides:** External systems, high-level architecture
- **Use when:** Onboarding developers, planning sprints, debugging

### Create Conditionally:

**4. Data Flow View**
- **Create when:** System has complex data pipelines, multiple data stores, or data compliance requirements
- **Skip when:** Simple CRUD operations with one database

**5. Security View**
- **Create when:** Handling sensitive data, compliance requirements (HIPAA, PCI, GDPR)
- **Skip when:** Internal tools with no sensitive data

**6. Operational View**
- **Create when:** Complex deployment, multiple environments, disaster recovery requirements
- **Skip when:** Simple deployment (single service + database)

## Common Mistakes I See All the Time

### Mistake #1: The "One Diagram to Rule Them All"

**What happens:** You create one massive diagram with everything.

**Why it fails:** 
- Executives can't find business value
- Developers can't find implementation details
- Architects can't see service boundaries
- Nobody can read it after 10 boxes

**The fix:** Start with your audience. What decision do they need to make? Show only what supports that decision.

### Mistake #2: Creating Multiple Models Instead of Multiple Views

**What happens:** You create separate Sruja files for executives, architects, and developers.

**Why it fails:**
- Views get out of sync
- "Which file is the source of truth?"
- Update one, forget the others
- Conflicting information

**The fix:** One model. Multiple `view` blocks. Always in sync.

### Mistake #3: Forgetting the Developer View

**What happens:** You have great executive and architect views, but nothing for developers.

**Why it fails:**
- New developers struggle to understand the codebase
- "Where do I add this feature?"
- Inconsistent implementation patterns

**The fix:** Always create a developer view showing components, their responsibilities, and key dependencies.

### Mistake #4: Showing the Wrong Detail Level

**What happens:** You show database schema to executives, or business context to developers.

**Why it fails:**
- Audience gets confused or frustrated
- "Why are you showing me this?"
- Loss of credibility

**The fix:** Match detail level to audience:
- **Executives:** Systems and users (5-10 boxes)
- **Architects:** Containers and databases (10-20 boxes)
- **Developers:** Components and classes (20-50 boxes)

### Mistake #5: The "Everything View" Default

**What happens:** Your `index` view is the only view, showing everything.

**Why it fails:**
- Default view is overwhelming
- People stop looking at your architecture docs
- "It's too complicated, I'll just ask someone"

**The fix:** Make the default view match your most common audience (usually container-level for teams).

## The VIEW Framework: A Decision Guide

When someone asks for architecture documentation, use this framework:

**V - Verify the Audience**
- "Who will see this diagram?"
- "What's their technical level?"
- "What decisions will they make?"

**E - Establish the Purpose**
- "What question are we answering?"
- "What level of detail do they need?"
- "What can we safely hide?"

**I - Identify the Scope**
- "Which systems/containers are relevant?"
- "What's out of scope?"
- "Should we show external dependencies?"

**E - Exclude Ruthlessly**
- Remove everything that doesn't serve the purpose
- Less is more
- Clarity over completeness

**W - Write the View**
- Create a named view with descriptive title
- Test it with someone from the target audience
- Iterate based on feedback

## Practical Exercise

Take the e-commerce example above and create a new view:

**Challenge:** Create a "Security View" for the compliance team that shows:
- Where customer data flows
- Which systems handle payments
- External integrations
- But hides implementation details

**Hint:** Focus on `PaymentGateway`, `Shop.API`, and data stores. Exclude UI components.

## What to Remember

1. **One model, multiple views** - Define once, present many ways
2. **Match the view to the audience** - Executives, architects, and developers need different things
3. **Use the VIEW framework** - Verify, Establish, Identify, Exclude, Write
4. **Less is more** - The best view shows only what's needed, nothing more
5. **Avoid duplication** - Multiple views from one model, not multiple models
6. **Test with your audience** - If they can't understand it in 30 seconds, simplify
7. **Keep views in sync** - One source of truth means automatic consistency

## When to Skip Views

You don't always need all views. Skip creating multiple views when:

- **Simple systems** - Single service + database doesn't need multiple perspectives
- **Single audience** - If only developers will see it, one developer view is enough
- **Prototyping** - During exploration, focus on the view that helps you think
- **Time pressure** - Better to have one good view than five bad ones

Start simple. Add views when you feel the pain of not having them.

---

**Next up:** We'll explore how to model complex relationships and data flows between your architectural elements.
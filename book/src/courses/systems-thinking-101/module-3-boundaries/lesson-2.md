title: "Lesson 2: Internal vs External"
weight: 2
summary: "Differentiating and marking boundary elements"
time: "5 min"
---

# Inside and Outside: Internal vs External

Remember those "Keep Out" signs you'd see on fences as a kid? They were clear markers: this side is private, that side is public. Inside the fence, you had your rules. Outside, it was anyone's territory.

Boundaries in software work the same way, but instead of fences, we use metadata and tags. These markers tell everyone what's inside your boundary (what you control) and what's outside (what you depend on but don't control).

In this lesson, you'll learn to mark these boundaries clearly in Sruja. You'll discover how to annotate external systems, document ownership, and make your diagrams instantly readable by anyone who picks them up.

## Learning Goals

By the end of this lesson, you'll be able to:

- Use Sruja metadata to mark external systems clearly
- Document ownership and responsibility for each system
- Model team and organizational boundaries effectively
- Create diagrams that make internal/external distinctions instantly visible
- Add meaningful context (SLAs, compliance, contact info) to external systems

## Marking External Systems: The Basics

The simplest way to mark something as external is using metadata tags. Let me show you the pattern I use consistently:

```sruja
// partial
// Internal: Your system (no tags = internal by default)
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API Service"
}

// External: Third-party system you depend on
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}
```

The `tags ["external"]` is the marker. It tells anyone reading your diagram: "This is outside our boundary. We don't control it. Plan accordingly."

This simple tag transforms a diagram from "a bunch of boxes" into "a clear picture of what we own versus what we depend on."

## Adding Rich Context to External Systems

I've learned that just marking something as "external" often isn't enough. People want to know: Who owns it? What's the SLA? Who do I call when it breaks? How do we connect?

Let me show you how I add this context.

### External System with Ownership

```sruja
Stripe = system "Stripe" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe Inc."
    support "support@stripe.com"
  }
}
```

Now anyone looking at the diagram knows:
- It's external (`tags ["external"]`)
- It's a vendor (`tags ["vendor"]`)
- Who owns it (`owner "Stripe Inc."`)
- Who to contact for support (`support "support@stripe.com"`)

I include this information because when something breaks at 3 AM (and it will), you don't want to be hunting through documentation trying to figure out who owns what.

### External System with SLA Information

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    sla "99.9% uptime"
    mttr "4 hours"
    support "24/7 enterprise support"
  }
}
```

SLA (Service Level Agreement) information is crucial for understanding risk:
- **99.9% uptime** means ~43 minutes of downtime per month
- **mttr (Mean Time To Repair)** tells you how quickly they commit to fixing issues
- **24/7 enterprise support** tells you response time expectations

I always add SLA information for critical external dependencies. It helps teams understand what they're committing to.

### External System with Compliance Requirements

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "pci-compliant"]
    compliance ["PCI-DSS Level 1"]
    security ["TLS 1.3", "Mutual TLS"]
  }
}
```

For highly regulated industries (healthcare, finance, government), compliance information is essential. It tells you:
- What standards the external system meets
- What security controls are in place
- Whether you can use it for sensitive data

I once worked on a healthcare project where someone chose an email provider without checking HIPAA compliance. We had to rebuild the integration later. Lesson learned: **document compliance requirements upfront**.

## Common Boundary Patterns

After modeling hundreds of systems, I've noticed patterns that repeat constantly. Let me show you the ones I see most often.

### Pattern 1: Third-Party Services

This is the most common pattern—integrating with vendors who provide specialized services.

```sruja
// partial
// Your system
Shop = system "Shop"

// Third-party integrations
Stripe = system "Stripe" {
  metadata {
    tags ["external", "vendor", "pci-compliant"]
    owner "Stripe"
    sla "99.99% uptime"
    api_endpoint "https://api.stripe.com/v1"
    authentication "API Key"
  }
}

Twilio = system "Twilio" {
  metadata {
    tags ["external", "vendor"]
    owner "Twilio"
    sla "99.9% uptime"
    api_endpoint "https://api.twilio.com"
  }
}

GoogleAnalytics = system "Google Analytics" {
  metadata {
    tags ["external", "vendor"]
    owner "Google"
    privacy_policy "https://policies.google.com/privacy"
  }
}

// Integration relationships
Shop.API -> Stripe "Process payment" [encrypted, tls1.3]
Shop.API -> Twilio "Send SMS" [encrypted]
Shop.API -> GoogleAnalytics "Track events" [anonymous]
```

Notice how each vendor gets different context based on what matters:
- **Stripe**: PCI compliance, strict SLA, API endpoint details
- **Twilio**: SLA, API endpoint
- **Google Analytics**: Privacy policy (because it's tracking users)

The key insight: **add metadata that matters for that specific integration**. Don't copy-paste the same structure for every external system.

### Pattern 2: Partner Integrations

Partners are different from vendors—they're external organizations you have contractual relationships with.

```sruja
// partial
// Your system
Shop = system "Shop"

// Partner systems (B2B integrations)
LogisticsPartner = system "Logistics Partner API" {
  metadata {
    tags ["external", "partner"]
    owner "FedEx"
    sla "99.5% uptime"
    api_documentation "https://partners.fedex.com/api"
    contact "partnership@fedex.com"
  }
}

InventoryPartner = system "Inventory Partner" {
  metadata {
    tags ["external", "partner"]
    owner "Vendor X"
    sla "99.0% uptime"
    contract "CONTRACT-2024-INV-001"
  }
}

// Partner relationships
Shop.API -> LogisticsPartner "Ship order"
Shop.API -> InventoryPartner "Check stock"
```

Partner integrations often have:
- Different SLA expectations than public vendors
- Contractual obligations
- Dedicated partnership contacts
- Custom API documentation

I always add contract references for partner systems. When disputes arise about service levels, you want the contract number readily available.

### Pattern 3: Internal Team Boundaries

Sometimes the boundary isn't between your company and outside world—it's between teams within your organization.

```sruja
// partial
// Your team's system
Shop = system "Shop" {
  metadata {
    tags ["internal", "shop-team"]
    owner "Shop Team"
    slack "#shop-team"
    repository "github.com/company/shop"
  }
  WebApp = container "Web App"
  API = container "API"
}

// Another team's systems (internal but different ownership)
UserPlatform = system "User Platform" {
  metadata {
    tags ["internal", "platform-team"]
    owner "Platform Team"
    slack "#platform-team"
  }
  AuthService = container "Auth Service"
  UserProfile = container "User Profile"
}

AnalyticsPlatform = system "Analytics Platform" {
  metadata {
    tags ["internal", "data-team"]
    owner "Data Team"
    slack "#data-team"
  }
  EventCollector = container "Event Collector"
  DataWarehouse = database "Data Warehouse"
}

// Cross-team boundaries
Shop.API -> UserPlatform.AuthService "Authenticate user"
Shop.API -> AnalyticsPlatform.EventCollector "Send events"
```

Internal team boundaries are crucial for:
- **Communication**: Who do I talk to when something breaks?
- **Coordination**: How do we coordinate changes?
- **Escalation**: Who's responsible when issues arise?

I include Slack channel references for internal systems. It's the fastest way to get help or ask questions.

## People: Always External

Here's a rule I never break: **people are always outside your system boundary**.

They're not "inside" your system—they're actors who interact with it from the outside.

```sruja
// partial
// External actors (always outside boundary)
Customer = person "Customer"
Administrator = person "Administrator"
SupportAgent = person "Customer Support"

// Your system (the boundary)
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
  Database = database "Database"
}

// People interact with your system, they're not part of it
Customer -> Shop.WebApp "Browses products"
Customer -> Shop.WebApp "Adds to cart"
Customer -> Shop.WebApp "Checks out"

Administrator -> Shop.WebApp "Manages products"
Administrator -> Shop.WebApp "Views reports"

SupportAgent -> Shop.WebApp "Monitors health"
```

Why does this matter?

1. **Ownership clarity**: People aren't "owned" by your system. They're autonomous actors who choose to interact with it.
2. **Security perspective**: People are untrusted by default. You need to authenticate, authorize, and validate everything they do.
3. **Testing strategy**: You can unit test internal components, but you need to test user interactions differently (end-to-end tests, user acceptance tests).

I've seen diagrams where people are modeled inside the system, and it always creates confusion. Are they part of the system? Do you control them? No—people are always external.

## Modeling Boundary Crossings

Every relationship that goes from internal to external (or vice versa) crosses a boundary. These crossings are integration points that need special attention.

### Single Boundary Crossing

```sruja
// partial
// Internal system
Shop = system "Shop" {
  API = container "API"
}

// External system
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Crossing the boundary
Shop.API -> PaymentGateway "Process payment"
```

This relationship crosses from internal (your API) to external (payment gateway). That means:
- You need integration tests
- You need error handling for failures
- You need to consider timeout strategies
- You need to understand the failure modes

### Multiple Boundary Crossings

```sruja
// partial
Customer = person "Customer"

// Internal
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// External systems (multiple)
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external"]
  }
}

AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external"]
  }
}

// Multiple boundary crossings
Customer -> Shop.WebApp "Places order"           // Person to System
Shop.WebApp -> Shop.API "Process order"          // Internal (same boundary)
Shop.API -> PaymentGateway "Charge payment"       // Internal → External
Shop.API -> EmailService "Send confirmation"      // Internal → External
Shop.API -> AnalyticsService "Track event"         // Internal → External
```

This system has three external dependencies. Each crossing represents:
- **Risk**: What happens if that external service is down?
- **Complexity**: Integration testing, error handling, fallback strategies
- **Performance**: Network latency, rate limits, timeouts

When I see many boundary crossings like this, I ask: "Do we really need all these external services?" Sometimes consolidating reduces complexity and risk.

## Team Boundaries in Practice

Let me show you a real-world example of how team boundaries work in a larger organization.

### Single Team, One System (Simple)

```sruja
// partial
// One team owns everything
Shop = system "Shop" {
  metadata {
    tags ["internal", "shop-team"]
    owner "Shop Team"
    slack "#shop-team"
  }
  WebApp = container "Web App"
  API = container "API"
  Database = database "Database"
}
```

This is the ideal scenario. One team, one system, clear ownership. Everything inside is your team's responsibility.

### Multiple Teams, Bounded Contexts (Realistic)

```sruja
// partial
// Team A: Shop team
Shop = system "Shop" {
  metadata {
    tags ["internal", "shop-team"]
    owner "Shop Team"
    slack "#shop-team"
    repository "github.com/company/shop"
  }
  WebApp = container "Web App"
  API = container "API"
}

// Team B: Payment team
Payment = system "Payment Service" {
  metadata {
    tags ["internal", "payment-team"]
    owner "Payment Team"
    slack "#payment-team"
    repository "github.com/company/payment"
  }
  Processor = container "Payment Processor"
}

// Team C: Notification team
Notifications = system "Notification Service" {
  metadata {
    tags ["internal", "notification-team"]
    owner "Notification Team"
    slack "#notification-team"
    repository "github.com/company/notifications"
  }
  Sender = container "Notification Sender"
}

// Cross-team boundaries (each is a coordination point)
Shop.API -> Payment.Processor "Process payment"
Shop.API -> Notifications.Sender "Send notification"
```

This is more realistic in larger organizations. Different teams own different systems, each with their own repositories, Slack channels, and processes.

The boundary crossings between teams matter because:
- **Coordination**: Changes need to be coordinated across teams
- **Testing**: Cross-team integrations need integration tests
- **Communication**: Who do you talk to when something breaks?

I include Slack channels and repository links for every internal system. It saves so much time when you're trying to figure out who to contact or where to look at code.

## Creating Views for Different Audiences

One of the most powerful features of Sruja is creating different views for different audiences. Let me show you how.

### System Context View (Shows All Boundaries)

```sruja
view system_context {
  title "System Context - Internal vs External"
  include *
}
```

This view shows everything:
- Your systems
- External systems
- People
- All relationships

Perfect for stakeholders who want to see the big picture, including dependencies and risks.

### Internal-Only View (Shows Just Your System)

```sruja
// partial
view internal_view of Shop {
  title "Internal Architecture"
  include Shop.*
}
```

This view shows only what's inside your boundary:
- Your containers
- Your components
- Your internal relationships

Perfect for developers who are working on the system and don't need to see external dependencies.

### External-Only View (Shows Dependencies)

```sruja
// partial
view external_view of Shop {
  title "External Dependencies"
  exclude Shop.*
}
```

This view shows only what's external:
- Third-party systems
- Partner systems
- Dependencies you rely on

Perfect for risk management, dependency reviews, and vendor assessments.

## What to Remember

Marking internal vs. external is about clarity—clarity of ownership, clarity of risk, clarity of responsibility. When you annotate your diagrams:

- **Always mark external systems**: Use `metadata { tags ["external"] }`
- **Add meaningful context**: Ownership, SLA, support contacts, compliance
- **Document team boundaries**: Include Slack channels, repository links
- **Remember: people are external**: They're never inside your system boundary
- **Show boundary crossings**: Every crossing is an integration point
- **Create multiple views**: Different audiences need different perspectives

If you take away one thing, let it be this: **clearly annotated boundaries prevent confusion and make your diagrams immediately useful** to anyone who picks them up, whether they're on your team or not.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're modeling a healthcare platform and want to mark external dependencies correctly. Which metadata structure is best for an external payment gateway?

**A)**
```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    owner "Stripe"
  }
}
```

**B)**
```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    owner "Stripe"
    sla "99.9% uptime"
    support "support@stripe.com"
  }
}
```

**C)**
```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["internal", "payment"]
    owner "Payment Team"
  }
}
```

**D)**
```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["vendor", "pci-compliant"]
    api_endpoint "https://api.stripe.com"
  }
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: B) Tags as external, plus ownership, SLA, and support**

Let's analyze each option:

**A)** Incorrect. It's missing the crucial `tags ["external"]` marker. Without this tag, anyone reading the diagram won't know this is an external system. They might assume it's internal and under your control.

**B)** Correct! This includes:
- `tags ["external"]` — Clearly marks it as external
- `owner "Stripe"` — Documents who owns and maintains it
- `sla "99.9% uptime"` — Documents the service level commitment
- `support "support@stripe.com"` — Provides contact information for when things break

This metadata provides all the context someone needs to understand the dependency, assess the risk, and know who to contact for support.

**C)** Incorrect. This marks the system as `tags ["internal", "payment"]`, which means it's owned by the "Payment Team." But Stripe is a third-party vendor, not an internal team. This is misleading.

**D)** Incorrect. While it includes some useful tags (`vendor`, `pci-compliant`) and the API endpoint, it's missing the most important tag: `["external"]`. Without this, the system isn't clearly marked as external. Also missing SLA and support information.

**Key insight:** Always use `tags ["external"]` to mark external systems. Then add context that matters: ownership, SLA, support contacts, API endpoints, compliance requirements. Make your diagrams useful, not just correct.

</details>

---

### Question 2

You're reviewing an architecture diagram and notice this structure:

```sruja
// partial
Shop = system "Shop"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"

Shop.API -> PaymentGateway "Process payment"
Shop.API -> EmailService "Send confirmation"
```

What's missing from a boundaries perspective?

**A)** Containers for Shop system
**B)** Metadata tags marking external systems
**C)** Person elements for users
**D)** Component-level breakdown

<details>
<summary>Click to see the answer</summary>

**Answer: B) Metadata tags marking external systems**

The diagram has three systems, but there's no way to tell which one is yours (internal) and which are external dependencies. They all look identical.

**What the diagram should include:**

```sruja
// partial
// Your system (internal)
Shop = system "Shop" {
  metadata {
    tags ["internal"]
    owner "Shop Team"
  }
  WebApp = container "Web App"
  API = container "API"
}

// External systems (clearly marked)
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe"
    sla "99.9% uptime"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "vendor"]
    owner "SendGrid"
    sla "99.9% uptime"
  }
}

Shop.API -> PaymentGateway "Process payment"
Shop.API -> EmailService "Send confirmation"
```

**Why other options are wrong:**

- **A)** Incorrect. While Shop does have containers in the corrected version, the main issue isn't missing containers—it's that external systems aren't marked as external. You could have a valid diagram without containers if you're showing a high-level system context view.

- **C)** Incorrect. Person elements would be great to add (Customer, Administrator, etc.), but the primary boundaries issue isn't about missing people. The problem is that PaymentGateway and EmailService aren't marked as external.

- **D)** Incorrect. Component-level breakdown is optional and depends on your audience. The boundaries issue isn't about missing components—it's about not distinguishing internal from external systems.

**Key insight:** Always use metadata tags to mark external systems. This is the single most important thing you can do to make boundaries clear in your diagrams. Without it, your diagrams tell an incomplete story.

</details>

---

## What's Next?

Now you know how to mark internal and external components clearly. You can annotate external systems with ownership, SLAs, and support information. You can model team boundaries and create different views for different audiences.

But we've only talked about **defining** boundaries. We haven't talked about what happens when you **cross** them.

In the next lesson, you'll learn about crossing boundaries—how to model integrations, plan for failures, document interface contracts, and design fallback strategies. You'll discover why every boundary crossing is both an opportunity and a risk.

See you there!

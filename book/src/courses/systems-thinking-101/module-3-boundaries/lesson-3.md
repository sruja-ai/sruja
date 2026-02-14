title: "Lesson 3: Crossing Boundaries"
weight: 3
summary: "Modeling integrations and planning for boundary crossings"
time: "5 min"
---

# Crossing the Line: Integrations at Boundaries

Imagine crossing a border between countries. You need a passport, you might wait in customs, and there are rules about what you can bring across. Sometimes the border is open and easy to cross. Sometimes it's closed, and you're stuck.

Boundaries in software architecture work the same way. Every time you cross from your internal system to an external one, you're dealing with integration complexity, potential failures, and risks you don't control.

In this lesson, you'll learn to model these boundary crossings effectively. You'll discover how to plan for failures, document interface contracts, and design fallback strategies that keep your system resilient when external dependencies misbehave.

Let's start by understanding what boundary crossings actually are.

## Learning Goals

By the end of this lesson, you'll be able to:

- Model integrations across boundaries clearly
- Identify different integration patterns and when to use each
- Plan for common failure scenarios at boundaries
- Document interface contracts that prevent misunderstandings
- Design fallback strategies that keep your system resilient

## Boundary Crossings: The Reality

Every relationship that goes from internal to external (or external to internal) is a boundary crossing. These are the riskiest parts of your system.

```sruja
// Internal → External = Boundary crossing
Shop.API -> PaymentGateway "Process payment"

// External → Internal = Boundary crossing
PaymentGateway -> Shop.API "Payment result"
```

Why are these risky? Because **you don't control what's on the other side**.

I learned this the hard way early in my career. We had an e-commerce site that depended on a single payment gateway. When that gateway went down for six hours during Black Friday, we lost millions in sales because we hadn't planned for boundary failures.

Lesson learned: **every boundary crossing is a potential failure point**. Plan accordingly.

## Integration Patterns You'll Use

After years of building systems, I've found there are really three main integration patterns you'll encounter. Understanding which one you're using helps you plan correctly.

### Pattern 1: Request-Response (Synchronous)

This is the most common pattern. You send a request, you wait for a response.

```sruja
Shop.API -> PaymentGateway "Process payment"
PaymentGateway -> Shop.API "Payment result"
```

**Characteristics:**
- **Synchronous** — Your system waits for the response
- **Real-time** — The customer sees the result immediately
- **Tight coupling** — If the external service is down, you're down too
- **Simple to implement** — One call, one response

**When to use it:**
- When you need an immediate response (payment processing, real-time validation)
- When the operation is critical to the user's workflow
- When the external service has good uptime guarantees

**Risks:**
- Your users wait if the external service is slow
- Your system fails if the external service is down
- Timeouts need to be configured carefully

This is the pattern I see most often. It's simple, but it's fragile if you don't handle failures properly.

### Pattern 2: Event-Driven (Asynchronous)

You publish an event, and other systems process it whenever they can.

```sruja
Shop.API -> EventQueue "Publish order created"
EventQueue -> PaymentProcessor "Consume order event"
EventQueue -> EmailService "Consume order event"
```

**Characteristics:**
- **Asynchronous** — You don't wait for a response
- **Decoupled** — Your system continues even if others are slow
- **Resilient** — If a consumer fails, the queue buffers events
- **More complex** — You need infrastructure (Kafka, RabbitMQ, etc.)

**When to use it:**
- When the operation can happen in the background (sending emails, updating analytics)
- When you have multiple consumers who need to process the same event
- When you want resilience and fault tolerance

**Risks:**
- Eventual consistency (the user sees "processing" before it's actually done)
- More complex infrastructure to maintain
- Harder to debug when things go wrong

I used to think event-driven was overkill for small systems. Then I built a notification system that sent welcome emails, onboarding sequences, and marketing emails. Trying to send all those synchronously was a nightmare. Moving to events made everything so much smoother.

### Pattern 3: Polling

Your system periodically checks for updates from an external service.

```sruja
Shop.API -> ExternalAPI "Check order status"
ExternalAPI -> Shop.API "Return status"
```

**Characteristics:**
- **Periodic** — You check on a schedule (every minute, every hour, etc.)
- **Simple** — No webhooks or real-time infrastructure needed
- **Less efficient** — You make calls even when nothing has changed
- **Eventual** — There's always a delay between an event and when you discover it

**When to use it:**
- When the external service doesn't support webhooks
- When you need to check status periodically (order fulfillment, shipment tracking)
- When the external API doesn't have push notifications

**Risks:**
- You're wasting resources polling when nothing changes
- There's always a delay before you see updates
- Rate limiting can become an issue

I only use polling when I have no other choice. It's simple, but it's inefficient and introduces latency.

## Integration Considerations (What Actually Matters)

Now that you know the patterns, let's talk about what you actually need to think about when crossing boundaries.

### 1. Error Handling: What Happens When It Breaks?

External services fail. It's not a question of "if," it's "when."

```sruja
// Document expected failure modes
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    sla "99.9% uptime"
    failure_modes ["timeout", "service unavailable", "network error", "rate limit"]
  }
}

// Model fallbacks
Shop.API -> PrimaryPayment "Process payment" [primary]
Shop.API -> BackupPayment "Process payment" [fallback]
```

I've seen systems that don't document failure modes. When something breaks, nobody knows what to expect. Does the external service retry automatically? Do they return specific error codes? What's the timeout?

Document this upfront. It saves hours of debugging later.

### 2. Timeouts and Latency: How Long Is Too Long?

External services can be slow. You need to configure timeouts that protect your system without being too aggressive.

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    timeout "30s"
    expected_latency "500ms"
    max_latency "5s"
  }
}

Shop.API = container "API Service" {
  slo {
    latency {
      p95 "200ms"  // 95% of requests complete in 200ms
      p99 "500ms"  // 99% of requests complete in 500ms
    }
  }
}
```

I once worked on a system that had no timeout configured. An external service got slow, and our threads hung indefinitely. The entire system ground to a halt.

Set timeouts. Always. Even if the external service is usually fast.

### 3. Data Consistency: What Happens When Things Go Wrong?

What if the payment succeeds but saving the order fails? Or the order saves but the payment fails?

```sruja
Shop.API -> PaymentGateway "Process payment"
Shop.API -> Shop.Database "Save order"

// If payment succeeds but order save fails:
// - Did you charge the customer?
// - Is the order lost?
// - How do you reconcile?
```

You need strategies for handling this:
- **Idempotent payment calls** — Calling the same payment ID twice should only charge once
- **Compensating transactions** — If the order save fails after payment, refund automatically
- **Eventual consistency** — Accept that things might be inconsistent briefly, then reconcile
- **Two-phase commits** — Complex, but guarantees consistency

I once worked on a system where we charged customers but lost their orders. We spent weeks manually reconciling payments and orders. Awful experience.

Plan for consistency issues at boundaries. They will happen.

### 4. Security: What Protects Your Data?

Crossing a boundary is where attacks happen. This is where you need to be most careful.

```sruja
// Security at the boundary
Shop.API -> PaymentGateway "Process payment" [encrypted, authenticated, tls1.3]

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "pci-compliant"]
    security ["mutual TLS", "API key authentication"]
    compliance ["PCI-DSS Level 1"]
  }
}
```

Security controls at boundaries:
- **Authentication** — Prove who you are
- **Authorization** — Prove you're allowed to do what you're asking
- **Encryption** — Protect data in transit
- **Validation** — Don't trust anything coming from outside

I learned this lesson painfully. We had an internal API that we exposed to the web without proper validation. Someone sent malformed requests that brought down our database.

Validate everything at your boundaries. Trust nothing from external systems.

## Documenting Interface Contracts

One of the most important things you can do for boundary crossings is document the interface contract. This is the agreement between your system and the external one.

### API Contract

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    api_endpoint "https://api.payment.com/v1"
    authentication "API Key (Bearer token)"
    rate_limit "1000 req/min"
    supported_methods ["POST /charges", "GET /charges/:id", "POST /refunds"]
  }
}

Shop.API = container "API Service" {
  metadata {
    api_consumer "Payment Gateway Client"
    retry_policy "3 retries with exponential backoff"
    circuit_breaker "Enabled (5 failures = open for 60s)"
  }
}
```

This contract tells everyone:
- **Where** the API is
- **How** to authenticate
- **What** methods are available
- **What** limits exist
- **How** to handle retries and failures

I've seen so many integration disasters because nobody documented the contract. Teams assumed different APIs, different limits, different behaviors. When something changed, everything broke.

Document your interface contracts. Make them explicit.

### Data Format

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    data_format "JSON"
    schema_version "v1.2"
    validation "Strict schema validation"
    date_format "ISO 8601 (UTC)"
    currency_format "ISO 4217 (e.g., 'USD')"
  }
}
```

Don't let data format be implicit. Specify:
- JSON vs. XML vs. Protocol Buffers
- Schema version (what happens when it changes?)
- Date formats (timezone matters!)
- Currency formats
- Number formats (decimal precision, rounding)

I once dealt with a system where dates were sometimes in US format (MM/DD/YYYY) and sometimes in ISO format (YYYY-MM-DD), depending on which service you called. Bugs everywhere.

### SLA and Reliability

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    sla "99.9% uptime"
    mttr "4 hours"  // Mean Time To Repair
    support_tier "24/7 enterprise support"
    escalation_path "Support → Account Manager → CTO"
  }
}
```

SLA documentation tells you:
- **What** uptime they're committing to
- **How fast** they'll fix things when they break
- **Who** to contact and how to escalate
- **What** compensation you get if they violate SLA

Knowing the SLA helps you decide: do you need a fallback? Can you tolerate 43 minutes of downtime per month (99.9%)? Or do you need 99.99% (4 minutes)?

## Fallback Strategies: Planning for Failure

External systems fail. You need fallback strategies. Here are the ones I use most often.

### Strategy 1: Redundant Providers

Have a backup provider you can switch to if the primary fails.

```sruja
// Primary provider
PrimaryPayment = system "Stripe" {
  metadata {
    tags ["external", "primary"]
    sla "99.99% uptime"
    owner "Stripe"
  }
}

// Backup provider
BackupPayment = system "PayPal" {
  metadata {
    tags ["external", "backup"]
    sla "99.9% uptime"
    owner "PayPal"
  }
}

// Try primary, fall back to backup
Shop.API -> PrimaryPayment "Process payment" [primary]
Shop.API -> BackupPayment "Process payment" [fallback]
```

**Why this works:** If Stripe is down, you can still process payments through PayPal.

**Challenge:** Supporting two payment gateways is complex. You need to reconcile transactions, handle different APIs, manage different fee structures.

I've used this strategy for critical paths (payments, messaging, notifications). It adds complexity, but it buys you resilience.

### Strategy 2: Circuit Breaker

Stop calling a failing service automatically instead of hammering it with requests.

```sruja
Shop.API = container "API Service" {
  metadata {
    circuit_breaker {
      enabled true
      failure_threshold 5  // Open after 5 failures
      recovery_timeout "60s"  // Try again after 60 seconds
      half_open_attempts 3  // Send 3 test requests before closing
    }
  }
}
```

**How it works:**
1. **Closed** — Normal operation, requests go through
2. **Open** — After N failures, stop sending requests
3. **Half-open** — After timeout, send a few test requests
4. **Closed** — If tests succeed, go back to normal

**Why this works:** Instead of hammering a failing service with 1000 requests/second (which might make recovery worse), you stop calling it and fail fast.

This is one of my favorite patterns. It's saved me from cascading failures more times than I can count.

### Strategy 3: Degraded Mode

Continue operating even if a non-critical external service is down.

```sruja
// Non-critical: If analytics fails, continue
Shop.API -> AnalyticsService "Track events" [non_critical]

// Queue for later: If email fails, queue it
Shop.API -> EmailService "Send notifications" [async_queue]
```

**Why this works:** Not every external dependency is critical. If analytics is down, you can still process orders. If email is down, queue messages and send later.

**What this requires:** You need to distinguish between:
- **Critical paths** — System can't function without them (payments)
- **Important paths** — System functions, but with degraded UX (email, push notifications)
- **Nice-to-have paths** — System functions perfectly without them (analytics)

I used to treat all dependencies as critical. Then I realized: if analytics is down for an hour, does anyone actually care? No. Mark it non-critical and move on.

### Strategy 4: Cache External Data

Cache responses from external APIs so you can serve from cache if the external service is down.

```sruja
// External API call
Shop.API -> ExchangeRateAPI "Get exchange rates"

// Cache for backup
Shop.API -> Shop.Cache "Get cached rates"

// Fallback strategy
Shop.API -> ExchangeRateAPI "Get exchange rates" [primary]
Shop.API -> Shop.Cache "Get cached rates" [fallback]
```

**Why this works:** Even if the external API is down, you can serve slightly stale data from cache.

**What to consider:**
- How stale is acceptable? (10 minutes? 1 hour? 24 hours?)
- How do you detect when the external service is back up?
- Do you need to warm the cache before the service goes down?

I've used this for exchange rates, product catalogs, weather data—anything that's expensive to fetch and acceptable to serve slightly stale.

## Complete Integration Example

Let me show you a complete example that brings everything together.

```sruja
import { * } from 'sruja.ai/stdlib'

// People
Customer = person "Customer"

// Your system
Shop = system "Shop" {
  metadata {
    tags ["internal"]
    owner "Shop Team"
    slack "#shop-team"
  }

  WebApp = container "Web Application"
  API = container "API Service" {
    metadata {
      timeout "30s"
      retry_policy "3 retries with exponential backoff"
      circuit_breaker {
        enabled true
        failure_threshold 5
        recovery_timeout "60s"
      }
    }
  }
  Cache = database "Redis Cache"
}

// Primary payment provider
Stripe = system "Stripe" {
  metadata {
    tags ["external", "primary", "vendor"]
    owner "Stripe Inc."
    sla "99.99% uptime"
    mttr "4 hours"
    api_endpoint "https://api.stripe.com/v1"
    authentication "API Key (Bearer token)"
    rate_limit "1000 req/min"
    data_format "JSON"
    schema_version "v1.2"
    security ["TLS 1.3", "API key authentication"]
    compliance ["PCI-DSS Level 1"]
    support "24/7 enterprise support"
    escalation_path "Support → Account Manager → CTO"
  }
}

// Backup payment provider
PayPal = system "PayPal" {
  metadata {
    tags ["external", "backup", "vendor"]
    owner "PayPal"
    sla "99.9% uptime"
    api_endpoint "https://api.paypal.com/v2"
    authentication "OAuth 2.0"
  }
}

// Email service
SendGrid = system "SendGrid" {
  metadata {
    tags ["external", "vendor"]
    owner "SendGrid"
    sla "99.9% uptime"
    timeout "10s"
    api_endpoint "https://api.sendgrid.com/v3"
  }
}

// Integrations
Customer -> Shop.WebApp "Checkout"
Shop.WebApp -> Shop.API "Process order"

// Primary payment (encrypted, authenticated)
Shop.API -> Stripe "Process payment" [primary, encrypted, tls1.3]
Stripe -> Shop.API "Payment result"

// Fallback to backup if Stripe fails
Shop.API -> PayPal "Process payment" [fallback, encrypted]

// Email (non-critical, can queue)
Shop.API -> SendGrid "Send confirmation" [non_critical, async_queue]

// Cache exchange rates
Shop.API -> ExchangeRateAPI "Get exchange rates"
Shop.API -> Shop.Cache "Get cached rates" [fallback]

view index {
  include *
}
```

This example shows:
- Clear external/internal boundaries with rich metadata
- Multiple integration patterns (synchronous payment, asynchronous email)
- Fallback strategies (backup provider, cache)
- Failure documentation (timeouts, circuit breaker, retry policy)
- Interface contracts (API endpoints, authentication, data formats)

This is the kind of documentation that saves you when things go wrong at 3 AM.

## What to Remember

Crossing boundaries is where systems are most fragile. When you model and plan for boundary crossings:

- **Document everything** — API contracts, failure modes, SLAs, security requirements
- **Plan for failures** — Timeouts, retries, circuit breakers, fallbacks
- **Use the right pattern** — Synchronous for critical paths, asynchronous for background work
- **Protect your system** — Authentication, encryption, validation at every boundary
- **Design resilience** — Redundant providers, caching, degraded modes
- **Test thoroughly** — Integration tests, chaos engineering, failure scenarios

If you take away one thing, let it be this: **every boundary crossing is both an opportunity and a risk**. The opportunity is integrating with powerful external services. The risk is depending on something you don't control. Plan for that risk.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're modeling a weather application that fetches data from an external API. Which integration pattern is most appropriate?

> "The weather app needs to display current weather and 7-day forecasts for cities around the world. Users expect to see real-time weather data. The external weather API has good uptime and supports synchronous calls."

**A)** Request-Response (Synchronous)
**B)** Event-Driven (Asynchronous)
**C)** Polling
**D)** All of the above are equally appropriate

<details>
<summary>Click to see the answer</summary>

**Answer: A) Request-Response (Synchronous)**

Let's analyze each option:

**A)** Correct! Request-response is the right choice here because:
- The weather data is needed **in real-time** (users expect to see current weather immediately)
- The operation is **critical to the user's workflow** (the app's main purpose is displaying weather)
- The external API has **good uptime** (so reliability risk is manageable)
- It's **simple to implement** (one call, one response)

**B)** Incorrect. Event-driven is asynchronous—you publish an event, and consumers process it whenever they can. This doesn't work for real-time weather display because:
- There's a delay between requesting and receiving data
- You'd need a background worker to consume events
- The user would see "loading..." longer than necessary
- It adds unnecessary complexity for a simple request-response scenario

**C)** Incorrect. Polling is periodic checking on a schedule (e.g., checking every hour). This doesn't work here because:
- Users want to see weather **when they open the app**, not according to a schedule
- There's unnecessary latency (user opens app, but has to wait for next poll cycle)
- It's inefficient (you'd be calling the API even when no one's viewing data)

**D)** Incorrect. These patterns are not equally appropriate. Request-response is clearly the best fit for this scenario. The other patterns introduce unnecessary complexity or don't meet the real-time requirement.

**Key insight:** Choose integration patterns based on your requirements. Need immediate results? Use synchronous. Can happen in background? Use asynchronous. No webhooks available? Use polling (as a last resort). Match the pattern to the problem.

</details>

---

### Question 2

You're designing the payment processing flow for an e-commerce platform. The payment gateway has an SLA of 99.9% uptime. What does this mean for your system?

**A)** You don't need to worry about failures—99.9% is very reliable
**B)** You should have a fallback strategy because 99.9% still means ~43 minutes of downtime per month
**C)** You should only process payments when the gateway is at 100% uptime
**D)** You should switch to a different payment gateway immediately

<details>
<summary>Click to see the answer</summary>

**Answer: B) You should have a fallback strategy because 99.9% still means ~43 minutes of downtime per month**

Let's break down what 99.9% actually means:

**The math:**
- 99.9% uptime = 0.1% downtime
- 0.1% of a month (30 days × 24 hours × 60 minutes = 43,200 minutes) = ~43 minutes
- That's **43 minutes per month** of potential payment processing outages

**Why other options are wrong:**

**A)** Incorrect. 99.9% might sound high, but 43 minutes of downtime is significant if it happens during peak shopping hours (Black Friday, Cyber Monday, etc.). You absolutely need to worry about failures and plan for them.

**C)** Incorrect. No system has 100% uptime. Waiting for perfect uptime means your system never processes payments. This is unrealistic. You need to work with the reality that failures will happen.

**D)** Incorrect. Switching to a different payment gateway "immediately" is overkill. 99.9% uptime means the gateway is working 99.9% of the time. You should:
- Have a **backup gateway** as a fallback
- Implement **circuit breakers** to detect and route around failures
- Use **caching** for less-critical payment info
- Consider **degraded modes** (e.g., show "payment processing temporarily unavailable" instead of failing completely)

**What you should actually do:**
- Document the 99.9% SLA in your metadata
- Calculate the business impact of 43 minutes/month downtime
- Design fallback strategies (backup provider, queueing, retry logic)
- Set up monitoring and alerting for gateway outages
- Have an escalation path with the gateway vendor

**Key insight:** SLAs give you information to make decisions. 99.9% tells you to plan for ~43 minutes of monthly downtime. Don't ignore it—plan for it.

</details>

---

## What's Next?

Congratulations! You've completed Module 3: Boundaries. You now understand:

- **What boundaries are** and why they matter for ownership, risk, and clarity
- **How to mark internal vs. external components** using metadata and tags
- **How to model boundary crossings** with proper planning for failures and fallbacks

You can now create architectures that clearly distinguish what you control from what you depend on. You can plan for failures at boundaries instead of being surprised by them. You can document interface contracts that prevent integration disasters.

You're building resilient systems.

In the next module, you'll learn about **flows**—how information moves through your system over time. You'll discover how to model data flow, process flows, and temporal behaviors that tell a richer story than static diagrams can.

See you there!

---

## Module 3 Complete!

You've now mastered the art of defining and crossing boundaries. Here's what you've learned:

**Lesson 1: Understanding Boundaries**
- Boundaries separate what's inside from what's outside
- Multiple types of boundaries: system, team, organization, deployment, trust
- Clear boundaries prevent confusion and clarify ownership

**Lesson 2: Internal vs. External**
- Use metadata tags to mark external systems clearly
- Document ownership, SLAs, and support contacts
- Remember: people are always outside your system boundary
- Create different views for different audiences

**Lesson 3: Crossing Boundaries**
- Every boundary crossing is an integration point and potential failure
- Choose the right integration pattern (synchronous, asynchronous, polling)
- Document interface contracts (API endpoints, data formats, security)
- Design fallback strategies (redundant providers, circuit breakers, degraded modes, caching)
- Plan for failures—they will happen

You're ready to tackle more advanced concepts. Let's continue!
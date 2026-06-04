title: "Module 3 Quiz: Boundaries"
weight: 4
summary: "Test your understanding of boundaries, internal vs. external components, and crossing boundaries"
time: "10 min"
---

# Module 3 Quiz: Boundaries

Test your understanding of how to define system boundaries, differentiate internal vs. external components, and model integrations at boundaries.

This quiz covers all three lessons in Module 3. Take your time, think through each question, and read the explanations to reinforce your learning.

---

## Question 1: Understanding Boundaries

You're reviewing an architecture diagram for a healthcare platform and notice this structure:

```sruja
// partial
// People
Patient = person "Patient"
Doctor = person "Doctor"

// Systems
HospitalSystem = system "Hospital Scheduling"
InsuranceAPI = system "Insurance API"
Twilio = system "Twilio SMS"
```

Based on what you've learned about boundaries, what's the main problem with this diagram?

**A)** No containers are defined within HospitalSystem
**B)** External systems (InsuranceAPI, Twilio) aren't marked as external
**C)** People should be modeled as systems, not persons
**D)** There are too many systems for a simple healthcare platform

<details>
<summary>Click to see the answer</summary>

**Answer: B) External systems (InsuranceAPI, Twilio) aren't marked as external**

**Explanation:**

The diagram has three systems, but there's no way to tell which one is being built internally (HospitalSystem) and which are external dependencies (InsuranceAPI, Twilio). They all look identical.

**What the diagram should include:**

```sruja
// partial
// Internal system (what you own and control)
HospitalSystem = system "Hospital Scheduling" {
  // No tags = internal by default
  metadata {
    owner "Hospital IT Team"
    slack "#hospital-it"
  }
  WebApp = container "Web App"
  API = container "API Service"
  Database = database "Database"
}

// External systems (clearly marked)
InsuranceAPI = system "Insurance API" {
  metadata {
    tags ["external", "vendor"]
    owner "External Insurance Provider"
    sla "99.5% uptime"
    support "support@insurance.com"
  }
}

Twilio = system "Twilio SMS" {
  metadata {
    tags ["external", "vendor"]
    owner "Twilio"
    sla "99.9% uptime"
    api_endpoint "https://api.twilio.com"
  }
}
```

**Why other options are wrong:**

- **A)** Incorrect. While adding containers would make the diagram more detailed, the primary boundaries problem isn't about missing containers—it's that external systems aren't marked. You could have a valid high-level diagram without containers, but boundaries should still be clear.

- **C)** Incorrect. People (Patient, Doctor) should absolutely be modeled as persons, not systems. People are always outside system boundaries—they're actors who interact with your system, not parts of it.

- **D)** Incorrect. Having multiple systems is normal for a healthcare platform that integrates with insurance APIs and SMS services. The problem isn't the number of systems—it's that boundaries between internal and external aren't clear.

**Key Takeaway:** Always use metadata tags (`tags ["external"]`) to mark external systems. This makes boundaries immediately visible to anyone reading your diagram. It prevents confusion about ownership, risk, and responsibility.

</details>

---

## Question 2: Types of Boundaries

You're modeling a large e-commerce platform with multiple teams. Which boundary type is most relevant when the **Shop Team** owns the "Shop" system and the **Payment Team** owns the "Payment" system?

**A)** System Boundary
**B)** Organizational Boundary
**C)** Team Boundary
**D)** Deployment Boundary

<details>
<summary>Click to see the answer</summary>

**Answer: C) Team Boundary**

**Explanation:**

Team boundaries exist when different teams own different systems, even within the same organization. This is exactly what's described here:

- **Shop Team** owns the **Shop** system
- **Payment Team** owns the **Payment** system

These are internal systems (same company), but different teams are responsible for them. That's a team boundary.

**What this looks like in Sruja:**

```sruja
// partial
// Team A's system
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

// Team B's system
Payment = system "Payment" {
  metadata {
    tags ["internal", "payment-team"]
    owner "Payment Team"
    slack "#payment-team"
    repository "github.com/company/payment"
  }
  Processor = container "Payment Processor"
}

// Cross-team boundary (integration point)
Shop.API -> Payment.Processor "Process payment"
```

**Why other options are wrong:**

- **A)** Incorrect. System boundary separates your main application from the world. While Shop and Payment are separate systems, the specific issue described is about **team ownership**, not just system separation.

- **B)** Incorrect. Organizational boundaries separate internal (your company) from external (other companies). Both Shop and Payment are internal systems within the same organization, so this isn't an organizational boundary.

- **D)** Incorrect. Deployment boundaries separate what deploys together. While Shop and Payment might deploy independently (which would make it a deployment boundary), the scenario specifically describes **team ownership**, not deployment strategy.

**Key Takeaway:** Team boundaries are crucial for communication, coordination, and escalation. Include metadata about team ownership (Slack channels, repositories) so everyone knows who to contact when issues arise.

</details>

---

## Question 3: Integration Patterns

You're building a real-time stock trading application. Users place buy/sell orders that must execute immediately. You're integrating with an external market data API to get current stock prices. Which integration pattern is most appropriate?

**A)** Request-Response (Synchronous)
**B)** Event-Driven (Asynchronous)
**C)** Polling
**D)** All patterns are equally appropriate

<details>
<summary>Click to see the answer</summary>

**Answer: A) Request-Response (Synchronous)**

**Explanation:**

Request-response is the right choice here because:

- **Real-time requirement** — Users need current stock prices immediately when they're trading
- **Critical to user workflow** — The app's main purpose is showing real-time prices and executing trades
- **Simple and direct** — One call, one response gets you the data you need
- **User expectation** — Stock traders expect to see live prices, not prices from 10 minutes ago

**What this looks like:**

```sruja
// partial
TradingApp = system "Trading App" {
  WebApp = container "Web Application"
  API = container "API Service"
}

MarketDataAPI = system "Market Data API" {
  metadata {
    tags ["external", "vendor"]
    owner "Financial Data Provider"
    sla "99.99% uptime"
    timeout "100ms"
  }
}

// Synchronous: Request and wait for response
TradingApp.API -> MarketDataAPI "Get current stock price" [real-time]
MarketDataAPI -> TradingApp.API "Return price"

Trader -> TradingApp.WebApp "Place buy order"
```

**Why other options are wrong:**

- **B)** Incorrect. Event-driven is asynchronous—you publish an event and consumers process it whenever they can. This doesn't work for real-time stock trading because:
  - There's a delay between requesting and receiving price data
  - Users need live prices, not "eventual" prices
  - Adds unnecessary infrastructure (queues, event brokers)
  - You'd need background workers to consume events

- **C)** Incorrect. Polling checks for updates periodically (e.g., every minute). This doesn't work because:
  - Users want prices **when they load the page**, not according to a schedule
  - Unnecessary latency—if you poll every minute, prices are stale for 59 seconds
  - Inefficient—you're calling the API even when no one's viewing that stock
  - Polling only makes sense when the external service doesn't support push notifications or webhooks

- **D)** Incorrect. These patterns are not equally appropriate. Request-response is clearly the best fit for real-time requirements. Using event-driven or polling would introduce unacceptable latency and complexity for this use case.

**Key Takeaway:** Choose integration patterns based on your requirements. Need immediate, real-time responses? Use synchronous. Can processing happen in background? Use asynchronous. No webhooks available? Use polling as a last resort. Match the pattern to the problem.

</details>

---

## Question 4: Marking External Systems

You're modeling a ride-sharing app that integrates with Google Maps for navigation and Stripe for payments. Which metadata structure is best for these external dependencies?

**A)**
```sruja
GoogleMaps = system "Google Maps" {
  owner "Google"
}

Stripe = system "Stripe" {
  owner "Stripe"
}
```

**B)**
```sruja
GoogleMaps = system "Google Maps" {
  metadata {
    tags ["external"]
    owner "Google"
    api_endpoint "https://maps.googleapis.com"
  }
}

Stripe = system "Stripe" {
  metadata {
    tags ["external", "pci-compliant"]
    owner "Stripe"
    sla "99.9% uptime"
    support "support@stripe.com"
  }
}
```

**C)**
```sruja
GoogleMaps = system "Google Maps" {
  metadata {
    tags ["internal", "maps"]
  }
}

Stripe = system "Stripe" {
  metadata {
    tags ["internal", "payments"]
  }
}
```

**D)**
```sruja
GoogleMaps = system "Google Maps" {
  metadata {
    tags ["vendor"]
    api_documentation "https://developers.google.com/maps"
  }
}

Stripe = system "Stripe" {
  metadata {
    tags ["vendor"]
  }
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: B) Both marked as external with relevant context**

**Explanation:**

This is the best structure because:

- **Both systems are marked as external** (`tags ["external"]`), making boundaries immediately clear
- **Context is added that matters for each integration**:
  - **Google Maps**: API endpoint for integration
  - **Stripe**: PCI compliance (critical for payments), SLA, support contact
- **Ownership is documented** so everyone knows who to contact

**Why this works:**

```sruja
// partial
GoogleMaps = system "Google Maps" {
  metadata {
    tags ["external"]  // Clearly external
    owner "Google"  // Who owns it
    api_endpoint "https://maps.googleapis.com"  // Where to integrate
  }
}

Stripe = system "Stripe" {
  metadata {
    tags ["external", "pci-compliant"]  // External and regulated
    owner "Stripe"  // Who owns it
    sla "99.9% uptime"  // Reliability commitment
    support "support@stripe.com"  // Who to contact for issues
  }
}
```

**Why other options are wrong:**

- **A)** Incorrect. Both systems are missing the crucial `tags ["external"]` marker. Without this, anyone reading the diagram won't know these are external dependencies. They might assume they're internal systems under your control.

- **C)** Incorrect. Both systems are marked as `tags ["internal"]`, which means they're owned by your organization. But Google Maps and Stripe are third-party vendors, not internal teams. This is misleading and hides the fact that these are external dependencies.

- **D)** Incorrect. While both have `tags ["vendor"]`, they're missing the most important tag: `["external"]`. Without this, the boundary isn't clear. Also missing:
  - API endpoint for Google Maps
  - SLA information for Stripe
  - Support contact information
  - These are useful for debugging and planning

**Key Takeaway:** Always use `tags ["external"]` to mark external systems. Then add context that matters for that specific integration:
- **For payments**: PCI compliance, SLA, support contacts
- **For APIs**: API endpoints, rate limits, authentication methods
- **For data services**: Data freshness, caching strategies, privacy policies

Make your diagrams useful, not just correct.

</details>

---

## Question 5: Fallback Strategies

You're building an e-commerce platform that processes payments through Stripe (99.9% SLA). You need to design a fallback strategy. Which approach is most appropriate?

**A)** No fallback needed—99.9% uptime is very reliable
**B)** Queue failed payments for manual processing
**C)** Switch to a backup payment provider (e.g., PayPal) if Stripe fails
**D)** Retry Stripe indefinitely until it succeeds

<details>
<summary>Click to see the answer</summary>

**Answer: C) Switch to a backup payment provider if Stripe fails**

**Explanation:**

Let's analyze what 99.9% uptime actually means:

**The math:**
- 99.9% uptime = 0.1% downtime
- 0.1% of a month (30 days × 24 hours × 60 minutes = 43,200 minutes) = **~43 minutes per month**

**The impact:**
- 43 minutes of payment processing outages per month
- If these 43 minutes happen during peak shopping hours (Black Friday, Cyber Monday, evening rush), you could lose significant revenue
- 43 minutes is not acceptable for a critical path like payments

**The right approach:**

```sruja
// partial
// Primary payment provider
Stripe = system "Stripe" {
  metadata {
    tags ["external", "primary", "vendor"]
    owner "Stripe"
    sla "99.9% uptime"
    mttr "4 hours"
    support "24/7 enterprise support"
    api_endpoint "https://api.stripe.com/v1"
  }
}

// Backup payment provider
PayPal = system "PayPal" {
  metadata {
    tags ["external", "backup", "vendor"]
    owner "PayPal"
    sla "99.9% uptime"
    api_endpoint "https://api.paypal.com/v2"
  }
}

// Your system
Shop = system "Shop" {
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
}

// Primary path
Shop.API -> Stripe "Process payment" [primary]

// Fallback path
Shop.API -> PayPal "Process payment" [fallback]
```

**Why this works:**
- If Stripe is down, you can still process payments through PayPal
- Circuit breaker prevents hammering Stripe with requests while it's down
- You have redundancy for your critical payment path
- Users can still complete purchases

**Why other options are wrong:**

- **A)** Incorrect. As calculated, 99.9% means ~43 minutes of downtime per month. For a critical path like payments, this is significant risk. You absolutely need a fallback strategy. Don't confuse "reliable" with "infinitely available."

- **B)** Incorrect. Queueing failed payments for manual processing is not acceptable for e-commerce:
  - Users expect immediate purchase confirmation
  - Manual processing is slow, error-prone, and doesn't scale
  - Creates poor user experience (did my payment go through? do I have my items?)
  - High operational overhead

- **D)** Incorrect. Retrying indefinitely causes multiple problems:
  - Users wait indefinitely for their payment to process
  - If Stripe is down for hours, retries accumulate and create a backlog
  - Potential for duplicate charges if retry logic isn't idempotent
  - Wastes resources hammering a down service

**Key Takeaway:** For critical paths (payments, messaging, authentication), design redundancy. Multiple providers, circuit breakers, and failover strategies protect your system from outages. 99.9% uptime means plan for 43 minutes of monthly downtime, not ignore it.

</details>

---

## How Did You Do?

Count your correct answers:

- **5 correct:** Excellent! You have a solid understanding of boundaries, internal vs. external differentiation, and integration strategies. You're ready to apply these concepts in real projects.
- **4 correct:** Great work! You understand most concepts well. Review the explanation for the question you missed to solidify your understanding.
- **3 correct:** Good effort! You understand the basics but need to practice on some concepts. Re-read the relevant lessons and try the quiz again.
- **1-2 correct:** Keep learning! You're on the right track, but need to review the lessons more carefully. Focus on understanding the "why" behind each concept, not just the "how."

---

## What's Next?

Ready to move on? In [Module 4: Flows](../module-4-flows/module-overview.md), you'll learn about how information moves through your system over time. This is crucial for:

- Understanding system behavior beyond static structure
- Modeling data flow and process flows
- Identifying bottlenecks and performance issues
- Visualizing temporal patterns (feedback loops, delays, queues)


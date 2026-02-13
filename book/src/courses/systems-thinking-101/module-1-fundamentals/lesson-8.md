---
title: "Lesson 8: Context"
weight: 8
summary: "Your system doesn't exist in a vacuum—context shapes everything."
time: "2 minutes"
---

# Lesson 8: Context

## Learning Goals

By the end of this lesson, you'll be able to:
- Understand what context is and why it matters
- Identify stakeholders, dependencies, and constraints
- Model context in Sruja
- Design systems that fit their context

## Understanding Context

Imagine you're designing a house. You think about the layout, materials, style. All important.

But wait—what's the climate like? What's the neighborhood like? What's your budget? Who will live there?

These factors—climate, location, budget, occupants—are **context**. They shape what's possible and what's not.

A house designed for a desert (no insulation, cool floors, large windows) would fail in a rainforest (too much rain, humidity issues). A luxury house design would fail for a budget-conscious family.

Software systems are the same. They don't exist in a vacuum—they exist in a context that shapes everything.

## What Is Context?

**Context** is the environment your system operates in. It includes everything that affects or is affected by your system.

### The Three Context Layers

**1. Stakeholder Context: People**

Who uses, depends on, or influences your system?

**Examples:**
- Users who interact with your application
- Administrators who manage it
- Developers who build and maintain it
- Business owners who pay for it
- Compliance officers who ensure it meets regulations

**Why it matters:** Different stakeholders have different needs. If you design for users only, you might ignore administrators' needs for monitoring or compliance officers' needs for security.

**2. Dependency Context: What You Rely On**

What external systems, APIs, libraries, or infrastructure do you depend on but don't control?

**Examples:**
- Payment gateways (Stripe, PayPal)
- Email services (SendGrid, Mailchimp)
- Analytics platforms (Google Analytics, Mixpanel)
- Cloud infrastructure (AWS, GCP, Azure)
- Third-party libraries and SDKs

**Why it matters:** Dependencies are risks. If a payment gateway goes down, your order processing stops. If an analytics platform changes its API, your tracking breaks. Understanding dependencies helps you plan for failures and changes.

**3. Constraint Context: What Limits You**

What technical, business, regulatory, or organizational constraints shape what's possible?

**Examples:**
- **Technical:** Performance requirements (p95 latency < 200ms), scalability (10,000 concurrent users), technology stack choices
- **Business:** Budget ($500/month for infrastructure), timeline (launch in Q3), revenue targets
- **Regulatory:** GDPR, HIPAA, PCI-DSS compliance requirements
- **Organizational:** Team size (3 engineers), skills (Node.js expertise), processes (CI/CD pipeline, code reviews)

**Why it matters:** You can't design a system that violates constraints. A system that costs $2,000/month when budget is $500/month will fail. A system that takes 6 months to build when deadline is Q3 will fail. Constraints are hard limits—you must work within them.

### Why Context Matters

Context determines success or failure.

A system designed without understanding its context will fail. It might work in theory, but fail in practice because it doesn't fit its environment.

**Example:**
You design a beautiful e-commerce system with real-time inventory, personalized recommendations, and instant notifications.

But you didn't consider the context:
- Your team has 3 developers with no experience with recommendation algorithms
- Your budget is $500/month, but real-time inventory and ML recommendations would cost $2,000/month
- Your compliance requirements include GDPR, but your system stores user data without proper consent mechanisms

The result? Your system is over-budget, over-engineered, and non-compliant. It fails despite having great features.

**With context:**
You understand your constraints (3 developers, $500/month budget, GDPR compliance) and design accordingly:
- Use simple rules-based recommendations (not ML)
- Use external recommendation service as dependency
- Implement proper consent mechanisms for GDPR
- Stay within budget and team capabilities

This system fits its context—it will succeed.

## Context Layers in Depth

### Stakeholder Context: Who Cares?

Let's explore stakeholder types in detail.

**Users:**
- **What they care about:** Easy to use, fast, reliable, works on their devices
- **Design implications:** Responsive design, offline support, graceful degradation, clear error messages
- **Example:** E-commerce users want fast checkout, order history, order tracking

**Administrators:**
- **What they care about:** Easy to manage, good visibility into operations, easy to troubleshoot
- **Design implications:** Admin dashboards, logging and monitoring, configuration management, clear error reporting
- **Example:** E-commerce admins want inventory management, order management, sales reports

**Developers:**
- **What they care about:** Easy to build, easy to test, clear code, good documentation
- **Design implications:** Clean architecture, well-defined APIs, modular code, integration tests
- **Example:** E-commerce developers want clear APIs for frontend, separation of concerns, testable code

**Business Owners:**
- **What they care about:** Delivered on time, under budget, generates revenue, meets goals
- **Design implications:** Milestone tracking, cost monitoring, feature prioritization, metrics dashboards
- **Example:** E-commerce owners want conversion metrics, revenue tracking, campaign performance

**Compliance Officers:**
- **What they care about:** Meets regulations, data security, audit trails, proper controls
- **Design implications:** Encryption at rest and in transit, access controls, audit logging, consent management
- **Example:** E-commerce compliance officers want PCI-DSS compliance (payment security), GDPR compliance (user data privacy), audit trails

**The key insight:** Each stakeholder has different needs. A system that works great for users might fail for administrators. You must consider all stakeholder types and make appropriate trade-offs.

### Dependency Context: What Do You Depend On?

Dependencies are everywhere in modern software. Understanding them is critical.

**Types of Dependencies:**

**Infrastructure dependencies:**
- Cloud providers (AWS, GCP, Azure)
- Data centers
- CDNs (Cloudflare, Akamai)
- DNS providers

**Service dependencies:**
- Payment gateways (Stripe, PayPal, Braintree)
- Email services (SendGrid, Mailchimp, AWS SES)
- Authentication providers (Auth0, Firebase Auth, Okta)
- Analytics platforms (Google Analytics, Mixpanel, Amplitude)
- Storage services (AWS S3, Google Cloud Storage)

**Library dependencies:**
- Frontend frameworks (React, Vue, Angular)
- Backend frameworks (Express, Django, Rails)
- Database drivers
- Utility libraries (moment, lodash, axios)

**Why dependencies matter:**

Each dependency is a potential single point of failure.

**Example:**
Your e-commerce system depends on:
- Stripe for payments
- SendGrid for emails
- AWS for hosting

What happens if Stripe goes down? You can't process payments. Orders stop coming in. Revenue stops.

What happens if SendGrid goes down? You can't send order confirmations. Users don't know their orders succeeded. They might place duplicate orders.

What happens if AWS has an outage? Your entire system is down.

**Designing for dependencies:**
- **Plan for failures:** What if this dependency goes down? Do you have a fallback?
- **Understand SLAs:** What uptime guarantees does this dependency provide?
- **Consider alternatives:** Could you switch providers if needed?
- **Document dependencies:** Make it clear what your system depends on

### Constraint Context: What Limits You?

Constraints are the boundaries you must work within. They determine what's possible.

**Types of Constraints:**

**Technical constraints:**
- **Performance:** Response time requirements (p95 < 200ms), throughput requirements (10,000 requests/second)
- **Scalability:** Concurrent user requirements (1,000, 10,000, 100,000)
- **Technology stack:** Must use Node.js because that's what your team knows
- **Data retention:** Must keep data for 7 years (regulatory requirement)

**Business constraints:**
- **Budget:** $500/month for infrastructure, $50,000/year total project budget
- **Timeline:** Launch in Q3, complete by end of year
- **Revenue targets:** Must generate $100,000/month by Q4
- **Team size:** 3 developers, 1 designer, 1 PM

**Regulatory constraints:**
- **GDPR:** User consent for data processing, right to be forgotten, data portability
- **PCI-DSS:** Secure payment card data, regular security audits
- **HIPAA:** Protect patient health information, access controls
- **SOC 2:** Security controls, audit trails, vulnerability management

**Organizational constraints:**
- **Team capabilities:** 3 developers with Node.js experience, no ML expertise
- **Processes:** Must use 2-week sprint cycle, code reviews required, deployment on Fridays
- **Culture:** No overtime allowed, remote-first team, distributed across time zones

**Designing for constraints:**
- **Identify constraints early:** What are your hard limits?
- **Make trade-offs explicit:** If you must stay under budget, what features get cut?
- **Design within constraints:** Don't design a system that violates constraints
- **Raise conflicts:** If constraints conflict (budget vs. timeline vs. scope), surface the issue

**Example:**
You're designing an e-commerce system with these constraints:
- Budget: $500/month for infrastructure
- Timeline: Launch in Q3 (3 months)
- Team: 3 Node.js developers, no ML expertise
- Business requirement: Personalized product recommendations

**Conflict:** Business wants ML-based recommendations, but team has no ML expertise and budget doesn't support ML infrastructure (would need $1,500/month for GPU instances).

**Resolution:** Use external recommendation service as a dependency. It costs $100/month and uses their ML expertise. System stays within budget and leverages external dependency.

This is context-aware design—you're working within constraints, not fighting them.

## Modeling Context in Sruja

Now let's see how to model context in Sruja.

### Stakeholders

```sruja
// People who interact with or influence your system
Customer = person "Customer"
Administrator = person "Administrator"
Developer = person "Developer"
BusinessOwner = person "Product Manager"
ComplianceOfficer = person "Compliance Officer"
```

### Dependencies

```sruja
// External systems you depend on
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical"]
    sla "99.9% uptime"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external"]
    priority "low"  // Email can be delayed
  }
}

AnalyticsService = system "Analytics Platform" {
  metadata {
    tags ["external"]
  }
}

CDN = system "Content Delivery Network" {
  metadata {
    tags ["external", "infrastructure"]
  }
}
```

### Constraints

```sruja
// Your system with constraints
Shop = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
  
  metadata {
    constraints {
      performance {
        p95 "200ms"
      }
      business {
        budget "$500/month"
      }
      compliance {
        regulations ["PCI-DSS", "GDPR"]
      }
      organizational {
        team_size "3 engineers"
      }
    }
  }
}
```

### Putting It Together

```sruja
import { * } from 'sruja.ai/stdlib'

// Stakeholders
Customer = person "Customer"
Administrator = person "Administrator"
Developer = person "Developer"
BusinessOwner = person "Product Manager"
ComplianceOfficer = person "Compliance Officer"

// Dependencies
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical"]
    sla "99.9% uptime"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external"]
    priority "low"
  }
}

AnalyticsService = system "Analytics Platform" {
  metadata {
    tags ["external"]
  }
}

// Your system with constraints
Shop = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
  
  metadata {
    constraints {
      performance {
        p95 "200ms"
      }
      business {
        budget "$500/month"
      }
      compliance {
        regulations ["PCI-DSS", "GDPR"]
      }
      organizational {
        team_size "3 engineers"
      }
    }
  }
}

// Stakeholder relationships
Customer -> Shop "Wants fast, reliable shopping"
Administrator -> Shop "Wants easy management"
Developer -> Shop "Wants clean, testable code"
BusinessOwner -> Shop "Wants revenue and on-time delivery"
ComplianceOfficer -> Shop "Wants data security and audit trails"

// Dependency relationships
Shop -> PaymentGateway "Depends on for payments"
Shop -> EmailService "Depends on for notifications"
Shop -> AnalyticsService "Depends on for tracking"
Shop -> CDN "Depends on for asset delivery"
```

### Analyzing the Context Model

**What this reveals:**

**Stakeholder needs:**
- Customer wants fast, reliable shopping → need performance and reliability
- Administrator wants easy management → need admin dashboards and monitoring
- Developer wants clean, testable code → need clean architecture and APIs
- Business owner wants revenue and on-time delivery → need metrics and monitoring
- Compliance officer wants data security and audit trails → need encryption, access controls, logging

**Dependency risks:**
- Payment gateway is critical (99.9% uptime) but external → if it's down, no revenue
- Email service is low priority (can be delayed) → less critical if it's down
- Analytics and CDN are external → outages possible but not revenue-critical

**Constraints:**
- Performance: p95 < 200ms → must optimize database queries, use caching
- Budget: $500/month → can't use expensive services, must be cost-conscious
- Compliance: PCI-DSS and GDPR → must encrypt payment data, get consent for user data
- Team: 3 engineers → can't over-engineer, must stay within capabilities

This context model tells you everything you need to know to design the right system.

## Common Mistakes

### Ignoring Stakeholders

**Mistake:** Designing for only one type of stakeholder (usually users).

**Example:** You design a great user experience but ignore administrators. Now they can't manage inventory, troubleshoot issues, or generate reports.

**Solution:** Identify all stakeholder types and consider their needs. Make trade-offs explicit.

### Forgetting Dependencies

**Mistake:** Assuming external systems will always be available.

**Example:** You design your system assuming the payment gateway is always up. It goes down during Black Friday. Orders stop processing. Revenue is lost.

**Solution:** Model all dependencies explicitly. Plan for failures. Have fallbacks (save orders as pending, show "try again later" message).

### Violating Constraints

**Mistake:** Designing a system that doesn't fit within constraints.

**Example:** You design an ML-based recommendation system with real-time inventory. It costs $2,000/month, but your budget is $500/month. You can't launch.

**Solution:** Identify constraints early. Design within them. If constraints conflict, surface the issue and make trade-offs explicit.

### Context Drift

**Mistake:** Designing a system for one context, then using it in a different context.

**Example:** You design an e-commerce system for a startup (small team, fast iterations, no heavy compliance). Your company gets acquired by an enterprise (large team, slow processes, heavy compliance). The same system doesn't fit the new context.

**Solution:** Make context explicit in your designs. When context changes, reevaluate whether the system still fits.

## What to Remember

The core idea of this lesson is simple but powerful: **Never design in isolation.**

Every system exists in a context—stakeholders who use it, dependencies it relies on, and constraints that limit it.

When you design without understanding context, you build systems that work in theory but fail in practice.

When you design with context:
- You understand stakeholder needs → build systems that work for everyone
- You understand dependencies → plan for failures and make trade-offs
- You understand constraints → design systems that fit within limits

The result? Systems that actually work—systems that succeed in their environment, not just on paper.

Context isn't just nice-to-have—it's essential.

## Check Your Understanding

Let's see if context makes sense to you.

### Quick Check

**1. You're designing a new analytics dashboard. Your business owner wants real-time metrics (updates every second), but your budget only allows for hourly batch data. What should you do?**

[ ] A. Ignore the budget constraint and build real-time
[ ] B. Build real-time anyway and hope for more budget later
[ ] C. Design within budget (hourly batch) and explain trade-offs to business owner
[ ] D. Ask for a bigger team to build real-time

**2. Your system depends on an external API that's documented as having 99.5% uptime. What does this tell you about availability planning?**

[ ] A. Your system will have 99.5% uptime
[ ] B. Your system's maximum possible uptime is 99.5%
[ ] C. You can ignore this dependency in availability calculations
[ ] D. You need to plan for 0.5% downtime and design fallbacks

---

### Answers & Discussion

**1. C. Design within budget (hourly batch) and explain trade-offs to business owner** – Budget is a hard constraint—you can't violate it. Build what's possible within constraints, then explain to the business owner what they're giving up (real-time updates) and what they're getting (cost savings). This is context-aware design—working within limits instead of ignoring them. If real-time is critical, they'll need to provide more budget.

**2. B. Your system's maximum possible uptime is 99.5%** – Your system depends on this external API, which means your system's availability is limited by its availability. Even if your internal components are perfect, your system can at most be 99.5% available. You need to plan for that 0.5% downtime—what happens when the API is down? Do you have cached data to show users? Can you queue requests for later? These are context-aware design decisions.

## What's Next

This completes **Module 1: Fundamentals**! You now have a complete toolkit for systems thinking:

- ✅ Systems thinking fundamentals
- ✅ The iceberg model (seeing deeper than surface events)
- ✅ Systems as systems of systems
- ✅ Parts and relationships
- ✅ Boundaries (what's inside vs. outside)
- ✅ Flows (how things move)
- ✅ Feedback loops (how systems adapt)
- ✅ Context (the environment your system lives in)

These eight concepts form the foundation for everything you'll learn in this course. You can now think about systems holistically—seeing them as complete, connected entities in context, not just isolated parts.

In the next modules, you'll apply these concepts to specific domains: modeling parts and relationships, defining boundaries, visualizing flows, designing feedback loops, and capturing context.

You're ready to dive deeper!
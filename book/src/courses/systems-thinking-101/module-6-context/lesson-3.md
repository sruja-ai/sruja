---
title: "Lesson 3: Dependencies and Constraints"
weight: 3
summary: "The external service you didn't document will be the one that goes down at 3 AM"
time: "8 min"
---

# The 3 AM Page: What Dependencies Really Cost

It was 3:14 AM when my phone buzzed. The payment system was down. Customers couldn't check out. Revenue was bleeding.

I stumbled to my laptop, pulled up the dashboards, and started debugging. Everything looked fine—our services were up, databases responding, APIs healthy. But payments kept failing.

Two hours later, I discovered the problem: an external email verification service we used had changed their API. We'd added it as a quick fix six months earlier, never documented it as a critical dependency, and forgotten about it. When they deprecated the old API endpoint, our checkout flow silently broke.

The cost? Four hours of downtime, thousands in lost sales, and a very uncomfortable conversation with the CEO at 6 AM.

The root cause wasn't technical complexity. It was missing documentation. We'd never modeled our dependencies properly, so we didn't know what we depended on—or how to fix it when it broke.

This lesson is about avoiding that 3 AM page. You'll learn how to document every external dependency, understand what constraints actually limit your choices, and define success criteria that prevent surprises.

## Learning Goals

By the end of this lesson, you'll be able to:

- Identify and categorize all external dependencies, not just the obvious ones
- Document dependencies with the information you'll need at 3 AM
- Recognize the four types of constraints and how they shape design
- Define success criteria and SLOs that actually reflect what matters
- Model complete context in Sruja so nothing gets forgotten

## Dependencies: The Systems You Don't Control

Here's a question that took me too long to ask: **What happens when your dependencies fail?**

Not if. When. Every external service goes down eventually. Every API changes. Every vendor has outages. The question isn't whether it will happen—it's whether you'll be prepared when it does.

Dependencies are external systems, services, or resources your system relies on to function. They're not part of your architecture, but they determine whether your architecture works.

Let me share three dependency failures that taught me why this matters.

### The Payment Gateway Outage

A startup I worked with built their entire checkout flow around Stripe. Clean integration, well-tested, ready to launch. Two days before launch, Stripe had a three-hour outage.

They had no fallback. No backup payment processor. No way to process payments manually. The launch was delayed a week while they scrambled to add a second payment provider.

**The lesson:** Critical dependencies need fallbacks. If you can't function without it, you need a Plan B.

### The Email Service Change

I consulted for a company that used SendGrid for transactional emails—password resets, order confirmations, welcome emails. After a year, SendGrid changed their pricing model, and the company's email costs tripled overnight.

They hadn't documented the dependency properly, so they didn't know how many emails they were sending or what alternatives existed. It took three months to migrate to a different provider because the email service was woven throughout the codebase.

**The lesson:** Document what you depend on, including costs and alternatives. Vendor changes happen.

### The Analytics Gap

A team built a real-time dashboard that depended on an analytics API. The dashboard was beautiful—until the analytics provider had an outage. The dashboard didn't fail gracefully; it showed zeros everywhere, causing panic among business users who thought all their traffic had disappeared.

**The lesson:** Know what happens when dependencies fail. Design for graceful degradation.

## Categorizing Dependencies (So You Know What Matters)

Not all dependencies are equal. I've learned to sort them into three buckets:

### Critical Dependencies: The System-Stoppers

These are dependencies your system cannot function without. If they go down, you go down.

**Examples:** Payment gateways, primary databases, authentication services, core APIs.

**How to handle them:**
- Document them as critical
- Have fallbacks or backups
- Monitor them closely
- Know your SLA and theirs

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical", "vendor"]
    owner "Stripe Inc."
    sla "99.9% uptime"
    mttr "4 hours"
    contact "support@stripe.com"
    fallback "Backup payment processor configured"
    fallback_activation "Manual switch, < 15 minutes"
    cost "$0.30 per transaction"
    compliance ["PCI-DSS Level 1"]
    
    // Critical info for 3 AM debugging
    monitoring "https://status.stripe.com"
    last_incident "2024-01-15 (2 hour outage)"
  }
}
```

Notice how much information I include. This isn't bureaucracy—it's the information you need at 3 AM when things break. Who owns it? What's the SLA? What's the fallback? How do you contact them? Where's the status page?

### Important Dependencies: The Degraded-Experience Ones

These are dependencies that cause problems when they fail, but the system still works in a degraded mode.

**Examples:** Email services, analytics, CDNs, notification systems.

**How to handle them:**
- Document the degradation behavior
- Queue work for later if possible
- Don't block core functionality

```sruja
EmailService = system "Email Service" {
  metadata {
    tags ["external", "important", "vendor"]
    owner "SendGrid"
    sla "99.0% uptime"
    impact "Emails delayed but system works"
    fallback "Queue emails locally, retry when service recovers"
    degradation "Users won't receive notifications immediately"
    
    cost "$14.95/month base + $0.0010/email"
    volume "50k emails/month"
  }
}
```

The key question: What happens when this goes down? If the answer is "users are annoyed but can still use the system," it's important but not critical.

### Optional Dependencies: The Nice-to-Haves

These dependencies add value but aren't essential. If they fail, the system works normally, just with fewer features.

**Examples:** Analytics services, A/B testing tools, non-essential integrations.

**How to handle them:**
- Don't let them block core functionality
- Fail gracefully and silently
- Monitor but don't alert at 3 AM

```sruja
AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "optional", "vendor"]
    owner "Google Analytics"
    impact "Analytics data lost but core functionality works"
    degradation "Dashboards show gaps in data"
    fallback "None - data loss acceptable"
    
    // Don't wake me up for this
    alerting "Business hours only"
  }
}
```

Optional doesn't mean unimportant. It means the system can function without it.

## Constraints: The Real Design Limits

Constraints are the limitations that shape your architecture. They're not suggestions—they're the boundaries you have to work within.

After years of fighting constraints, I've learned to embrace them. Constraints aren't obstacles; they're design inputs. The best architectures work with constraints, not against them.

### Technical Constraints: What's Technically Possible or Required

These are the technical realities you can't ignore.

**Examples:** "Must use PostgreSQL for ACID transactions," "Must deploy to AWS," "API response time under 200ms," "Support 10k concurrent users."

```sruja
Shop = system "Shop" {
  metadata {
    technical_constraints {
      "PostgreSQL required for transactional integrity",
      "Maximum API response time: 200ms (p95)",
      "Must support 10,000 concurrent users",
      "Deploy to AWS us-east-1 region",
      "Real-time inventory updates required"
    }
    
    // Why these constraints matter
    rationale {
      "PostgreSQL: Financial transactions require ACID",
      "200ms: User research shows >200ms feels slow",
      "10k users: Peak traffic from marketing campaigns"
    }
  }
}
```

Technical constraints often feel limiting, but they actually clarify decisions. When you know you need ACID transactions, you stop considering NoSQL databases. That's not a limitation—it's focus.

### Business Constraints: The Organizational Realities

These are the business realities: budgets, timelines, team size, strategic goals.

**Examples:** "Launch by Q4," "Budget is $500k/year," "Team of 3 engineers," "Must support international currencies."

```sruja
Shop = system "Shop" {
  metadata {
    business_constraints {
      "Launch date: December 1, 2024",
      "Infrastructure budget: $500k/year",
      "Team size: 3 engineers (growing to 5)",
      "Must support USD, EUR, GBP",
      "Mobile-first: 70% of traffic from mobile"
    }
    
    // These are as real as technical constraints
    rationale {
      "Dec 1 launch: Board commitment, marketing scheduled",
      "$500k: Approved budget, no flexibility",
      "3 engineers: Hiring takes 3 months per engineer"
    }
  }
}
```

Business constraints often frustrate engineers. "Why can't we have more budget? Why is the deadline fixed?" But fighting them doesn't help. Better to understand them and design within them.

### Compliance Constraints: The Rules You Must Follow

These are regulatory and legal requirements. You don't get to choose them; they choose you.

**Examples:** "PCI-DSS for payments," "GDPR for EU users," "HIPAA for health data," "SOC 2 for enterprise customers."

```sruja
Shop = system "Shop" {
  metadata {
    compliance_constraints {
      "PCI-DSS Level 1 (processing > 6M transactions/year)",
      "GDPR (EU customers)",
      "CCPA (California customers)",
      "SOC 2 Type II (enterprise customers require it)"
    }
    
    // Compliance drives architecture decisions
    implications {
      "PCI-DSS: Cannot store credit card data, must use tokenization",
      "GDPR: Right to deletion, data portability, consent management",
      "SOC 2: Audit logging, access controls, encryption required"
    }
  }
}
```

Compliance constraints are non-negotiable. You can't launch without them. Building them in from the start is far cheaper than adding them later.

### Security Constraints: The Protection Requirements

These are security requirements that shape how you build.

**Examples:** "All data encrypted at rest," "All API calls authenticated," "No PII in logs," "Minimum TLS 1.3."

```sruja
Shop = system "Shop" {
  metadata {
    security_constraints {
      "All data encrypted at rest (AES-256)",
      "All API calls authenticated (JWT)",
      "No PII in logs or error messages",
      "Minimum TLS 1.3 for all connections",
      "Secrets in vault, not in code"
    }
  }
}
```

Security constraints feel like overhead until something goes wrong. Then they're the difference between "we had a security incident" and "we went out of business."

## Success Criteria: How You Know You've Won

Success criteria answer a simple question: **How do you know if your system is successful?**

This seems obvious until you try to answer it. "It works"? Too vague. "Users like it"? Not measurable. "It makes money"? That's a business outcome, not a system property.

I've learned to define success at two levels: business outcomes and system properties.

### Business Outcomes (The "Why")

These are the business reasons the system exists:

```sruja
overview {
  summary "E-commerce platform for online retail"
  
  goals [
    "Increase online revenue by 25%",
    "Reduce abandoned carts by 15%",
    "Enable international expansion (EU, UK)",
    "Reduce support tickets by 30%"
  ]
  
  success_criteria [
    "Checkout completion rate > 80%",
    "Average checkout time < 2 minutes",
    "Customer satisfaction (NPS) > 50",
    "Support tickets per 1000 orders < 5"
  ]
}
```

These criteria connect the system to business value. They answer "why are we building this?"

### System Properties (The "How")

These are measurable system behaviors that support business outcomes. I use SLOs (Service Level Objectives):

```sruja
Shop = system "Shop" {
  slo {
    availability {
      target "99.9%"
      window "30 days"
      rationale "Less than 9 hours downtime per year"
    }

    latency {
      p95 "200ms"
      p99 "500ms"
      window "7 days"
      rationale "Research shows >200ms feels slow to users"
    }

    errorRate {
      target "0.1%"
      window "7 days"
      rationale "< 1 error per 1000 requests"
    }

    throughput {
      target "10000 req/s"
      window "peak hour"
      rationale "Peak traffic during marketing campaigns"
    }
  }
}
```

SLOs give you concrete targets. Is the system performing well? Check the SLOs. Are we ready to launch? Check the SLOs. Is something wrong? Check the SLOs.

### Non-Goals: What You're NOT Building

I've also learned to document what we're NOT doing. This prevents scope creep and sets expectations:

```sruja
overview {
  goals [
    "Fast checkout",
    "Mobile-first design",
    "Real-time inventory"
  ]
  
  non_goals [
    "Social features (reviews, sharing)",
    "Mobile app (web-only for now)",
    "Marketplace (first-party sales only)",
    "Subscription billing"
  ]
}
```

Non-goals are liberating. They let you say "that's a good idea, but it's out of scope" without feeling guilty.

## A Complete Context Example

Let me show you how everything in this module comes together. This is what a complete context model looks like—stakeholders, dependencies, constraints, and success criteria all in one place:

```sruja
import { * } from 'sruja.ai/stdlib'

// ============ OVERVIEW ============
// What are we building and why?

overview {
  summary "E-commerce platform for online retail"
  audience "Customers, administrators, business owners"
  scope "Shopping, checkout, order management, inventory"
  
  goals [
    "Increase online revenue by 25%",
    "Reduce abandoned carts by 15%",
    "Support international customers (EU, UK)"
  ]
  
  non_goals [
    "Social features (reviews, sharing)",
    "Mobile app (web-responsive only)",
    "Marketplace (first-party sales only)"
  ]
  
  risks [
    "Payment gateway downtime (critical dependency)",
    "Database scaling limits at peak traffic",
    "GDPR compliance complexity"
  ]
  
  success_criteria [
    "Checkout completion rate > 80%",
    "Average checkout time < 2 minutes",
    "Support 99.9% availability",
    "Page load time < 2s (p95)"
  ]
}

// ============ STAKEHOLDERS ============
// Who matters?

// Primary users
Customer = person "Customer" {
  description "Shoppers purchasing products"
  metadata {
    needs ["Fast checkout", "Easy search", "Mobile-friendly"]
    priority "critical"
  }
}

Administrator = person "Administrator" {
  description "Manages products, orders, inventory"
  metadata {
    needs ["Bulk operations", "Reporting", "Quick updates"]
    priority "high"
  }
}

// Secondary users
SupportAgent = person "Support Agent" {
  description "Helps customers with order issues"
  metadata {
    needs ["Customer lookup", "Order history", "Refund processing"]
    priority "high"
  }
}

// Business stakeholders
BusinessOwner = person "Business Owner" {
  description "Accountable for revenue and profit"
  metadata {
    needs ["Revenue reports", "Conversion metrics"]
    priority "high"
  }
}

// Compliance
ComplianceOfficer = person "Compliance Officer" {
  description "Ensures PCI-DSS and GDPR compliance"
  metadata {
    can_block_launch true
  }
}

// ============ SYSTEM ============
// What are we building?

Shop = system "Shop" {
  WebApp = container "Web Application" {
    technology "React"
  }
  
  API = container "API Service" {
    technology "Node.js"
  }
  
  Database = database "PostgreSQL" {
    technology "PostgreSQL 15"
  }
  
  Cache = database "Redis" {
    technology "Redis 7"
  }
  
  // Constraints
  metadata {
    team ["platform-team"]
    budget "$500k/year infrastructure"
    launch_date "2024-12-01"
    
    technical_constraints {
      "PostgreSQL required for ACID transactions",
      "Maximum API response time: 200ms (p95)",
      "Support 10,000 concurrent users"
    }
    
    business_constraints {
      "Launch by December 1, 2024",
      "Team of 3 engineers (growing to 5)",
      "Must support USD, EUR, GBP"
    }
    
    compliance_constraints {
      "PCI-DSS Level 1",
      "GDPR for EU customers",
      "CCPA for California customers"
    }
  }
  
  // Success criteria
  slo {
    availability {
      target "99.9%"
      window "30 days"
    }
    latency {
      p95 "200ms"
      p99 "500ms"
    }
    errorRate {
      target "0.1%"
    }
  }
}

// ============ DEPENDENCIES ============
// What do we depend on?

// Critical
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical", "vendor"]
    owner "Stripe Inc."
    sla "99.9% uptime"
    mttr "4 hours"
    contact "support@stripe.com"
    fallback "Backup payment processor (PayPal)"
    cost "$0.30/transaction + $0.30% fee"
    compliance ["PCI-DSS Level 1"]
  }
}

// Important
EmailService = system "Email Service" {
  metadata {
    tags ["external", "important", "vendor"]
    owner "SendGrid"
    sla "99.0% uptime"
    fallback "Queue locally, retry when service recovers"
    cost "$14.95/month + overage"
  }
}

// Optional
AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "optional", "vendor"]
    owner "Google Analytics"
    fallback "None - data loss acceptable"
  }
}

// ============ RELATIONSHIPS ============
// How does everything connect?

// Stakeholder interactions
Customer -> Shop.WebApp "Browses and purchases"
Administrator -> Shop.WebApp "Manages products and orders"
SupportAgent -> Shop.WebApp "Looks up customer info"
BusinessOwner -> Shop "Reviews revenue"
ComplianceOfficer -> Shop "Audits compliance"

// Dependencies
Shop.API -> PaymentGateway "Process payment" [critical]
Shop.API -> EmailService "Send notifications" [important]
Shop.API -> AnalyticsService "Track events" [optional]

view index {
  include *
}
```

This diagram tells a complete story. You can see:
- **What** we're building (overview)
- **Who** it's for (stakeholders)
- **What** we're building (system)
- **What** we depend on (dependencies)
- **What** limits us (constraints)
- **How** we measure success (SLOs)

That's the power of complete context modeling. Nothing gets forgotten.

## Documenting Decisions (And Why You Made Them)

One more thing I've learned: document your decisions, not just your architecture.

When you choose PostgreSQL over MongoDB, write down why. When you choose Stripe over building payments in-house, write down why. These decisions seem obvious now, but six months from now, you'll forget.

I use Architecture Decision Records (ADRs):

```sruja
ADR001 = adr "Use PostgreSQL for primary database" {
  status "accepted"
  date "2024-06-15"
  
  context "Need ACID transactions for orders and payments. Team has PostgreSQL experience. Must support complex queries for reporting."
  
  decision "Use PostgreSQL instead of MongoDB or MySQL"
  
  consequences {
    benefits "Strong consistency, ACID transactions, team expertise, mature tooling"
    tradeoffs "Horizontal scaling harder than NoSQL, requires careful schema design"
  }
  
  alternatives [
    "MongoDB: Rejected - no ACID transactions",
    "MySQL: Rejected - team less experienced, fewer advanced features"
  ]
}

ADR002 = adr "Use Stripe for payments" {
  status "accepted"
  date "2024-06-20"
  
  context "Need PCI-compliant payment processing. Building in-house would take 6+ months and require PCI certification."
  
  decision "Use Stripe instead of building in-house or using multiple providers"
  
  consequences {
    benefits "PCI compliance handled, fast integration, excellent documentation",
    tradeoffs "Per-transaction fees, vendor lock-in, limited customization"
  }
}
```

ADRs save you later when someone asks "Why did we do it this way?" or when you're considering a change and want to understand the original reasoning.

## What to Remember

**Dependencies will fail.** Document them before they do. Include criticality, SLAs, fallbacks, and 3 AM debugging information.

**Categorize dependencies:** Critical (need fallbacks), important (degrade gracefully), optional (fail silently).

**Constraints are design inputs, not obstacles.** Technical, business, compliance, and security constraints all shape your architecture. Work with them, not against them.

**Define success before you build.** Business outcomes connect to business value. SLOs give you measurable targets. Non-goals prevent scope creep.

**Document decisions.** ADRs capture why you made choices. They're invaluable when revisiting decisions or onboarding new team members.

**Context prevents 3 AM pages.** The dependency you didn't document, the constraint you ignored, the success criteria you never defined—these are the things that break in production at the worst possible time.

Modeling context isn't bureaucracy. It's survival.

## What's Next

Congratulations! You've completed **Module 6: Context** and the entire **Systems Thinking 101** course!

## 🎉 Course Complete!

You did it. You've made it through all six modules of Systems Thinking 101. That's no small achievement—this material fundamentally changes how you see software systems.

Let me recap what you've learned:

**Module 1: Fundamentals** - You learned what systems thinking is, the iceberg model, and why seeing the whole system matters more than seeing individual parts.

**Module 2: Parts and Relationships** - You learned how to identify system components and model how they connect and interact.

**Module 3: Boundaries** - You learned where systems start and end, what's inside vs. outside, and how to draw meaningful boundaries.

**Module 4: Flows** - You learned how data and control move through systems, and how to model the pathways that connect components.

**Module 5: Feedback Loops** - You learned about positive and negative feedback, self-regulating systems, and why cycles aren't errors.

**Module 6: Context** - You learned about the environment surrounding your system—stakeholders, dependencies, constraints, and success criteria.

You now think differently about architecture. You don't just see code—you see systems. You don't just see features—you see stakeholders and their competing needs. You don't just see databases—you see dependencies and failure modes.

This is the foundation. Everything else in architecture builds on this.

## What's Next for You?

You're ready for what comes next:

- **Practice:** Take a system you're working on and model it in Sruja. Apply what you've learned. See what you discover.

- **Go deeper:** The [System Design 101](../system-design-101/course-overview.md) course dives into specific patterns, trade-offs, and real-world architectures.

- **Explore:** Check out the [tutorials](../../tutorials/README.md) for hands-on exercises building real architectures.

- **Share:** Teach someone else what you've learned. The best way to solidify knowledge is to teach it.

## A Final Thought

I want to leave you with something that took me years to understand: **Great architecture isn't about being perfect. It's about being aware.**

You won't always make the right decisions. You won't always anticipate every problem. You'll still have 3 AM pages. But with systems thinking, you'll understand WHY things break, WHAT to do about them, and HOW to prevent the same problems next time.

That awareness—the ability to see systems holistically, to model their complexity, to anticipate their failures—that's what makes you an architect.

Now go build something amazing. And when it breaks at 3 AM (and it will), you'll know what to do.

**Congratulations on completing Systems Thinking 101!** 🚀
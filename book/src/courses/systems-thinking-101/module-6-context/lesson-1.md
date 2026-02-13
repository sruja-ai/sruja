---
title: "Lesson 1: Understanding Context"
weight: 1
summary: "Why do some technically perfect systems fail while messy ones succeed?"
time: "6 min"
---

# The Context Trap: Why Great Architecture Needs More Than Great Code

I once worked on what we thought was the perfect e-commerce platform. Clean microservices architecture, elegant APIs, comprehensive test coverage, the works. We were proud. Three months after launch, the project was cancelled.

The problem? We'd built the wrong thing.

Our payment processing was elegant—but the company had negotiated a deal with a specific payment provider we couldn't use. Our real-time inventory tracking was brilliant—but the warehouse team needed daily batches, not real-time updates. Our admin interface was beautiful—but the support team needed bulk operations, not pretty screens.

We had built a technical masterpiece that solved nobody's actual problems.

The missing piece was context. We'd designed the system in isolation, without understanding the organizational constraints, stakeholder needs, and business realities that surrounded it. The architecture was technically sound but organizationally wrong.

This lesson is about avoiding that trap. You'll learn how to see the invisible environment that shapes every system—the stakeholders, dependencies, constraints, and success criteria that determine whether your architecture succeeds or fails, regardless of how elegant your code might be.

## Learning Goals

By the end of this lesson, you'll be able to:

- Identify the multiple layers of context that surround any system
- Recognize why context matters just as much as technical architecture
- Model stakeholder, technical, and organizational context using Sruja
- Avoid the trap of designing systems in isolation
- Document constraints and dependencies that affect design decisions

## Context: The Invisible Environment

Here's a question that changed how I think about architecture: **What surrounds your system?**

Not what's inside it. Not what code it runs. But what's around it—the environment it operates in, the people who use it, the systems it depends on, the constraints it must satisfy.

Context is everything that affects your system or is affected by it, even though it's not part of the system itself. Think of it like the water a fish swims in—the fish doesn't see it, but it determines everything about how the fish lives.

```
┌─────────────────────────────────────────────┐
│              ORGANIZATIONAL CONTEXT          │
│  Company culture, processes, constraints    │
│                                             │
│   ┌─────────────────────────────────────┐   │
│   │       TECHNICAL CONTEXT            │   │
│   │  Dependencies, infrastructure, APIs  │   │
│   │                                   │   │
│   │  ┌─────────────────────────────┐  │   │
│   │  │    STAKEHOLDER CONTEXT     │  │   │
│   │  │  Users, teams, customers   │  │   │
│   │  │                           │  │   │
│   │  │  ┌───────────────────────┐ │  │   │
│   │  │  │   YOUR SYSTEM         │ │  │   │
│   │  │  └───────────────────────┘ │  │   │
│   │  └─────────────────────────────┘  │   │
│   └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

Each of these layers matters. Miss any of them, and you're designing in the dark.

### The Three Layers of Context

**Stakeholder context** answers the question: Who cares about this system? This includes users, certainly, but also business owners, support teams, compliance officers, and anyone else affected by what you build. Each has different needs, and ignoring any of them creates problems.

**Technical context** covers the systems and services yours depends on: payment gateways, email services, databases, APIs. These dependencies constrain what's possible and create failure modes you need to handle.

**Organizational context** captures the business realities: budget constraints, team size, compliance requirements, strategic goals, timelines. These often matter more than technical constraints but are easier to overlook.

## Why Context Matters (Beyond Theory)

Let me share three real experiences that taught me why context isn't optional.

### The Payment Gateway Surprise

A startup I advised built their entire checkout flow around a specific payment gateway's API. Clean code, well-tested, ready to ship. Two weeks before launch, they learned that their business deal required using a different gateway with a completely different API.

The technical work wasn't wasted, but they had to rewrite significant portions and delay launch by a month. Had they modeled their payment gateway as an external dependency with the constraint that it might change, they would have built an abstraction layer from the start.

The cost of missing context? A month of delay and a lot of rework.

### The Compliance Awakening

I consulted for a healthcare company that built a beautiful patient data system. It was fast, user-friendly, and technically impressive. Then they tried to deploy it and learned about HIPAA compliance requirements they'd never considered.

The system needed audit logging, data encryption at rest, access controls, and a dozen other features they hadn't built. Six months of work had to be redone to add compliance.

The cost of missing context? Six months of rework.

### The Stakeholder Disconnect

A team built an internal deployment tool that developers loved. It was elegant and powerful. But the DevOps team couldn't use it—it didn't integrate with their monitoring systems or provide the audit trails they needed for compliance.

Two teams, one tool, completely different needs. The developers got their tool, but the DevOps team continued using their old scripts, creating fragmentation and confusion.

The cost of missing context? A tool that solved half the problem and created new ones.

## Modeling Context in Sruja

This is where architecture diagrams become powerful. Sruja gives you specific tools to capture context, not just system internals.

### Documenting Stakeholders

Use `person` to capture who cares about your system:

```sruja
// The people who matter
Customer = person "Customer"
Administrator = person "Administrator"
SupportTeam = person "Support Team"
BusinessOwner = person "Business Owner"
ComplianceOfficer = person "Compliance Officer"
```

This isn't just documentation—it's a reminder that your system serves different people with different needs.

### Capturing External Dependencies

Use `system` for everything your system depends on:

```sruja
// What you depend on
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical", "pci-compliant"]
    sla "99.9% uptime"
    fallback "Manual payment processing"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "low-priority"]
    impact "Notifications delayed but system works"
  }
}

AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "optional"]
    impact "Analytics lost but core functionality works"
  }
}
```

The metadata captures critical information: Is this dependency critical? What happens if it fails? What's the SLA? This isn't just documentation—it's risk assessment.

### Showing Stakeholder Needs

Relationships reveal what different stakeholders need:

```sruja
// Different stakeholders, different needs
Customer -> Shop "Wants fast checkout"
Administrator -> Shop "Wants easy management"
BusinessOwner -> Shop "Wants high conversion"
ComplianceOfficer -> Shop "Wants data security"
```

These arrows seem simple, but they remind you that you're balancing competing needs. Fast checkout might conflict with security. Easy management might conflict with simplicity. These tensions are real and ignoring them doesn't make them go away.

### Documenting Constraints

Use `metadata` to capture organizational realities:

```sruja
Shop = system "Shop" {
  metadata {
    constraints {
      "PCI-DSS compliance required",
      "Maximum response time: 2s",
      "Budget: $500/month infrastructure",
      "Team size: 3 engineers"
    }
  }
}
```

These constraints are just as real as technical constraints. A $500/month budget limits your infrastructure choices. A 3-person team limits how complex the system can be. Pretending these don't exist doesn't help anyone.

### Defining Success

Use `slo` and `success_criteria` to define what "good" looks like:

```sruja
Shop = system "Shop" {
  slo {
    availability {
      target "99.9%"
    }
    latency {
      p95 "200ms"
    }
  }

  metadata {
    success_criteria {
      "Support 10k concurrent users",
      "Less than 1% abandoned carts",
      "Checkout completion rate > 80%"
    }
  }
}
```

Without explicit success criteria, how do you know if you've succeeded? How do you make trade-offs? How do you prioritize features?

## A Complete Example

Let me show you how this comes together in a real architecture:

```sruja
import { * } from 'sruja.ai/stdlib'

// Stakeholder context: Who cares?
Customer = person "Customer"
Administrator = person "Administrator"
BusinessOwner = person "Business Owner"
SupportTeam = person "Support Team"

// Your system
Shop = system "Shop" {
  metadata {
    constraints {
      "PCI-DSS compliance required",
      "Team size: 3 engineers"
    }
  }
}

// Technical context: What do you depend on?
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical"]
    sla "99.9% uptime"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "low-priority"]
  }
}

AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external"]
  }
}

// Show the dependencies
Shop -> PaymentGateway "Depends on for payments"
Shop -> EmailService "Depends on for notifications"
Shop -> AnalyticsService "Depends on for tracking"

// Show stakeholder needs
Customer -> Shop "Wants fast, reliable shopping"
Administrator -> Shop "Wants easy management"
BusinessOwner -> Shop "Wants high revenue"
SupportTeam -> Shop "Wants clear error messages"

view index {
  include *
}
```

This diagram tells a story. You can see who matters, what the system depends on, what the constraints are, and what success looks like. That's the power of modeling context.

## Common Context Mistakes

After years of watching teams struggle with this, I've seen a few patterns repeat.

**Mistake 1: Ignoring context entirely.** Teams build elegant systems that solve the wrong problems. The architecture is technically beautiful but organizationally useless.

**Mistake 2: Adding too much context.** Some teams try to document everything and everyone, creating diagrams that are more confusing than helpful. Focus on stakeholders who directly affect or are affected by the system.

**Mistake 3: Context at the wrong level.** Documenting individual databases and caches instead of higher-level services. Context should clarify, not overwhelm.

The key is balance: enough context to understand the system's environment, not so much that you lose the system itself.

## What to Remember

Context isn't optional—it's half the architecture. The most elegant technical solution fails if it ignores stakeholder needs, organizational constraints, or external dependencies.

**Model stakeholders explicitly.** Use `person` to capture who cares about your system and what they need.

**Document dependencies clearly.** Use `system` for external services and metadata to capture criticality and fallbacks.

**Capture constraints honestly.** Budget, team size, compliance, timelines—these are real constraints that shape design decisions.

**Define success criteria.** Without explicit goals, you can't make trade-offs or know when you're done.

**Balance detail.** Enough context to understand the environment, not so much that the system gets lost.

Context-aware architecture isn't about adding more documentation—it's about seeing the full picture before you start building. The time you spend understanding context pays back tenfold in avoided rework and better decisions.

## What's Next

Now that you understand context as a concept, [Lesson 2](lesson-2.md) dives deep into stakeholders—the people who use, build, and depend on your system. You'll learn how to identify different stakeholder types and model their needs in Sruja.
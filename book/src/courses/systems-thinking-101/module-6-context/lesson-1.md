# Lesson 1: Understanding Context

## Learning Goals

- Understand what context is in systems thinking
- Recognize the different layers of context
- Learn why context matters for architecture

## What Is Context?

Context is the environment your system operates in. It includes everything that affects or is affected by your system, even if it's not part of the system itself.

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

## Layers of Context

### 1. Stakeholder Context

Who cares about your system?

```sruja
// People who use or influence the system
Customer = person "Customer"
Administrator = person "Administrator"
SupportTeam = person "Support Team"
BusinessOwner = person "Business Owner"
```

### 2. Technical Context

What technical dependencies exist?

```sruja
// External systems and services
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
Analytics = system "Analytics Platform"
CDN = system "Content Delivery Network"
```

### 3. Organizational Context

What organizational factors affect the system?

```sruja
// Documented in metadata
Shop = system "Shop" {
  metadata {
    team ["platform-team"]
    business_unit ["e-commerce"]
    compliance ["PCI-DSS"]
    cost_center ["engineering"]
  }
}
```

## Why Context Matters

### 1. Stakeholder Alignment

Understand who your system serves:

```sruja
// Different stakeholders have different needs
Customer -> Shop "Wants fast checkout"
Administrator -> Shop "Wants easy management"
BusinessOwner -> Shop "Wants high conversion"
ComplianceOfficer -> Shop "Wants data security"
```

### 2. Dependency Management

Know what you depend on:

```sruja
Shop = system "Shop"

// Explicit dependencies
Shop -> PaymentGateway "Depends on for payments"
Shop -> EmailService "Depends on for notifications"
Shop -> Analytics "Depends on for tracking"

// If any dependency fails, what's the impact?
```

### 3. Constraint Awareness

Understand limitations:

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

### 4. Success Criteria

Define what success looks like:

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

## Context in Sruja

### Using `overview` Block

```sruja
overview {
  summary "E-commerce platform for online retail"
  audience "Customers, administrators, business owners"
  scope "Shopping cart, checkout, order management"
  goals [
    "Fast and reliable checkout",
    "Easy order management",
    "Real-time inventory tracking"
  ]
  non_goals [
    "Social features",
    "Mobile app (web-only)"
  ]
  risks [
    "Payment gateway downtime",
    "Database scaling limits"
  ]
}
```

### Using `person` for Stakeholders

```sruja
// Different types of stakeholders
Customer = person "Customer"
Administrator = person "Administrator"
SupportAgent = person "Support Agent"
ProductManager = person "Product Manager"
ComplianceOfficer = person "Compliance Officer"
```

### Using `system` for External Dependencies

```sruja
// Document external services
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
```

### Using `requirements`

```sruja
// Capture constraints and requirements
R1 = requirement functional "Must support multiple payment methods"
R2 = requirement constraint "Must be PCI-DSS compliant"
R3 = requirement performance "Page load time < 2 seconds"
R4 = requirement security "All data encrypted at rest"
```

## Context Examples

### Example 1: E-Commerce Platform

```sruja
import { * } from 'sruja.ai/stdlib'

// Stakeholder context
Customer = person "Customer"
Administrator = person "Administrator"
BusinessOwner = person "Business Owner"
SupportTeam = person "Support Team"

// System
Shop = system "Shop"

// Technical context (dependencies)
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical", "pci-compliant"]
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

// Relationships show dependencies
Shop -> PaymentGateway "Depends on for payments"
Shop -> EmailService "Depends on for notifications"
Shop -> AnalyticsService "Depends on for tracking"

Customer -> Shop "Wants fast, reliable shopping"
Administrator -> Shop "Wants easy management"
BusinessOwner -> Shop "Wants high revenue"
SupportTeam -> Shop "Wants clear error messages"

view index {
  include *
}
```

### Example 2: Internal Tool

```sruja
// Stakeholder context (internal only)
Developer = person "Developer"
QAEngineer = person "QA Engineer"
DevOpsEngineer = person "DevOps Engineer"

// System
DeploymentTool = system "Deployment Tool"

// Technical context
GitHub = system "GitHub" {
  metadata {
    tags ["external"]
  }
}

AWS = system "Amazon Web Services" {
  metadata {
    tags ["external", "infrastructure"]
  }
}

Slack = system "Slack" {
  metadata {
    tags ["external", "communication"]
  }
}

// Dependencies
DeploymentTool -> GitHub "Fetches code"
DeploymentTool -> AWS "Deploys to AWS"
DeploymentTool -> Slack "Sends notifications"

// Context shows different stakeholders have different needs
Developer -> DeploymentTool "Wants simple interface"
QAEngineer -> DeploymentTool "Wants detailed logs"
DevOpsEngineer -> DeploymentTool "Wants monitoring & alerts"
```

## Context Anti-Patterns

### Anti-Pattern 1: No Context

```sruja
// Bad: System in isolation
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}
```

**Solution**: Add stakeholders and dependencies.

### Anti-Pattern 2: Too Much Context

```sruja
// Bad: Everything is context, system is lost
Customer = person "Customer"
Manager = person "Manager"
Support = person "Support"
Finance = person "Finance"
Legal = person "Legal"
HR = person "HR"
Marketing = person "Marketing"
// ... 20 more stakeholders
```

**Solution**: Focus on key stakeholders directly affected.

### Anti-Pattern 3: Wrong Level of Context

```sruja
// Bad: Too detailed (implementation, not context)
Shop = system "Shop"
PostgreSQL = system "PostgreSQL"
Redis = system "Redis"
Nginx = system "Nginx"
```

**Solution**: Group into higher-level services.

## Exercise

Identify the context layers for a hotel booking system:

**Requirements**: "A hotel booking system allows guests to search rooms, make reservations, and manage bookings. Hotel staff can manage room inventory and view reports. The system integrates with a payment gateway for payments and sends SMS confirmations. Hotel management needs reporting on occupancy and revenue."

**Identify:**

1. Stakeholders: ******\_******
2. External dependencies: ******\_******
3. Organizational constraints: ******\_******

## Key Takeaways

1. **Context is the environment**: Everything affecting or affected by your system
2. **Multiple layers**: Stakeholder, technical, organizational
3. **Use Sruja features**: `overview`, `person`, `system`, `requirements`
4. **Balance detail**: Don't ignore context, don't overwhelm
5. **Context matters**: Affects design, decisions, and success

## Next Lesson

In [Lesson 2](lesson-2.md), you'll learn how to model stakeholders and their relationships to your system.

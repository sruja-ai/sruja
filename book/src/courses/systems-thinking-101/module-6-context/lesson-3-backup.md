# Lesson 3: Dependencies and Constraints

## Learning Goals

- Identify and document external dependencies
- Define constraints and limitations
- Capture success criteria and SLOs
- Use Sruja features to model context

## External Dependencies

### What Are Dependencies?

Dependencies are external systems, services, or resources your system relies on to function.

```sruja
// Your system
Shop = system "Shop"

// External dependencies
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"
AnalyticsService = system "Analytics Service"
CDN = system "Content Delivery Network"
```

### Categorizing Dependencies

#### Critical Dependencies

System cannot function without:

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical", "pci-compliant"]
    sla "99.9% uptime"
    impact "If down, checkout fails"
  }
}
```

#### Important Dependencies

System can function but degraded:

```sruja
EmailService = system "Email Service" {
  metadata {
    tags ["external", "important"]
    sla "99.0% uptime"
    impact "If down, notifications delayed but system works"
  }
}
```

#### Optional Dependencies

System works fine without:

```sruja
AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "optional"]
    sla "98.0% uptime"
    impact "If down, analytics lost but core functionality works"
  }
}
```

## Documenting Dependencies

### Using Metadata

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe Inc."
    sla "99.9% uptime"
    mttr "4 hours"
    contact "support@stripe.com"
    fallback "Manual payment processing"
    cost "$0.30 per transaction"
    compliance ["PCI-DSS Level 1"]
  }
}
```

### Using Relationships

```sruja
// Shows dependency
Shop.API -> PaymentGateway "Process payment" [critical]
Shop.API -> EmailService "Send notifications" [important]
Shop.API -> AnalyticsService "Track events" [optional]
```

### Using Fallbacks

```sruja
// Primary provider
PrimaryPayment = system "Primary Payment Gateway"
BackupPayment = system "Backup Payment Gateway"

Shop.API -> PrimaryPayment "Process payment" [primary]
Shop.API -> BackupPayment "Process payment" [fallback]
```

## Constraints

### What Are Constraints?

Constraints are limitations that affect your design and implementation choices.

### Types of Constraints

#### Technical Constraints

```sruja
Shop = system "Shop" {
  metadata {
    technical_constraints {
      "Must use PostgreSQL for transactions",
      "Must support 10k concurrent users",
      "Maximum API response time: 2s",
      "Must be deployable to AWS"
    }
  }
}
```

#### Business Constraints

```sruja
Shop = system "Shop" {
  metadata {
    business_constraints {
      "Launch date: Q4 2024",
      "Budget: $500k/year infrastructure",
      "Team size: 3 engineers",
      "Must support multi-currency"
    }
  }
}
```

#### Compliance Constraints

```sruja
Shop = system "Shop" {
  metadata {
    compliance_constraints {
      "PCI-DSS Level 1 for payments",
      "GDPR for EU customer data",
      "CCPA for California customers",
      "SOX compliance for financial reporting"
    }
  }
}
```

#### Security Constraints

```sruja
Shop = system "Shop" {
  metadata {
    security_constraints {
      "All data encrypted at rest",
      "All API calls authenticated",
      "No PII in logs",
      "Minimum TLS 1.3"
    }
  }
}
```

## Success Criteria

### What Is Success?

Define what "good" looks like for your system.

### Using `overview` Block

```sruja
overview {
  summary "E-commerce platform for online retail"
  audience "Customers, administrators, business owners"

  goals [
    "Fast and reliable checkout",
    "Easy product discovery",
    "Real-time inventory tracking",
    "Scalable to 10k concurrent users"
  ]

  non_goals [
    "Social features",
    "Mobile app (web-only)",
    "Marketplace (first-party only)"
  ]

  success_criteria [
    "Checkout completion rate > 80%",
    "Average checkout time < 2 minutes",
    "Page load time < 2s",
    "Customer satisfaction > 4.5/5"
  ]
}
```

### Using `slo` Block

```sruja
Shop = system "Shop" {
  slo {
    availability {
      target "99.9%"
      window "30 days"
    }

    latency {
      p95 "200ms"
      p99 "500ms"
      window "7 days"
    }

    errorRate {
      target "0.1%"
      window "7 days"
    }

    throughput {
      target "10000 req/s"
      window "peak hour"
    }
  }
}
```

## Complete Context Example

```sruja
import { * } from 'sruja.ai/stdlib'

// OVERVIEW
overview {
  summary "E-commerce platform for online retail"
  audience "Customers, administrators, business owners"
  scope "Shopping, checkout, order management, inventory"
  goals [
    "Fast and reliable checkout",
    "Real-time inventory",
    "Scalable architecture"
  ]
  non_goals [
    "Social features",
    "Marketplace"
  ]
  risks [
    "Payment gateway downtime",
    "Database scaling limits",
    "High traffic surges"
  ]
}

// STAKEHOLDERS
Customer = person "Customer"
Administrator = person "Administrator"
BusinessOwner = person "Business Owner"
SupportAgent = person "Support Agent"

// SYSTEM
Shop = system "Shop" {
  metadata {
    team ["platform-team"]
    budget "$500k/year"
    launch_date "2024-12-01"
  }

  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "PostgreSQL"
  Cache = database "Redis"

  // CONSTRAINTS
  constraints {
    "Must support 10k concurrent users",
    "Maximum response time: 2s",
    "PCI-DSS Level 1 compliance",
    "All data encrypted at rest"
  }

  // SUCCESS CRITERIA (SLOs)
  slo {
    availability {
      target "99.9%"
      window "30 days"
    }

    latency {
      p95 "200ms"
      p99 "500ms"
    }
  }
}

// DEPENDENCIES
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "critical", "vendor"]
    owner "Stripe Inc."
    sla "99.9% uptime"
    mttr "4 hours"
    cost "$0.30/transaction"
    compliance ["PCI-DSS Level 1"]
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "important", "vendor"]
    owner "SendGrid"
    sla "99.0% uptime"
    fallback "Queue for later"
  }
}

AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "optional", "vendor"]
    owner "Google"
    sla "98.0% uptime"
  }
}

// RELATIONSHIPS
Customer -> Shop.WebApp "Purchases products"
Administrator -> Shop.WebApp "Manages system"

Shop.API -> PaymentGateway "Process payment" [critical]
Shop.API -> EmailService "Send notifications" [important]
Shop.API -> AnalyticsService "Track events" [optional]

view index {
  include *
}
```

## Requirements and Constraints

### Using `requirements`

```sruja
// Functional requirements
R1 = requirement functional "Must support multiple payment methods"
R2 = requirement functional "Guest checkout available"
R3 = requirement functional "Order tracking"

// Non-functional requirements
R4 = requirement performance "Page load time < 2s"
R5 = requirement availability "99.9% uptime"
R6 = requirement scalability "Support 10k concurrent users"

// Security requirements
R7 = requirement security "All data encrypted at rest"
R8 = requirement security "TLS 1.3 for all connections"

// Compliance requirements
R9 = requirement constraint "PCI-DSS Level 1"
R10 = requirement constraint "GDPR compliance for EU users"
```

### Using `constraints` Block

```sruja
constraints {
  "All APIs must use HTTPS",
  "Database must be encrypted at rest",
  "No PII in logs",
  "Maximum API response time: 2s",
  "Must support 99.9% uptime",
  "PCI-DSS compliance required"
}
```

## Documenting Trade-offs

### Decision Records (ADRs)

```sruja
ADR001 = adr "Use PostgreSQL for primary database" {
  status "accepted"
  context "Need ACID transactions, strong consistency for orders"
  decision "Use PostgreSQL over MongoDB"
  consequences {
    benefits "Strong consistency, ACID transactions",
    tradeoffs "Scaling requires more effort than NoSQL"
  }
}

ADR002 = adr "Use Stripe for payments" {
  status "accepted"
  context "Need PCI-compliant payment processing"
  decision "Use Stripe over building in-house"
  consequences {
    benefits "PCI compliance, focus on core product",
    tradeoffs "Per-transaction fee, vendor lock-in"
  }
}
```

## Exercise

Document context for a fitness tracking app:

> "A fitness tracking app allows users to log workouts and view progress. Users can sync data from wearables. The app integrates with HealthKit and Google Fit. The app must work offline and sync when online. HIPAA compliance required for health data. Target 1 million users by year-end."

**Document:**

1. Stakeholders
2. External dependencies (with criticality)
3. Constraints (technical, business, compliance)
4. Success criteria (SLOs)

## Key Takeaways

1. **Document dependencies**: External systems you rely on
2. **Categorize by importance**: Critical, important, optional
3. **Define constraints**: Limitations affecting design
4. **Set success criteria**: What "good" looks like
5. **Capture trade-offs**: ADRs for important decisions

## Module 6 Complete

You've completed Context! You now understand:

- What context is and why it matters
- How to model stakeholders
- How to document dependencies and constraints

## 🎉 Course Complete!

You've finished **Systems Thinking 101**! You now understand:

- ✅ Fundamentals of systems thinking
- ✅ Parts and relationships
- ✅ Boundaries
- ✅ Flows
- ✅ Feedback loops
- ✅ Context

## What's Next?

- Try the [System Design 101](../system-design-101/course-overview.md) course
- Explore the [Systems Thinking tutorial](../../tutorials/basic/systems-thinking.md)
- Review the [Beginner path](../../docs/beginner-path.md)
- Build your own systems thinking architecture!

Congratulations on completing the course! 🚀

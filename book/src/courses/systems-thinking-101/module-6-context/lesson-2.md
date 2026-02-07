# Lesson 2: Stakeholders

## Learning Goals

- Identify key stakeholders for your system
- Model different stakeholder types
- Document stakeholder needs and concerns

## Who Are Stakeholders?

Stakeholders are people or groups who are affected by or can affect your system. They include users, customers, team members, and anyone else with an interest in the system.

## Stakeholder Categories

### 1. Primary Users

Direct users of the system:

```sruja
Customer = person "Customer" {
  description "End users who purchase products"
  metadata {
    needs ["Fast checkout", "Easy product search", "Order tracking"]
    pain_points ["Complex checkout", "Slow search results"]
  }
}

Administrator = person "Administrator" {
  description "Manages products, orders, and users"
  metadata {
    needs ["Dashboard", "Bulk operations", "Reporting"]
  }
}
```

### 2. Secondary Users

Users who use the system indirectly:

```sruja
SupportAgent = person "Support Agent" {
  description "Helps customers with orders"
  metadata {
    needs ["Order history", "Customer lookup", "Quick access"]
  }
}
```

### 3. Business Stakeholders

Decision-makers and business owners:

```sruja
ProductManager = person "Product Manager" {
  description "Owns product strategy"
  metadata {
    needs ["Analytics", "User feedback", "Feature usage"]
  }
}

BusinessOwner = person "Business Owner" {
  description "Responsible for revenue and profit"
  metadata {
    needs ["Sales reports", "Conversion metrics", "ROI data"]
  }
}
```

### 4. Technical Stakeholders

People who build and maintain the system:

```sruja
Developer = person "Developer" {
  description "Builds and maintains the system"
  metadata {
    needs ["API documentation", "Clear architecture", "Debugging tools"]
  }
}

DevOpsEngineer = person "DevOps Engineer" {
  description "Deploys and operates the system"
  metadata {
    needs ["Monitoring", "Logs", "Health checks"]
  }
}
```

### 5. Compliance & Governance

People ensuring rules are followed:

```sruja
ComplianceOfficer = person "Compliance Officer" {
  description "Ensures regulatory compliance"
  metadata {
    needs ["Audit logs", "Data privacy controls", "Access logs"]
  }
}

SecurityAuditor = person "Security Auditor" {
  description "Reviews security posture"
  metadata {
    needs ["Security reports", "Vulnerability assessments", "Penetration test results"]
  }
}
```

## Modeling Stakeholders in Sruja

### Basic Stakeholder

```sruja
Customer = person "Customer"
```

### With Details

```sruja
Customer = person "Customer" {
  description "End users who purchase products"
  metadata {
    tags ["primary-user", "external"]
    priority "high"
    needs [
      "Fast and easy checkout",
      "Product search and filtering",
      "Order tracking"
    ]
    pain_points [
      "Complex forms",
      "Slow page loads",
      "Lack of mobile support"
    ]
  }
}
```

### With Relationships

```sruja
Customer = person "Customer"
Shop = system "Shop"

// Customer relationship shows interaction
Customer -> Shop "Purchases products"
Shop -> Customer "Sends order updates"
```

## Stakeholder Interactions

### Example: E-Commerce Platform

```sruja
import { * } from 'sruja.ai/stdlib'

// Stakeholders
Customer = person "Customer"
Administrator = person "Administrator"
SupportAgent = person "Support Agent"
ProductManager = person "Product Manager"
BusinessOwner = person "Business Owner"

// System
Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "Database"
}

// Stakeholder interactions
Customer -> Shop.WebApp "Browses products"
Customer -> Shop.WebApp "Purchases products"
Customer -> Shop.WebApp "Tracks orders"

Administrator -> Shop.WebApp "Manages products"
Administrator -> Shop.WebApp "Views reports"

SupportAgent -> Shop.WebApp "Assists customers"
SupportAgent -> Shop.WebApp "Views order history"

ProductManager -> Shop.WebApp "Reviews analytics"
ProductManager -> Shop.WebApp "Monitors user behavior"

BusinessOwner -> Shop.WebApp "Views revenue reports"
BusinessOwner -> Shop.WebApp "Monitors KPIs"

view index {
  include *
}
```

## Stakeholder Matrix

| Stakeholder     | Role             | Needs                        | Concerns             |
| --------------- | ---------------- | ---------------------------- | -------------------- |
| Customer        | End user         | Fast checkout, easy search   | Privacy, reliability |
| Administrator   | System admin     | Management tools, reports    | Ease of use          |
| Support Agent   | Customer support | Customer data, order history | Quick access         |
| Product Manager | Product owner    | Analytics, user feedback     | Feature adoption     |
| Business Owner  | Decision maker   | Revenue, conversion, ROI     | Profitability, costs |

## Prioritizing Stakeholders

### MoSCoW Method

```sruja
// Must Have (Critical)
Customer = person "Customer"

// Should Have (Important)
Administrator = person "Administrator"

// Could Have (Nice to have)
ProductManager = person "Product Manager"

// Won't Have (Out of scope)
MarketingAnalyst = person "Marketing Analyst"
```

### RICE Scoring (for Features)

```sruja
// High impact, low effort
Customer = person "Customer" {
  metadata {
    rice_score {
      reach "10000 users"
      impact "High"
      confidence "80%"
      effort "2 weeks"
    }
  }
}
```

## Stakeholder Personas

### Creating Personas

```sruja
// Persona 1: Primary user
Sarah = person "Sarah (Customer)" {
  description "Busy professional, 35, shops on mobile"
  metadata {
    goals ["Fast checkout", "Mobile-friendly", "Easy returns"]
    frustrations ["Complex forms", "Slow loading"]
    context ["Shops during commute", "Uses iPhone"]
  }
}

// Persona 2: Secondary user
John = person "John (Administrator)" {
  description "Operations manager, 45, manages inventory"
  metadata {
    goals ["Quick updates", "Bulk operations", "Real-time data"]
    frustrations ["Slow interface", "Lack of filters"]
    context ["Desktop user", "Excel background"]
  }
}
```

## Stakeholder-System Relationships

### Direct Users

```sruja
// Interact directly with the system
Customer -> Shop.WebApp "Uses"
Administrator -> Shop.WebApp "Manages"
```

### Indirect Users

```sruja
// System serves their needs without direct interaction
BusinessOwner -> Shop "Reviews revenue"
// Business owner doesn't use the app directly
```

### External Stakeholders

```sruja
// Affected by system but don't use it
ComplianceOfficer = person "Compliance Officer"
ComplianceOfficer -> Shop "Reviews audit logs"
```

## Documenting Stakeholder Needs

### Using Requirements

```sruja
// Customer needs
R1 = requirement functional "Guest checkout available"
R2 = requirement performance "Page load < 2s"

// Administrator needs
R3 = requirement functional "Bulk product upload"
R4 = requirement usability "Intuitive dashboard"

// Business owner needs
R5 = requirement reporting "Daily revenue reports"
R6 = requirement analytics "Conversion funnel tracking"
```

### Using SLOs

```sruja
Shop = system "Shop" {
  slo {
    availability {
      target "99.9%"
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
```

## Exercise

Identify stakeholders for a hospital appointment scheduling system:

> "The system allows patients to book appointments, doctors to view their schedules, and receptionists to manage appointments. Hospital administrators need reporting on utilization. The system integrates with insurance APIs and sends SMS reminders."

**Identify:**

1. Primary stakeholders: ******\_******
2. Secondary stakeholders: ******\_******
3. Business stakeholders: ******\_******
4. Technical stakeholders: ******\_******

**For each, document:**

- Their role
- What they need from the system
- Any concerns or pain points

## Key Takeaways

1. **Identify all stakeholders**: Users, business, technical, compliance
2. **Categorize them**: Primary, secondary, business, technical
3. **Document needs**: Goals, frustrations, context
4. **Prioritize**: Focus on most important stakeholders
5. **Model relationships**: Show how stakeholders interact with the system
6. **Use personas**: Create detailed user profiles

## Next Lesson

In [Lesson 3](lesson-3.md), you'll learn how to document dependencies, constraints, and success criteria.

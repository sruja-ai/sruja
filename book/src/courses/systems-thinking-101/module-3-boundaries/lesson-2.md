# Lesson 2: Internal vs External

## Learning Goals

- Learn how to mark components as internal or external
- Use Sruja metadata to annotate boundary elements
- Model team and organizational boundaries

## Marking External Systems

### Basic Pattern

```sruja
// Internal: Your system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API Service"
}

// External: Third-party system
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}
```

## Metadata for External Systems

### External Tag

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}
```

### External with Ownership

```sruja
Stripe = system "Stripe" {
  metadata {
    tags ["external", "vendor"]
    owner "Stripe Inc."
  }
}
```

### External with SLA

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    sla "99.9% uptime"
  }
}
```

### External with Compliance

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "pci-compliant"]
    compliance ["PCI-DSS Level 1"]
  }
}
```

## Internal vs External Patterns

### Pattern 1: Third-Party Services

```sruja
// Your system
Shop = system "Shop"

// Third-party integrations
Stripe = system "Stripe" {
  metadata {
    tags ["external", "vendor", "pci-compliant"]
    owner "Stripe"
  }
}

Twilio = system "Twilio" {
  metadata {
    tags ["external", "vendor"]
    owner "Twilio"
  }
}

GoogleAnalytics = system "Google Analytics" {
  metadata {
    tags ["external", "vendor"]
    owner "Google"
  }
}
```

### Pattern 2: Partner Integrations

```sruja
// Your system
Shop = system "Shop"

// Partner systems
LogisticsPartner = system "Logistics Partner API" {
  metadata {
    tags ["external", "partner"]
    owner "FedEx"
  }
}

InventoryPartner = system "Inventory Partner" {
  metadata {
    tags ["external", "partner"]
    owner "Vendor X"
  }
}
```

### Pattern 3: Internal Team Boundaries

```sruja
// Your team's system
Shop = system "Shop"

// Another team's systems
UserPlatform = system "User Platform" {
  metadata {
    tags ["internal", "platform-team"]
    owner "Platform Team"
  }
}

AnalyticsPlatform = system "Analytics Platform" {
  metadata {
    tags ["internal", "data-team"]
    owner "Data Team"
  }
}
```

## People: Always External

People are always outside the system boundary:

```sruja
// External actors
Customer = person "Customer"
Administrator = person "Administrator"
Support = person "Customer Support"

// Your system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API Service"
}

// People interact with internal components
Customer -> Shop.WebApp "Browses"
Administrator -> Shop.WebApp "Manages"
Support -> Shop.WebApp "Monitors"
```

## Boundary Crossings

### Crossing to External Systems

```sruja
// External dependency
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Crossing the boundary
Shop.API -> PaymentGateway "Process payment"
PaymentGateway -> Shop.API "Payment result"
```

### Multiple Boundary Crossings

```sruja
Customer = person "Customer"

// Internal
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// External 1
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// External 2
EmailService = system "Email Service" {
  metadata {
    tags ["external"]
  }
}

// External 3
AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external"]
  }
}

// Multiple crossings
Customer -> Shop.WebApp "Places order"
Shop.WebApp -> Shop.API "Process order"
Shop.API -> PaymentGateway "Charge payment"
Shop.API -> EmailService "Send confirmation"
Shop.API -> AnalyticsService "Track event"
```

## Teams and Boundaries

### Single Team, One System

```sruja
// Your team owns everything
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
  Database = database "Database"
}
```

### Multiple Teams, Bounded Contexts

```sruja
// Team A: Shop team
Shop = system "Shop" {
  metadata {
    tags ["internal", "shop-team"]
    owner "Shop Team"
  }
  WebApp = container "Web App"
  API = container "API"
}

// Team B: Payment team
Payment = system "Payment Service" {
  metadata {
    tags ["internal", "payment-team"]
    owner "Payment Team"
  }
  Processor = container "Payment Processor"
}

// Team C: Notification team
Notifications = system "Notification Service" {
  metadata {
    tags ["internal", "notification-team"]
    owner "Notification Team"
  }
  Sender = container "Notification Sender"
}

// Cross-team boundaries
Shop.API -> Payment.Processor "Process payment"
Shop.API -> Notifications.Sender "Send notification"
```

## Complete Example

```sruja
import { * } from 'sruja.ai/stdlib'

// External actors (always external)
Customer = person "Customer"
Administrator = person "Administrator"

// Internal system
Shop = system "Shop" {
  metadata {
    tags ["internal"]
    owner "Shop Team"
  }

  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "PostgreSQL"
}

// External systems
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor", "pci-compliant"]
    owner "Stripe"
    sla "99.9% uptime"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "vendor"]
    owner "SendGrid"
  }
}

AnalyticsService = system "Analytics Service" {
  metadata {
    tags ["external", "vendor"]
    owner "Google"
  }
}

// Relationships
Customer -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Sends requests"
Shop.API -> Shop.Database "Persists data"

// Crossing boundaries
Shop.API -> PaymentGateway "Process payment"
PaymentGateway -> Shop.API "Payment result"
Shop.API -> EmailService "Send notifications"
Shop.API -> AnalyticsService "Track events"

view index {
  include *
}
```

## Boundary Views

### System Context View (Shows boundaries)

```sruja
view index {
  title "System Context - Shows Internal vs External"
  include *
}
```

### Internal-Only View

```sruja
view internal_view of Shop {
  title "Internal Architecture"
  include Shop.*
  exclude external
}
```

## Exercise

Mark internal and external components:

```sruja
import { * } from 'sruja.ai/stdlib'

// Add metadata to mark external systems

Patient = person "Patient"
Doctor = person "Doctor"

Hospital = system "Hospital Scheduling" {
  WebApp = container "Web App"
  API = container "API Service"
  Database = database "Database"
}

InsuranceAPI = system "Insurance API"
SMSProvider = system "SMS Provider"
```

**Add metadata to mark:**

- External systems
- External dependencies
- Owner information

## Key Takeaways

1. **Use metadata tags**: Mark external systems clearly
2. **People are external**: Users are outside the system
3. **Document ownership**: Who owns each system
4. **Show crossings**: How boundaries are crossed
5. **Team boundaries**: Internal boundaries between teams

## Next Lesson

In [Lesson 3](lesson-3.md), you'll learn how to model integrations and plan for boundary crossings.

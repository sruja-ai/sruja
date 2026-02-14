# Lesson 3: Crossing Boundaries

## Learning Goals

- Model integrations across boundaries
- Plan for failures at boundaries
- Document interface contracts
- Design fallback strategies

## Boundary Crossings

Every time a relationship crosses from internal to external, it's a boundary crossing:

```sruja
// Internal → External = Boundary crossing
Shop.API -> PaymentGateway "Process payment"

// External → Internal = Boundary crossing
PaymentGateway -> Shop.API "Payment result"
```

## Integration Patterns

### Pattern 1: Request-Response

```sruja
Shop.API -> PaymentGateway "Process payment"
PaymentGateway -> Shop.API "Payment result"

// Characteristics:
// - Synchronous
// - Real-time
// - Tight coupling
```

### Pattern 2: Event-Driven

```sruja
Shop.API -> EventQueue "Publish order event"
EventQueue -> PaymentProcessor "Consume order event"

// Characteristics:
// - Asynchronous
// - Decoupled
// - Resilient
```

### Pattern 3: Polling

```sruja
Shop.API -> ExternalAPI "Check status"
ExternalAPI -> Shop.API "Return status"

// Characteristics:
// - Periodic checks
// - No webhooks
// - Simpler but less efficient
```

## Integration Considerations

### 1. Error Handling

What happens when external service fails?

```sruja
// Document expected failure modes
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    sla "99.9% uptime"
    failure_modes ["timeout", "service unavailable", "network error"]
  }
}

// Model fallbacks
Shop.API -> PaymentGateway "Process payment" [primary]
Shop.API -> PaymentGatewayBackup "Process payment" [fallback]
```

### 2. Timeouts and Latency

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
      p95 "200ms"
      p99 "500ms"
    }
  }
}
```

### 3. Data Consistency

```sruja
// Document consistency guarantees
Shop.API -> PaymentGateway "Process payment"
// If payment succeeds but order save fails:
// - Idempotent payment calls
// - Compensating transactions
// - Eventual consistency
```

### 4. Security

```sruja
// Security at boundary
Shop.API -> PaymentGateway "Process payment" [encrypted, authenticated, tls1.3]

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "pci-compliant"]
    security ["mutual TLS", "API key authentication"]
  }
}
```

## Documenting Interface Contracts

### API Contract

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    api_endpoint "https://api.payment.com/v1"
    authentication "API Key"
    rate_limit "1000 req/min"
  }
}

Shop.API = container "API Service" {
  metadata {
    api_consumer "Payment Gateway Client"
    retry_policy "3 retries with exponential backoff"
  }
}
```

### Data Format

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    data_format "JSON"
    schema_version "v1.2"
    validation "Strict schema validation"
  }
}
```

### SLA and Reliability

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

## Fallback Strategies

### Strategy 1: Redundant Providers

```sruja
// Primary provider
PrimaryPayment = system "Primary Payment Gateway" {
  metadata {
    tags ["external", "primary"]
  }
}

// Backup provider
BackupPayment = system "Backup Payment Gateway" {
  metadata {
    tags ["external", "backup"]
  }
}

// Try primary, fall back to backup
Shop.API -> PrimaryPayment "Process payment" [primary]
Shop.API -> BackupPayment "Process payment" [fallback]
```

### Strategy 2: Circuit Breaker

```sruja
Shop.API = container "API Service" {
  metadata {
    circuit_breaker {
      enabled true
      failure_threshold "5"
      recovery_timeout "60s"
    }
  }
}
```

### Strategy 3: Degraded Mode

```sruja
// If external analytics fails, continue operation
Shop.API -> AnalyticsService "Track events" [non_critical]

// If email fails, queue for later
Shop.API -> EmailService "Send notifications" [async_queue]
```

### Strategy 4: Cache External Data

```sruja
// Cache external API responses
Shop.API -> ExternalAPI "Get exchange rates"
Shop.Cache -> Shop.API "Return cached rates"

// Fallback to cache if external fails
Shop.API -> Shop.Cache "Get cached rates" [fallback]
```

## Complete Integration Example

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

Shop = system "Shop" {
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
    tags ["external", "primary", "pci-compliant"]
    owner "Stripe Inc."
    sla "99.99% uptime"
    api_endpoint "https://api.stripe.com/v1"
    authentication "API Key"
    rate_limit "1000 req/min"
    data_format "JSON"
  }
}

// Backup payment provider
PayPal = system "PayPal" {
  metadata {
    tags ["external", "backup"]
    owner "PayPal"
    sla "99.9% uptime"
  }
}

// Email service
SendGrid = system "SendGrid" {
  metadata {
    tags ["external"]
    sla "99.9% uptime"
    timeout "10s"
  }
}

// Integrations
Customer -> Shop.WebApp "Checkout"
Shop.WebApp -> Shop.API "Process order"

// Primary payment integration (encrypted, authenticated)
Shop.API -> Stripe "Process payment" [primary, encrypted, tls1.3]
Stripe -> Shop.API "Payment result"

// Fallback to backup
Shop.API -> PayPal "Process payment" [fallback, encrypted]

// Email (non-critical, can queue)
Shop.API -> SendGrid "Send confirmation" [non_critical, async_queue]

// Cache external API calls
Shop.Cache -> Shop.API "Return cached data"

view index {
  include *
}
```

## Boundary Testing

### What to Test at Boundaries

```sruja
// Document test requirements
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
    tests [
      "Happy path integration test",
      "Timeout handling",
      "Error response handling",
      "Rate limiting",
      "Authentication failure"
    ]
  }
}
```

## Exercise

Design an integration for:

> "A weather application displays current weather and forecasts. The app fetches weather data from OpenWeatherMap API and sends location-based ads to users. The ad service is provided by an external vendor."

**Consider:**

1. Boundary crossings
2. Error handling
3. Caching strategy
4. Fallbacks (if any)

## Key Takeaways

1. **Every boundary crossing is an integration point**
2. **Document interface contracts**: API endpoints, data formats, SLAs
3. **Plan for failures**: Timeouts, errors, service outages
4. **Design fallbacks**: Redundant providers, degraded modes, caching
5. **Test boundaries**: Integration tests, error scenarios

## Module 3 Complete

You've completed Boundaries! You now understand:

- What boundaries are and why they matter
- How to mark internal vs. external components
- How to model integrations and plan for boundary crossings

**Next**: Learn about [Module 4: Flows](../module-4-flows/module-overview.md).

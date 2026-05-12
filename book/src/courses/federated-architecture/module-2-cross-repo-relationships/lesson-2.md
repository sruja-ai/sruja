---
title: "Lesson 2: External References"
weight: 2
summary: "Link to components in other repositories."
---

# Lesson 2: External References

## Importing External Bundles

Reference components from other repos:

```sruja
// In order-service/repo.sruja
import {
  user-service,
  payment-service,
  notification-service
} from "./bundles/"

OrderService = system "Order Service" {
  OrderProcessor = container "Order Processor"
}
```

## Referencing External Components

```sruja
// Create relationships to external services
OrderProcessor -> user-service::UserAPI "Validates customer"
OrderProcessor -> payment-service::PaymentGateway "Processes payment"
OrderProcessor -> notification-service::EmailService "Sends confirmation"
```

## External System Modeling

For external services you don't control:

```sruja
// Model external systems you depend on
ExternalSystems = system "External Services" {
  Stripe = system "Stripe API" {
    description "Payment processor"
  }

  SendGrid = system "SendGrid" {
    description "Email service"
  }
}

OrderProcessor -> ExternalSystems.Stripe "Payment processing"
OrderProcessor -> ExternalSystems.SendGrid "Email delivery"
```

## Validation of External References

```bash
# Validate all external refs resolve
sruja validate -r . --check-external

# Check specific bundle
sruja validate --bundle ./bundles/user-service.bundle.json
```

Validation checks:
- ✅ External bundle exists
- ✅ Referenced component exists
- ✅ Relationship is valid

## Versioning External References

```sruja
// Pin to specific version
import { user-service v1.2.0 } from "./bundles/user-service.bundle.json"

// Or use latest
import { user-service @ latest } from "./bundles/user-service.bundle.json"
```

## Broken Reference Detection

```bash
# Find broken refs
sruja validate -r . --find-broken-refs

# Output:
# ERROR: user-service::NonExistent not found in bundle
# ERROR: payment-service::LegacyAPI has been deprecated
```

## Next Steps

Lesson 3 covers ownership and contracts.
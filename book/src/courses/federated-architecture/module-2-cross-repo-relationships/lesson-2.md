---
title: "Lesson 2: External References"
weight: 2
summary: "Link to components in other repositories."
---

# Lesson 2: External References

## Importing External Bundles

Reference components from other repos:

```sruja
<!-- partial -->
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
<!-- partial -->
// Create relationships to external services
OrderProcessor -> user-service::UserAPI "Validates customer"
OrderProcessor -> payment-service::PaymentGateway "Processes payment"
OrderProcessor -> notification-service::EmailService "Sends confirmation"
```

## External System Modeling

For external services you don't control:

```sruja
<!-- partial -->
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
# Validate DSL syntax and constraints
sruja lint repo.sruja

# Validate with architecture file
sruja lint repo.sruja --format json

# Run drift check to see what changed
sruja drift -r . -a repo.sruja
```

Validation checks:
- ✅ DSL syntax is correct
- ✅ All references resolve
- ✅ Relationships are valid

## Broken Reference Detection

```bash
# Find drift and broken refs
sruja drift -r . -a repo.sruja

# Use impact analysis to understand dependencies
sruja impact OrderService -r . --depth 2
```

## Hands-On: Work with External References

1. **Create a bundle to reference:**
   ```bash
   sruja publish -r ./user-service -o ./bundles/user-service.bundle.json
   ```

2. **Set up cross-repo references in your DSL:**
   ```sruja
   import { user-service } from "./bundles/user-service.bundle.json"

   OrderService = system "Order Service" {
     OrderProcessor = container "Order Processor"
     OrderProcessor -> user-service::UserAPI "Validates customer"
   }
   ```

3. **Validate the references:**
   ```bash
   sruja lint repo.sruja
   sruja drift -r . -a repo.sruja
   ```

4. **Check impact of external dependencies:**
   ```bash
   sruja impact OrderProcessor -r . --depth 3
   ```

## Learning Outcomes

- ✅ Import and reference components from external bundles
- ✅ Model external systems you depend on but don't control
- ✅ Validate external references using `sruja lint`
- ✅ Use impact analysis to understand cross-repo dependencies

## Quiz: Test Your Understanding

### Q1: What command is used to validate Sruja DSL files?

A) `sruja validate`
B) `sruja lint`
C) `sruja check`
D) `sruja verify`

### Q2: When modeling external services you don't control, what should you use?

A) Import statements with no changes
B) A separate system definition with description
C) Direct database access
D) Git submodules

### Q3: What does `sruja drift` help detect?

A) Git conflicts
B) Changes between architecture and code
C) Network issues
D) Memory leaks

## Next Steps

Lesson 3 covers ownership and contracts.
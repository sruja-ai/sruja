---
title: "Lesson 3: Scenario Validation"
weight: 3
summary: "Test architectural behavior with scenarios and automated validation."
---

# Lesson 3: Scenario Validation

## What Are Scenarios?

**Scenarios** are executable specifications of architectural behavior. They describe sequences of actions and expected outcomes that can be automatically validated.

## Why Scenarios?

Scenarios bridge the gap between:
- **Architecture diagrams**: What we designed
- **Behavior tests**: What actually happens

With scenarios, you can verify:
- Users can actually complete checkout
- Failed payments trigger appropriate handling
- System recovers from component failures
- Compliance requirements are met

## Writing Scenarios in Sruja

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

scenario "Customer completes checkout" {
  description "Happy path: customer browses, adds to cart, pays"

  actors {
    Customer = person "Online Customer"
  }

  system EcommercePlatform

  steps {
    1. Customer visits homepage
    2. Customer searches for "wireless headphones"
    3. System displays product list
    4. Customer adds item to cart
    5. Customer proceeds to checkout
    6. Customer enters shipping address
    7. Customer selects payment method
    8. Customer confirms purchase
    9. System processes payment
    10. System sends confirmation email
    11. Customer receives order confirmation
  }

  expected_outcomes {
    - "Order created with status 'confirmed'"
    - "Payment charged to customer"
    - "Inventory updated"
    - "Confirmation email sent"
  }
}

scenario "Payment fails during checkout" {
  description "Card declined scenario"

  steps {
    1. Customer reaches payment step
    2. Customer enters card details
    3. Payment gateway declines card
    4. System displays error message
    5. Customer prompted to retry or use different method
  }

  expected_outcomes {
    - "Order status remains 'pending_payment'"
    - "No charge to customer"
    - "Error logged for audit"
  }
}
```

## Running Scenario Validation

```bash
# Validate all scenarios
sruja validate --scenarios

# Run specific scenario
sruja validate --scenario "Customer completes checkout"

# Generate test report
sruja validate --scenarios --format json > report.json
```

## Coverage Analysis

```bash
# Show scenario coverage
sruja validate --scenarios --coverage

# Shows which components have scenario coverage
# and which are untested
```

## CI/CD Integration

```yaml
# .github/workflows/scenarios.yml
- name: Run Scenario Validation
  run: |
    sruja validate --scenarios
    sruja validate --contracts
```

## Advanced: Scenario with State Machine

```sruja
<!-- partial -->
scenario "Order lifecycle end-to-end" {
  steps {
    1. submit order (from New state)
    2. payment approved (transitions to Processing)
    3. order fulfilled (transitions to Shipped)
    4. order delivered (transitions to Delivered)
  }

  validate {
    state_machine OrderLifecycle
    final_state Delivered
  }
}
```

## Module Complete!

You've completed the Behavioral Modeling module. You now understand:
- ✅ State machines and state transitions
- ✅ API contracts with request/response schemas
- ✅ Scenario writing and validation
- ✅ CI/CD integration for behavioral testing

This module completes the Advanced Architects course with skills to model both structural and behavioral aspects of complex architectures.
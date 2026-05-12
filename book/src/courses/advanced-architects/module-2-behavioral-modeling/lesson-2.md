---
title: "Lesson 2: API Contracts"
weight: 2
summary: "Define and validate service interfaces with Sruja contracts."
---

# Lesson 2: API Contracts

## What Are API Contracts?

An **API contract** defines the expected interface between services—what they exchange, in what format, and what errors can occur. Contracts enable:

- **Consumer-driven contracts**: API consumers define what they need
- **Contract testing**: Automatically verify implementations match
- **Documentation**: Self-documenting APIs
- **Versioning**: Manage API evolution safely

## Why Contracts in Architecture?

Contracts are the *boundaries* between components. Good contracts:
- Make dependencies explicit
- Enable independent evolution
- Prevent breaking changes
- Enable contract testing in CI/CD

## Defining Contracts in Sruja

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

contract "Payment Contract" {
  description "Contract for payment processing service"

  input {
    amount integer "Payment amount in cents"
    currency string "ISO 4217 currency code"
    customer_id string "Customer identifier"
  }

  output {
    transaction_id string "Unique transaction identifier"
    status string "Payment status"
  }

  error {
    "400" "Invalid request parameters"
    "402" "Payment declined"
    "500" "Internal server error"
  }

  constraint "Response time must be < 50ms"
}
```

## Contract Fields

Each field in input/output has a name, type, and description:

```sruja
input {
  field_name type "Description of the field"
}
```

Supported types: `string`, `integer`, `boolean`, `array`, `object`

## Contract Integrity Validation

```bash
# Validate all contracts
sruja validate --contracts

# Check specific contract
sruja validate payment.sruja
```

Validation checks:
- ✅ All fields have valid types
- ✅ Error codes are properly formatted
- ✅ Constraints are well-defined
- ✅ Contracts are used by existing elements

## Contracts on Containers

Apply contracts to containers to define their interfaces:

```sruja
// partial
PaymentService = container "Payment Service" {
  technology "Node.js"
  description "Handles payment processing"

  contract "Process Payment" {
    input {
      amount integer "Amount in cents"
      currency string "Currency code"
    }
    output {
      transaction_id string "Transaction ID"
      status string "Payment status"
    }
    error {
      "400" "Invalid request"
      "402" "Payment declined"
    }
  }
}

OrderService = container "Order Service" {
  technology "Go"
  description "Manages orders"
}

// OrderService uses PaymentService's contract
OrderService -> PaymentService "Processes payment via contract"
```

## Using Contracts for Testing

Generate test cases from contracts:

```bash
# Generate mock implementations
sruja contract mock "Payment Contract" --output ./tests/mocks/

# Generate test cases
sruja contract test "Payment Contract" --framework jest
```

## Module Complete!

You've completed Behavioral Modeling. You now understand:
- ✅ State machines and state transitions
- ✅ API contracts with input/output/error definitions
- ✅ Scenario writing and validation
- ✅ CI/CD integration for behavioral testing

Next steps: Module 3 covers Intent-Driven Development.
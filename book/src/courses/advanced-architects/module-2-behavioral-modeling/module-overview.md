---
title: "Module 2 - Behavioral Modeling"
weight: 2
summary: "Model state machines, API contracts, and scenarios with Sruja DSL."
---

# Module 2 - Behavioral Modeling

**State machines, API contracts, and scenario-based validation.**

## Overview

This module teaches behavioral modeling with Sruja—capturing how systems behave over time, not just their structure.

## Lessons

### Lesson 1: [State Machines](lesson-1.md)
_Modeling system states and transitions_

- What are state machines
- Defining states and transitions
- Guard conditions and actions
- State machine validation

### Lesson 2: [API Contracts](lesson-2.md)
_Defining and validating service interfaces_

- Contract syntax
- Request/response schemas
- Error handling patterns
- Contract integrity validation

### Lesson 3: [Scenario Validation](lesson-3.md)
_Testing architectural behavior with scenarios_

- Writing scenarios
- Running validation
- Coverage analysis
- Integration testing patterns

## Learning Outcomes

- ✅ Model stateful systems with state machines
- ✅ Define API contracts with schemas
- ✅ Write and validate scenarios
- ✅ Test architectural behavior in CI/CD

## Prerequisites

- Completed Advanced Architects Module 1 (Policy as Code)
- Familiarity with Sruja DSL basics
- Understanding of microservices patterns

## Estimated Time

2-3 hours

---

## Raw Thoughts from Analysis

### v0.34.0 Features to Cover:
- `state_machine` syntax in DSL
- `contract` syntax for API contracts
- `scenario` validation
- State machine integrity rules
- Contract integrity validation

### Related Concepts:
- Behavioral DSL
- State machine validation
- Contract-first design
- API versioning
- Scenario-based testing

### Key DSL Elements:

```sruja
// State Machine
Order = state_machine "Order Lifecycle" {
  initial New
  state New {
    on submit -> Pending
    on cancel -> Cancelled
  }
  state Pending {
    on accept -> Processing
    on reject -> Cancelled
  }
  // ...
}

// API Contract
PaymentContract = contract "Payment API" {
  endpoint POST /payments
  request {
    amount: integer
    currency: string
  }
  response 200 {
    transaction_id: string
  }
  response 400 {
    error: string
  }
}
```

### Lesson Ideas:

1. **State Machines**
   - Model order lifecycle, user sessions, workflow processes
   - Validate no invalid transitions
   - Show guard conditions and actions
   - Hands-on: Model checkout flow

2. **API Contracts**
   - Define service interfaces contract-first
   - Validate request/response schemas
   - Error handling and versioning
   - Integration with existing services

3. **Scenario Validation**
   - Write happy path and edge case scenarios
   - Run `sruja validate` to check scenarios
   - Coverage reporting
   - CI/CD integration for behavioral testing
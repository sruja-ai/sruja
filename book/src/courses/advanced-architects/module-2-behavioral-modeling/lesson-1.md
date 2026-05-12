---
title: "Lesson 1: State Machines"
weight: 1
summary: "Model system states and transitions with Sruja state machines."
---

# Lesson 1: State Machines

## What Are State Machines?

A **state machine** models a system's behavior as a series of discrete states and the transitions between them. State machines are perfect for:

- Order processing workflows
- User session management
- Business process orchestration
- Protocol implementations

## Why State Machines in Architecture?

Traditional architecture diagrams show *structure*—components and their relationships. State machines add *behavior*—how the system evolves over time.

## Defining State Machines in Sruja

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

OrderLifecycle = state_machine "Order Lifecycle" {
  initial New

  state New {
    description "Order created, awaiting payment"
    on submit -> Pending
    on cancel -> Cancelled
  }

  state Pending {
    description "Payment submitted, processing"
    on accept -> Processing
    on reject -> Failed
    on timeout -> Cancelled
  }

  state Processing {
    description "Order being fulfilled"
    on complete -> Shipped
    on fail -> Failed
  }

  state Shipped {
    description "Order shipped to customer"
    on deliver -> Delivered
    on return -> Returned
  }

  state Delivered {
    description "Order completed"
    final
  }

  state Cancelled {
    description "Order cancelled"
    final
  }

  state Failed {
    description "Order failed"
    on retry -> Pending
  }

  state Returned {
    description "Order returned"
    final
  }
}
```

## Guard Conditions

Guard conditions restrict when transitions can occur:

```sruja
state Pending {
  description "Payment submitted"

  on accept [amount > 0] -> Processing
  on reject [reason != null] -> Failed
}
```

## Actions

Actions execute on transitions:

```sruja
state New {
  on submit [validate()] -> Pending {
    action send_email("Payment received")
    action update_inventory()
  }
}
```

## State Machine Validation

Sruja validates state machines automatically:

```bash
sruja validate order-lifecycle.sruja
```

Validation checks:
- ✅ All states are reachable from initial
- ✅ All final states are actually terminal
- ✅ No orphan states
- ✅ Guard conditions are valid
- ✅ Actions reference existing functions

## Hands-On: Model a User Session

```sruja
UserSession = state_machine "User Session" {
  initial Unauthenticated

  state Unauthenticated {
    description "User not logged in"
    on login[credentials_valid] -> Authenticated
    on login[!credentials_valid] -> Failed
  }

  state Authenticated {
    description "User logged in"
    on logout -> Unauthenticated
    on session_expire -> Expired
  }

  state Failed {
    description "Authentication failed"
    on retry -> Unauthenticated
  }

  state Expired {
    description "Session timed out"
    on reauthenticate -> Authenticated
    on logout -> Unauthenticated
  }
}
```

## Next Steps

Lesson 2 covers API contracts—defining service interfaces with schemas and validation.
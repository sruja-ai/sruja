---
title: "Lesson 2: Managing Technical Debt"
weight: 2
summary: "Using Deprecation and ADRs to manage legacy code."
---

# Lesson 2: Managing Technical Debt

Every codebase has skeletons. The key is to label them.

## Deprecating Components

We decided to move from `Stripe` to `Adyen` for lower fees. But we can't switch overnight.

```sruja
import { * } from 'sruja.ai/stdlib'

Stripe = system "Legacy Payment Gateway" {
metadata {
  tags ["deprecated"]
}
description "Do not use for new features. Migration in progress."
}

Adyen = system "New Payment Gateway" {
metadata {
  tags ["preferred"]
}
}
```

## Governance Policies

We can enforce this with a policy!

```sruja
policy Migration "No New Stripe Integrations" {
  rule deny edge from { tag "payment-system" } to { tag "stripe" }
    message "New Stripe integrations are forbidden under the migration policy"
    suggest "Use Adyen or an internal payment gateway instead"
}
```

This prevents developers from accidentally adding dependencies to the system you are trying to kill.

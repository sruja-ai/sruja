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
system Stripe "Legacy Payment Gateway" {
    external
    tags ["deprecated"]
    description "Do not use for new features. Migration in progress."
}

system Adyen "New Payment Gateway" {
    external
    tags ["preferred"]
}
```

## Governance Policies
We can enforce this with a policy!

```sruja
policy Migration "No New Stripe Integrations" {
    rule "BanStripe" {
        // Pseudo-code: Fail if any NEW relation points to Stripe
        check "relation.to != 'Stripe'"
    }
}
```

This prevents developers from accidentally adding dependencies to the system you are trying to kill.

---
title: "Lesson 3: Governance as Code"
weight: 3
summary: "Automating architectural compliance with Policies and Rules."
---

# Lesson 3: Governance as Code

As your organization scales, manually reviewing every architectural change becomes impossible. You need automated guardrails to ensure consistency and security.

## What is Governance as Code?

Governance as Code treats architectural policies (e.g., "All databases must be encrypted", "No circular dependencies") as executable code that can be validated automatically in your CI/CD pipeline.

## The `policy` Keyword

Sruja allows you to define policies directly in your architecture.

### 1. Global Policies
Apply to the entire architecture.

```sruja
architecture "E-Commerce" {
    // Define a security policy
    policy Security "Security Standards" {
        rule Encryption "Data at Rest" {
            check "tags contains 'encrypted'"
        }
    }

    system PaymentService "Payment Service" {
        // This system must adhere to the Security policy
        // (Validation logic will check if it meets the rules)
        tags ["encrypted"] 
    }
}
```

### 2. Scoped Policies
You can also define policies within a specific system or container if they only apply locally.

```sruja
system OrderService {
    policy Consistency "Data Consistency" {
        rule ACID "Must use transactional DB"
    }
}
```

## Automated Validation

The real power comes when you run the Sruja CLI. It can check your architecture against these policies and fail the build if violations are found.

```bash
sruja validate architecture.sruja
```

This ensures that your architecture isn't just a diagram—it's a contract that is continuously verified.

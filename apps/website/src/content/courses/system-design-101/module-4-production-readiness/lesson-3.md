---
title: "Lesson 3: Governance as Code"
weight: 3
summary: "Automating architectural compliance with Policies and Rules."
---

# Lesson 3: Governance as Code

As your organization scales, manually reviewing every architectural change becomes impossible. You need automated guardrails to ensure consistency and security.

## What is Governance as Code?

Governance as Code treats architectural policies (e.g., "All databases must be encrypted", "No circular dependencies") as executable code that can be validated automatically in your CI/CD pipeline.

## Built-in Validation Rules

Sruja validates common architectural concerns automatically:

```sruja
architecture "E-Commerce" {
    system PaymentService "Payment Service" {
        container API {
            tags ["encrypted"]
        }
        datastore DB
    }

    person Auditor "Security Auditor"
    Auditor -> API "Reviews"
    API -> DB "Reads/Writes"
}
```

## Automated Validation

The real power comes when you run the Sruja CLI. It can check your architecture against these policies and fail the build if violations are found.

```bash
sruja validate architecture.sruja
```

This ensures that your architecture isn't just a diagram—it's a contract that is continuously verified.

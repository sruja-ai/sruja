---
title: "Lesson 1: Documenting Decisions (ADRs)"
weight: 1
summary: "Why document? What is an ADR?"
---

# Lesson 1: Documenting Decisions (ADRs)

## What is an ADR?

An **Architecture Decision Record (ADR)** is a document that captures an important architectural decision made along with its context and consequences.

### Why use ADRs?
*   **Context:** Explains *why* a decision was made (e.g., "Why did we choose Postgres over Mongo?").
*   **Onboarding:** Helps new team members understand the history of the system.
*   **Alignment:** Ensures everyone agrees on the path forward.

### Structure of an ADR
1.  **Title:** Short summary.
2.  **Status:** Proposed, Accepted, Deprecated.
3.  **Context:** The problem we are solving.
4.  **Decision:** What we are doing.
5.  **Consequences:** The pros and cons of this decision.

---

## 🛠️ Sruja Perspective: Native ADR Support

Sruja treats ADRs as first-class citizens. You can define them directly in your architecture file.

```sruja
architecture "Payment System" {
    // Define an ADR
    adr ADR001 "Use Stripe for Payments" {
        status "Accepted"
        context "We need a reliable payment processor that supports global currencies."
        decision "We will use Stripe as our primary payment gateway."
        consequences "Vendor lock-in, but faster time to market."
    }

    system PaymentService "Payment Service" {
        // Link the ADR to the component it affects
        adr ADR001
        description "Handles credit card processing."
    }
}
```

This ensures that your documentation lives right next to the code it describes, making it harder to ignore or lose.

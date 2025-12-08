---
title: "Lesson 1: Monolith vs. Microservices (ADRs)"
weight: 1
summary: "Using Architecture Decision Records to document critical choices."
---

# Lesson 1: Monolith vs. Microservices

This is the most debated topic in software engineering. Should we start with a monolith or microservices?

## The Trade-off
*   **Monolith**: Easier to develop, deploy, and debug. Harder to scale teams and components independently.
*   **Microservices**: Independent scaling and deployment. High complexity (network, consistency, observability).

## Our Decision: Modular Monolith
For Shopify-lite, we will start with a **Modular Monolith**. We will have clear boundaries (modules) but deploy as a single unit initially. This gives us speed now and flexibility later.

## Documenting with ADRs
We don't just make this decision; we *document* it so future engineers know why.

```sruja
architecture "Shopify-Lite" {
    
    adr ArchitectureStyle "Modular Monolith Strategy" {
        status "Accepted"
        context "We are a small team building a new product. Speed is critical."
        
        option "Microservices" {
            pros "Independent scaling"
            cons "High operational complexity"
        }
        
        option "Modular Monolith" {
            pros "Simple deployment, code sharing"
            cons "Risk of tight coupling if not disciplined"
        }

        decision "Modular Monolith"
        reason "We prioritize iteration speed. We will enforce boundaries using Sruja domains."
    }
}
```

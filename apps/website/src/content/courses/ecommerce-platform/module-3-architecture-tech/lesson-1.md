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
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
    // Requirements that drive the architecture decision
    requirement R1 functional "Must support 10,000+ stores"
    requirement R2 performance "API response < 200ms p95"
    requirement R3 scalability "Scale components independently"
    requirement R4 development "Small team, need fast iteration"
    
    // Architecture Decision Record
    adr ArchitectureStyle "Modular Monolith Strategy" {
        status "Accepted"
        context "We are a small team building a new product. Speed is critical, but we need to scale to 10k+ stores."
        
        option "Microservices" {
            pros "Independent scaling, technology diversity"
            cons "High operational complexity, network latency, data consistency challenges"
        }
        
        option "Monolith" {
            pros "Simplest deployment, no network calls"
            cons "Cannot scale components independently, single point of failure"
        }
        
        option "Modular Monolith" {
            pros "Simple deployment, code sharing, clear boundaries"
            cons "Risk of tight coupling if not disciplined, harder to scale independently later"
        }

        decision "Modular Monolith"
        reason "We prioritize iteration speed for MVP. We will enforce boundaries using Sruja domains and can split to microservices later if needed."
        consequences "Faster initial development, may need refactoring to microservices at scale"
    }
    
    // Architecture that implements the decision
    Platform = system "E-Commerce Platform" {
        description "Modular monolith - single deployment with clear module boundaries"
        adr ArchitectureStyle
        
        // Modules as containers (can be split to microservices later)
        StorefrontModule = container "Storefront Module" {
            technology "Next.js"
            description "Handles product browsing and storefronts"
        }
        
        AdminModule = container "Admin Module" {
            technology "Next.js"
            description "Merchant admin dashboard"
        }
        
        APIModule = container "API Module" {
            technology "Go"
            description "Core business logic - can scale independently"
            scale {
                min 3
                max 50
                metric "cpu > 70%"
            }
        }
        
        OrderDB = datastore "Order Database" {
            technology "PostgreSQL"
            description "Stores orders and transactions"
        }
        
        StorefrontModule -> APIModule "Fetches product data"
        AdminModule -> APIModule "Manages inventory"
        APIModule -> OrderDB "Reads/Writes"
    }
}

views {
  view index {
    title "Platform Architecture Overview"
    include *
  }
  
  // Module view: Show module boundaries
  view modules {
    title "Module View"
    include Platform.StorefrontModule Platform.AdminModule Platform.APIModule Platform.OrderDB
  }
  
  // Scalability view: Focus on scalable components
  view scalability {
    title "Scalability View"
    include Platform.APIModule Platform.OrderDB
  }
}
```

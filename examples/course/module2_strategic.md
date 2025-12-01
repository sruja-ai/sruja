# Module 2: Strategic Design

## Strategic Design

Strategic Design focuses on high-level architecture and defining the boundaries between different parts of the system. It helps in managing large-scale complexity.

## Bounded Contexts

A **Bounded Context** is a semantic boundary within which a particular domain model is defined and applicable. Inside the boundary, all terms and concepts have a specific, unambiguous meaning.

### Identifying Bounded Contexts
- Look for linguistic boundaries (e.g., "Customer" means something different in Sales vs. Support).
- Align with business capabilities.
- Often map to microservices or modules.

## Context Mapping

Context Mapping describes the relationships between Bounded Contexts.
- **Partnership**: Two teams working together.
- **Shared Kernel**: Shared model/code.
- **Customer-Supplier**: Upstream/Downstream relationship.
- **Conformist**: Downstream conforms to Upstream.
- **Anti-Corruption Layer (ACL)**: Downstream translates Upstream model.

## Sruja Syntax

In Sruja, you define a Bounded Context using the `context` keyword inside a `domain`.

```sruja
domain ECommerce "E-Commerce" {
    context Sales "Sales Context" {
        description "Handles orders and sales"
    }
    
    context Shipping "Shipping Context" {
        description "Handles delivery"
    }
}
```

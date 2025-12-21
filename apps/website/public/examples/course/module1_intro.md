# Module 1: Introduction to Domain-Driven Design (DDD)

## What is DDD?

Domain-Driven Design (DDD) is an approach to software development that centers the design on the **domain logic** and **business rules**. It aims to tackle complexity in the heart of software by connecting the implementation to an evolving model of the core business concepts.

## Why DDD?

- **Alignment with Business**: Ensures software accurately reflects business needs.
- **Ubiquitous Language**: Creates a shared language between developers and domain experts.
- **Manage Complexity**: Breaks down complex systems into manageable parts (Bounded Contexts).
- **Better Software Design**: Encourages decoupling and high cohesion.

## Core Concepts

1.  **Domain**: The sphere of knowledge and activity around which the business logic revolves.
2.  **Model**: A system of abstractions that describes selected aspects of a domain and can be used to solve problems related to that domain.
3.  **Ubiquitous Language**: A common language used by developers and domain experts to describe the domain model.

## DDD in Sruja

Sruja supports DDD by allowing you to model your domain directly in the DSL. You can define Domains, Bounded Contexts, Aggregates, Entities, and more.

### Example

```sruja
architecture "My System" {
    domain ECommerce "E-Commerce" {
        description "The core business domain"
    }
}
```

In the next module, we will explore **Strategic Design** and how to define Bounded Contexts.

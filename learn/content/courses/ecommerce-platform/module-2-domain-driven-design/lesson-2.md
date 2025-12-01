---
title: "Lesson 2: Modeling Data & Rules (Tactical Design)"
weight: 2
summary: "Defining Aggregates, Entities, and Value Objects."
---

# Lesson 2: Modeling Data & Rules

Now we zoom in. Inside our `OrderManagement` domain, what does the data look like?

## DDD Building Blocks

*   **Entity**: An object defined by its identity (e.g., an Order).
*   **Value Object**: An object defined by its attributes (e.g., an Address).
*   **Aggregate**: A cluster of objects treated as a unit (e.g., an Order + its Line Items).

## Modeling in Sruja

In Sruja, we use **Simple Syntax** to represent these advanced concepts without the jargon:

*   **`module`**: Represents the Bounded Context (a logical group).
*   **`data`**: Represents both Entities and Value Objects.
    *   If it has an `id`, it's an **Entity**.
    *   If it has no `id`, it's a **Value Object**.

Let's flesh out the `Orders` module:

```sruja
module Orders "Order Processing" {
    
    // Aggregate Root: Order
    // It has an ID, so it's an Entity.
    data Order {
        id string
        customer_id string
        items OrderLineItem[] // Relationship to other data
        shipping_address Address
    }

    // Local Entity inside the aggregate
    data OrderLineItem {
        product_id string
        quantity int
        price Money
    }

    // Value Object: No ID, just attributes
    data Address {
        street string
        city string
        zip_code string
    }

    data Money {
        amount float
        currency string
    }

    // Business Rules
    requirement "Inventory Check" {
        description "Cannot place order if inventory is insufficient."
    }
    
    policy "Max Order Value" {
        rule "Limit" {
            check "totalAmount < 10000"
        }
    }
}
```

This isn't just documentation. Sruja can use these definitions to generate database schemas or validation logic later.

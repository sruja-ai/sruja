---
title: "Lesson 1: Identifying Domains (Strategic Design)"
weight: 1
summary: "Breaking the monolith into core, supporting, and generic domains."
---

# Lesson 1: Identifying Domains

In DDD, we split the problem space into **Domains**. Not all domains are equal.

## 1. The Core Domain
This is what makes your business money. For Shopify-lite, it's **Order Management**. This is where the complexity lies (discounts, taxes, shipping rules).

## 2. Supporting Domains
These are necessary but not your main differentiator. Example: **Inventory**. You need it, but "tracking counts" isn't why people buy from you.

## 3. Generic Domains
These are solved problems. Example: **Authentication** or **Payments**. You shouldn't build your own; you should buy or use a library.

## Modeling in Sruja

Let's update our `architecture/main.sruja` to reflect this.

```sruja
architecture "Shopify-Lite" {

    domain OrderManagement "Core Domain" {
        description "Handles the lifecycle of customer orders."
        
        context Orders "Order Processing" {
            // We will fill this in later
        }
    }

    domain Inventory "Supporting Domain" {
        description "Tracks stock levels across warehouses."
        
        context Stock "Stock Tracking"
    }

    domain Identity "Generic Domain" {
        description "Authentication and Authorization."
        
        context Auth "User Management"
    }
}
```

By grouping our systems into domains, we create clear boundaries that will help us decide team structures and microservice splits later.

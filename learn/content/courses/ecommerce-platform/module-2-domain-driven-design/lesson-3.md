---
title: "Lesson 3: User Journeys (Stories)"
weight: 3
summary: "Mapping the Happy Path using Stories."
---

# Lesson 3: User Journeys

We have our domains and data. Now, how do users interact with them?

We use **Stories** to map the "Happy Path".

## The "Buy T-Shirt" Story

Let's model the flow of a customer buying a product.

```sruja
story Purchase "Customer Purchase Journey" {
    description "End-to-end flow from landing to payment"
    
    // 1. Discovery
    Shopper -> Storefront "Browses Products"
    Storefront -> Inventory "Check Stock"
    
    // 2. Checkout
    Shopper -> Checkout "Adds to Cart"
    Checkout -> Identity "Login / Guest"
    
    // 3. Payment
    Checkout -> PaymentGateway "Process Payment" {
        latency "2s" // Payments are slow!
    }
    
    // 4. Fulfillment
    PaymentGateway -> Orders "Payment Success"
    Orders -> Inventory "Reserve Stock"
}
```

## Why model this?
1.  **Validation**: Does our architecture support this flow? (Do we have the right connections?)
2.  **Performance**: We can see that the Payment step is slow (2s). This suggests we might need async processing here later.

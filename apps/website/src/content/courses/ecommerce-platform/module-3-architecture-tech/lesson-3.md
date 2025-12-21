---
title: "Lesson 3: Defining Interfaces (Contracts)"
weight: 3
summary: "Design-first API development using Contracts."
---

# Lesson 3: Defining Interfaces

Before frontend and backend teams start working, they need to agree on the API. This is where **Contracts** come in.

## API-First Design
Instead of writing code and then documenting it, we define the contract first. This allows frontend devs to mock the API while backend devs build it.

## Modeling in Sruja
We can define contracts directly inside our containers.

```sruja
specification {
  element system
  element container
  element datastore
}

model {
  ECommerce = system "E-Commerce Platform" {
    API = container "Core API" {
      technology "Go, Gin"
      
      contracts {
        api CreateOrder {
          endpoint "/orders"
          method "POST"
          description "Creates a new customer order."
          
          request {
            items List<OrderItem>
            paymentMethod string
          }
          
          response {
            orderId string
            status string
          }
        }
        
        api GetOrder {
          endpoint "/orders/{orderId}"
          method "GET"
          description "Retrieves order details by ID"
        }
      }
    }
    
    OrderDB = datastore "Order Database" {
      technology "PostgreSQL"
    }
    
    API -> OrderDB "Reads/Writes"
  }
}

views {
  view index {
    title "API Contracts View"
    include *
  }
  
  // API-focused view
  view api {
    title "API Contracts"
    include ECommerce.API
  }
}
```

## Why this matters
1.  **Single Source of Truth**: The architecture file *is* the API documentation.
2.  **Code Generation**: We can potentially generate Go structs or TypeScript interfaces from this definition.

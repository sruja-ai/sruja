---
title: "Lesson 2: Design a Rate Limiter"
weight: 2
summary: "Token Bucket, Distributed Caching, Middleware."
---

# Lesson 2: Design a Rate Limiter

**Goal:** Design a system to limit the number of requests a client can send to an API within a time window (e.g., 10 requests per second).

## Why Rate Limit?
*   **Prevent Abuse:** Stop DDoS attacks or malicious bots.
*   **Fairness:** Ensure one user doesn't hog all resources.
*   **Cost Control:** Prevent auto-scaling bills from exploding.

## Algorithms

### Token Bucket
*   A "bucket" holds tokens.
*   Tokens are added at a fixed rate (e.g., 10 tokens/sec).
*   Each request consumes a token.
*   If the bucket is empty, the request is dropped (429 Too Many Requests).

### Leaky Bucket
*   Requests enter a queue (bucket) and are processed at a constant rate.
*   If the queue is full, new requests are dropped.

## Architecture Location
Where does the rate limiter live?
1.  **Client-side:** Unreliable (can be forged).
2.  **Server-side:** Inside the application code.
3.  **Middleware:** In a centralized API Gateway (Best practice).

---

## 🛠️ Sruja Perspective: Middleware Modeling

In Sruja, we can model the Rate Limiter as a component within the API Gateway, backed by a fast datastore like Redis.

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
    APIGateway = system "API Gateway" {
        GatewayService = container "Gateway" {
            technology "Nginx / Kong"
            
            RateLimiter = component "Rate Limiter Middleware" {
                description "Implements Token Bucket algorithm"
            }
        }
        
        Redis = datastore "Rate Limit Store" {
            technology "Redis"
            description "Stores token counts per user/IP"
        }

        APIGateway.GatewayService -> APIGateway.Redis "Stores tokens"
    }
    
    Backend = system "Backend Service"
    
    APIGateway.GatewayService -> Backend "Forward Requests"
    Client = person "Client"

    // Scenario: Request allowed (has tokens)
    scenario RateLimitAllowed "Rate Limit Check - Allowed" {
        Client -> APIGateway.GatewayService "API Request"
        APIGateway.GatewayService -> APIGateway.Redis "DECR user_123_tokens"
        APIGateway.Redis -> APIGateway.GatewayService "Result: 5 (tokens remaining)"
        APIGateway.GatewayService -> Backend "Forward request"
        Backend -> APIGateway.GatewayService "Response"
        APIGateway.GatewayService -> Client "200 OK"
    }
    
    // Scenario: Request rate limited (no tokens)
    scenario RateLimitBlocked "Rate Limit Check - Blocked" {
        Client -> APIGateway.GatewayService "API Request"
        APIGateway.GatewayService -> APIGateway.Redis "DECR user_123_tokens"
        APIGateway.Redis -> APIGateway.GatewayService "Result: -1 (Empty bucket)"
        APIGateway.GatewayService -> Client "429 Too Many Requests"
    }
    
    // Scenario: Token refill (background process)
    scenario TokenRefill "Token Bucket Refill" {
        APIGateway.Redis -> APIGateway.Redis "Add 10 tokens/sec (background)"
        APIGateway.Redis -> APIGateway.Redis "Cap at max bucket size"
    }
}

views {
  view index {
    include *
  }
}
```

---
title: "Lesson 2: Scalability & Performance"
weight: 2
summary: "Vertical vs. Horizontal Scaling, Latency vs. Throughput."
---

# Lesson 2: Scalability & Performance

## What is Scalability?

Scalability is the ability of a system to handle increased load without performance degradation. It's not just about "handling more users"; it's about doing so cost-effectively and reliably.

### Vertical Scaling (Scaling Up)
Adding more power (CPU, RAM) to an existing server.
*   **Pros:** Simple to implement.
*   **Cons:** Hardware limits, single point of failure, expensive at the high end.

### Horizontal Scaling (Scaling Out)
Adding more servers to the pool of resources.
*   **Pros:** Theoretically infinite scale, better fault tolerance.
*   **Cons:** Increased complexity (load balancing, data consistency).

## Performance Metrics

### Latency
The time it takes for a system to process a request.
*   *Goal:* Minimize latency (e.g., "API response < 100ms").

### Throughput
The number of requests a system can handle per unit of time.
*   *Goal:* Maximize throughput (e.g., "Handle 10,000 requests per second").

---

## 🛠️ Sruja Perspective: Modeling Scalability

In Sruja, you can represent horizontal scaling using the native `scale` block. This allows you to define minimum and maximum replicas and the scaling metric.

```sruja
system ECommerce "E-Commerce System" {
    container WebServer "Web App" {
        technology "Go, Gin"
        
        // Define horizontal scaling properties
        scale {
            min 3
            max 10
            metric "cpu > 80%"
        }
    }
    
    container Database "Primary DB" {
        technology "PostgreSQL"
        // Vertical scaling example
        description "Running on a high-memory instance (AWS r5.2xlarge)."
    }

    WebServer -> Database "Reads/Writes"
}
```

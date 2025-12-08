---
title: "Lesson 3: Observability"
weight: 3
summary: "Mapping metrics and logs to your architecture."
---

# Lesson 3: Observability

You can't fix what you can't see. Observability is about understanding the internal state of your system from the outside.

## The Three Pillars
1.  **Logs**: "What happened?" (Error: Payment Failed)
2.  **Metrics**: "How often?" (Error Rate: 5%)
3.  **Traces**: "Where?" (Checkout -> API -> DB)

## Mapping to Sruja
Your Sruja components should map 1:1 to your observability dashboards.

*   **System `OrderService`** -> Dashboard `Order Service Overview`
*   **Container `Database`** -> Metric `postgres_cpu_usage`

## Standardizing with Policies
You can enforce observability standards using Sruja Policies.

```sruja
policy Observability "Must have metrics" {
    rule "HealthCheck" {
        check "contracts contains 'HealthCheck'"
    }
}
```

This ensures every new service you build comes with the necessary hooks for monitoring.

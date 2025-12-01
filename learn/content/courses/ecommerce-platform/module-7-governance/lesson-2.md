---
title: "Lesson 2: Cost Optimization"
weight: 2
summary: "Tracking and controlling infrastructure costs."
---

# Lesson 2: Cost Optimization

Cloud bills kill startups. Sruja helps you visualize where the money is going.

## Modeling Cost
We can add cost metadata to our deployment nodes.

```sruja
deployment Production {
    node DB "RDS Large" {
        metadata {
            cost "$500/month"
            type "db.r5.large"
        }
    }
}
```

## Cost Policies
We can prevent expensive mistakes in non-production environments.

```sruja
policy CostControl "Dev Environment Limits" {
    rule "NoLargeInstances" {
        // Pseudo-code: Dev nodes cannot cost more than $50/month
        check "deployment.name == 'Dev' implies cost < 50"
    }
}
```

This prevents someone from accidentally spinning up a massive cluster for a weekend hackathon.

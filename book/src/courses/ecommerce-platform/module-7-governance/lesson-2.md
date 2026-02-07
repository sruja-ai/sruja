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
Use metadata and CI checks to prevent expensive mistakes in non‑production environments.

```sruja
deployment Dev {
    node App "Small Instance" {
        metadata {
            cost "$20/month"
            type "t3.small"
        }
    }
}
```

Add a CI rule to flag dev nodes exceeding budget thresholds.

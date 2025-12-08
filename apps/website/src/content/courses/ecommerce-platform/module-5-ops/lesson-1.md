---
title: "Lesson 1: Deployment Strategies"
weight: 1
summary: "Modeling Blue/Green and Canary deployments."
---

# Lesson 1: Deployment Strategies

When you deploy to production, you don't just "copy files". You need a strategy to minimize downtime and risk.

## Blue/Green Deployment
You have two identical environments (Blue and Green). One is live, the other is idle. You deploy to the idle one, test it, and then switch traffic.

## Modeling in Sruja
We can use **Deployment Nodes** to represent these environments.

```sruja
deployment Production {
    node Blue "Active Cluster" {
        // ...
    }
    
    node Green "Idle Cluster" {
        // ...
    }
}
```

## Canary Deployment
You roll out the new version to a small percentage of users (e.g., 5%) and monitor for errors.

We can document this strategy in our ADRs or Metadata.

```sruja
system Platform {
    metadata {
        deploymentStrategy "Canary"
        rolloutSteps "5%, 25%, 50%, 100%"
    }
}
```

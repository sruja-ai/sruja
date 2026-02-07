---
title: "Lesson 2: Environments (Deployment Nodes)"
weight: 2
summary: "Modeling Dev, Staging, and Production environments."
---

# Lesson 2: Environments

Your software runs differently in Production than it does on your laptop. Sruja models this using **Deployment Nodes**.

## Modeling Production

```sruja
deployment Production "AWS Production" {
    node Region "US-East-1" {
        node K8s "EKS Cluster" {
            containerInstance WebApp
            containerInstance API
        }
        
        node DB "RDS Postgres" {
            containerInstance Database
        }
    }
}
```

## Modeling Local Dev

```sruja
deployment Local "Docker Compose" {
    node Laptop "My MacBook" {
        containerInstance WebApp
        containerInstance API
        containerInstance Database
    }
}
```

## Why model this?
It helps you visualize the *physical* differences. Maybe in Prod you have a Load Balancer that doesn't exist locally. Sruja makes these differences explicit.

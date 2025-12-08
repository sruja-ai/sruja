---
title: "Lesson 2: Deployment Architecture"
weight: 2
summary: "Cloud, On-prem, Containers."
---

# Lesson 2: Deployment Architecture

## Logical vs. Physical Architecture

*   **Logical Architecture:** The software components and how they interact (Containers, Components).
*   **Physical Architecture:** Where those components actually run (Servers, VMs, Kubernetes Pods).

## Deployment Strategies

### On-Premises
Running on your own hardware in a data center.
*   **Pros:** Total control, security.
*   **Cons:** High maintenance, capital expense.

### Cloud (AWS, GCP, Azure)
Renting infrastructure from a provider.
*   **Pros:** Pay-as-you-go, infinite scale.
*   **Cons:** Vendor lock-in, variable costs.

### Containers & Orchestration
Packaging code with dependencies (Docker) and managing them at scale (Kubernetes).

---

## 🛠️ Sruja Perspective: Deployment Nodes

Sruja allows you to map your logical containers to physical deployment nodes.

```sruja
architecture "Web App" {
    system WebApp "Web Application" {
        container WebServer "Nginx"
        container AppServer "Python App"
        container Database "Postgres"

        WebServer -> AppServer "Reverse Proxy"
        AppServer -> Database "SQL"
    }

    // Define the deployment environment
    // Define the deployment environment
    deployment Production "Production" {
        node AWS "AWS" {
            node USEast1 "US-East-1" {
                node EC2 "EC2 Instance" {
                    // Map the logical container to this node
                    containerInstance WebServer
                    containerInstance AppServer
                }
                
                node RDS "RDS" {
                    containerInstance Database
                }
            }
        }
    }
}
```

This mapping allows you to visualize exactly where your code is running in production.

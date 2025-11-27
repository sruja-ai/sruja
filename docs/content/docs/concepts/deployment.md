---
title: Deployment
weight: 9
---

# Deployment

The **Deployment** view allows you to map your software containers to infrastructure. This corresponds to the C4 Deployment Diagram.

## Deployment Node

A **Deployment Node** is something like physical hardware, a virtual machine, a Docker container, a Kubernetes pod, etc. Nodes can be nested.

### Syntax

```sruja
deployment "Environment" {
    node "Region" {
        node "Server" {
            // ...
        }
    }
}
```

## Infrastructure Node

An **Infrastructure Node** represents infrastructure software that isn't one of your containers (e.g., DNS, Load Balancer, External Database Service).

### Syntax

```sruja
infrastructure ID "Label" {
    description "Description"
}
```

## Container Instance

A **Container Instance** represents a runtime instance of one of your defined Containers running on a Deployment Node.

### Syntax

```sruja
containerInstance ContainerID {
    instanceId 1 // Optional
}
```

## Example

```sruja
deployment "Production" {
    node "AWS" {
        node "US-East-1" {
            infrastructure LB "Load Balancer"
            
            node "EC2 Instance" {
                containerInstance WebApp
            }
            
            node "RDS" {
                containerInstance DB
            }
        }
    }
}
```

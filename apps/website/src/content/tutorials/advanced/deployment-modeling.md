---
title: "Deployment Modeling"
weight: 70
summary: "Map logical elements to deployment nodes for environment diagrams."
tags: ["deployment", "infrastructure"]
---

# Deployment Modeling

Model production environments and map containers onto infrastructure nodes.

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
  system WebApp {
    WebServer = container "Nginx"
    AppServer = container "Python App"
    Database = container "Postgres"
  }

  deployment Production "Production" {
    node AWS "AWS" {
      node USEast1 "US-East-1" {
        node EC2 "EC2 Instance" {
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

views {
  view index {
    include *
  }
}
```

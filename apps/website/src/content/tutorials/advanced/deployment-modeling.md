---
title: "Deployment Modeling"
weight: 70
summary: "Map logical elements to deployment nodes for environment diagrams."
tags: ["deployment", "infrastructure"]
---

# Deployment Modeling

Model production environments and map containers onto infrastructure nodes.

```sruja
import { * } from 'sruja.ai/stdlib'


WebServer = container "Nginx"
AppServer = container "Python App"
Database = database "Postgres"


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

view index {
include *
}
```

---
title: "Deployment Modeling"
weight: 70
summary: "Map logical elements to deployment nodes for environment diagrams."
tags: [deployment]
aliases: ["/tutorials/deployment-modeling/"]
---

# Deployment Modeling

Model production environments and map containers onto infrastructure nodes.

```sruja
architecture "Web App" {
  system WebApp {
    container WebServer "Nginx"
    container AppServer "Python App"
    container Database "Postgres"
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
```


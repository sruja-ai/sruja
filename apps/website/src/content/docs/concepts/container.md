---
title: "Container"
weight: 14
summary: "A Container represents an application or a data store."
---

# Container

A **Container** represents an application or a data store. It is something that needs to be running in order for the overall software system to work.

**Note**: In C4, "Container" does _not_ mean a Docker container. It means a deployable unit like:

- Server-side web application (e.g., Java Spring, ASP.NET Core)
- Client-side web application (e.g., React, Angular)
- Mobile app
- Database schema
- File system

## Syntax

```sruja
container = kind "Container"

ID = container "Label/Name" {
technology "Technology Stack"
tags ["tag1", "tag2"]
// ... contains components
}
```

## Example

```sruja
system = kind "System"
container = kind "Container"

BankingSystem = system "Internet Banking System" {
WebApp = container "Web Application" {
  technology "Java and Spring MVC"
  tags ["web", "frontend"]
}
}
```

## Scaling Configuration

Containers can define horizontal scaling properties using the `scale` block:

```sruja
container = kind "Container"

API = container "API Service" {
technology "Go, Gin"
scale {
  min 3
  max 10
  metric "cpu > 80%"
}
}
```

### Scale Block Fields

- **`min`** (optional): Minimum number of replicas
- **`max`** (optional): Maximum number of replicas
- **`metric`** (optional): Scaling metric trigger (e.g., "cpu > 80%", "memory > 90%")

This helps document your auto-scaling strategy and can be used by deployment tools.

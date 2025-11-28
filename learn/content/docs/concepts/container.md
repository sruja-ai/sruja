---
title: Container
weight: 3
summary: "A Container represents an application or a data store."
---

# Container

A **Container** represents an application or a data store. It is something that needs to be running in order for the overall software system to work.

**Note**: In C4, "Container" does *not* mean a Docker container. It means a deployable unit like:
-   Server-side web application (e.g., Java Spring, ASP.NET Core)
-   Client-side web application (e.g., React, Angular)
-   Mobile app
-   Database schema
-   File system

## Syntax

```sruja
container ID "Label/Name" {
    technology "Technology Stack"
    tags ["tag1", "tag2"]
    // ... contains components
}
```

## Example

```sruja
container WebApp "Web Application" {
    technology "Java and Spring MVC"
    tags ["web", "frontend"]
}
```

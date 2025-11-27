---
title: Dynamic Views
weight: 10
---

# Dynamic Views

**Dynamic Views** describe how elements (Containers, Components) interact at runtime to implement a specific user story or feature. This corresponds to the C4 Dynamic Diagram.

## Syntax

```sruja
dynamic "View Title" {
    description "Optional description"
    
    Source -> Destination "Description of interaction"
    // or
    Source -> Destination "Description" {
        order "1" // Explicit ordering
    }
}
```

## Example

```sruja
dynamic "Login Flow" {
    User -> WebApp "Submits credentials"
    WebApp -> AuthComponent "Validates credentials"
    AuthComponent -> DB "Checks user record"
    DB -> AuthComponent "Returns user data"
    AuthComponent -> WebApp "Returns token"
    WebApp -> User "Redirects to dashboard"
}
```

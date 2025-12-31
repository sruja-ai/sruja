---
title: "Syntax Reference"
weight: 51
summary: "Core constructs and fields for Sruja DSL."
---

# Syntax Reference

## Elements

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"
database = kind "Database"
queue = kind "Queue"
component = kind "Component"

ID = person "Label"
ID = system "Label" { ... }
ID = container "Label" { ... }
ID = database "Label" { ... }
ID = queue "Label" { ... }
ID = component "Label" { ... }
```

## Relations

```sruja
Source -> Target "Label"
// Use fully qualified names when referring to nested elements:
System.Container -> System.API "Label"
System.Container.Component -> System.API.Component "Label"
```

## Metadata

```sruja
metadata {
  team "Platform"
  tier "critical"
}
```

## Deployment

```sruja
deployment Prod {
  node Cloud {
    node Region {
      node Service {
        containerInstance Web
      }
    }
  }
}
```

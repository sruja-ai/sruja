---
title: "Syntax Reference"
weight: 51
summary: "Core constructs and fields for Sruja DSL."
---

# Syntax Reference

## Elements

```sruja
person ID "Label"
system ID "Label" { ... }
container ID "Label" { ... }
datastore ID "Label" { ... }
queue ID "Label" { ... }
component ID "Label" { ... }
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

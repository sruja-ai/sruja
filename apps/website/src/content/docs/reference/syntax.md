---
title: "Syntax Reference"
weight: 51
summary: "Core constructs and fields for Sruja DSL."
---

# Syntax Reference

## Elements

```sruja
import { * } from 'sruja.ai/stdlib'


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
overview {
  summary "Syntax Reference Overview"
}

MySystem = system "MySystem" {
  metadata {
    team "Platform"
    tier "critical"
  }
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

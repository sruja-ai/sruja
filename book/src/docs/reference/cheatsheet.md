---
title: "Sruja Cheatsheet"
summary: "Quick syntax and common patterns for fast modeling."
---

# Sruja Cheatsheet

## Elements

```sruja
import { * } from 'sruja.ai/stdlib'

User = person "User"
App = system "App" {
  Web = container "Web"
  API = container "API"
  DB = database "DB"
}
User -> App.Web "Uses"
App.Web -> App.API "Calls"
App.API -> App.DB "Reads/Writes"

view index {
  include *
}
```

> [!TIP]
> The `import { * } from 'sruja.ai/stdlib'` line provides all standard kinds. You can also declare kinds manually if needed: `person = kind "Person"`, `system = kind "System"`, etc.

## Component

```sruja
import { * } from 'sruja.ai/stdlib'

App = system "App" {
  Web = container "Web" {
    Cart = component "Cart"
  }
}
```

## Scenario

```sruja
import { * } from 'sruja.ai/stdlib'

User = person "User"

App = system "App" {
  Web = container "Web"
  API = container "API"
  DB = database "Database"
}

scenario Checkout "Checkout Flow" {
  User -> App.Web "adds items"
  App.Web -> App.API "validates"
  App.API -> App.DB "stores order"
}
```

## Deployment

```sruja
<!-- partial -->
deployment Prod {
  node Cloud {
    node Region {
      node Service {
        containerInstance App.Web
      }
    }
  }
}
```

## Try it

Use the [VS Code extension](../../vscode.md) to paste these snippets into a `.sruja` file and see the diagram preview.

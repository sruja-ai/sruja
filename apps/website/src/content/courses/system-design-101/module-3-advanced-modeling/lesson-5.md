---
title: "Lesson 5: Views & Styling"
weight: 5
summary: "Focus diagrams with views; improve legibility with styling."
---

# Lesson 5: Views & Styling

## Why Views?
Views let you spotlight specific paths (API, data, auth) without redrawing the whole system.

## Sruja: Views and Styles

```sruja
architecture "E-Commerce Platform" {
  person User
  system Shop {
    container WebApp
    container API
    datastore DB
  }

  User -> Shop.WebApp "Uses"
  Shop.WebApp -> Shop.API "Calls"
  Shop.API -> Shop.DB "Reads/Writes"

  style {
    element "Datastore" { shape cylinder color "#22c55e" }
    relation "Calls" { color "#ef4444" }
  }

  views {
    container Shop "API Focus" {
      include Shop.API Shop.DB
      exclude Shop.WebApp
    }
    styles {
      element "API" { color "#0ea5e9" }
    }
  }
}
```

## Practice
- Create an "Data Flow" view focusing on `DB` reads/writes.
- Use view `styles` to highlight critical edges.


---
title: "Lesson 5: Views & Styling"
weight: 5
summary: "Focus diagrams with views; improve legibility with styling."
---

# Lesson 5: Views & Styling

## Why Views?

Views let you spotlight specific paths (API, data, auth) without redrawing the whole system.

## Sruja: Views and Styles

Views let you create focused diagrams from a single model. Styles make them visually clear.

### Basic Views Example

```sruja
element person
element system
element container
element component
element datastore
element queue

User = person "User"

Shop = system "E-Commerce Shop" {
WebApp = container "Web Application"
API = container "API Service"
DB = datastore "Database"
}

User -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> Shop.DB "Reads/Writes"

// Default view: Everything
view index {
title "Complete System View"
include *
}

// API-focused view
view api {
title "API Focus View"
include Shop.API Shop.DB
exclude Shop.WebApp User
}

// User experience view
view user {
title "User Experience View"
include User Shop.WebApp Shop.API
exclude Shop.DB
}
```

### Views with Custom Styling

```sruja
element person
element system
element container
element datastore

Customer = person "Customer"

ECommerce = system "E-Commerce System" {
WebApp = container "Web Application"
API = container "API Service"
OrderDB = datastore "Order Database"
ProductDB = datastore "Product Database"
}

Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.OrderDB "Stores orders"
ECommerce.API -> ECommerce.ProductDB "Queries products"

// Global styles
style {
element "Datastore" {
  shape cylinder
  color "#22c55e"
}
relation "Fetches data" {
  color "#3b82f6"
}
relation "Stores" {
  color "#ef4444"
}
}

view index {
title "Complete View"
include *
}

// Data flow view with custom styling
view dataflow {
title "Data Flow View"
include ECommerce.API ECommerce.OrderDB ECommerce.ProductDB
exclude Customer ECommerce.WebApp

// View-specific styles override global styles
styles {
  element "API" { color "#0ea5e9" }
  relation "Stores" { color "#10b981" }
}
}
```

## Practice

- Create an "Data Flow" view focusing on `DB` reads/writes.
- Use view `styles` to highlight critical edges.

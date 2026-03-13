---
title: "Generated View Output"
weight: 46
summary: "See actual rendered output from custom view exports."
---

# Generated View Output

This page shows the **actual rendered output** from running Sruja view export commands.

## Example 1: Single View (Mermaid)

Running:

```bash
sruja export mermaid book/valid-examples/advanced-views.sruja --view api_focus
```

**Output:**

```mermaid
graph LR

classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000
classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef containerStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef databaseStyle fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000

Customer["Customer<br>Customer"]
class Customer personStyle
subgraph Shop["E-Commerce Shop"]
    direction TB
    Shop_API["REST API<br>API"]
    class Shop_API containerStyle
    Shop_DB["PostgreSQL<br>DB"]
    class Shop_DB databaseStyle
end
```

---

## Example 2: All Views (Markdown)

Running:

```bash
sruja export markdown book/valid-examples/advanced-views.sruja --all-views
```

**Output:**

````markdown
## Custom Views

### API Architecture

Shows the API layer and its dependencies

```mermaid
graph LR

classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000
classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef containerStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef databaseStyle fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000

Customer["Customer<br>Customer"]
class Customer personStyle
subgraph Shop["E-Commerce Shop"]
    direction TB
    Shop_API["REST API<br>API"]
    class Shop_API containerStyle
    Shop_DB["PostgreSQL<br>DB"]
    class Shop_DB databaseStyle
end
```
````

### Customer Experience

Components directly visible to customers

```mermaid
graph LR

classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000
classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef containerStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000

Customer["Customer<br>Customer"]
class Customer personStyle
subgraph Shop["E-Commerce Shop"]
    direction TB
    Shop_WebApp["Web Application<br>WebApp"]
    class Shop_WebApp containerStyle
    Shop_WebApp_Cart["Shopping Cart<br>Cart"]
    class Shop_WebApp_Cart componentStyle
    Shop_WebApp_Checkout["Checkout Service<br>Checkout"]
    class Shop_WebApp_Checkout componentStyle
    Shop_WebApp_Catalog["Product Catalog<br>Catalog"]
    class Shop_WebApp_Catalog componentStyle
end
```

### System Context

External actors and system overview

```mermaid
graph LR

classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000
classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000

Customer["Customer<br>Customer"]
class Customer personStyle
Admin["Administrator<br>Admin"]
class Admin personStyle
Shop["E-Commerce Shop<br>Shop"]
class Shop systemStyle
```

### Data Layer

Shows databases and their consumers

```mermaid
graph LR

classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef containerStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef databaseStyle fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000

Shop["E-Commerce Shop<br>Shop"]
class Shop systemStyle
subgraph Shop["Shop"]
    direction TB
    Shop_API["REST API<br>API"]
    class Shop_API containerStyle
    Shop_DB["PostgreSQL<br>DB"]
    class Shop_DB databaseStyle
    Shop_Cache["Redis Cache<br>Cache"]
    class Shop_Cache databaseStyle
end
```

````

---

## Example 3: Wildcard Pattern

With this view definition:
```sruja
view all_containers {
    title "All Containers"
    include Shop.*
}
````

Running:

```bash
sruja export mermaid book/valid-examples/advanced-views.sruja --view all_containers
```

**Output:**

```mermaid
graph LR

classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000
classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef containerStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000
classDef componentStyle fill:#e6f7ff,stroke:#333,stroke-width:2px,color:#000
classDef databaseStyle fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000

Customer["Customer<br>Customer"]
class Customer personStyle
Admin["Administrator<br>Admin"]
class Admin personStyle
subgraph Shop["E-Commerce Shop"]
    direction TB
    Shop_WebApp["Web Application<br>WebApp"]
    class Shop_WebApp containerStyle
    Shop_WebApp_Cart["Shopping Cart<br>Cart"]
    class Shop_WebApp_Cart componentStyle
    Shop_WebApp_Checkout["Checkout Service<br>Checkout"]
    class Shop_WebApp_Checkout componentStyle
    Shop_WebApp_Catalog["Product Catalog<br>Catalog"]
    class Shop_WebApp_Catalog componentStyle
    Shop_API["REST API<br>API"]
    class Shop_API containerStyle
    Shop_API_Auth["Authentication<br>Auth"]
    class Shop_API_Auth componentStyle
    Shop_API_Orders["Order Handler<br>Orders"]
    class Shop_API_Orders componentStyle
    Shop_DB["PostgreSQL<br>DB"]
    class Shop_DB databaseStyle
    Shop_Cache["Redis Cache<br>Cache"]
    class Shop_Cache databaseStyle
end
```

---

## How to Generate These Outputs

### From CLI

```bash
# Single view to mermaid
sruja export mermaid book/valid-examples/advanced-views.sruja --view api_focus

# All views to markdown
sruja export markdown book/valid-examples/advanced-views.sruja --all-views > output.md
```

### From mdBook (Auto-generation)

Add a build step to your `book.toml` preprocessor or Makefile:

```bash
# Generate views as part of build
sruja export markdown book/valid-examples/advanced-views.sruja --all-views > book/src/generated/views.md
```

Then include in your markdown:

```markdown
{{#include ./generated/views.md}}
```

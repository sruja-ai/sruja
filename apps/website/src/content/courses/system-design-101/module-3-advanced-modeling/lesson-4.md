---
title: "Lesson 4: Architectural Perspectives"
weight: 4
summary: "Understanding context, containers, and components without special DSL keywords."
---

# Lesson 4: Architectural Perspectives

As your system grows, a single diagram becomes too cluttered. You need different "maps" for different audiences:

- **Executives:** Need a high-level overview (Context).
- **Architects:** Need to see service boundaries (Containers).
- **Developers:** Need to see internal details (Components).

Sruja models naturally support multiple perspectives without special keywords. Use the built‑in elements, and tooling presents the right level of detail.

## One Model, Multiple Perspectives

Sruja's `views` block lets you create custom perspectives from a single model. This is powerful for communicating with different audiences.

```sruja
import { * } from 'sruja.ai/stdlib'


Customer = person "Customer"
Admin = person "Administrator"

Shop = system "E-Commerce Shop" {
WebApp = container "Web Application" {
  technology "React"
  CartComponent = component "Shopping Cart"
  ProductComponent = component "Product Catalog"
}
API = container "API Service" {
  technology "Go"
  OrderController = component "Order Controller"
  PaymentController = component "Payment Controller"
}
DB = database "Database" {
  technology "PostgreSQL"
}
Cache = database "Cache" {
  technology "Redis"
}
}

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

// Relations
Customer -> Shop.WebApp "Browses products"
Admin -> Shop.WebApp "Manages inventory"
Shop.WebApp -> Shop.API "Fetches data"
Shop.API -> Shop.DB "Reads/Writes"
Shop.API -> Shop.Cache "Caches queries"
Shop.API -> PaymentGateway "Processes payments"

// Executive view: High-level context
view executive {
title "Executive Overview"
include Customer
include Admin
include Shop
include PaymentGateway
exclude Shop.WebApp
exclude Shop.API
exclude Shop.DB
exclude Shop.Cache
}

// Architect view: Container-level architecture
view architect {
title "Architectural View"
include Shop Shop.WebApp Shop.API Shop.DB Shop.Cache
include PaymentGateway
exclude Customer Admin
exclude Shop.CartComponent Shop.ProductComponent Shop.OrderController Shop.PaymentController
}

// Developer view: Component-level details
view developer {
title "Developer View"
include Shop.WebApp Shop.WebApp.CartComponent Shop.WebApp.ProductComponent
include Shop.API Shop.API.OrderController Shop.API.PaymentController
include Shop.DB Shop.Cache
exclude Customer Admin PaymentGateway
}

// Data flow view: Focus on data dependencies
view dataflow {
title "Data Flow View"
include Shop.API Shop.DB Shop.Cache
exclude Customer Admin Shop.WebApp PaymentGateway
}

// Default view: Everything
view index {
title "Complete System View"
include *
}
```

### Key Benefits

1. **Context View (Executive)**: Shows systems and actors - perfect for stakeholders
2. **Container View (Architect)**: Shows deployable units and their relationships
3. **Component View (Developer)**: Shows internal structure and implementation details
4. **Data Flow View**: Focuses on data dependencies and storage
5. **Complete View**: Shows everything for comprehensive documentation

### When to Use Views

- **Different Audiences**: Tailor diagrams to what each audience needs to see
- **Reduce Complexity**: Hide irrelevant details for specific discussions
- **Documentation**: Create multiple diagrams from one source of truth
- **Presentations**: Switch views during presentations to zoom in/out

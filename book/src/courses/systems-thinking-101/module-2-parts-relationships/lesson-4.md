# Lesson 4: Hierarchy and Nesting

## Learning Goals

- Understand how to organize parts hierarchically
- Learn nesting patterns for containers and components
- Model complex systems with clear structure
- Balance detail and clarity

## The C4 Hierarchy

Sruja follows the C4 model hierarchy:

```
Person (Level 1)
  ↓
System (Level 2)
  ↓
Container (Level 3)
  ↓
Component (Level 4)
```

## Nesting Containers in Systems

Systems contain containers:

```sruja
import { * } from 'sruja.ai/stdlib'

Shop = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
  }

  API = container "API Service" {
    technology "Node.js"
  }

  DB = database "PostgreSQL" {
    technology "PostgreSQL 14"
  }
}
```

### Benefits of Nesting

- **Clear ownership**: Containers belong to systems
- **Logical grouping**: Related components together
- **Simpler references**: Use dot notation
- **Better organization**: Easier to navigate

## Nesting Components in Containers

Containers contain components:

```sruja
API = container "API Service" {
  AuthService = component "Authentication Service"
  ProductService = component "Product Service"
  CartService = component "Cart Service"
  OrderService = component "Order Service"
}
```

### When to Add Components

Add components when:

- Container is complex (>5 logical parts)
- Different teams own different parts
- Need to show internal architecture
- Documenting for implementation

Skip components when:

- Container is simple (monolith)
- Too much detail for audience
- Container level is sufficient

## Reference Patterns

### Level 1 to Level 2 (Person to System)

```sruja
Customer -> Shop "Uses"
```

### Level 2 to Level 3 (System to Container)

Use dot notation:

```sruja
Customer -> Shop.WebApp "Browses"
Customer -> Shop.API "Submits order"
```

### Level 3 to Level 3 (Container to Container)

```sruja
Shop.WebApp -> Shop.API "Sends requests"
Shop.API -> Shop.DB "Persists data"
```

### Level 3 to Level 4 (Container to Component)

```sruja
Shop.API -> Shop.API.ProductService "Get products"
Shop.API -> Shop.API.OrderService "Create order"
```

### Level 4 to Level 4 (Component to Component)

```sruja
Shop.API.ProductService -> Shop.API.CartService "Get product details"
Shop.API.CartService -> Shop.API.OrderService "Create order"
```

## Multi-Level Systems

### Complex Example

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
  }

  API = container "API Service" {
    technology "Node.js"

    ProductService = component "Product Service"
    CartService = component "Cart Service"
    OrderService = component "Order Service"
    PaymentService = component "Payment Service"
    NotificationService = component "Notification Service"
  }

  Database = database "PostgreSQL" {
    technology "PostgreSQL 14"
  }

  Cache = queue "Redis" {
    technology "Redis 7"
  }
}

PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external"]
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external"]
  }
}

// Relationships at different levels
Customer -> ECommerce.WebApp "Browses"
ECommerce.WebApp -> ECommerce.API "Makes requests"
ECommerce.API.ProductService -> ECommerce.Cache "Reads cache"
ECommerce.API.PaymentService -> PaymentGateway "Process payment"
ECommerce.API.NotificationService -> EmailService "Send notifications"
```

## Hierarchy Best Practices

### 1. Be Consistent

Don't mix levels:

```sruja
// Bad: Inconsistent nesting
App = system "App" {
  Frontend = container "Frontend"
  Backend = container "Backend" {
    API = component "API Service"
  }
  Database = database "Database"
}

// Good: Consistent nesting
App = system "App" {
  Frontend = container "Frontend" {
    UI = component "UI Components"
  }
  Backend = container "Backend" {
    API = component "API Service"
  }
  Database = database "Database"
}
```

### 2. Right Level of Detail

Match detail to audience:

```sruja
// For business stakeholders
Platform = system "Platform" {
  WebApp = container "Web App"
  API = container "API"
}

// For developers
Platform = system "Platform" {
  WebApp = container "Web App" {
    Auth = component "Auth Module"
    Dashboard = component "Dashboard"
  }
  API = container "API" {
    UserService = component "User Service"
    ProductService = component "Product Service"
  }
}
```

### 3. Logical Grouping

Group related components:

```sruja
// Good: Logical grouping
API = container "API" {
  UserServices = component "User Service"
  ProductServices = component "Product Service"
  OrderServices = component "Order Service"
}

// Also Good: Domain-based grouping
API = container "API" {
  UserDomain = component "User Module"
  ProductDomain = component "Product Module"
  OrderDomain = component "Order Module"
}
```

## Implied Relationships

Sruja automatically infers some relationships:

```sruja
Customer -> Shop.WebApp "Uses"
// Implies: Customer -> Shop
```

This reduces boilerplate while maintaining accuracy.

## Views and Hierarchy

Create views at different hierarchy levels:

```sruja
// Level 1 view (System Context)
view index {
  title "System Context"
  include *
}

// Level 2 view (System)
view system_view of Shop {
  title "Shop System"
  include Shop
}

// Level 3 view (Containers)
view container_view of Shop {
  title "Shop Containers"
  include Shop.*
  exclude Shop.Database
}

// Level 4 view (Components)
view component_view of Shop.API {
  title "API Components"
  include Shop.API.*
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Deep Nesting

```sruja
// Bad: Too deep (hard to read)
App = system "App" {
  Frontend = container "Frontend" {
    Layout = component "Layout" {
      Header = component "Header" {
        Navigation = component "Navigation"
      }
    }
  }
}
```

**Solution**: Keep nesting to 3-4 levels max.

### Anti-Pattern 2: Orphaned Elements

```sruja
// Bad: Component without container
Shop = system "Shop"
WebApp = container "Web App"
API = component "API Service"  // Should be in a container
```

**Solution**: Nest components properly.

### Anti-Pattern 3: Flat Everything

```sruja
// Bad: Everything at same level
Customer = person "Customer"
WebApp = container "Web App"
API = container "API"
Database = database "Database"
AuthService = component "Auth Service"
```

**Solution**: Show proper hierarchy.

## Exercise

Create a nested architecture for:

> "A social media platform with users, posts, and comments. The platform has a web frontend, mobile API, and backend services. The backend has user management, post management, and notification services. Data is stored in PostgreSQL and cached in Redis."

**Create:**

- Person level
- System level with nested containers
- Component level for at least one container

## Key Takeaways

1. **Follow C4 hierarchy**: Person → System → Container → Component
2. **Nest logically**: Group related parts together
3. **Use dot notation**: Reference nested elements
4. **Right level of detail**: Match audience needs
5. **Create multiple views**: Show different hierarchy levels

## Module 2 Complete

You've completed Parts and Relationships! You now understand:

- How to identify system parts
- Sruja's element types
- Defining meaningful relationships
- Organizing parts hierarchically

**Next**: Learn about [Module 3: Boundaries](../module-3-boundaries/module-overview.md).

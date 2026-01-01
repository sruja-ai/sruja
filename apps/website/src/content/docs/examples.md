---
title: "Usage Examples"
weight: 80
summary: "Production-ready architectures. Learn how to model Fintech, E-Commerce, and SaaS systems with Sruja."
---

# Examples & Patterns

Theory is good, but code is better. Below are **production-grade** Sruja models that you can copy, paste, and adapt.

Every example here follows our ["FAANG-level" quality standards](/docs/style-guide):

1.  **Clear Requirements**: Functional & Non-functional.
2.  **Proper Hierarchies**: Context -> Container -> Component.
3.  **Real Tech Stacks**: No generic "Database" boxes.

---

## 1. Banking System (Fintech)

> [!NOTE]
> **Ideally Suited For**: Highly regulated industries requiring audit trails, security policies, and strict latency SLAs.

**Scenario**: A regional bank needs to modernize its legacy mainframe interactions while providing a slick mobile experience.

### Why review this example?

- **Security**: Uses `policy` blocks for PCI-DSS.
- **Hybrid Cloud**: Connects modern Cloud Containers to an on-premise "Mainframe" System.
- **Complexity**: Models the "Legacy Core" vs "Modern Interface" pattern often seen in enterprise.

```sruja
import { * } from 'sruja.ai/stdlib'


// --- REQUIREMENTS ---
// We start with the 'Why'. These drive the architecture.
R1 = requirement functional "Customers must be able to view balances"
R2 = requirement functional "Customers can transfer money internally"
R3 = requirement security "All PII must be encrypted at rest (PCI-DSS)"
R4 = requirement stability "99.99% Availability (Target: <52m downtime/year)"

// --- ACTORS ---
Customer = person "Banking Customer" {
    description "A holder of one or more accounts"
}

// --- SYSTEMS ---
BankingSystem = system "Internet Banking Platform" {
    description "Allows customers to view information and make payments."

    // Containers (Deployable units)
    WebApp = container "Single Page App" {
        technology "React / TypeScript"
    }

    MobileApp = container "Mobile App" {
        technology "Flutter"
    }

    API = container "Main API Gateway" {
        technology "Java / Spring Boot"
        description "Orchestrates calls to core services"
    }

    Database = container "Main RDBMS" {
        technology "PostgreSQL"
        tags ["database", "storage"]
    }

    // Relationships
    WebApp -> API "Uses (JSON/HTTPS)"
    MobileApp -> API "Uses (JSON/HTTPS)"
    API -> Database "Reads/Writes (JDBC)"
}

// --- EXTERNAL SYSTEMS ---
Mainframe = system "Legacy Core Banking" {
    tags ["external"] // This is outside our scope of control
    description "The heavy iron that stores the actual money."
}

EmailSystem = system "Email Service" {
    tags ["external"]
    description "SendGrid / AWS SES"
}

// --- INTEGRATIONS ---
Customer -> BankingSystem.WebApp "Views dashboard"
BankingSystem.API -> Mainframe "Syncs transactions (XML/SOAP)"
BankingSystem.API -> EmailSystem "Sends alerts"

view index {
include *
}
```

👉 **[Deep Dive this Architecture using our Course](/courses/system-design-101/module-3-advanced-modeling)**

---

## 2. Global E-Commerce Platform

> [!NOTE]
> **Ideally Suited For**: High-scale B2C applications. Focuses on caching, asynchronous processing, and eventual consistency.

**Scenario**: An Amazon-like store preparing for Black Friday traffic spikes.

### Why review this example?

- **Scalability**: Explains how to handle high reads (Product Catalog) vs transactional writes (Checkout).
- **Async Messaging**: Shows usages of Queues/Topics (`Apache Kafka`) to decouple services.
- **Caching**: Strategic placement of Redis caches.

```sruja
import { * } from 'sruja.ai/stdlib'


R1 = requirement scale "Handle 100k concurrent users"
R2 = requirement performance "Product pages load in <100ms"

ShopScale = system "E-Commerce Platform" {

    // --- EDGE LAYER ---
    CDN = container "Content Delivery Network" {
        technology "Cloudflare"
        description "Caches static assets and product images"
    }

    LoadBalancer = container "Load Balancer" {
        technology "NGINX"
    }

    // --- SERVICE LAYER ---
    Storefront = container "Storefront Service" {
        technology "Node.js"
        description "SSR for SEO-friendly product pages"
    }

    Checkout = container "Checkout Service" {
        technology "Go"
        description "Handles payments and inventory locking"
    }

    // --- DATA LAYER ---
    ProductCache = container "Product Cache" {
        technology "Redis Cluster"
        description "Stores hot product data"
    }

    MainDB = database "Product Database" {
        technology "MongoDB"
        description "Flexible schema for diverse product attributes"
    }

    OrderQueue = queue "Order Events" {
        technology "Kafka"
        description "Async order processing pipeline"
    }

    // --- FLOWS ---
    CDN -> LoadBalancer "Forwards dynamic requests"
    LoadBalancer -> Storefront "Routes traffic"
    Storefront -> ProductCache "Read-through cache"
    Storefront -> MainDB "Cache miss / heavy query"

    // The Checkout Flow
    Checkout -> OrderQueue "Publishes 'OrderCreated'"
}

view index {
include *
}
```

---

## What Next?

- **New to Sruja?** Try the **[First Architecture Tutorial](/tutorials/basic/getting-started)**.
- **Need more depth?** Check out the full **[Content Library](/courses)**.

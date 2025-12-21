// Auto-generated from examples/manifest.json - DO NOT EDIT MANUALLY
// Run: go run scripts/generate-playground-examples.go

import type { PlaygroundExample } from './types';

export const EXAMPLES: PlaygroundExample[] = [
  {
    name: "Overview",
    code: `// examples/overview_showcase.sruja
// Example demonstrating the new overview block for architecture reviews

architecture "E-Commerce Platform" {
  overview {
    summary "High-performance e-commerce platform supporting 10M+ users with sub-200ms latency"
    audience "Engineering teams, architects, stakeholders, and reviewers"
    scope "Covers ordering, payments, inventory management, and analytics subsystems"
    goals ["Scale to 50M users by Q4", "Maintain sub-200ms p95 latency", "Achieve 99.99% uptime SLA"]
    nonGoals ["Real-time inventory sync (eventual consistency acceptable)", "Multi-currency support (Phase 2)"]
    risks ["Payment Gateway single point of failure", "Database write bottleneck during flash sales", "CDN vendor lock-in"]
  }
  
  description "Enterprise-grade e-commerce solution with microservices architecture"
  
  metadata {
    team "E-Commerce Platform Team"
    owner "platform-team@example.com"
    version "2.1.0"
    status "production"
    scale "10M+ users, 1M+ requests/day"
    cost "$15k/month operational cost"
  }
  
  person Customer "Customer"
  person Admin "Administrator"
  
  system ECommerce "E-Commerce Platform" "Main platform handling all e-commerce operations" {
    container WebApp "Web Application" {
      technology "React"
    }
    container API "REST API" {
      technology "Go"
    }
    datastore ProductDB "Product Database"
    datastore OrderDB "Order Database"
  }
  
  system PaymentGateway "Payment Gateway" "External payment processing service" {
    container StripeAPI "Stripe API" {
      technology "Stripe"
    }
  }
  
  Customer -> ECommerce.WebApp "Browses and purchases"
  ECommerce.WebApp -> ECommerce.API "Calls"
  ECommerce.API -> ECommerce.ProductDB "Reads products"
  ECommerce.API -> ECommerce.OrderDB "Writes orders"
  ECommerce.API -> PaymentGateway.StripeAPI "Processes payments"
  
  requirement REQ001 functional "Users must be able to browse products by category"
  requirement REQ002 performance "Product catalog page must load in under 2 seconds"
  requirement REQ003 security "All API endpoints must use HTTPS"
  
  adr ADR001 "Use Microservices Architecture" {
    status "Accepted"
    context "Need to scale different parts of the system independently"
    decision "Adopt microservices with service boundaries aligned to business capabilities"
    consequences "Gain: Independent scaling. Trade-off: Increased operational complexity"
  }
  
  policy SecurityPolicy "Enforce TLS 1.3 for all external communications" category "security" enforcement "required"
  policy DataRetentionPolicy "Retain order data for 7 years for tax compliance" category "compliance" enforcement "required"
  
  flow CheckoutFlow "Customer Checkout Flow" {
    Customer -> ECommerce.WebApp "Initiates checkout"
    ECommerce.WebApp -> ECommerce.API "Submit order"
    ECommerce.API -> PaymentGateway.StripeAPI "Process payment"
    ECommerce.API -> ECommerce.OrderDB "Save order"
  }
}`
  },
  {
    name: "Queues & DataStore",
    code: `architecture "Queues and DataStore Example" {
  system Service "Service" {
    container API "API" {
    }
    queue MQ "Message Queue"
    datastore DB "Database"
  }
  Service.API -> Service.MQ publishes "Publishes messages"
  Service.API -> Service.DB reads "Reads data"
  Service.API -> Service.DB writes "Writes data"
}`
  },
  {
    name: "Tags & Filtering",
    code: `architecture "Tags Showcase Example" {
  system App "Application" {
    container API "HTTP API" {
      tags ["http", "public"]
    }
    datastore DB "PostgreSQL"
    queue MQ "Events"
  }
  App.API -> App.DB reads "Reads data"
  App.API -> App.DB writes "Writes data"
  App -> App.MQ publishes "Publishes events"
}`
  },
  {
    name: "Metadata",
    code: `architecture "Metadata Showcase Example" {
  system BillingAPI {
    metadata {
      team "Payments"
      tier "critical"
      rate_limit_per_ip "50/s"
      error_budget "99.5%"
      cloud "aws"
      lambda_memory "512MB"
      tracing "enabled"
    }
    container API {
      metadata {
        rate_limit "100/s"
        auth "jwt"
        upstream "BillingAPI"
      }
    }
  }
  
  datastore InvoiceDB {
    metadata {
      engine "PostgreSQL"
      version "14"
      encryption_at_rest "AES-256"
      compliance "PCI-DSS"
    }
  }
  
  queue Events {
    metadata {
      provider "RabbitMQ"
      durable "true"
      prefetch "10"
    }
  }
  
  person Customer {
    metadata {
      persona "end-user"
      access_level "standard"
    }
  }
}`
  },
  {
    name: "Implied Relationships",
    code: `// examples/real_world_implied_relationships.sruja
// Real-world example: Microservices E-commerce Platform
// Demonstrates how implied relationships reduce boilerplate in complex architectures

architecture "Microservices E-commerce Platform" {
  // External actors
  person Customer "Customer"
  person Admin "Administrator"
  person SupportAgent "Support Agent"
  
  // Main system with multiple services
  system ECommerce "E-Commerce Platform" {
    // Frontend services
    container WebApp "Web Application" {
      technology "React, TypeScript"
      component ProductCatalog "Product Catalog"
      component ShoppingCart "Shopping Cart"
      component Checkout "Checkout Flow"
    }
    
    container MobileApp "Mobile Application" {
      technology "React Native"
      component ProductBrowser "Product Browser"
      component CartManager "Cart Manager"
    }
    
    // Backend services
    container ProductService "Product Service" {
      technology "Go, gRPC"
      component ProductAPI "Product API"
      component SearchEngine "Search Engine"
    }
    
    container OrderService "Order Service" {
      technology "Java, Spring Boot"
      component OrderAPI "Order API"
      component OrderProcessor "Order Processor"
    }
    
    container PaymentService "Payment Service" {
      technology "Node.js, Express"
      component PaymentAPI "Payment API"
      component PaymentGateway "Payment Gateway"
    }
    
    container UserService "User Service" {
      technology "Python, FastAPI"
      component AuthAPI "Authentication API"
      component ProfileAPI "Profile API"
    }
    
    // Data stores
    datastore ProductDB "Product Database" {
      technology "PostgreSQL"
    }
    
    datastore OrderDB "Order Database" {
      technology "PostgreSQL"
    }
    
    datastore UserDB "User Database" {
      technology "MongoDB"
    }
    
    datastore Cache "Redis Cache" {
      technology "Redis"
    }
    
    // Message queue
    queue EventQueue "Event Queue" {
      technology "Kafka"
    }
  }
  
  // External services
  system PaymentProvider "Payment Provider" {
    metadata {
      tags ["external"]
    }
  }
  
  system EmailService "Email Service" {
    metadata {
      tags ["external"]
    }
  }
  
  system AnalyticsService "Analytics Service" {
    metadata {
      tags ["external"]
    }
  }
  
  // Relationships - With implied relationships, we only need to define
  // relationships to specific containers/components, and parent relationships
  // are automatically inferred
  
  // Customer interactions - only define specific container relationships
  Customer -> ECommerce.WebApp "Browses products"
  Customer -> ECommerce.WebApp.ProductCatalog "Views products"
  Customer -> ECommerce.WebApp.ShoppingCart "Adds items"
  Customer -> ECommerce.WebApp.Checkout "Completes purchase"
  Customer -> ECommerce.MobileApp "Uses mobile app"
  
  // Service-to-service communication
  ECommerce.WebApp -> ECommerce.ProductService "Fetches products"
  ECommerce.WebApp -> ECommerce.OrderService "Creates orders"
  ECommerce.WebApp -> ECommerce.PaymentService "Processes payments"
  ECommerce.WebApp -> ECommerce.UserService "Authenticates users"
  
  ECommerce.MobileApp -> ECommerce.ProductService "Fetches products"
  ECommerce.MobileApp -> ECommerce.OrderService "Creates orders"
  
  // Service to database relationships
  ECommerce.ProductService -> ECommerce.ProductDB "Reads/Writes"
  ECommerce.ProductService -> ECommerce.Cache "Caches products"
  ECommerce.OrderService -> ECommerce.OrderDB "Reads/Writes"
  ECommerce.UserService -> ECommerce.UserDB "Reads/Writes"
  
  // Event-driven communication
  ECommerce.OrderService -> ECommerce.EventQueue "Publishes order events"
  ECommerce.PaymentService -> ECommerce.EventQueue "Publishes payment events"
  
  // External service integrations
  ECommerce.PaymentService -> PaymentProvider "Processes payments"
  ECommerce.OrderService -> EmailService "Sends order confirmations"
  ECommerce.UserService -> EmailService "Sends verification emails"
  ECommerce.WebApp -> AnalyticsService "Tracks user behavior"
  
  // Admin and support interactions
  Admin -> ECommerce.OrderService "Manages orders"
  Admin -> ECommerce.ProductService "Manages products"
  SupportAgent -> ECommerce.OrderService "Views order details"
  SupportAgent -> ECommerce.UserService "Views user profiles"
  
  // The following relationships are automatically inferred:
  //   Customer -> ECommerce (from Customer -> ECommerce.WebApp.*)
  //   Customer -> ECommerce (from Customer -> ECommerce.MobileApp)
  //   ECommerce.WebApp -> ECommerce (from ECommerce.WebApp -> ECommerce.*Service)
  //   ECommerce.ProductService -> ECommerce (from ECommerce.ProductService -> ECommerce.ProductDB)
  //   Admin -> ECommerce (from Admin -> ECommerce.OrderService)
  //   SupportAgent -> ECommerce (from SupportAgent -> ECommerce.*Service)
  //   ECommerce -> PaymentProvider (from ECommerce.PaymentService -> PaymentProvider)
  //   ECommerce -> EmailService (from ECommerce.*Service -> EmailService)
  //   ECommerce -> AnalyticsService (from ECommerce.WebApp -> AnalyticsService)
  
  requirement R1 functional "Must handle 100k concurrent users"
  requirement R2 performance "API response time < 200ms (p95)"
  requirement R3 availability "99.9% uptime SLA"
  requirement R4 security "PCI-DSS compliant for payment processing"
  
  adr ADR001 "Microservices Architecture" {
    status "accepted"
    context "Need independent scaling and deployment"
    decision "Adopt microservices architecture with service mesh"
    consequences "Gain: Independent scaling, deployment. Trade-off: Increased complexity, network latency"
  }
  
  adr ADR002 "Event-Driven Communication" {
    status "accepted"
    context "Services need to communicate asynchronously"
    decision "Use Kafka for event streaming"
    consequences "Gain: Loose coupling, scalability. Trade-off: Eventual consistency, complexity"
  }
}`
  },
  {
    name: "Views Customization",
    code: `// examples/real_world_views_customization.sruja
// Real-world example: SaaS Platform with Custom Views
// Demonstrates practical use cases for views block customization

architecture "SaaS Analytics Platform" {
  person EndUser "End User"
  person DataAnalyst "Data Analyst"
  person DevOpsEngineer "DevOps Engineer"
  person ProductManager "Product Manager"
  
  system AnalyticsPlatform "Analytics Platform" {
    // Frontend
    container Dashboard "Analytics Dashboard" {
      technology "React, D3.js"
      tags ["frontend", "ui"]
      component ChartBuilder "Chart Builder"
      component ReportGenerator "Report Generator"
      component DataExplorer "Data Explorer"
    }
    
    container AdminPanel "Admin Panel" {
      technology "React"
      tags ["frontend", "admin"]
      component UserManagement "User Management"
      component BillingManagement "Billing Management"
    }
    
    // API Layer
    container API "REST API" {
      technology "Node.js, Express"
      tags ["backend", "api"]
      component QueryEngine "Query Engine"
      component AuthMiddleware "Auth Middleware"
      component RateLimiter "Rate Limiter"
    }
    
    container GraphQLAPI "GraphQL API" {
      technology "Node.js, Apollo"
      tags ["backend", "api"]
      component SchemaResolver "Schema Resolver"
      component DataLoader "Data Loader"
    }
    
    // Processing Services
    container DataIngestion "Data Ingestion Service" {
      technology "Go"
      tags ["backend", "processing"]
      component EventCollector "Event Collector"
      component DataTransformer "Data Transformer"
      component BatchProcessor "Batch Processor"
    }
    
    container QueryService "Query Service" {
      technology "Go"
      tags ["backend", "processing"]
      component QueryParser "Query Parser"
      component QueryOptimizer "Query Optimizer"
      component ResultAggregator "Result Aggregator"
    }
    
    container AlertService "Alert Service" {
      technology "Python"
      tags ["backend", "processing"]
      component RuleEngine "Rule Engine"
      component NotificationSender "Notification Sender"
    }
    
    // Data Layer
    datastore TimeSeriesDB "Time Series Database" {
      technology "InfluxDB"
      metadata {
        tags ["Database", "TimeSeries"]
      }
    }
    
    datastore DataWarehouse "Data Warehouse" {
      technology "Snowflake"
      metadata {
        tags ["Database", "Warehouse"]
      }
    }
    
    datastore MetadataDB "Metadata Database" {
      technology "PostgreSQL"
      metadata {
        tags ["Database", "Relational"]
      }
    }
    
    datastore Cache "Cache Layer" {
      technology "Redis"
      metadata {
        tags ["Database", "Cache"]
      }
    }
    
    // Message Queue
    queue EventStream "Event Stream" {
      technology "Kafka"
      metadata {
        tags ["Queue", "Streaming"]
      }
    }
    
    queue TaskQueue "Task Queue" {
      technology "RabbitMQ"
      metadata {
        tags ["Queue", "Tasks"]
      }
    }
  }
  
  // External services
  system PaymentGateway "Payment Gateway" {
    metadata {
      tags ["external"]
    }
  }
  
  system EmailProvider "Email Provider" {
    metadata {
      tags ["external"]
    }
  }
  
  // Relationships
  EndUser -> AnalyticsPlatform.Dashboard "Uses"
  DataAnalyst -> AnalyticsPlatform.Dashboard "Creates reports"
  ProductManager -> AnalyticsPlatform.AdminPanel "Manages platform"
  DevOpsEngineer -> AnalyticsPlatform.API "Monitors"
  
  AnalyticsPlatform.Dashboard -> AnalyticsPlatform.API "Fetches data"
  AnalyticsPlatform.Dashboard -> AnalyticsPlatform.GraphQLAPI "Queries data"
  AnalyticsPlatform.AdminPanel -> AnalyticsPlatform.API "Manages"
  
  AnalyticsPlatform.API -> AnalyticsPlatform.QueryService "Executes queries"
  AnalyticsPlatform.GraphQLAPI -> AnalyticsPlatform.QueryService "Resolves queries"
  AnalyticsPlatform.DataIngestion -> AnalyticsPlatform.TimeSeriesDB "Writes data"
  AnalyticsPlatform.QueryService -> AnalyticsPlatform.TimeSeriesDB "Reads data"
  AnalyticsPlatform.QueryService -> AnalyticsPlatform.DataWarehouse "Queries"
  AnalyticsPlatform.QueryService -> AnalyticsPlatform.Cache "Caches results"
  
  AnalyticsPlatform.DataIngestion -> AnalyticsPlatform.EventStream "Publishes events"
  AnalyticsPlatform.AlertService -> AnalyticsPlatform.EventStream "Consumes events"
  AnalyticsPlatform.QueryService -> AnalyticsPlatform.TaskQueue "Queues tasks"
  
  AnalyticsPlatform.AdminPanel -> PaymentGateway "Processes payments"
  AnalyticsPlatform.AlertService -> EmailProvider "Sends alerts"
  
  // Custom views for different stakeholders
  views {
    // View for developers: Focus on API and processing services
    container AnalyticsPlatform "Developer View" {
      include AnalyticsPlatform.API AnalyticsPlatform.GraphQLAPI
      include AnalyticsPlatform.QueryService AnalyticsPlatform.DataIngestion
      include AnalyticsPlatform.AlertService
      exclude AnalyticsPlatform.Dashboard AnalyticsPlatform.AdminPanel
    }
    
    // View for product managers: Focus on user-facing components
    container AnalyticsPlatform "Product View" {
      include AnalyticsPlatform.Dashboard AnalyticsPlatform.AdminPanel
      include AnalyticsPlatform.API AnalyticsPlatform.GraphQLAPI
      exclude AnalyticsPlatform.DataIngestion AnalyticsPlatform.AlertService
    }
    
    // View for data engineers: Focus on data flow
    container AnalyticsPlatform "Data Flow View" {
      include AnalyticsPlatform.DataIngestion
      include AnalyticsPlatform.TimeSeriesDB AnalyticsPlatform.DataWarehouse
      include AnalyticsPlatform.EventStream AnalyticsPlatform.TaskQueue
      include AnalyticsPlatform.QueryService
    }
    
    // System context view for executives
    systemContext AnalyticsPlatform "Executive Overview" {
      include *
    }
    
    // Custom styling for different element types
    styles {
      element "Database" {
        shape "cylinder"
        color "#4A90E2"
        stroke "#2E5C8A"
        strokeWidth "2"
      }
      
      element "TimeSeries" {
        shape "cylinder"
        color "#50C878"
        stroke "#2E7D4E"
      }
      
      element "Warehouse" {
        shape "cylinder"
        color "#9B59B6"
        stroke "#6C3483"
      }
      
      element "Cache" {
        shape "cylinder"
        color "#F39C12"
        stroke "#D68910"
      }
      
      element "Queue" {
        shape "box"
        color "#E74C3C"
        stroke "#C0392B"
      }
      
      element "Streaming" {
        shape "box"
        color "#E67E22"
        stroke "#D35400"
      }
      
      element "external" {
        shape "box"
        color "#95A5A6"
        stroke "#7F8C8D"
        strokeDasharray "5,5"
      }
      
      relationship "Relationship" {
        thickness "2"
        color "#34495E"
      }
    }
  }
  
  requirement R1 functional "Must support 1M events/minute ingestion"
  requirement R2 performance "Query response time < 500ms (p95)"
  requirement R3 scalability "Must scale to 10TB+ data"
  requirement R4 reliability "99.99% uptime SLA"
}`
  },
  {
    name: "Governance",
    code: `architecture "E-Commerce" {
  system Shop "Shop" {
    adr DB_Choice "Database Selection" {
      status "Accepted"
      context "Choose between Postgres and Mongo"
      decision "Use Postgres for relational needs"
      consequences "Reliable ACID; more ops complexity"
    }
  }

  policy SecurityPolicy "Encrypt data at rest" category "security" enforcement "required"
}`
  },
  {
    name: "Service Level Objectives",
    code: `architecture "SLO Example" {
  requirement R1 performance "Must maintain 99.9% availability"
  requirement R2 performance "API response time must be under 200ms p95"

  system API "API Service" {
    
    slo {
      availability {
        target "99.9%"
        window "30 days"
        current "99.95%"
      }
      latency {
        p95 "200ms"
        p99 "500ms"
        window "7 days"
        current {
          p95 "180ms"
          p99 "420ms"
        }
      }
      errorRate {
        target "0.1%"
        window "7 days"
        current "0.08%"
      }
      throughput {
        target "10000 req/s"
        window "peak hour"
        current "8500 req/s"
      }
    }
    
    container WebApp "Web Application" {
      slo {
        availability {
          target "99.9%"
          window "30 days"
        }
        latency {
          p95 "150ms"
          p99 "300ms"
          window "7 days"
        }
      }
    }
  }
}`
  },
  {
    name: "Scenarios",
    code: `architecture "User Stories with Qualified Actors" {
    person User "Customer"
    system Inventory "Inventory System"
    system ECommerce "E-Commerce System" {
        container CartPage "Cart Page"
    }
    
    story Checkout "User Checkout Flow" {
        User -> ECommerce.CartPage "adds item to cart"
        ECommerce.CartPage -> ECommerce "clicks checkout"
        ECommerce -> Inventory "Check Stock"
    }
}`
  },
  {
    name: "Shared Libraries",
    code: `architecture "Shared Model" {
  system Shared "Shared" {
    datastore SharedDB "Shared DB"
  }
}`
  },
  {
    name: "Microservices Pattern",
    code: `// examples/real_world_microservices.sruja
// Real-world example: Microservices Architecture with Implied Relationships
// Shows how implied relationships simplify complex microservices diagrams

architecture "E-commerce Microservices Platform" {
  person Customer "Customer"
  person Merchant "Merchant"
  person Admin "Platform Administrator"
  
  system ECommerce "E-Commerce Platform" {
    // API Gateway
    container APIGateway "API Gateway" {
      technology "Kong"
      component Routing "Routing"
      component Auth "Authentication"
      component RateLimit "Rate Limiting"
    }
    
    // Core Services
    container CatalogService "Catalog Service" {
      technology "Java, Spring Boot"
      component ProductAPI "Product API"
      component SearchAPI "Search API"
      component CategoryAPI "Category API"
    }
    
    container InventoryService "Inventory Service" {
      technology "Go"
      component StockAPI "Stock API"
      component ReservationAPI "Reservation API"
    }
    
    container CartService "Cart Service" {
      technology "Node.js"
      component CartAPI "Cart API"
      component SessionManager "Session Manager"
    }
    
    container OrderService "Order Service" {
      technology "Java, Spring Boot"
      component OrderAPI "Order API"
      component OrderProcessor "Order Processor"
      component FulfillmentCoordinator "Fulfillment Coordinator"
    }
    
    container PaymentService "Payment Service" {
      technology "Go"
      component PaymentAPI "Payment API"
      component RefundProcessor "Refund Processor"
    }
    
    container UserService "User Service" {
      technology "Python, FastAPI"
      component AuthAPI "Auth API"
      component ProfileAPI "Profile API"
      component PreferenceAPI "Preference API"
    }
    
    container NotificationService "Notification Service" {
      technology "Node.js"
      component EmailSender "Email Sender"
      component SMSSender "SMS Sender"
      component PushNotifier "Push Notifier"
    }
    
    // Data Stores
    datastore CatalogDB "Catalog Database" {
      technology "PostgreSQL"
    }
    
    datastore InventoryDB "Inventory Database" {
      technology "PostgreSQL"
    }
    
    datastore CartDB "Cart Database" {
      technology "Redis"
    }
    
    datastore OrderDB "Order Database" {
      technology "PostgreSQL"
    }
    
    datastore UserDB "User Database" {
      technology "MongoDB"
    }
    
    // Message Queue
    queue OrderEvents "Order Events" {
      technology "Kafka"
    }
    
    queue NotificationQueue "Notification Queue" {
      technology "RabbitMQ"
    }
  }
  
  // External Services
  system PaymentProvider "Payment Provider" {
    metadata {
      tags ["external"]
    }
  }
  
  system ShippingProvider "Shipping Provider" {
    metadata {
      tags ["external"]
    }
  }
  
  system EmailService "Email Service" {
    metadata {
      tags ["external"]
    }
  }
  
  // Customer flow - only define specific service relationships
  // Parent relationships are automatically inferred
  Customer -> ECommerce.APIGateway "Accesses platform"
  Customer -> ECommerce.CatalogService "Browses products"
  Customer -> ECommerce.CartService "Manages cart"
  Customer -> ECommerce.OrderService "Places orders"
  Customer -> ECommerce.PaymentService "Makes payments"
  
  // Service-to-service communication
  ECommerce.APIGateway -> ECommerce.CatalogService "Routes requests"
  ECommerce.APIGateway -> ECommerce.CartService "Routes requests"
  ECommerce.APIGateway -> ECommerce.OrderService "Routes requests"
  ECommerce.APIGateway -> ECommerce.PaymentService "Routes requests"
  ECommerce.APIGateway -> ECommerce.UserService "Routes requests"
  
  ECommerce.CatalogService -> ECommerce.InventoryService "Checks availability"
  ECommerce.CartService -> ECommerce.InventoryService "Reserves items"
  ECommerce.OrderService -> ECommerce.InventoryService "Deducts stock"
  ECommerce.OrderService -> ECommerce.PaymentService "Processes payment"
  ECommerce.OrderService -> ECommerce.NotificationService "Sends notifications"
  
  // Service to database
  ECommerce.CatalogService -> ECommerce.CatalogDB "Reads/Writes"
  ECommerce.InventoryService -> ECommerce.InventoryDB "Reads/Writes"
  ECommerce.CartService -> ECommerce.CartDB "Reads/Writes"
  ECommerce.OrderService -> ECommerce.OrderDB "Reads/Writes"
  ECommerce.UserService -> ECommerce.UserDB "Reads/Writes"
  
  // Event-driven communication
  ECommerce.OrderService -> ECommerce.OrderEvents "Publishes order events"
  ECommerce.NotificationService -> ECommerce.NotificationQueue "Queues notifications"
  
  // External integrations
  ECommerce.PaymentService -> PaymentProvider "Processes payments"
  ECommerce.OrderService -> ShippingProvider "Creates shipments"
  ECommerce.NotificationService -> EmailService "Sends emails"
  
  // Merchant interactions
  Merchant -> ECommerce.CatalogService "Manages products"
  Merchant -> ECommerce.OrderService "Views orders"
  
  // Admin interactions
  Admin -> ECommerce.UserService "Manages users"
  Admin -> ECommerce.OrderService "Monitors orders"
  
  // Implied relationships automatically created:
  //   Customer -> ECommerce (from Customer -> ECommerce.*Service)
  //   ECommerce.APIGateway -> ECommerce (from ECommerce.APIGateway -> ECommerce.*Service)
  //   ECommerce.*Service -> ECommerce (from ECommerce.*Service -> ECommerce.*DB)
  //   ECommerce -> PaymentProvider (from ECommerce.PaymentService -> PaymentProvider)
  //   ECommerce -> ShippingProvider (from ECommerce.OrderService -> ShippingProvider)
  //   ECommerce -> EmailService (from ECommerce.NotificationService -> EmailService)
  //   Merchant -> ECommerce (from Merchant -> ECommerce.*Service)
  //   Admin -> ECommerce (from Admin -> ECommerce.*Service)
  
  views {
    // View for API team: Focus on API Gateway and services
    container ECommerce "API Architecture" {
      include ECommerce.APIGateway
      include ECommerce.CatalogService ECommerce.CartService
      include ECommerce.OrderService ECommerce.PaymentService
      include ECommerce.UserService
      exclude ECommerce.InventoryService ECommerce.NotificationService
    }
    
    // View for data team: Focus on data flow
    container ECommerce "Data Architecture" {
      include ECommerce.CatalogService ECommerce.InventoryService
      include ECommerce.OrderService ECommerce.UserService
      include ECommerce.CatalogDB ECommerce.InventoryDB
      include ECommerce.OrderDB ECommerce.UserDB
      include ECommerce.OrderEvents ECommerce.NotificationQueue
    }
    
    // View for operations: Focus on external dependencies
    systemContext ECommerce "External Dependencies" {
      include ECommerce PaymentProvider ShippingProvider EmailService
    }
    
    styles {
      element "Database" {
        shape "cylinder"
        color "#3498DB"
        stroke "#2980B9"
      }
      
      element "Queue" {
        shape "box"
        color "#E67E22"
        stroke "#D35400"
      }
      
      element "external" {
        shape "box"
        color "#95A5A6"
        stroke "#7F8C8D"
        strokeDasharray "5,5"
      }
    }
  }
  
  requirement R1 functional "Must handle 50k orders/day"
  requirement R2 performance "API response time < 300ms (p95)"
  requirement R3 availability "99.9% uptime"
  requirement R4 scalability "Scale to 10x traffic during sales events"
}`
  },
  {
    name: "RAG Pipeline",
    code: `architecture "Enterprise RAG Pipeline" {
  overview {
    summary "Retrieval-Augmented Generation (RAG) pipeline for enterprise knowledge management"
    audience "Data Engineers, AI Architects, Compliance Officers"
    scope "Document ingestion, embedding generation, vector retrieval, and LLM answer generation"
    goals ["Provide accurate answers from internal docs", "Ensure data freshness < 1hr", "Enforce access controls"]
    nonGoals ["General purpose chatbot", "Creative writing"]
    risks ["Hallucinations", "Leaking sensitive document data to unauthorized users"]
  }

  person Employee "Employee" {
    description "Knowledge worker seeking internal information"
  }

  person DataAdmin "Data Administrator" {
    description "Manages document sources and access policies"
  }

  system RAGPlatform "RAG Platform" {
    description "End-to-end RAG system"

    container UI "Chat Interface" {
      technology "React/Tailwind"
      description "Chatbot UI with citation support"
      component ChatWindow "Message stream"
      component CitationViewer "Source document preview"
    }

    container Gateway "API Gateway" {
      technology "Python/FastAPI"
      description "Orchestrates retrieval and generation"
      slo { latency { p95 "2s" p99 "5s" } }
      component QueryPreprocessor "Rewrites queries for better retrieval"
      component Guardrails "Input/Output safety checks"
    }

    container Ingestion "Ingestion Service" {
      technology "Python/LangChain"
      description "Processes documents into vectors"
      component PDFLoader "Parses PDFs"
      component Chunker "Splits text (RecursiveCharacterTextSplitter)"
      component Embedder "Generates embeddings (OpenAI/Cohere)"
    }

    container Retrieval "Retrieval Service" {
      technology "Python"
      description "Semantic search and reranking"
      component HybridSearch "Keyword + Vector search"
      component Reranker "Cross-encoder reranking (Cohere)"
    }

    datastore VectorDB "Vector Database" {
      technology "Qdrant / Milvus"
      description "Stores document embeddings and metadata"
      scale { min 3 max 5 metric "memory > 80%" }
    }

    datastore DocStore "Document Store" {
      technology "S3 / MinIO"
      description "Raw document storage"
    }

    datastore Cache "Semantic Cache" {
      technology "Redis"
      description "Caches frequent queries and answers"
    }

    queue IngestionQueue "Ingestion Queue" {
      technology "RabbitMQ"
      description "Decouples upload from processing"
    }
    
    // Internal Flows
    Gateway.QueryPreprocessor -> Retrieval.HybridSearch "Search"
    Retrieval.HybridSearch -> VectorDB "KNN Search"
    Retrieval.Reranker -> Retrieval.HybridSearch "Re-rank results"
    Gateway -> Cache "Check hit"
    Ingestion.Chunker -> Ingestion.Embedder "Embed chunks"
    Ingestion.Embedder -> VectorDB "Upsert vectors"
    Ingestion.PDFLoader -> DocStore "Fetch raw"
    IngestionQueue -> Ingestion.PDFLoader "Trigger job"

    // External dependencies
    system LLM "LLM Provider" {
      description "OpenAI GPT-4 or Azure OpenAI"
      metadata { tags ["external", "cost-sensitive"] }
    }
    
    Gateway -> LLM "Generate answer"
    Ingestion.Embedder -> LLM "Get embeddings"
  }

  system InternalSources "Internal Data Sources" {
    container SharePoint "SharePoint"
    container Confluence "Confluence"
    container GoogleDrive "Google Drive"
  }

  // User Flows
  Employee -> RAGPlatform.UI "Asks question"
  RAGPlatform.UI -> RAGPlatform.Gateway "Submit query"
  
  // Admin Flows
  DataAdmin -> RAGPlatform.Ingestion "Configure sources"
  InternalSources.SharePoint -> RAGPlatform.IngestionQueue "Webhook/Sync"

  // Scenarios
  scenario "Answer Question" {
    Employee -> RAGPlatform.UI "What is the travel policy?"
    RAGPlatform.UI -> RAGPlatform.Gateway "Query"
    RAGPlatform.Gateway -> RAGPlatform.Cache "Check cache"
    // Cache Miss
    RAGPlatform.Gateway -> RAGPlatform.Retrieval "Retrieve context"
    RAGPlatform.Retrieval -> RAGPlatform.VectorDB "Semantic search"
    RAGPlatform.VectorDB -> RAGPlatform.Retrieval "Top 20 chunks"
    RAGPlatform.Retrieval -> RAGPlatform.Gateway "Context"
    RAGPlatform.Gateway -> RAGPlatform.LLM "Generate(Query + Context)"
    RAGPlatform.LLM -> RAGPlatform.Gateway "Answer"
    RAGPlatform.Gateway -> RAGPlatform.UI "Stream response"
  }

  flow IngestionFlow "Document Indexing" {
    InternalSources -> IngestionQueue "New Document Event"
    IngestionQueue -> Ingestion.PDFLoader "Process"
    Ingestion.PDFLoader -> Ingestion.Chunker "Text"
    Ingestion.Chunker -> Ingestion.Embedder "Chunks"
    Ingestion.Embedder -> VectorDB "Vectors"
  }

  // Governance
  policy ACLPolicy "Users can only retrieve docs they have access to" category "security" enforcement "required"
  policy CitationsPolicy "All answers must include citations" category "compliance" enforcement "required"
  
  constraints {
    "VectorDB must be isolated per tenant"
    "PII must be redacted before embedding"
  }

  adr ADR001 "Hybrid Search" {
    status "accepted"
    context "Vector search misses exact keyword matches (e.g., project codes)."
    decision "Implement Hybrid Search (Sparse + Dense vectors)."
    consequences "Higher storage cost, better retrieval accuracy."
  }
}`
  },
  {
    name: "Agentic AI",
    code: `architecture "Customer Support AI Agents" {
  overview {
    summary "Agentic automation for e-commerce support: triage, investigation, and resolution"
    audience "Support operations, product, platform engineering"
    scope "Ticket triage, refunds, missing orders, chargebacks, proactive outreach"
    goals ["Reduce resolution time", "Ensure approvals", "Maintain auditability"]
    nonGoals ["General-purpose RPA outside support domain"]
    risks ["Prompt injection", "PII leakage", "Incorrect refunds", "Vendor lock-in"]
  }

  person Customer "Customer" {
    description "Reports issues, requests refunds, tracks orders"
  }
  person SupportAgent "Support Agent" {
    description "Approves sensitive actions, handles escalations"
  }

  system Commerce "E-commerce" {
    description "Order management, catalog, and shipping integrations"

    container ShopFrontend "Web/App Frontend" {
      technology "React/Next.js"
      description "Customer portal for orders and support"
      version "2.3.0"
      tags ["customer","frontend"]
      scale { min 3 max 12 metric "cpu > 70%" }
    }

    container OrderAPI "Order Service" {
      technology "Node.js"
      description "Order management and fulfillment API"
      scale { min 4 max 10 }
      slo { latency { p95 "300ms" p99 "800ms" } }
      component OrdersRead "Order read endpoints"
      component OrdersWrite "Order write endpoints"
    }

    datastore OrdersDB "Orders Database" {
      technology "PostgreSQL"
      description "Canonical source of order and payment state"
    }

    container ShippingAPI "Shipping Integration" {
      technology "Go"
      description "Carrier integrations and tracking"
    }
  }

  system Support "Support Operations" {
    description "Ticketing, CRM, and communications"

    container Ticketing "Ticketing System" {
      technology "Zendesk"
      description "Customer support tickets and workflows"
      component TicketAPI "Ticket CRUD and macros"
    }

    container CRM "CRM" {
      technology "Salesforce"
      description "Customer profiles and entitlements"
      component CRMAPI "Contact, case, entitlement APIs"
    }

    container Email "Transactional Email" {
      technology "SendGrid"
      description "Outbound communications and notifications"
      component EmailAPI "Email send and templates"
    }
  }

  system Agentic "Agentic Platform" {
    description "Planning, tool-use, execution, memory, and governance for support automation"

    container API "Agent Gateway" {
      technology "TypeScript/Node.js"
      description "Ingress, contracts, streaming of results"
      version "1.1.0"
      tags ["api","backend"]
      scale { min 2 max 6 metric "cpu > 65%" }
      slo { latency { p95 "1200ms" p99 "2500ms" } }
      component RequestHandler "Validates and receives tasks"
      component StreamServer "SSE/WebSocket streams"
    }

    container Orchestrator "Task Orchestrator" {
      technology "Go"
      description "Lifecycle control, routing, safety checks, trace context"
      component PlannerRouter "Planner routing and retries"
      component SafetyGate "Pre-execution guardrails"
      component MemoryManager "Retrieval hooks and updates"
      slo { availability { target "99.9%" window "30d" } }
    }

    container Planner "Planning Agent Service" {
      technology "Python"
      description "Generates plans and selects tools"
      scale { min 3 max 10 }
      component ReActPlanner "Reason+Act planning"
      component ToTPlanner "Tree-of-Thought for complex cases"
      component ToolSelector "Tool allowlist and selection"
    }

    container Tools "Support Tools" {
      technology "Mixed"
      description "External tools invoked by agents"
      component WebBrowser "Web search and scraping"
      component DatabaseQuery "Structured data access"
      component CodeRunner "Ephemeral code execution"
      component EmailSender "Templated email dispatch"
    }

    container Executor "Execution Sandbox" {
      technology "Docker"
      description "Network-restricted sandbox for code execution"
      component SandboxRuntime "Runtime with quotas"
      component TraceEmitter "Execution traces"
    }

    container Monitor "Telemetry & Policy" {
      technology "Go"
      description "Metrics, logs, policy evaluation, audit trail"
      component PolicyEngine "Evaluates enforcement rules"
      component MetricsCollector "Latency, error rate, throughput"
      component AuditLog "Immutable audit trail"
    }

    datastore VectorDB "Vector Store" {
      technology "Pinecone/Weaviate"
      description "Long-term memory for retrieval"
    }

    datastore Cache "Context Cache" {
      technology "Redis"
      description "Short-term context, rate limits, throttles"
    }

    queue TaskQueue "Agent Tasks" {
      technology "Kafka/SQS"
      description "Decouples planning from execution"
    }

    API.RequestHandler -> Orchestrator.PlannerRouter "Dispatch task"
    Orchestrator.PlannerRouter -> Planner.ReActPlanner "Request plan"
    Planner.ToolSelector -> Tools.WebBrowser "Search" [ToolCall]
    Planner.ToolSelector -> Tools.DatabaseQuery "Query" [ToolCall]
    Planner.ToolSelector -> Tools.CodeRunner "Prototype" [ToolCall]
    Planner.ToolSelector -> Tools.EmailSender "Notify" [ToolCall]
    Planner.ReActPlanner -> Orchestrator.MemoryManager "Retrieve context"
    Orchestrator.SafetyGate -> Monitor.PolicyEngine "Evaluate" [Guardrail]
    Orchestrator.PlannerRouter -> Executor.SandboxRuntime "Execute step"
    Executor.TraceEmitter -> Monitor.AuditLog "Emit traces"
    Planner.ReActPlanner -> VectorDB "Write embeddings"
    Planner.ReActPlanner -> Cache "Warm context"
    Orchestrator.PlannerRouter -> TaskQueue "Enqueue execution"
    TaskQueue -> Executor.SandboxRuntime "Dequeue"
    Monitor.MetricsCollector -> API.StreamServer "Stream telemetry"
  }

  system External "External Services" {
    description "Third-party APIs"
    container Payments "Stripe"
    container CommerceHost "Shopify"
    container SearchAPI "Search API"
    container Carriers "Carrier APIs"
  }

  Customer -> Commerce.ShopFrontend "Places orders"
  Customer -> Support.Ticketing "Submits ticket"
  SupportAgent -> Support.Ticketing "Works tickets"
  SupportAgent -> Agentic.API "Approves actions" [HumanInLoop]

  Agentic.API -> Support.Ticketing "Create/update tickets"
  Agentic.Tools.DatabaseQuery -> Commerce.OrdersDB "Read order state"
  Agentic.Tools.WebBrowser -> External.SearchAPI "Query"
  Agentic.Tools.WebBrowser -> External.Carriers "Track shipment"
  Agentic.Tools.EmailSender -> Support.Email "Send email"
  Agentic.Tools.DatabaseQuery -> Support.CRM "Fetch customer profile"
  Agentic.Tools.CodeRunner -> Agentic.Executor "Invoke code"
  Agentic.Tools.DatabaseQuery -> External.CommerceHost "Fetch order"
  Agentic.Tools.DatabaseQuery -> External.Payments "Payment lookup"

  requirement R1 functional "Automate triage and investigation for top 5 issue types"
  requirement R2 performance "Reduce average resolution time by 40%"
  requirement R3 security "Mask PII in prompts and logs"
  requirement R4 constraint "Refunds over $100 require human approval"

  adr ADR001 "Adopt ReAct for planning" {
    status "accepted"
    context "Reliable tool selection and iterative refinement needed"
    decision "Use ReAct; fall back to ToT for complex cases"
    consequences "Higher success rate; increased token usage"
  }
  adr ADR002 "Use vector DB for memory" {
    status "accepted"
    context "Require retrieval-augmented planning and case recall"
    decision "Persist embeddings and traces to VectorDB"
    consequences "Better recall; vendor lock-in risk"
  }
  adr ADR003 "Introduce task queue" {
    status "accepted"
    context "Decouple planning from execution and absorb spikes"
    decision "Use Kafka/SQS between orchestrator and executor"
    consequences "Eventual consistency; simplified scaling"
  }
  adr ADR004 "Human-in-the-loop approvals" {
    status "accepted"
    context "Financial risk and customer trust require oversight"
    decision "Require agent approval for sensitive operations"
    consequences "Adds latency; improves correctness and compliance"
  }

  policy RefundPolicy "Refunds over $100 require approval" category "finance" enforcement "required"
  policy PIIProtection "Mask PII in prompts and logs" category "security" enforcement "required"
  policy ToolAccessPolicy "Restrict tools to allowlist" category "governance" enforcement "required"

  constraints {
    "Executor runs in sandbox with default egress blocked"
    "All tool calls pass SafetyGate"
    "Only allowlisted domains for WebBrowser"
    "DatabaseQuery limited to read-only for OrdersDB"
  }

  conventions {
    "Emit JSON traces with step and ticket IDs"
    "Use typed contracts for agent ingress and approvals"
    "Tag sensitive relations with [HumanInLoop]"
  }

  slo {
    latency { p95 "2000ms" p99 "5000ms" window "7d" }
    errorRate { target "0.5%" window "7d" }
    throughput { target "500 tickets/min" window "peak hour" }
    availability { target "99.9%" window "30d" }
  }

  contracts {
    contract SubmitSupportRequest api {
      version "1.0"
      endpoint "/api/support/submit"
      method "POST"
      request {
        ticketId string
        customerId string
        issueType string
        description string
      }
      response {
        ticketId string
        status string
        streamUrl string
      }
      errors ["INVALID_REQUEST","POLICY_VIOLATION","TOOL_FAILURE"]
    }
    contract ApproveAction api {
      version "1.0"
      endpoint "/api/support/approve"
      method "POST"
      request {
        ticketId string
        action string
        amount string
        approverId string
      }
      response {
        approved string
        reason string
      }
      errors ["NOT_ALLOWED","MISSING_APPROVAL","INVALID_ACTION"]
    }
  }

  deployment Prod "Production" {
    node AWS "AWS" {
      node USEast1 "us-east-1" {
        infrastructure LB "Load Balancer"
        containerInstance Agentic.API
        containerInstance Agentic.Orchestrator
        containerInstance Agentic.Planner
        containerInstance Agentic.Executor
        containerInstance Agentic.Monitor
        containerInstance Support.Ticketing
        containerInstance Support.CRM
        containerInstance Support.Email
        containerInstance Commerce.OrderAPI
      }
    }
  }

  scenario TicketTriage "Triage incoming ticket" {
    Customer -> Support.Ticketing "Submit"
    Support.Ticketing -> Agentic.API "Webhook"
    Agentic.API -> Agentic.Orchestrator "Dispatch"
    Agentic.Orchestrator -> Agentic.Planner "Plan"
    Agentic.Planner -> Agentic.Tools.DatabaseQuery "Order lookup" [ToolCall]
    Agentic.Planner -> Agentic.Tools.WebBrowser "Carrier tracking" [ToolCall]
    Agentic.Orchestrator -> Agentic.Executor "Execute macro"
    Agentic.Executor -> Agentic.Monitor "Trace"
    Agentic.API -> Support.Ticketing "Update"
  }

  scenario RefundInvestigation "Process refund request" {
    Support.Ticketing -> Agentic.API "Refund requested"
    Agentic.API -> Agentic.Orchestrator "Dispatch"
    Agentic.Orchestrator -> Agentic.Planner "Plan investigation"
    Agentic.Planner -> Agentic.Tools.DatabaseQuery "Payment lookup" [ToolCall]
    Agentic.Planner -> Agentic.Tools.DatabaseQuery "Order state" [ToolCall]
    Agentic.Orchestrator -> Agentic.Monitor.PolicyEngine "Evaluate policy" [Guardrail]
    SupportAgent -> Agentic.API "Approve refund" [HumanInLoop]
    Agentic.API -> External.Payments "Initiate refund"
    Agentic.API -> Support.Email "Notify customer"
  }

  scenario LostPackage "Handle not received order" {
    Customer -> Support.Ticketing "Report issue"
    Agentic.API -> Agentic.Orchestrator "Dispatch"
    Agentic.Planner -> Agentic.Tools.WebBrowser "Carrier tracking" [ToolCall]
    Agentic.Planner -> Agentic.Tools.EmailSender "Proactive outreach" [ToolCall]
    Agentic.API -> Support.Ticketing "Add tracking info"
  }

  flow TriageFlow "Data flow for triage" {
    Agentic.Planner -> Agentic.Tools.DatabaseQuery "Order lookup"
    Agentic.Tools.DatabaseQuery -> Agentic.Planner "Order details"
    Agentic.Planner -> Agentic.Tools.WebBrowser "Carrier query"
    Agentic.Tools.WebBrowser -> Agentic.Planner "Tracking data"
    Agentic.Planner -> Agentic.Tools.EmailSender "Email"
    Agentic.Tools.EmailSender -> Support.Email "Send"
  }
}`
  },
  {
    name: "E-Commerce Platform",
    code: `architecture "E-Commerce Platform" {
  system ECommerce "E-Commerce Platform" "Comprehensive platform for online retail operations handling browsing, ordering, and payments." {
    metadata {
      capacity "10M+ users, 1M+ requests/day"
      scaling "Horizontal Pod Autoscaling (HPA) on K8s"
      projectedGrowth "2x traffic expected in next 6 months due to holiday season"
      bottlenecks "Database write IOPS on OrderDB during flash sales"
      metrics "Request rate, Error rate, Latency (p95, p99), Saturation"
      logging "Structured JSON logs -> Fluentd -> Elasticsearch (14 days retention)"
      alerting "PagerDuty for P0/P1 (Availability < 99.9%, Error Rate > 1%)"
    }
    container WebApp "Web Application" "React-based storefront for customers to browse and buy products." {
      technology "React"
      component ProductCatalog "Product Catalog"
      component ShoppingCart "Shopping Cart"
      component Checkout "Checkout"
      component UserAuth "User Authentication"
      ProductCatalog -> API "Queries products"
      ShoppingCart -> API "Updates cart"
      Checkout -> API "Calls checkout"
      UserAuth -> API "Uses user authentication"
    }
    container API "REST API" "Core business logic API exposed to frontend and mobile clients." {
      technology "Go"
      component ProductService "Product Service"
      component OrderService "Order Service"
      component PaymentService "Payment Service"
      component AuthService "Authentication Service"
      component NotificationService "Notification Service"
      component InventoryService "Inventory Service"
      ProductService -> ProductDB "Reads products"
      ProductService -> SearchEngine "Queries search index"
      ProductService -> Redis "Reads cache"
      OrderService -> OrderDB "Writes orders"
      OrderService -> InventoryService "Updates inventory"
      InventoryService -> ProductDB "Updates stock"
      AuthService -> UserDB "Reads, Writes users"
      NotificationService -> EmailService.SMTP "Sends emails"
      PaymentService -> PaymentGateway.StripeAPI "Processes payments"
    }
    datastore ProductDB "Product Database"
    datastore OrderDB "Order Database"
    datastore UserDB "User Database"
    queue EventQueue "Event Queue"
    container AnalyticsService "Analytics Service" {
      technology "Python"
    }
    container SearchEngine "Search Engine" {
      technology "Elasticsearch"
    }
    container Redis "Cache" {
      technology "Redis"
    }
  }
  system PaymentGateway "Payment Gateway" "External service for securely processing customer payments." {
    container StripeAPI "Stripe API" {
      technology "Stripe"
    }
  }
  system EmailService "Email Service" {
    container SMTP "SMTP Server" {
      technology "SendGrid"
    }
  }
  system CDN "Content Delivery Network" "Global CDN for caching static assets and reducing latency." {
    container CloudFront "CloudFront" "AWS CloudFront distribution serving static content." {
      technology "AWS"
    }
  }
  person Customer "Customer"
  person Admin "Administrator"
  person SupportAgent "Support Agent"
  Customer -> CDN.CloudFront "Uses Web App"
  CDN.CloudFront -> ECommerce.WebApp "Sends content"
  Customer -> ECommerce.WebApp "Reads products"
  Customer -> ECommerce.WebApp "Creates orders"
  Admin -> ECommerce.API "Updates products"
  Admin -> ECommerce.API "Reads analytics"
  SupportAgent -> ECommerce.API "Reads order data"
  ECommerce.WebApp -> ECommerce.API "Calls API"
  ECommerce.API -> ECommerce.ProductDB "Reads product data"
  ECommerce.API -> ECommerce.OrderDB "Writes order data"
  ECommerce.API -> ECommerce.UserDB "Reads user data"
  ECommerce.API -> ECommerce.UserDB "Writes user data"
  ECommerce.API -> ECommerce.EventQueue "Notifies events"
  ECommerce.API -> PaymentGateway.StripeAPI "Processes payments"
  ECommerce.API -> EmailService.SMTP "Sends emails"
  ECommerce.EventQueue -> ECommerce.AnalyticsService "Sends analytics events"
  ECommerce.API -> ECommerce.SearchEngine "Queries products"
  ECommerce.API -> ECommerce.Redis "Reads, Writes cache"
  Customer -> CDN "Uses Web App"
  CDN.CloudFront -> ECommerce "Sends content"
  Customer -> ECommerce "Reads products"
  Admin -> ECommerce "Updates products"
  SupportAgent -> ECommerce "Reads order data"
  ECommerce.API -> PaymentGateway "Processes payments"
  ECommerce.API -> EmailService "Sends emails"
  NotificationService -> EmailService "Sends emails"
  PaymentService -> PaymentGateway "Processes payments"
  requirement REQ001 functional "Users must be able to browse products by category" {
    tags "ECommerce.WebApp", "ECommerce.API.ProductService"
  }
  requirement REQ012 performance "Product catalog page must load in under 2 seconds" {
    tags "ECommerce.WebApp", "ECommerce.API.ProductService", "ECommerce.ProductDB"
  }
  adr ADR005 "Use React for Frontend" {
    tags "ECommerce.WebApp"
    status "Accepted"
    context "Need interactive UI with real-time updates, large team familiar with React ecosystem."
    decision "Use React with TypeScript for frontend. Use component-based architecture for reusability."
    consequences "Gain: Large ecosystem, developer productivity, component reusability. Trade-off: Bundle size, client-side rendering complexity."
  }
  policy SecurityPolicy "Enforce TLS 1.3 for all external communications" {
    category "security"
    enforcement "required"
    tags "ECommerce.API", "PaymentGateway"
  }
  flow PaymentFlow "Payment processing flow" {
    
    ECommerce.WebApp -> ECommerce.API "Payment request"
    ECommerce.API.PaymentService -> PaymentGateway.StripeAPI "Process payment"
    PaymentGateway.StripeAPI -> ECommerce.API.PaymentService "Payment confirmation"
    ECommerce.API.PaymentService -> ECommerce.OrderDB "Update order status"
    ECommerce.API.NotificationService -> ECommerce.EventQueue "Payment event"
  }
  flow OrderFulfillmentFlow "Order fulfillment flow" {
    ECommerce.WebApp -> ECommerce.API "Order request"
    ECommerce.API.OrderService -> ECommerce.OrderDB "Save order"
    ECommerce.API.OrderService -> ECommerce.API.InventoryService "Check inventory"
    ECommerce.API.InventoryService -> ECommerce.ProductDB "Reserve inventory"
    ECommerce.API.OrderService -> ECommerce.EventQueue "Order event"
    ECommerce.EventQueue -> ECommerce.AnalyticsService "Analytics data"
  }
  metadata {
    team "E-Commerce Platform Team"
    owner "platform-team@example.com"
    version "2.1.0"
    status "production"
    scale "10M+ users, 1M+ requests/day"
    cost "$15k/month operational cost"
    team "E-Commerce Team"
    owner "platform-team@example.com"
    stakeholders ["product-team@example.com", "security-team@example.com"]
    version "1.0.0"
    status "production"
  }
}`
  },
  {
    name: "IoT Platform",
    code: `// Real-world IoT Platform with many devices, gateways, and data processing
// Tests layout with complex hierarchy and many edges

architecture "IoT Platform Architecture" {
  person DeviceOwner "Device Owner"
  person DataAnalyst "Data Analyst"
  person SystemAdmin "System Administrator"
  
  system IoTPlatform "IoT Platform" {
    container DeviceGateway "Device Gateway" "Manages device connections" {
      technology "MQTT Broker"
      component ConnectionManager "Connection Manager"
      component MessageRouter "Message Router"
      component ProtocolAdapter "Protocol Adapter"
      ConnectionManager -> MessageBroker "Routes messages"
      MessageRouter -> DeviceRegistry "Queries devices"
      ProtocolAdapter -> MessageBroker "Publishes messages"
    }
    
    container IngestionService "Ingestion Service" "Processes incoming data" {
      technology "Go"
      component MessageConsumer "Message Consumer"
      component DataValidator "Data Validator"
      component DataNormalizer "Data Normalizer"
      MessageConsumer -> MessageBroker "Consumes messages"
      DataValidator -> DeviceRegistry "Validates devices"
      DataNormalizer -> TimeSeriesDB "Writes data"
    }
    
    container DeviceManagementService "Device Management Service" "Manages device lifecycle" {
      technology "Java, Spring Boot"
      component DeviceAPI "Device API"
      component ProvisioningService "Provisioning Service"
      component FirmwareManager "Firmware Manager"
      DeviceAPI -> DeviceRegistry "Manages devices"
      ProvisioningService -> DeviceRegistry "Registers devices"
      FirmwareManager -> ObjectStorage "Stores firmware"
    }
    
    container AnalyticsService "Analytics Service" "Real-time and batch analytics" {
      technology "Python, Spark"
      component StreamProcessor "Stream Processor"
      component BatchProcessor "Batch Processor"
      component AlertEngine "Alert Engine"
      StreamProcessor -> MessageBroker "Processes streams"
      BatchProcessor -> TimeSeriesDB "Reads historical data"
      AlertEngine -> NotificationService "Sends alerts"
    }
    
    container RuleEngine "Rule Engine" "Executes business rules" {
      technology "Node.js"
      component RuleEvaluator "Rule Evaluator"
      component ActionExecutor "Action Executor"
      RuleEvaluator -> TimeSeriesDB "Queries data"
      ActionExecutor -> DeviceGateway "Sends commands"
    }
    
    container DashboardService "Dashboard Service" "Provides web dashboard" {
      technology "React"
      component DashboardAPI "Dashboard API"
      component VisualizationEngine "Visualization Engine"
      DashboardAPI -> TimeSeriesDB "Queries data"
      VisualizationEngine -> DashboardAPI "Fetches data"
    }
    
    datastore DeviceRegistry "Device Registry" {
      technology "PostgreSQL"
    }
    
    datastore TimeSeriesDB "Time Series Database" {
      technology "InfluxDB"
    }
    
    datastore MetadataDB "Metadata Database" {
      technology "PostgreSQL"
    }
    
    queue MessageBroker "Message Broker" {
      technology "Kafka"
    }
    
    container ObjectStorage "Object Storage" {
      technology "S3"
    }
  }
  
  system NotificationService "Notification Service" {
    container EmailService "Email Service" {
      technology "SendGrid"
    }
    
    container SMSService "SMS Service" {
      technology "Twilio"
    }
    
    container PushService "Push Service" {
      technology "Firebase"
    }
  }
  
  system EdgeGateway "Edge Gateway" "On-premise edge gateway" {
    container EdgeProcessor "Edge Processor" {
      technology "Docker"
    }
  }
  
  // User interactions
  DeviceOwner -> IoTPlatform.DashboardService "Views data"
  DataAnalyst -> IoTPlatform.DashboardService "Analyzes data"
  SystemAdmin -> IoTPlatform.DeviceManagementService "Manages devices"
  
  // Data flow
  EdgeGateway -> IoTPlatform.DeviceGateway "Sends telemetry"
  IoTPlatform.DeviceGateway -> IoTPlatform.IngestionService "Routes data"
  IoTPlatform.IngestionService -> IoTPlatform.TimeSeriesDB "Stores data"
  
  // Device management
  IoTPlatform.DeviceManagementService -> IoTPlatform.DeviceRegistry "Manages registry"
  IoTPlatform.DeviceGateway -> IoTPlatform.DeviceRegistry "Queries devices"
  
  // Analytics flow
  IoTPlatform.AnalyticsService -> IoTPlatform.TimeSeriesDB "Reads data"
  IoTPlatform.DashboardService -> IoTPlatform.TimeSeriesDB "Queries data"
  IoTPlatform.RuleEngine -> IoTPlatform.TimeSeriesDB "Queries data"
  
  // Rule execution
  IoTPlatform.RuleEngine -> IoTPlatform.DeviceGateway "Sends commands"
  IoTPlatform.RuleEngine -> IoTPlatform.NotificationService "Sends notifications"
  
  // Notifications
  IoTPlatform.AnalyticsService -> NotificationService.EmailService "Sends emails"
  IoTPlatform.AnalyticsService -> NotificationService.SMSService "Sends SMS"
  IoTPlatform.AnalyticsService -> NotificationService.PushService "Sends push"
  
  // Storage
  IoTPlatform.DeviceManagementService -> IoTPlatform.ObjectStorage "Stores firmware"
  
  requirement REQ001 functional "Handle 1M+ devices"
  requirement REQ002 performance "Process 100k messages/second"
  requirement REQ003 reliability "99.9% message delivery"
  requirement REQ004 latency "Sub-second rule execution"
}`
  },
  {
    name: "SaaS Platform",
    code: `// Real-world SaaS Platform with multiple systems, microservices, and complex hierarchy
// Tests layout engine with many containers, components, and cross-system relationships

architecture "SaaS Platform Architecture" {
  person Customer "Customer"
  person Admin "Platform Administrator"
  person Developer "Developer"
  
  system SaaSPlatform "SaaS Platform" {
    container WebApp "Web Application" "React-based dashboard for customers" {
      technology "React, TypeScript"
      component Dashboard "Dashboard"
      component Settings "Settings"
      component Billing "Billing"
      component Analytics "Analytics"
      Dashboard -> API "Fetches data"
      Settings -> API "Updates settings"
      Billing -> API "Manages billing"
      Analytics -> API "Fetches analytics"
    }
    
    container API "REST API Gateway" "Main API entry point" {
      technology "Go, Gin"
      component AuthMiddleware "Authentication Middleware"
      component RateLimiter "Rate Limiter"
      component RequestRouter "Request Router"
      component ResponseFormatter "Response Formatter"
      AuthMiddleware -> UserService "Validates tokens"
      RateLimiter -> Redis "Checks limits"
      RequestRouter -> ProjectService "Routes requests"
      RequestRouter -> BillingService "Routes requests"
      ResponseFormatter -> ResponseCache "Caches responses"
    }
    
    container UserService "User Service" "Manages users and authentication" {
      technology "Node.js, Express"
      component AuthAPI "Auth API"
      component ProfileAPI "Profile API"
      component SessionManager "Session Manager"
      AuthAPI -> UserDB "Reads/Writes users"
      ProfileAPI -> UserDB "Updates profiles"
      SessionManager -> Redis "Manages sessions"
    }
    
    container ProjectService "Project Service" "Manages customer projects" {
      technology "Java, Spring Boot"
      component ProjectAPI "Project API"
      component WorkspaceManager "Workspace Manager"
      component CollaborationService "Collaboration Service"
      ProjectAPI -> ProjectDB "Reads/Writes projects"
      WorkspaceManager -> StorageService "Manages storage"
      CollaborationService -> MessageQueue "Publishes events"
    }
    
    container BillingService "Billing Service" "Handles subscriptions and payments" {
      technology "Python, FastAPI"
      component SubscriptionAPI "Subscription API"
      component InvoiceGenerator "Invoice Generator"
      component PaymentProcessor "Payment Processor"
      SubscriptionAPI -> BillingDB "Manages subscriptions"
      InvoiceGenerator -> BillingDB "Generates invoices"
      PaymentProcessor -> PaymentGateway "Processes payments"
    }
    
    container AnalyticsService "Analytics Service" "Processes analytics data" {
      technology "Python, Flask"
      component EventCollector "Event Collector"
      component DataProcessor "Data Processor"
      component ReportGenerator "Report Generator"
      EventCollector -> EventQueue "Consumes events"
      DataProcessor -> AnalyticsDB "Writes analytics"
      ReportGenerator -> AnalyticsDB "Reads analytics"
    }
    
    container StorageService "Storage Service" "Manages file storage" {
      technology "Go"
      component FileAPI "File API"
      component MetadataManager "Metadata Manager"
      FileAPI -> ObjectStorage "Stores files"
      MetadataManager -> MetadataDB "Manages metadata"
    }
    
    datastore UserDB "User Database" {
      technology "PostgreSQL"
    }
    
    datastore ProjectDB "Project Database" {
      technology "PostgreSQL"
    }
    
    datastore BillingDB "Billing Database" {
      technology "PostgreSQL"
    }
    
    datastore AnalyticsDB "Analytics Database" {
      technology "ClickHouse"
    }
    
    datastore MetadataDB "Metadata Database" {
      technology "PostgreSQL"
    }
    
    queue MessageQueue "Message Queue" {
      technology "RabbitMQ"
    }
    
    queue EventQueue "Event Queue" {
      technology "Kafka"
    }
    
    container Redis "Cache & Sessions" {
      technology "Redis"
    }
    
    container ObjectStorage "Object Storage" {
      technology "S3-compatible"
    }
    
    container ResponseCache "Response Cache" {
      technology "Redis"
    }
  }
  
  system PaymentGateway "Payment Gateway" "External payment processing" {
    container StripeAPI "Stripe API" {
      technology "Stripe"
    }
  }
  
  system NotificationService "Notification Service" "External notification service" {
    container EmailAPI "Email API" {
      technology "SendGrid"
    }
    
    container SMSAPI "SMS API" {
      technology "Twilio"
    }
  }
  
  // User interactions
  Customer -> SaaSPlatform.WebApp "Uses platform"
  Admin -> SaaSPlatform.API "Manages platform"
  Developer -> SaaSPlatform.API "Uses API"
  
  // Internal service communication
  SaaSPlatform.WebApp -> SaaSPlatform.API "API calls"
  SaaSPlatform.API -> SaaSPlatform.UserService "User operations"
  SaaSPlatform.API -> SaaSPlatform.ProjectService "Project operations"
  SaaSPlatform.API -> SaaSPlatform.BillingService "Billing operations"
  SaaSPlatform.ProjectService -> SaaSPlatform.StorageService "Storage operations"
  SaaSPlatform.BillingService -> SaaSPlatform.AnalyticsService "Usage analytics"
  
  // Service to database
  SaaSPlatform.UserService -> SaaSPlatform.UserDB "Reads/Writes"
  SaaSPlatform.ProjectService -> SaaSPlatform.ProjectDB "Reads/Writes"
  SaaSPlatform.BillingService -> SaaSPlatform.BillingDB "Reads/Writes"
  SaaSPlatform.AnalyticsService -> SaaSPlatform.AnalyticsDB "Writes analytics"
  SaaSPlatform.StorageService -> SaaSPlatform.MetadataDB "Manages metadata"
  
  // Cache usage
  SaaSPlatform.API -> SaaSPlatform.Redis "Cache checks"
  SaaSPlatform.UserService -> SaaSPlatform.Redis "Session storage"
  SaaSPlatform.API -> SaaSPlatform.ResponseCache "Response caching"
  
  // Queue usage
  SaaSPlatform.ProjectService -> SaaSPlatform.MessageQueue "Publishes events"
  SaaSPlatform.AnalyticsService -> SaaSPlatform.EventQueue "Consumes events"
  
  // External integrations
  SaaSPlatform.BillingService -> PaymentGateway.StripeAPI "Processes payments"
  SaaSPlatform.ProjectService -> NotificationService.EmailAPI "Sends emails"
  SaaSPlatform.ProjectService -> NotificationService.SMSAPI "Sends SMS"
  
  // Storage
  SaaSPlatform.StorageService -> SaaSPlatform.ObjectStorage "Stores files"
  
  requirement REQ001 functional "Support 100k+ customers"
  requirement REQ002 performance "API response time < 200ms (p95)"
  requirement REQ003 availability "99.99% uptime SLA"
  requirement REQ004 scalability "Auto-scale based on load"
}`
  },
  {
    name: "C4 Model",
    code: `architecture "C4 Complete Example" {

  person User "End User"
  system WebApp "Web Application" {
    container API "API Service"
    container DB "Database"
  }

  // New Deployment View
  deployment Prod "Production" {
    node AWS "AWS" {
      node USEast1 "US-East-1" {
         infrastructure LB "Load Balancer"
         containerInstance API
      }
      node RDS "RDS" {
         containerInstance DB
      }
    }
  }

  // Login Flow Relations (dynamic view converted to relations)
  User -> WebApp "Uses"
  User -> WebApp.API "Credentials"
  WebApp.API -> WebApp.DB "Validate User"
}`
  },
  {
    name: "MVP Reference",
    code: `architecture "Full MVP Example" {
  person Customer "Customer"

  system App "Application" {
    container API "HTTP API" {
      tags ["http", "public"]
    }
    datastore DB "PostgreSQL"
    queue MQ "Events"
    container Web "Web App" {
      technology "React"
      component Cart "Cart" {
      }
      component Auth "Auth" {
      }
      Web.Cart -> Web.Auth uses "Uses"
    }
  }

  Customer -> App.API uses "Uses"
  App.API -> App.DB reads "Reads data"
  App.API -> App.DB writes "Writes data"
  App -> App.MQ publishes "Publishes events"

  requirement R1 functional "Persist user data"
  requirement R2 constraint "Expose public API"
  requirement R3 performance "Fast checkout"

  adr ADR001 "Adopt Stripe for payments"
}`
  },
  {
    name: "Systems Thinking",
    code: `architecture "Systems Thinking Example" {
    // ============================================================================
    // 1. PARTS AND RELATIONSHIPS
    // ============================================================================
    // Systems thinking starts with understanding how components connect and interact
    
    person Customer "End User"
    person Admin "System Administrator"
    
    system ECommerce "E-Commerce System" {
        container WebApp "Web Application" {
            technology "React"
        }
        
        container API "API Service" {
            technology "Go"
        }
        
        container Database "PostgreSQL Database" {
            technology "PostgreSQL 14"
        }
    }
    
    // Events show what happens in the system (declared at architecture level via context)
    // Events show what happens in the system (declared at architecture level via context)
    // context Events {
    //     event OrderPlaced {
    //         orderId string
    //         customerId string
    //         total float
    //     }
    //     
    //     event PaymentProcessed {
    //         orderId string
    //         amount float
    //         status string
    //     }
    // }
    
    system PaymentGateway "Third-party Payment Service" {
        metadata {
            tags ["external"]
        }
    }
    
    // Relationships show how parts interact
    Customer -> WebApp "Uses"
    WebApp -> API "Calls"
    API -> Database "Reads/Writes"
    API -> PaymentGateway "Processes payments"
    
    // ============================================================================
    // 2. BOUNDARIES
    // ============================================================================
    // system defines what's inside, person and external define what's outside
    
    // Inside boundary: ECommerce system contains WebApp, API, Database
    // Outside boundary: Customer, Admin, PaymentGateway are external
    
    // ============================================================================
    // 3. FLOWS
    // ============================================================================
    // Flows show how information/data moves through the system
    
    // Data Flow Diagram (DFD) style
    // Data Flow Diagram (DFD) style
    // flow OrderProcess "Order Processing Flow" {
    //     Customer -> WebApp "Submits Order"
    //     WebApp -> API "Sends Order Data"
    //     API -> Database "Saves Order"
    //     API -> PaymentGateway "Charges Payment"
    //     PaymentGateway -> API "Returns Confirmation"
    //     API -> Database "Updates Order Status"
    //     API -> WebApp "Returns Result"
    //     WebApp -> Customer "Shows Confirmation"
    // }
    
    // User Story/Scenario style
    // User Story/Scenario style
    // story Checkout "User Checkout Flow" {
    //     Customer -> "Cart Page" "adds items to cart"
    //     "Cart Page" -> WebApp "clicks checkout"
    //     WebApp -> API "validates cart"
    //     API -> Database "checks inventory"
    //     Database -> API "returns stock status"
    //     API -> PaymentGateway "processes payment"
    //     PaymentGateway -> API "confirms payment"
    //     API -> Database "creates order"
    //     API -> WebApp "returns order confirmation"
    //     WebApp -> Customer "displays success message"
    // }
    
    // Simple relationship flow (unidirectional to avoid cycles)
    Customer -> WebApp "Browses Products"
    WebApp -> API "Fetches Products"
    API -> Database "Queries Products"
    
    // ============================================================================
    // 4. FEEDBACK LOOPS
    // ============================================================================
    // Feedback loops show how actions create reactions
    
    // Simple feedback: User action triggers system response
    // Cycles are valid - this is a natural feedback loop
    Customer -> WebApp "Submits Form"
    WebApp -> API "Validates"
    API -> WebApp "Returns Validation Result"
    WebApp -> Customer "Shows Feedback"
    // The feedback affects customer's next action (feedback loop)
    
    // System feedback: Component A affects Component B, which affects A
    // Cycles are valid - this models event-driven or mutual dependency patterns
    API -> Database "Updates Inventory"
    Database -> API "Notifies Low Stock"
    API -> Admin "Sends Alert"
    Admin -> API "Adjusts Inventory"
    // This creates a feedback loop: API <-> Database <-> Admin
    // Cycles are natural in event-driven architectures and mutual dependencies
    
    // ============================================================================
    // 5. CONTEXT
    // ============================================================================
    // Context shows the environment the system operates in
    
    // Additional persons for context
    person Support "Customer Support"
    
    // Additional external systems show what the system depends on
    system EmailService "Email Notifications" {
        metadata {
            tags ["external"]
        }
    }
    
    system AnalyticsService "Usage Analytics" {
        metadata {
            tags ["external"]
        }
    }
    
    // Relationships show context interactions
    Customer -> ECommerce "Uses"
    Admin -> ECommerce "Manages"
    Support -> ECommerce "Monitors"
    ECommerce -> PaymentGateway "Depends on"
    ECommerce -> EmailService "Sends notifications"
    ECommerce -> AnalyticsService "Tracks usage"
    
    // ============================================================================
    // REQUIREMENTS AND ADRs
    // ============================================================================
    
    requirement R1 functional "System must handle 10k concurrent users"
    requirement R2 constraint "Must use PostgreSQL for data persistence"
    requirement R3 performance "API response time must be < 100ms"
    
    adr ADR001 "Use microservices architecture for scalability"
    adr ADR002 "Use PostgreSQL for strong consistency requirements"
}`
  },
  {
    name: "Sruja Architecture",
    code: `architecture "Sruja Architecture" {
  description "The Sruja Architecture-as-Code Platform"

  person Architect "Software Architect" {
    description "Uses Sruja to design and document systems"
  }

  person Developer "Developer" {
    description "Uses Sruja CLI and VS Code extension for development"
  }

  system Sruja "Sruja Platform" {
    description "Tools for defining, visualizing, and analyzing software architecture"

    container CLI "Sruja CLI" {
      technology "Go (Cobra)"
      description "Command-line interface (cmd/sruja)"

      component Commands "Commands" {
        description "compile, lint, fmt, export, import, tree, diff, explain, score, change"
      }
    }

    container Engine "Core Engine" {
      technology "Go"
      description "Core logic for validation, scoring, and analysis (pkg/engine)"

      component Validation "Validation Engine" {
        technology "Go"
        description "Validates AST against rules (pkg/engine)"
      }

      component Scorer "Scoring Engine" {
        technology "Go"
        description "Calculates architecture health score (pkg/engine/scorer.go)"
      }

      component Resolver "Reference Resolver" {
        technology "Go"
        description "Resolves references and relationships (pkg/engine/resolver.go)"
      }

      Scorer -> Validation "uses results from"
      Validation -> Resolver "validates references"
    }

    container Language "Language Service" {
      technology "Go"
      description "Parser, AST, and language implementation (pkg/language)"

      component Parser "Parser" {
        technology "Go"
        description "Parses .sruja files into AST (pkg/language/parser.go)"
      }

      component AST "AST" {
        technology "Go"
        description "Abstract Syntax Tree definitions (pkg/language/ast.go)"
      }

      component Printer "Printer" {
        technology "Go"
        description "Formats AST back to DSL (pkg/language/printer.go)"
      }

      Parser -> AST "builds"
      Printer -> AST "reads"
    }

    container LSP "Language Server" {
      technology "Go (jsonrpc2)"
      description "LSP implementation for IDE support (pkg/lsp)"

      component Handler "LSP Handler" {
        description "Handles textDocument/didChange, hover, definition, completion, etc."
      }
    }

    container Export "Exporters" {
      technology "Go"
      description "Export functionality (pkg/export)"

      component JSONExporter "JSON Exporter" {
        description "Exports architecture to JSON format (pkg/export/json)"
      }

      component ViewGenerator "View Generator" {
        description "Generates custom views (pkg/export/views)"
      }
    }

    container WASM "WASM Module" {
      technology "Go/WASM"
      description "WebAssembly build of the core engine (cmd/wasm)"
    }

    container VSCode "VS Code Extension" {
      technology "TypeScript"
      description "Editor extension (apps/vscode-extension)"
    }

    container Designer "Sruja Designer" {
      technology "React/Vite"
      description "Interactive designer for testing Sruja code (apps/designer)"
    }

    container Website "Documentation Site" {
      technology "Astro"
      description "Project documentation and guides (apps/website)"
    }

    // Internal Dependencies
    CLI -> Language "parses DSL using"
    CLI -> Engine "validates using"
    CLI -> Export "exports using"
    CLI -> WASM "builds"

    Engine -> Language "uses AST from"
    LSP -> Language "uses parser"
    LSP -> Engine "validates using"

    WASM -> Language "embeds"
    WASM -> Engine "embeds"

    VSCode -> LSP "uses for IDE features"
    VSCode -> WASM "uses for LSP and preview"

    Playground -> WASM "uses for parsing/rendering"
    Website -> Playground "embeds"
  }

  Architect -> Sruja.CLI "runs commands"
  Architect -> Sruja.VSCode "writes DSL"
  Architect -> Sruja.Playground "visualizes architecture"
  Architect -> Sruja.Website "reads docs"

  Developer -> Sruja.CLI "runs lint/compile"
  Developer -> Sruja.VSCode "writes code"

  requirement R1 functional "Must support full C4 model hierarchy"
  requirement R2 performance "Parsing must be < 100ms for typical files"
  requirement R3 portability "Core must compile to WASM"
  requirement R4 functional "Must export to JSON format"

  adr ADR001 "Use Participle for Parsing" {
    status "Accepted"
    context "Need a type-safe, declarative parser for Go."
    decision "Use github.com/alecthomas/participle."
    consequences "Gain: No external build tools needed, type-safe AST. Trade-off: Less flexibility than ANTLR"
  }

  adr ADR002 "Client-Side Rendering for Playground" {
    status "Accepted"
    context "Playground needs to run without a backend server for cost and simplicity"
    decision "Compile Go engine to WASM and run entirely client-side"
    consequences "Gain: Zero backend costs, works offline. Trade-off: Larger bundle size, WASM compatibility considerations"
  }

  adr ADR003 "JSON as Primary Export Format" {
    status "Accepted"
    context "Need a format that can be consumed by various tools and frontend applications"
    decision "Use JSON as the primary export format, with TypeScript exporters in frontend apps for other formats"
    consequences "Gain: Interoperability, easy integration. Trade-off: Frontend apps handle format conversion"
  }

  policy CodeStyle "Go code must be formatted with gofmt" {
    category "quality"
    enforcement "required"
  }

  policy NoCycles "Architecture graph must be acyclic" {
    category "structural"
    enforcement "strict"
  }

  flow AuthoringFlow "Authoring Architecture" {
    Architect -> Sruja.VSCode "Writes DSL"
    Sruja.VSCode -> Sruja.LSP "Validates in real-time"
    Architect -> Sruja.Playground "Opens Preview"
    Sruja.Playground -> Sruja.WASM "Renders diagram"
  }

  flow CICDFlow "CI/CD Verification" {
    Developer -> Sruja.CLI "Pushes code"
    Sruja.CLI -> Sruja.CLI "Runs lint"
    Sruja.CLI -> Sruja.CLI "Runs export json"
    Sruja.CLI -> Sruja.CLI "Generates artifacts"
  }

  metadata {
    version "0.1.0"
    owner "Sruja Team"
    repo "github.com/sruja-ai/sruja"
  }
}`
  },
  {
    name: "Course: E-Commerce",
    code: `architecture "E-Commerce System" {

    // Module 2: Context
    person Customer "Online Customer" {
        description "Browses and buys products."
    }

    system PaymentGateway "Stripe"
    system EmailService "SendGrid"

    system ECommerce "E-Commerce Platform" {
        description "Allows customers to buy products online."

        // Module 3: Containers
        container WebApp "Storefront" {
            technology "React"
            tags ["frontend"]
            
            // Module 7: Properties & Styles
            properties {
                "framework" "Next.js"
                "hosting" "Vercel"
            }
            style {
                shape "browser"
            }
        }

        container API "Product API" {
            technology "Go"
            tags ["backend"]

            // Module 4: Components
            component ProductController "Product Controller" {
                technology "Gin"
            }
            component ProductService "Product Service"
            component ProductRepo "Product Repository"

            ProductController -> ProductService "Uses"
            ProductService -> ProductRepo "Uses"
        }

        datastore DB "Product Database" {
            technology "PostgreSQL"
            properties {
                "version" "15"
                "storage" "100GB"
            }
        }

        queue Events "Order Events" {
            technology "Kafka"
        }

        // Internal Relations
        WebApp -> API "Fetches Data"
        API -> DB "Reads/Writes"
        API -> Events "Publishes"
    }

    // Context Relations
    Customer -> WebApp "Browses"
    API -> PaymentGateway "Process Payments"
    API -> EmailService "Send Emails"

    // Module 5: Scenarios
    scenario CheckoutFlow "Checkout Flow" "User checkout process" {
        Customer -> ECommerce.WebApp "Clicks Checkout"
        ECommerce.WebApp -> ECommerce.API "POST /checkout"
        ECommerce.API -> ECommerce.DB "Save Order"
        ECommerce.API -> PaymentGateway "Charge Card"
        ECommerce.API -> ECommerce.Events "Order Created"
    }

    // Module 6: Deployment
    // Module 6: Deployment
    deployment Production "Production" {
        node AWS "AWS" {
            node USEast1 "US-East-1" {
                node EKS "EKS" {
                    containerInstance API
                    containerInstance WebApp
                }
                node RDS "RDS" {
                    containerInstance DB
                }
            }
        }
    }

    // Module 7: NFRs
    requirement R1 performance "API latency < 100ms"
    adr ADR001 "Use Microservices"
}`
  }
];

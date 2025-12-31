---
title: "Lesson 1: Interview Question - Design an E-Commerce Platform (Microservices)"
weight: 1
summary: "Master microservices questions by designing Amazon-style platforms."
---

# Lesson 1: Interview Question - Design an E-Commerce Platform (Microservices)

## The Interview Question

**"Design an e-commerce platform like Amazon that can handle millions of users and products. Use a microservices architecture."**

This is one of the **most common system design interview questions**. It tests:

- System decomposition into microservices
- Service boundaries and responsibilities
- Inter-service communication
- Data consistency across services

## Step 1: Clarify Requirements

**You should ask:**

- "What are the core features? Shopping cart, checkout, recommendations?"
- "What's the scale? Users, products, orders per day?"
- "What about inventory? Real-time stock management?"
- "Payment processing? Do we integrate with payment gateways?"

**Interviewer's typical answer:**

- "Core features: Product catalog, shopping cart, checkout, order management, user accounts"
- "Scale: 100M users, 1B products, 10M orders/day"
- "Real-time inventory tracking required"
- "Integrate with payment gateways like Stripe"

## Step 2: Identify Microservices

**Key insight**: Break down by **business domain**, not technical layers.

**You should identify:**

1. **User Service** - Authentication, profiles
2. **Product Service** - Catalog, search, recommendations
3. **Cart Service** - Shopping cart management
4. **Order Service** - Order processing, tracking
5. **Payment Service** - Payment processing
6. **Inventory Service** - Stock management
7. **Notification Service** - Emails, SMS

## Step 3: Model with Sruja (Separate Systems)

Model each microservice as a **separate system** within the architecture. This clearly shows service boundaries.

```sruja
element person
element system
element container
element component
element datastore
element queue

Customer = person "Online Customer"

// Each microservice is a separate system
UserService = system "User Management" {
AuthAPI = container "Authentication API" {
  technology "Go, gRPC"
}

ProfileAPI = container "Profile API" {
  technology "Go, gRPC"
}

UserDB = datastore "User Database" {
  technology "PostgreSQL"
}
}

ProductService = system "Product Catalog" {
ProductAPI = container "Product API" {
  technology "Java, Spring Boot"
}

SearchAPI = container "Search API" {
  technology "Elasticsearch"
}

RecommendationAPI = container "Recommendation API" {
  technology "Python, ML"
}

ProductDB = datastore "Product Database" {
  technology "PostgreSQL"
}

SearchIndex = datastore "Search Index" {
  technology "Elasticsearch"
}
}

CartService = system "Shopping Cart" {
CartAPI = container "Cart API" {
  technology "Node.js, Express"
}

CartDB = datastore "Cart Database" {
  technology "Redis"
  description "In-memory cache for fast cart operations"
}
}

OrderService = system "Order Management" {
OrderAPI = container "Order API" {
  technology "Node.js, Express"
}

OrderProcessor = container "Order Processor" {
  technology "Node.js"
}

OrderDB = datastore "Order Database" {
  technology "PostgreSQL"
}

OrderQueue = queue "Order Queue" {
  technology "Kafka"
}
}

PaymentService = system "Payment Processing" {
PaymentAPI = container "Payment API" {
  technology "Go, gRPC"
}

PaymentDB = datastore "Payment Database" {
  technology "PostgreSQL"
}
}

InventoryService = system "Inventory Management" {
InventoryAPI = container "Inventory API" {
  technology "Java, Spring Boot"
}

InventoryDB = datastore "Inventory Database" {
  technology "PostgreSQL"
}
}

NotificationService = system "Notifications" {
NotificationAPI = container "Notification API" {
  technology "Python, FastAPI"
}

EmailQueue = queue "Email Queue" {
  technology "RabbitMQ"
}

SMSQueue = queue "SMS Queue" {
  technology "RabbitMQ"
}
}

// API Gateway - single entry point
ECommerceApp = system "E-Commerce Application" {
WebApp = container "Web Application" {
  technology "React, Next.js"
}

APIGateway = container "API Gateway" {
  technology "Kong, Nginx"
  description "Routes requests to appropriate microservices"
}
}

Stripe = system "Stripe Gateway" {
tags ["external"]
}

PayPal = system "PayPal Gateway" {
tags ["external"]
}

// User flow
Customer -> ECommerceApp.WebApp "Browses products"
ECommerceApp.WebApp -> ECommerceApp.APIGateway "Makes requests"
ECommerceApp.APIGateway -> UserService.AuthAPI "Authenticates"
ECommerceApp.APIGateway -> ProductService.ProductAPI "Fetches products"
ECommerceApp.APIGateway -> ProductService.SearchAPI "Searches products"
ECommerceApp.APIGateway -> ProductService.RecommendationAPI "Gets recommendations"

// Cart flow
ECommerceApp.APIGateway -> CartService.CartAPI "Manages cart"
CartService.CartAPI -> CartService.CartDB "Stores cart"

// Order flow
ECommerceApp.APIGateway -> OrderService.OrderAPI "Creates order"
OrderService.OrderAPI -> InventoryService.InventoryAPI "Checks stock"
OrderService.OrderAPI -> PaymentService.PaymentAPI "Processes payment"
OrderService.OrderAPI -> UserService.ProfileAPI "Gets user info"
OrderService.OrderAPI -> OrderService.OrderQueue "Enqueues for processing"
OrderService.OrderProcessor -> OrderService.OrderQueue "Processes orders"
OrderService.OrderProcessor -> NotificationService.NotificationAPI "Sends confirmation"

// Payment flow
PaymentService.PaymentAPI -> PaymentService.PaymentDB "Stores transaction"
PaymentService.PaymentAPI -> Stripe "Processes cards"
PaymentService.PaymentAPI -> PayPal "Processes PayPal"

// Notification flow
NotificationService.NotificationAPI -> NotificationService.EmailQueue "Sends emails"
NotificationService.NotificationAPI -> NotificationService.SMSQueue "Sends SMS"

view index {
include *
}
```

## What Interviewers Look For

### ✅ Good Answer (What You Just Did)

1. **Clear service boundaries** - Each service is a separate system
2. **Single responsibility** - Each service has one clear purpose
3. **Identified communication patterns** - API calls, queues, events
4. **Addressed data ownership** - Each service owns its database
5. **Explained trade-offs** - Why microservices vs monolith

### ❌ Bad Answer (Common Mistakes)

1. Services too granular (one service per function)
2. Services too coarse (monolith split incorrectly)
3. Not showing service boundaries clearly
4. Ignoring data consistency challenges
5. No API gateway or service mesh

## Key Points to Mention in Interview

### 1. Service Decomposition Strategy

**Say**: "I decompose by business domain, not technical layers. Each service owns its data and has clear boundaries. For example:

- User Service owns user data and authentication
- Product Service owns product catalog and search
- Order Service owns order lifecycle
- Each service is a separate system in the architecture"

### 2. Inter-Service Communication

**Say**: "Services communicate via:

- **Synchronous**: REST/gRPC for real-time operations (checkout, cart)
- **Asynchronous**: Message queues for eventual consistency (order processing, notifications)
- **API Gateway**: Single entry point, handles routing, auth, rate limiting"

### 3. Data Consistency

**Say**: "Each service owns its database (database per service pattern). For cross-service operations:

- **Saga pattern**: For distributed transactions (order → payment → inventory)
- **Eventual consistency**: Acceptable for non-critical paths (notifications)
- **Strong consistency**: Only within a service (cart operations)"

### 4. API Gateway Pattern

**Say**: "API Gateway provides:

- **Single entry point** for all client requests
- **Request routing** to appropriate microservices
- **Authentication/authorization** - validate tokens once
- **Rate limiting** and throttling
- **Load balancing** across service instances"

## Interview Practice: Add More Services

**Interviewer might ask**: "What about recommendations and analytics?"

Add them to your design (extending the main architecture):

```sruja
element person
element system
element container
element component
element datastore
element queue

Customer = person "Online Customer"

// Existing services (UserService, ProductService, OrderService, etc. from main design)
ProductService = system "Product Catalog" {
ProductAPI = container "Product API" {
  technology "Java, Spring Boot"
}
}

OrderService = system "Order Management" {
OrderAPI = container "Order API" {
  technology "Node.js, Express"
}
}

ECommerceApp = system "E-Commerce Application" {
APIGateway = container "API Gateway" {
  technology "Kong, Nginx"
}
}

// Additional services
RecommendationService = system "Recommendations" {
RecommendationAPI = container "Recommendation API" {
  technology "Python, ML"
}

UserBehaviorDB = datastore "User Behavior Database" {
  technology "MongoDB"
  description "Stores user clicks, views, purchases for ML"
}
}

AnalyticsService = system "Analytics" {
AnalyticsAPI = container "Analytics API" {
  technology "Go"
}

AnalyticsDB = datastore "Analytics Database" {
  technology "ClickHouse"
  description "Time-series data for analytics"
}
}

// Show how services interact
ECommerceApp.APIGateway -> ProductService.ProductAPI "Gets products"
ECommerceApp.APIGateway -> RecommendationService.RecommendationAPI "Gets recommendations"
OrderService.OrderAPI -> AnalyticsService.AnalyticsAPI "Tracks order events"

view index {
include *
}
```

## Common Follow-Up Questions

Be prepared for:

1. **"How do you handle failures?"**
   - Answer: "Circuit breakers prevent cascading failures. Retries with exponential backoff. Fallbacks (show cached data if service down). If payment service is down, queue the order for later processing."

2. **"How do you ensure data consistency?"**
   - Answer: "Saga pattern for distributed transactions. Each step can be compensated if later steps fail. For example, if payment fails after inventory is reserved, we release the inventory (compensating transaction)."

3. **"How do you handle service versioning?"**
   - Answer: "API versioning in URLs (/v1/, /v2/). Deploy new versions alongside old ones. Gradually migrate traffic. Deprecate old versions after migration."

4. **"How do you monitor microservices?"**
   - Answer: "Distributed tracing (Jaeger, Zipkin) to track requests across services. Centralized logging (ELK stack). Metrics (Prometheus) per service. Health checks for each service."

5. **"How do you handle service discovery?"**
   - Answer: "Service registry (Consul, Eureka) or DNS-based discovery. API Gateway can handle routing. Service mesh (Istio) for advanced features like load balancing, retries."

## Exercise: Practice This Question

Design an e-commerce platform and be ready to explain:

1. How you decomposed into services (why these services?)
2. How services communicate (sync vs async)
3. How you handle data consistency
4. How you handle failures
5. Your scaling strategy for each service

**Practice tip**: Time yourself (45-50 minutes). Draw the architecture, then model it with Sruja. Explain your decisions out loud as if in an interview.

## Key Takeaways for Interviews

1. **Decompose by business domain** - Not technical layers
2. **Each service is a separate system** - Clear boundaries in Sruja
3. **Each service owns its data** - Database per service
4. **Use API Gateway** - Single entry point
5. **Mix sync and async** - REST for real-time, queues for async
6. **Address failures** - Circuit breakers, retries, fallbacks
7. **Show with separate systems** - Clear service boundaries in architecture

## Next Steps

You've learned how to design microservices architectures. In the next module, we'll cover governance and policies - important for senior/staff level interviews!

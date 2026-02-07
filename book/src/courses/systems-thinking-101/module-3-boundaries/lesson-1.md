# Lesson 1: Understanding Boundaries

## Learning Goals

- Understand what boundaries are in software architecture
- Recognize different types of boundaries
- Learn why boundaries matter

## What Are Boundaries?

A boundary is the line that separates what's **inside** your system (what you build, own, and maintain) from what's **outside** (the environment, external dependencies, and stakeholders).

```
┌─────────────────────────────────────┐
│          EXTERNAL                   │
│  (Environment, Dependencies)        │
│                                     │
│    ┌───────────────────────────┐    │
│    │     INTERNAL             │    │
│    │   (Your System)          │    │
│    │                           │    │
│    │  [Components]             │    │
│    │                           │    │
│    └───────────────────────────┘    │
│             ↑                         │
│     Boundary Line                    │
└─────────────────────────────────────┘
```

## Why Boundaries Matter

### 1. Clear Ownership

Who's responsible for what?

```sruja
// Inside: Your team owns this
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API Service"
}

// Outside: Another team/service owns this
PaymentGateway = system "Payment Gateway"
```

### 2. Risk Management

What external risks exist?

```sruja
// External dependency = external risk
Shop.API -> PaymentGateway "Process payment"

// If PaymentGateway is down, what happens?
// What's your fallback? SLA?
```

### 3. Testing Scope

What needs integration tests vs. unit tests?

```sruja
// Internal: Unit tests sufficient
Shop.WebApp -> Shop.API

// External: Integration tests needed
Shop.API -> PaymentGateway
```

### 4. Security

What needs protection?

```sruja
// Inside boundary: Apply security controls
Shop.WebApp -> Shop.API

// Crossing boundary: Validate, authenticate
Shop.API -> PaymentGateway
```

## Types of Boundaries

### 1. System Boundary

Your main application vs. the world:

```sruja
// Inside
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Outside
Customer = person "Customer"
```

### 2. Team Boundary

What your team owns vs. what other teams own:

```sruja
// Your team's system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Another team's system
Analytics = system "Analytics Platform"
```

### 3. Organizational Boundary

Internal vs. external organizations:

```sruja
// Your company's system
Shop = system "Shop"

// External vendor
Stripe = system "Stripe" {
  metadata {
    tags ["external", "vendor"]
  }
}
```

### 4. Deployment Boundary

What's deployed together:

```sruja
// Same deployment
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Separate deployment
Database = system "Database Cluster"
```

### 5. Trust Boundary

Security and trust levels:

```sruja
// Trusted: Internal network
InternalAPI = container "Internal API"

// Untrusted: Public internet
PublicAPI = container "Public API"
```

## Boundary Examples

### Example 1: E-Commerce Platform

```sruja
// ┌──────────── EXTERNAL ────────────┐
// │                                  │
// │   Customer (person)               │
// │   Payment Gateway (system)        │
// │   Email Service (system)          │
// │                                  │
// │   ┌────── INTERNAL ─────────┐    │
// │   │ Shop (system)           │    │
// │   │   WebApp (container)    │    │
// │   │   API (container)       │    │
// │   │   Database (database)   │    │
// │   └─────────────────────────┘    │
// │                                  │
// └──────────────────────────────────┘

Customer -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> PaymentGateway "Process payment"
```

### Example 2: Microservices Architecture

```sruja
// Internal boundaries (within organization)
OrderService = system "Order Service"
PaymentService = system "Payment Service"
InventoryService = system "Inventory Service"

OrderService -> PaymentService "Request payment"
PaymentService -> OrderService "Payment result"
OrderService -> InventoryService "Reserve items"
```

## Boundary Anti-Patterns

### Anti-Pattern 1: No Clear Boundary

```sruja
// Bad: Everything looks internal
Customer = person "Customer"
Shop = system "Shop"
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"

// Without tags, it's unclear what's external
```

**Solution**: Use metadata tags to mark external systems.

### Anti-Pattern 2: Everything External

```sruja
// Bad: Everything marked external, no ownership
Shop = system "Shop" {
  tags ["external"]
}
WebApp = container "Web App" {
  tags ["external"]
}
```

**Solution**: Mark only truly external systems.

### Anti-Pattern 3: Too Many Boundaries

```sruja
// Bad: Overly fragmented, hard to understand
System1 = system "System 1"
System2 = system "System 2"
System3 = system "System 3"
// ... many small systems
```

**Solution**: Group related functionality.

## Defining Boundaries in Sruja

### Use `system` for Main Boundary

```sruja
Shop = system "Shop" {
  // Internal containers
  WebApp = container "Web App"
  API = container "API"
}
```

### Use Metadata for External Systems

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
    owner "Third-party"
  }
}
```

### Use `person` for External Actors

```sruja
// Users are outside the system boundary
Customer = person "Customer"
Administrator = person "Administrator"
```

## Exercise

Identify the boundaries in this scenario:

> "A hospital scheduling system allows patients to book appointments, doctors to manage their schedules, and administrators to oversee operations. The system integrates with an external insurance API for coverage verification and sends SMS notifications through Twilio. Patient data is stored in the hospital's database."

**Identify:**

1. Internal system: ******\_******
2. External systems: ******\_******
3. External actors: ******\_******
4. Boundary crossings: ******\_******

## Key Takeaways

1. **Boundaries define ownership**: What you build vs. what you depend on
2. **Multiple boundary types**: System, team, organization, deployment, trust
3. **Mark external systems**: Use metadata tags
4. **Document crossings**: Show how boundaries are crossed
5. **Avoid anti-patterns**: Clear but not over-fragmented boundaries

## Next Lesson

In [Lesson 2](lesson-2.md), you'll learn how to differentiate and mark internal vs. external components.

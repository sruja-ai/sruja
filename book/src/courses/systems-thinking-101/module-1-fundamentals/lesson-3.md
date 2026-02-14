---
title: "Lesson 3: Systems in Software Architecture"
weight: 3
summary: "Every application has layers beyond code—people, dependencies, processes, data."
time: "2 minutes"
---

# Lesson 3: Every App is a System of Systems

## Learning Goals

By the end of this lesson, you'll be able to:
- Identify the layers that make up a software system
- Recognize when you're ignoring dependencies or organizational factors
- Understand how Conway's Law affects your architecture
- Model systems holistically in Sruja

## Seeing Layers: Every App is a System of Systems

When you design software, what do you think about first? The frontend framework? The backend API? The database schema?

These are important parts. But they're not the whole system. Every application exists in a context—users who interact with it, dependencies it relies on, processes that keep it running, and data flowing through it.

Here's what happens when you ignore these layers: You design a perfect API, but your team doesn't know how to deploy it reliably. You optimize database queries, but external payment gateway has rate limits you didn't consider. You build features quickly, but organizational processes create technical debt that slows you down.

Seeing systems holistically—seeing all the layers—is what prevents these problems.

## The Layers of a Software System

Every application is built from multiple interconnected layers. Let's visualize them:

```
┌─────────────────────────────────────┐
│  People: Users, Developers, Ops      │  ← Stakeholders
├─────────────────────────────────────┤
│  Dependencies: APIs, Libraries       │  ← External Systems
├─────────────────────────────────────┤
│  Processes: Dev, Deploy, Monitor     │  ← Operational Systems
├─────────────────────────────────────┤
│  Data: State, Transactions, Logs     │  ← Information Systems
├─────────────────────────────────────┤
│  Application: UI, Logic, Storage     │  ← Your System
└─────────────────────────────────────┘
```

## Why These Layers Matter

Let's talk about why ignoring these layers causes problems. Here's what happens at each layer:

### Dependencies Can Fail

When you depend on external systems (payment gateways, APIs, libraries), those dependencies can fail. If you don't account for this in your architecture, a single external failure can bring down your entire application.

### Teams Shape Architecture (Conway's Law)

Ever heard of Conway's Law? It states that "organizations which design systems are constrained to produce designs which are copies of the communication structures of these organizations."

Translation: How your team communicates and is organized will directly influence your architecture. If your team is split into silos, you'll likely build tightly coupled systems. If your team has clear cross-team communication, you'll naturally build more modular, integrated systems.

### Operational Processes Determine Reliability

Your architecture design matters, but so do your processes. How do you deploy? How do you monitor? How do you handle incidents?

You might design the most resilient architecture possible, but if your deployment process is fragile (manual steps, no testing, no rollbacks), your system will be fragile too.

### Data Systems Create Integration Challenges

Data doesn't just sit in one place anymore. It flows between databases, caches, analytics, logs, and external systems. This movement creates integration challenges and consistency requirements.

If you don't consider these data flows, you'll hit problems: inconsistent data across services, race conditions, synchronization issues.

## Common Architecture Layers

Now that we understand why layers matter, let's look at common layers you'll encounter in real software systems.

### Application Layer: What You Build

This is what most developers think about first—the code and logic. It includes:

- **User interfaces:** Web applications, mobile apps, CLI tools
- **Business logic:** APIs, services, domain models
- **Data storage:** Databases, caches, file systems

This is the visible part of your system—what users interact with and what you spend most of your time building.

### Infrastructure Layer: What Runs Your Code

Your application doesn't run in a vacuum. It needs infrastructure:

- **Compute:** Servers, containers, cloud instances
- **Network:** Load balancers, CDNs, firewalls, VPNs
- **Monitoring:** Logs, metrics, alerts, dashboards

Infrastructure determines reliability, scalability, and performance. You might have perfect code, but if your infrastructure can't handle load or lacks monitoring, you won't know when things fail.

### Organizational Layer: Who Builds It

This layer often gets overlooked, but it's critical. It includes:

- **Teams and responsibilities:** Who owns what? How are decisions made?
- **Development workflows:** Code reviews, testing, CI/CD pipelines
- **Release processes:** How do features go from idea to production?

As Conway's Law states, how your organization is structured will influence your architecture. If your team is organized by technical skill (backend team, frontend team, database team), you'll likely build systems that mirror that structure—sometimes creating unnecessary boundaries.

## Modeling Systems Holistically in Sruja

Let's see how this looks in practice with a real e-commerce system.

### Example: E-Commerce Platform

```sruja
import { * } from 'sruja.ai/stdlib'

// People (stakeholders)
Customer = person "Customer"
Admin = person "Admin"

// External Systems (dependencies)
PaymentGateway = system "Payment Gateway"
EmailService = system "Email Service"

// Your System
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
  }
  API = container "API Service" {
    technology "Node.js"
  }
  DB = database "PostgreSQL"
}

// Relationships show system interactions
Customer -> ECommerce.WebApp "Browses products"
ECommerce.WebApp -> ECommerce.API "Fetches data"
ECommerce.API -> ECommerce.DB "Queries"
ECommerce.API -> PaymentGateway "Process payment"
PaymentGateway -> EmailService "Send receipt"
```

### Analyzing This Model

Notice how this shows multiple layers:

**People layer:** `Customer` and `Admin` are stakeholders who use and manage the system.

**Dependencies layer:** `PaymentGateway` and `EmailService` are external systems we depend on. If the payment gateway is down, our entire order flow breaks.

**Application layer:** `WebApp`, `API`, and `DB` are containers within our system. This is what we build and control.

**Relationships show dependencies:** The arrows between components show data flow and dependencies. `API -> PaymentGateway` is a critical dependency. If that external system is slow or fails, we need to handle it gracefully.

### What This Reveals

When you model systems holistically like this, you start seeing:

- Single points of failure (like `PaymentGateway`)
- Team ownership boundaries (what's in our system vs. external)
- Data flow paths (how do orders flow through the system?)
- Integration points (where do we cross system boundaries?)

This big-picture view helps you design more resilient systems.

## What to Remember

The key insight from this lesson is simple but powerful: Never design in isolation.

Every application exists in a context—people who use it, dependencies it relies on, processes that keep it running, and data flowing through it. When you ignore these layers, you build fragile systems that work in theory but fail in practice.

Think of it this way: A car designer who only thinks about the engine and drivetrain, but ignores the fuel system, cooling system, and driver experience, will create a car that can't run reliably.

The same applies to software. When you see systems holistically—all the layers together—you design better systems. You identify dependencies before they break, you consider team structure before it constrains you, and you plan for failure before it happens.

This is what systems thinking in architecture looks like: Not just code, but code in context.

## Check Your Understanding

Let's see if you're seeing systems holistically now.

### Quick Check

**1. Your team is organized by technical role: backend team, frontend team, database team. You're designing a new feature. What's likely to happen to your architecture, based on Conway's Law?**

[ ] A. You'll build a perfectly modular system because roles are clear
[ ] B. You'll build systems with boundaries that mirror your team structure
[ ] C. Team organization won't affect your architecture at all
[ ] D. You'll need to reorganize teams to match a better architecture

**2. You're designing an e-commerce system and decide to use an external payment gateway API. What should you consider, based on this lesson?**

[ ] A. Nothing—it's just another API call
[ ] B. The gateway might fail or have rate limits, which could break your entire order flow
[ ] C. Always build your own payment system instead of depending on external services
[ ] D. Only worry about the API documentation and authentication

---

### Answers & Discussion

**1. B. You'll build systems with boundaries that mirror your team structure** – Conway's Law states that systems mirror the communication structures of the organizations that build them. If your teams are organized by technical role (backend, frontend, database), your architecture will likely have similar boundaries, creating unnecessary coupling and communication challenges.

**2. B. The gateway might fail or have rate limits, which could break your entire order flow** – When you depend on external systems, those dependencies become single points of failure. You need to consider what happens if the gateway is down, has rate limits, or has performance issues. Do you have fallbacks? Retry logic? Degraded service modes? Ignoring these layers leads to fragile systems.

## What's Next

Now that you understand every application is a system of systems with multiple layers, let's dive into the specific components that make up these systems. In the next lesson, we'll explore **Parts & Relationships**—how to identify components and model their interactions.

This is where systems thinking gets practical: You'll learn to model real systems in Sruja with precision and clarity.
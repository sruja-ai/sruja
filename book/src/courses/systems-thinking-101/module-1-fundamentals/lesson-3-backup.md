---
title: "Lesson 3: Systems in Software Architecture"
weight: 3
summary: "Every software system is a system of systems - understand the layers and dependencies."
time: "2 minutes"
---

# Lesson 3: Systems in Software Architecture

## Learning Goal
Recognize that every software system is a system of systems with multiple layers. This awareness helps you design systems that are resilient, maintainable, and aligned with their environment.

## Software Systems Are Systems of Systems

Every application is built from multiple interconnected systems:

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

**Why this matters:**
- Dependencies can fail and bring down your system
- Teams have communication patterns that affect architecture (Conway's Law)
- Operational processes determine deployment speed and reliability
- Data systems create integration challenges and consistency requirements
- Ignoring any layer leads to fragile, brittle architectures

## Common Architecture Layers

**Application Layer:**
- User interfaces (Web, Mobile, CLI)
- Business logic (APIs, Services)
- Data storage (Databases, Caches)

**Infrastructure Layer:**
- Compute (Servers, Containers)
- Network (Load balancers, CDNs)
- Monitoring (Logs, Metrics, Alerts)

**Organizational Layer:**
- Teams and responsibilities
- Development workflows
- Release processes

## Sruja Example: E-Commerce System

```sruja
import { * } from 'sruja.ai/stdlib'

// People
Customer = person "Customer"
Admin = person "Admin"

// External Systems
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

## Key Takeaway
Never design in isolation. Consider **all the systems** your application depends on and interacts with.

## Quiz: Test Your Knowledge

**Question 1:** What does it mean that "every software system is a system of systems"?

- [ ] a) Every system contains multiple databases
- [ ] b) Every application is built from multiple interconnected layers: people, dependencies, processes, data, and the application itself
- [ ] c) Every system needs to be deployed on multiple servers
- [ ] d) Every system requires microservices architecture

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> Every application is built from multiple interconnected layers: people (stakeholders), dependencies (external APIs), processes (dev, deploy, monitor), data (state, transactions), and the application itself (UI, logic, storage).
  </div>
</div>

---

**Question 2:** Which of the following is NOT a common architecture layer?

- [ ] a) Application Layer (UI, logic, storage)
- [ ] b) Infrastructure Layer (compute, network, monitoring)
- [ ] c) Organizational Layer (teams, workflows, release processes)
- [ ] d) All of these ARE common architecture layers

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> All of these are common architecture layers: Application Layer (UI, logic, storage), Infrastructure Layer (compute, network, monitoring), and Organizational Layer (teams, workflows, release processes).
  </div>
</div>

---

**Question 3:** In the E-Commerce Sruja example, what represents the "external systems"?

- [ ] a) WebApp, API, DB
- [ ] b) Customer, Admin
- [ ] c) PaymentGateway, EmailService
- [ ] d) The entire E-Commerce platform

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> The external systems are `PaymentGateway` and `EmailService` - these are systems outside your platform that you depend on.
  </div>
</div>

---

**Question 4:** [REMOVED - Covered in Question 1 and external systems concept]

- [ ] a) Because it determines the programming language to use
- [ ] b) Because organizational structures directly affect how systems are built, deployed, and maintained
- [ ] c) Because it's only relevant for large enterprise systems
- [ ] d) Because it has no impact on software architecture

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> Because organizational structures (teams, responsibilities, development workflows) directly affect how systems are built, deployed, and maintained. Conway's Law states that systems tend to mirror the communication structures of the teams that build them.
  </div>
</div>

---

**Question 5:** [REMOVED - Too specific, covered in Question 3]

- [ ] a) The WebApp sends requests directly to the Payment Gateway
- [ ] b) The Database stores payment information locally
- [ ] c) `ECommerce.API -> PaymentGateway "Process payment"` - the API service sends a payment processing request
- [ ] d) The customer sends credit card details directly to the Payment Gateway

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> `ECommerce.API -> PaymentGateway "Process payment"` - the API service sends a payment processing request to the external payment gateway system.
  </div>
</div>
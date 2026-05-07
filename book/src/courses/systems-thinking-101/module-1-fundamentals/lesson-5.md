---
title: "Lesson 5: Boundaries"
weight: 5
summary: "Define what's inside your system vs. what's outside - understanding scope and context."
time: "2 minutes"
---

# Lesson 5: Boundaries

## Learning Goal
Learn to clearly define what's inside your system (what you build) and what's outside (what you depend on).

## What Are Boundaries?

**Boundaries** define the scope of your system:
- **Inside:** Components you build and control
- **Outside:** External systems, dependencies, and stakeholders

## Why Boundaries Matter

Boundaries clarify scope (what you own vs. what you depend on), guide integration strategies, define security controls, and determine testing scope.

## Practical Example

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// OUTSIDE: Stakeholders and dependencies
Customer = person "Customer"
PaymentGateway = system "Payment Service" { metadata { tags ["external"] } }

// INSIDE: Your system
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
}

// Cross-boundary (external) vs internal relationships
Customer -> ECommerce.WebApp "HTTPS/Uses"
ECommerce.API -> PaymentGateway "HTTPS/REST (External)"
ECommerce.WebApp -> ECommerce.API "HTTP/REST (Internal)"
ECommerce.API -> ECommerce.DB "Reads/Writes (Internal)"
```

## Boundary Types

- **System Boundary:** Your application's scope (contains containers and components)
- **Enterprise Boundary:** Multiple systems within one organization with shared infrastructure
- **Trust Boundary:** Requires authentication/authorization; separates public vs. private resources

## Key Takeaway
Clearly define boundaries to understand **what you own** vs. **what you depend on**. This drives architecture decisions, security controls, and integration strategies.

## Quiz: Test Your Knowledge

**Question 1:** What do boundaries define in systems thinking?

- [ ] a) The budget and timeline for a project
- [ ] b) The scope of your system - what's inside vs. what's outside
- [ ] c) The programming languages to use
- [ ] d) The team structure and roles

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> Boundaries define the scope of your system - what's inside (what you build and control) vs. what's outside (external systems, dependencies, and stakeholders).
  </div>
</div>

---

**Question 2:** In the e-commerce example, which components are considered "outside" the boundary?

- [ ] a) Web Application, API Service, Database
- [ ] b) Payment Gateway, Email Service, Customer
- [ ] c) Only the database
- [ ] d) Only the web application

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> Outside the boundary are: Payment Gateway (external dependency), and Customer (stakeholder). Inside are: Web Application, API Service, and Database.
  </div>
</div>

---

**Question 3:** Which of the following boundary types requires authentication/authorization?

- [ ] a) System Boundary
- [ ] b) Enterprise Boundary
- [ ] c) Trust Boundary
- [ ] d) All of the above

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Explanation:</strong> Trust Boundaries require authentication/authorization and separate public vs. private resources. Cross-boundary calls typically need security controls.
  </div>
</div>

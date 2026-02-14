---
title: "Lesson 4: Parts & Relationships"
weight: 4
summary: "Model the components of your system and how they interact with each other."
time: "2 minutes"
---

# Lesson 4: Parts & Relationships

## Learning Goal
Learn to identify components (parts) and model their interactions (relationships).

## What Are Parts & Relationships?

**Parts** are the distinct components of a system.

**Relationships** are how those components connect and interact with each other.

### The C4 Model Hierarchy

Sruja follows the C4 model: Person (users) → System (software) → Container (apps, databases) → Component (modules). Start with Person and System levels, then drill down as needed.

## Identifying Parts

Start by asking:
- **Who** interacts with the system? (People)
- **What** software systems are involved? (Systems)
- **How** are they built? (Containers & Components)

### Example: E-Commerce

**People:** Customer, Administrator

**Systems:** E-Commerce Platform, Payment Gateway, Email Service

**Containers:** Web App, API Service, Database

## Modeling Relationships

Label relationships with `[Protocol]/[Action]` format (e.g., `HTTPS/REST`, `gRPC/Reads`). Types include synchronous (real-time), asynchronous (message-based), one-way, and request-response.

```sruja
Customer = person "Customer"
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "PostgreSQL"
}

// Clear, labeled relationships
Customer -> ECommerce.WebApp "HTTPS/REST (Browses)"
ECommerce.WebApp -> ECommerce.API "HTTP/JSON (Submits)"
ECommerce.API -> ECommerce.DB "PostgreSQL (Reads/Writes)"
```

## Best Practices

- ✅ Good: `API → DB "PostgreSQL/Reads"`
- ❌ Bad: `API → DB "Uses"`
- Add timing and error handling notes where relevant
- Avoid overcrowding and vague labels

## Key Takeaway
**Parts define structure. Relationships define behavior.** Label relationships with protocols and actions to make diagrams actionable.

## Quiz: Test Your Knowledge

**Question 1:** What are the four levels of the C4 model from highest to lowest?

- [ ] a) System → Container → Component → Person
- [ ] b) Person → System → Component → Container
- [ ] c) Person → System → Container → Component
- [ ] d) Component → Container → System → Person

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: c)</strong> Person → System → Container → Component
  </div>
</div>

---

**Question 2:** What's the key difference between "parts" and "relationships"?

- [ ] a) Parts are technical, relationships are business
- [ ] b) Parts define structure, relationships define behavior
- [ ] c) Parts are visible, relationships are hidden
- [ ] d) Parts are static, relationships are dynamic

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: b)</strong> Parts define structure (what exists), relationships define behavior (how they interact)
  </div>
</div>

---

**Question 3:** Why label relationships with specific protocols?

- [ ] a) Makes diagrams look professional
- [ ] b) Makes diagrams precise and actionable
- [ ] c) Required for validation
- [ ] d) Helps documentation

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: b)</strong> Clear labels like "HTTPS/REST" make diagrams actionable for developers
  </div>
</div>


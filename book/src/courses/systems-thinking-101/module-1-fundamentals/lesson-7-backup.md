---
title: "Lesson 7: Feedback Loops"
weight: 7
summary: "Understand how systems respond, adapt, and self-regulate through feedback mechanisms."
time: "2 minutes"
---

# Lesson 7: Feedback Loops

## Learning Goal
Learn to identify and model feedback loops that make systems reactive and adaptive.

## What Are Feedback Loops?

**Feedback loops** are cycles where actions create reactions that affect future actions. They enable self-regulation, adaptation, and continuous improvement.

### Why Feedback Loops Matter

Feedback loops enable reactive behavior, system stability, adaptation, and monitoring of system health.

## Types of Feedback Loops

**Positive (Reinforcing):** Amplify changes (e.g., viral growth)
**Negative (Balancing):** Stabilize systems (e.g., auto-scaling)

## Sruja Example: Feedback Loops

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"
Admin = person "Admin"

ShopSystem = system "Shop System" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "Database"
  Analytics = container "Analytics Engine"
}

// Positive feedback: Satisfaction loop
scenario UserLoop "Customer Satisfaction" {
  Customer -> ShopSystem.WebApp "Purchases"
  ShopSystem.API -> ShopSystem.Analytics "Updates metrics"
  ShopSystem.API -> Customer "Sends confirmation"
  Customer -> ShopSystem.WebApp "Returns to buy again"
}

// Negative feedback: Auto-scaling loop
scenario SystemLoop "Auto-Scaling" {
  ShopSystem.API -> ShopSystem.Analytics "Reports high CPU"
  ShopSystem.Analytics -> Admin "Sends alert"
  Admin -> ShopSystem.API "Provisions server"
  ShopSystem.API -> ShopSystem.Analytics "Reports normal CPU"
}
```

## Common Feedback Patterns in Software

## Common Patterns

**User:** Action → Response → Behavior change
**System:** Metrics → Alerts → Remediation → Metrics
**Business:** Usage → Revenue → Investment → More usage
**DevOps:** Code → Deploy → Monitor → Learn → Improve

## Identifying Loops

Look for cycles (A → B → C → A), time delays, amplification, and self-regulation.

## Key Takeaway
Feedback loops make systems alive and responsive. Identify reinforcing (growth) and balancing (stability) loops to design for desired outcomes.

## Quiz: Test Your Knowledge

**Question 1:** What are the two main types of feedback loops?

- [ ] a) Fast loops and slow loops
- [ ] b) Positive (reinforcing) and negative (balancing)
- [ ] c) User loops and system loops
- [ ] d) Internal loops and external loops

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: b)</strong> Positive loops amplify changes; negative loops stabilize the system
  </div>
</div>

---

**Question 2:** Why is the satisfaction loop a positive feedback loop?

- [ ] a) Makes customers feel good
- [ ] b) Stabilizes satisfaction
- [ ] c) Amplifies changes - good experience → more purchases → more users
- [ ] d) Prevents customers from leaving

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: c)</strong> Amplifies changes through growth
  </div>
</div>

---

**Question 3:** What does a negative feedback loop do?

- [ ] a) Amplifies changes
- [ ] b) Destabilizes the system
- [ ] c) Stabilizes by counteracting changes
- [ ] d) Creates delays

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: c)</strong> Stabilizes by counteracting changes (e.g., auto-scaling)
  </div>
</div>


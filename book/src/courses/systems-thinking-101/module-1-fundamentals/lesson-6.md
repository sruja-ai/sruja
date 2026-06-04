title: "Lesson 6: Flows"
weight: 6
summary: "Visualize how data, information, and actions move through your system."
time: "2 minutes"
---

# Lesson 6: Flows

## Learning Goal
Learn to model and visualize the movement of data, information, and actions through your system.

## What Are Flows?

**Flows** represent how information, data, and actions move through a system from one component to another.

### Why Flows Matter

Flows show data lineage, process sequences, bottlenecks, and error paths.

## Types of Flows

**Data Flows:** Information movement (API → Database)
**Control Flows:** Operation sequences and decisions
**Event Flows:** Messages and notifications
**User Flows:** Complete user journeys

**Synchronous vs. Asynchronous:** Synchronous waits for response (HTTP); async is fire-and-forget (queues).

## Sruja Example: Order Processing Flow

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

OrderSystem = system "Order System" {
  WebApp = container "Web Application"
  API = container "API Service"
  OrderDB = database "Order Database"
}

PaymentGateway = system "Payment Gateway"
InventoryService = system "Inventory Service"
EmailService = system "Email Service"

scenario PlaceOrder "Order Processing Flow" {
  Customer -> OrderSystem.WebApp "Submits order"
  OrderSystem.WebApp -> OrderSystem.API "Sends data"
  OrderSystem.API -> OrderSystem.OrderDB "Saves record"
  OrderSystem.API -> PaymentGateway "Charge card"
  PaymentGateway -> OrderSystem.API "Confirmation"
  OrderSystem.API -> InventoryService "Reserve items"
  InventoryService -> OrderSystem.API "Confirmed"
  OrderSystem.API -> EmailService "Send email"
  OrderSystem.API -> OrderSystem.WebApp "Confirmed"
  OrderSystem.WebApp -> Customer "Shows confirmation"
}
```

## Visualizing Flows

Flows reveal parallel vs. sequential operations, dependencies, latency bottlenecks, and failure points.

## Flow Best Practices

1. Model happy path first
2. Add branches (error, retry)
3. Label clearly with actions
4. Show timing (sync/async)

## Key Takeaway
Flows reveal dynamic behavior, data movement, bottlenecks, and component collaboration.

## Quiz: Test Your Knowledge

**Question 1:** What is the primary purpose of modeling flows?

- [ ] a) Optimize database performance
- [ ] b) Visualize data and action movement
- [ ] c) Reduce components
- [ ] d) Generate code

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: b)</strong> Visualizes data lineage, bottlenecks, dependencies, and error paths
  </div>
</div>

---

**Question 2:** After customer submits an order, what happens next?

- [ ] a) Payment Gateway charges card
- [ ] b) Web App sends to API Service
- [ ] c) Email Service sends confirmation
- [ ] d) Inventory Service reserves items

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: b)</strong> Web App sends data to API Service
  </div>
</div>

---

**Question 3:** Why model both happy and error paths?

- [ ] a) Documentation requirement
- [ ] b) Make diagrams complete
- [ ] c) Design graceful error handling
- [ ] d) Error paths are more important

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback">
  <div class="feedback-text"></div>
  <div class="explanation">
    <strong>Answer: c)</strong> Helps identify failure points and design error handling
  </div>
</div>

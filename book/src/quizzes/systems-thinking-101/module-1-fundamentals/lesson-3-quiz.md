<!-- Auto-generated quiz from TOML -->
<!-- Source: lesson-3-quiz.toml -->

**1. In systems thinking for software architecture, what term describes the concept that 'every software system is built from multiple interconnected systems'?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** System of systems

**Alternative answers:**
- System-of-systems
- systems of systems

**Explanation:**
Software systems are never isolated. They're composed of and connected to multiple other systems including people, dependencies, processes, data systems, and infrastructure.


</details>

---

**2. Which of the following is NOT one of the layers commonly found in software architecture?**

- [ ] a) Application Layer (UI, Logic, Storage)
- [ ] b) Infrastructure Layer (Compute, Network, Monitoring)
- [ ] c) Organizational Layer (Teams, Workflows, Release processes)
- [ ] d) Marketing Layer (Branding, Advertising, Sales)
- [ ] e) Marketing Layer (Branding, Advertising, Sales)

<button class="check-answer-btn" data-correct="e">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    The main layers are: Application (your software), Infrastructure (computing resources), and Organizational (people and processes). Marketing is a business function, not an architecture layer.

  </div>
</div>

---

**3. In the application layer of software architecture, what three main components are typically included?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** User interfaces, business logic, data storage

**Alternative answers:**
- UI, logic, storage
- User interfaces, APIs, databases
- Web app, API service, database

**Explanation:**
The application layer typically has three main components: User interfaces (Web, Mobile, CLI), Business logic (APIs, Services), and Data storage (Databases, Caches).


</details>

---

**4. Load balancers, CDNs (Content Delivery Networks), and monitoring systems like logs and metrics are examples of which architecture layer?**

- [ ] a) Application Layer - because they're part of your code
- [ ] b) Organizational Layer - because they involve operations teams
- [ ] c) Data Layer - because they store information
- [ ] d) Infrastructure Layer - because they provide computing and networking resources

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Infrastructure layer includes compute (servers, containers), network (load balancers, CDNs), and monitoring (logs, metrics, alerts). These are the foundational services your application runs on.

  </div>
</div>

---

**5. In the Sruja E-Commerce example, what Sruja element type is used to model external systems like 'Payment Gateway' and 'Email Service'?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** system

**Alternative answers:**
- System
- external system

**Explanation:**
In Sruja, external systems are modeled using the `system` element type. For example: `PaymentGateway = system "Payment Gateway"` represents an external dependency.


</details>

---

**6. Why is it important to consider the 'Organizational Layer' when designing software architecture?**

- [ ] a) Because organizational charts make good system diagrams
- [ ] b) Because you need to list all employees in your documentation
- [ ] c) Because organizational changes should be handled by HR, not engineers
- [ ] d) Because teams, development workflows, and release processes directly impact how you build and operate the system

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    The organizational layer includes teams and their responsibilities, development workflows, and release processes. These shape how you structure your code, choose technologies, and deploy your system.

  </div>
</div>

---

**7. When modeling dependencies in Sruja, what kind of arrow is used to show the relationship 'ECommerce.API → PaymentGateway'?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** relationship arrow

**Alternative answers:**
- ->
- arrow
- connection

**Explanation:**
In Sruja, relationships between components are shown using arrows (`->`). For example: `ECommerce.API -> PaymentGateway "Process payment"` shows that the API service depends on the payment gateway.


</details>

---

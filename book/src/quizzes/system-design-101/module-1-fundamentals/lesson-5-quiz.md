<!-- Manual quiz for Lesson 5: User Scenarios -->

**1. What is a user scenario in system design?**

- [ ] a) A description of the system's architecture and components
- [ ] b) A series of steps a user takes to achieve a specific goal within the system
- [ ] c) A list of technical requirements for the system
- [ ] d) A diagram showing database relationships

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    A user scenario describes the series of steps a user takes to achieve a specific goal within your system. While static architecture diagrams show structure, user scenarios show behavior.
  </div>
</div>

---

**2. Which of the following is NOT a benefit of modeling user scenarios?**

- [ ] a) Validation - ensures all required components exist and are connected
- [ ] b) Performance - improves system response times
- [ ] c) Clarity - helps stakeholders understand the system from a user's perspective
- [ ] d) Testing - serves as a blueprint for integration and end-to-end tests

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    The three main benefits of modeling user scenarios are: Validation (ensuring components exist and are connected), Clarity (helping stakeholders understand), and Testing (providing blueprints for tests). Performance improvement is not a direct benefit of scenario modeling.
  </div>
</div>

---

**3. In the "Buying a Ticket" scenario example, what is the correct order of steps?**

- [ ] a) User searches for events → User selects a ticket → System sends confirmation email → User enters payment details → System processes payment
- [ ] b) User enters payment details → User searches for events → User selects a ticket → System processes payment → System sends confirmation email
- [ ] c) User searches for events → User selects a ticket → User enters payment details → System processes payment → System sends confirmation email
- [ ] d) System processes payment → User searches for events → User selects a ticket → User enters payment details → System sends confirmation email

<button class="check-answer-btn" data-correct="c">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    The correct order is: 1) User searches for events, 2) User selects a ticket, 3) User enters payment details, 4) System processes payment, 5) System sends confirmation email. This follows the natural flow from user action to system response.
  </div>
</div>

---

**4. What does the scenario keyword in Sruja allow you to do?**

- [ ] a) Define database schemas and relationships
- [ ] b) Model user interactions and visualize the flow of data across your architecture
- [ ] c) Create automated test scripts
- [ ] d) Generate API documentation

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Sruja's scenario keyword allows you to model user interactions explicitly and visualize the flow of data across your defined architecture. This helps bridge the gap between static architecture diagrams and dynamic user behavior.
  </div>
</div>

---

**5. How do user scenarios differ from static architecture diagrams?**

- [ ] a) Architecture diagrams show structure, user scenarios show behavior
- [ ] b) Architecture diagrams are for developers, user scenarios are for managers
- [ ] c) Architecture diagrams use UML, user scenarios use flowcharts
- [ ] d) Architecture diagrams are optional, user scenarios are required

<button class="check-answer-btn" data-correct="a">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    While static architecture diagrams show the structure of a system (components, connections, relationships), user scenarios show the behavior - the dynamic flow of actions and data as users interact with the system over time.
  </div>
</div>

---

**6. You're building an e-commerce platform. Which scenario best demonstrates a complete user journey?**

- [ ] a) "User adds item to cart"
- [ ] b) "User browses products → User adds item to cart → User enters shipping info → User completes payment → System confirms order"
- [ ] c) "Database stores product information"
- [ ] d) "Payment API processes transactions"

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    A complete user journey shows the full series of steps from start to finish. Option (b) shows the complete flow from browsing to order confirmation, representing a full user scenario. The other options are either too narrow (single action) or describe system behavior rather than user journey.
  </div>
</div>

---

**7. What is the primary purpose of validating user scenarios against your architecture?**

- [ ] a) To ensure the code compiles without errors
- [ ] b) To verify that all components required for a feature exist and are properly connected
- [ ] c) To reduce the cost of infrastructure
- [ ] d) To improve the user interface design

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Validation of user scenarios ensures that all the components, services, and connections needed to support a user feature actually exist in your architecture. This helps catch missing or incomplete implementations before development or deployment.
  </div>
</div>

---

**8. When modeling a scenario in Sruja, you define:**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Actors, systems, containers, and the flow of data between them

You define the actors (users or external systems), the systems and containers they interact with, and use arrows (->) to show the flow of data and interactions between components.

</details>

---

**9. A ride-sharing app has this scenario: "User requests ride → System matches driver → Driver accepts → User receives driver details → Trip begins". What architectural component is likely missing if the scenario fails at the matching step?**

- [ ] a) Payment processing service
- [ ] b) Driver matching algorithm or service
- [ ] c) User authentication system
- [ ] d) Email notification service

<button class="check-answer-btn" data-correct="b">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    If the scenario fails at the matching step, the driver matching algorithm or service is likely missing or malfunctioning. This component is responsible for connecting users with available drivers. The other components (payment, auth, email) are important but not directly related to the matching step.
  </div>
</div>

---

**10. Why are user scenarios valuable for testing?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** They serve as blueprints for integration and end-to-end tests

User scenarios provide a step-by-step description of how the system should behave from a user's perspective, making them perfect templates for writing integration tests and end-to-end tests. They help ensure that all components work together correctly to support real user workflows.

</details>

---
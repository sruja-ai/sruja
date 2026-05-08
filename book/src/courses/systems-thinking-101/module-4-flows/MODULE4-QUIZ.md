---
title: "Module 4 Quiz: Flows"
weight: 5
summary: "Test your understanding of flows, data flows, and user journeys"
time: "10 min"
---

# Module 4 Quiz: Flows

Test your understanding of how to model flows in software architecture—including data flows, user journeys, and behavioral scenarios.

This quiz covers all three lessons in Module 4. Take your time, think through each question, and read the explanations to reinforce your learning.

---

## Question 1: Understanding Flows

You're deciding between using a static relationship or a flow to document an order processing system. Which scenario requires a flow rather than a static relationship?

> "The system has these components: Customer, WebApp, API, Database, PaymentGateway, EmailService. We need to document the complete sequence of what happens when a customer places an order, including validation, payment processing, order creation, and confirmation email."

**A)** Static relationship showing Customer connects to WebApp
**B)** Flow showing the complete sequence from customer action to email delivery
**C)** Static relationships showing all connections between components
**D)** High-level diagram showing only the main systems

<details>
<summary>Click to see the answer</summary>

**Answer: B) Flow showing the complete sequence from customer action to email delivery**

**Explanation:**

The key phrase in this scenario is **"complete sequence"** and **"what happens when a customer places an order."** This tells you that order matters—you need to show the steps that happen, not just the connections.

Let's analyze each option:

**A)** Incorrect. A static relationship like `Customer -> WebApp "Uses"` only shows that a customer connects to the web app. It doesn't show what happens when they place an order. It doesn't show the sequence (validation → payment → order creation → email). Static relationships are for showing what's connected, not what happens.

**B)** Correct! A flow is the right choice here because:
- The scenario describes a **sequence** of events that happen in order
- It requires showing the **complete journey** from customer action (placing order) to final outcome (email delivery)
- It includes multiple steps that depend on each other (validation must happen before payment, payment must happen before order creation)
- The **order matters**—you can't show this with just static connections

Here's what the flow would look like:

```sruja
// partial
OrderFlow = scenario "Order Processing" {
  Customer -> WebApp "Places order"
  WebApp -> API "Validates cart"
  API -> Database "Checks inventory"
  Database -> API "Inventory available"
  API -> PaymentGateway "Processes payment"
  PaymentGateway -> API "Payment successful"
  API -> Database "Creates order"
  API -> EmailService "Sends confirmation"
}
```

This flow tells the complete story—every step, in order, from customer action to email delivery.

**C)** Incorrect. Static relationships show connections, not sequences. Even if you have all the connections (`Customer -> WebApp`, `WebApp -> API`, `API -> Database`, etc.), you're still not showing the **order** in which things happen. Does validation happen before payment? Does email happen after order creation? You can't tell from static relationships.

**D)** Incorrect. A high-level diagram showing only main systems (`Customer -> Shop`, `Shop -> PaymentGateway`) is even less detailed than static relationships. It shows a general overview, not the specific sequence of what happens during order processing.

**Key Takeaway:** Use flows when you need to show **sequence** and **order matters**. Static relationships are for showing what's connected. Flows are for showing what happens and in what order. If the scenario describes a process, a journey, or a sequence of events—use a flow.

</details>

---

## Question 2: Types of Flows

You're modeling a real-time fraud detection system. Transaction data comes from various sources (web, mobile, POS), gets processed through a stream processor, and fraud alerts are sent to a security team. Which flow type is most appropriate?

> "A fraud detection system processes transactions in real-time. Transactions originate from web, mobile, and POS systems. All transactions flow into a stream processor that analyzes patterns and flags suspicious activity. Flagged transactions trigger immediate alerts to the security team."

**A)** Data Flow (DFD Style)
**B)** User Journey / Scenario
**C)** Control Flow
**D)** Event Flow

<details>
<summary>Click to see the answer</summary>

**Answer: D) Event Flow**

**Explanation:**

The key characteristics of this scenario are:
- **Real-time processing** ("in real-time")
- **Multiple sources** (web, mobile, POS) contributing to a single stream
- **Stream processing** (transactions flow through a stream processor)
- **Event-like behavior** (transactions are discrete events that get processed)

Let's analyze each option:

**A)** Incorrect. A data flow (DFD-style) focuses on how **data** moves and transforms through a system—showing lineage, transformations, and aggregations. While this scenario involves data movement, it's specifically about **events** being processed in real-time through a stream. A data flow would be more appropriate for an ETL pipeline or batch analytics job, not real-time event processing.

**B)** Incorrect. A user journey shows how a **person** interacts with a system to achieve a goal—from the user's perspective. This scenario isn't about a user's experience. It's about how **transactions** (events) flow through a backend processing system. There's no human user clicking buttons or experiencing a journey—this is backend event processing.

**C)** Incorrect. A control flow shows decision points and branching logic—modeling "if this, then that" patterns. This scenario describes a **linear pipeline** where transactions flow from sources through a processor to alerts. There's no branching, no decision logic, no "if/else" conditions. Everything follows the same path: source → stream → processing → alert. Control flows are for modeling business logic, workflows, and approval processes.

**D)** Correct! An event flow is the right choice here because:
- The scenario describes **events** (transactions) that are published/consumed
- Multiple sources contribute events to a **stream** (web, mobile, POS)
- Events are processed **asynchronously** in real-time
- Events flow through a **processing pipeline** (stream processor)
- The focus is on **event propagation**—how events move through the system

Here's what the event flow would look like:

```sruja
// partial
FraudDetectionFlow = flow "Real-Time Fraud Detection" {
  // Events from multiple sources
  Web -> TransactionStream "Web transactions"
  Mobile -> TransactionStream "Mobile transactions"
  POS -> TransactionStream "POS transactions"
  
  // Stream processing
  TransactionStream -> StreamProcessor "Consumes and analyzes"
  StreamProcessor -> FraudEngine "Detects patterns"
  
  // Event propagation
  FraudEngine -> AlertService "Fraud alert"
  AlertService -> SecurityTeam "Sends notification"
}
```

This flow shows how events (transactions) from multiple sources flow into a stream, get processed, and trigger alerts when fraud is detected.

**Key Takeaway:** Event flows are for event-driven architectures where events flow through processing systems—often with multiple sources, asynchronous processing, and real-time or near-real-time characteristics. Data flows are for data lineage and transformation. User journeys are for human experiences. Control flows are for decision logic.

</details>

---

## Question 3: Data Flow Diagrams

You're documenting an ETL (Extract, Transform, Load) pipeline that moves customer data from operational databases to an analytics data warehouse. Which structure best documents the data transformations that happen at each step?

> "Customer data starts in the operational CRM database. Every night at 2 AM, an ETL job extracts customer records, validates email formats and removes invalid entries, normalizes phone numbers to E.164 format, standardizes dates to ISO 8601, enriches records with behavioral data from a clickstream, and loads the final, cleaned data into the analytics warehouse."

**A)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  CRM -> DataWarehouse "Move data"
}
```

**B)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  CRM -> ETLService "Extract"
  ETLService -> DataWarehouse "Load"
}
```

**C)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  CRM -> ETLService "Extract customer records"
  ETLService -> ValidatedData "Validate emails, remove invalids"
  ValidatedData -> NormalizedData "Normalize phones, standardize dates"
  NormalizedData -> EnrichedData "Add behavioral data"
  EnrichedData -> DataWarehouse "Load to warehouse"
}
```

**D)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  CRM -> ETLService "?"
  ETLService -> DataWarehouse "?"
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: C) Shows each transformation step clearly**

**Explanation:**

The scenario describes **multiple data transformations** that happen in sequence:
1. **Extract** - pull customer records from CRM
2. **Validate** - check email formats, remove invalid entries
3. **Normalize** - standardize phone numbers and dates
4. **Enrich** - add behavioral data from clickstream
5. **Load** - push final data to warehouse

Let's analyze each option:

**A)** Incorrect. This flow has only one step: `CRM -> DataWarehouse "Move data"`. This completely skips all the transformations. It tells you nothing about:
- Are emails validated? Invalid records removed?
- Are phone numbers normalized? How (E.164)?
- Are dates standardized? To what format (ISO 8601)?
- Is data enriched with behavioral data?

Anyone reading this flow wouldn't understand what actually happens to the data. It's too abstract to be useful.

**B)** Incorrect. This flow has two steps: `CRM -> ETLService "Extract"` and `ETLService -> DataWarehouse "Load"`. It captures the "Extract" and "Load" parts of ETL, but it completely skips the "Transform" part—the middle T! The transformations (validation, normalization, enrichment) are the most important and complex part of this pipeline, but they're not shown.

**C)** Correct! This flow documents each transformation step clearly:
- **Extract customer records** - Pulls raw data from CRM
- **Validate emails, remove invalids** - First transformation: checks data quality, removes bad records
- **Normalize phones, standardize dates** - Second transformation: standardizes formats (E.164 for phones, ISO 8601 for dates)
- **Add behavioral data** - Third transformation: enriches records with additional data from clickstream
- **Load to warehouse** - Final step: stores transformed, cleaned, enriched data

Each relationship label describes what transformation happens at that step. Anyone reading this flow understands the complete ETL process and what happens to the data at each stage.

**D)** Incorrect. While this has the right number of steps (two steps), the labels are meaningless ("?"). What does the first "?" mean? What about the second "?"? These labels provide no information about what transformations are happening. Each step is a black box—you know there are transformations, but you don't know what they are.

**Key Takeaway:** Data flows should document transformations clearly using descriptive relationship labels. Don't just show that data moves—show **how** it transforms. Label each step with what actually happens: "validate," "normalize," "enrich," "aggregate," "calculate." This makes your data flows informative and useful, not just correct.

</details>

---

## Question 4: User Journeys and BDD

You're documenting the login flow for a banking application and want to use BDD-style to make it unambiguous. Which scenario best represents a BDD "Given-When-Then" structure for successful login?

> "Users need to be able to log in with valid credentials to access their accounts. The login should be secure, quick, and provide clear feedback."

**A)**
```sruja
// partial
LoginFlow = scenario "Login" {
  User -> WebApp "Login"
  WebApp -> API "Authenticate"
  API -> Database "Check credentials"
  Database -> API "User found"
  API -> WebApp "Success"
  WebApp -> User "Dashboard"
}
```

**B)**
```sruja
// partial
LoginFlow = scenario "Successful Login" {
  // GIVEN: User has valid credentials
  User -> WebApp "Enters username and password"
  WebApp -> API "Submits login"
  
  // WHEN: User clicks login
  API -> Database "Verifies credentials"
  Database -> API "Credentials match"
  
  // THEN: User is logged in
  API -> TokenService "Generates session token"
  API -> WebApp "Returns token and user data"
  WebApp -> User "Shows dashboard"
}
```

**C)**
```sruja
// partial
LoginFlow = scenario "As a user, I want to log in" {
  Start -> End "Login completes"
}
```

**D)**
```sruja
// partial
LoginFlow = scenario "Login Flow" {
  WebApp -> API "Login request"
  API -> Database "Verify user"
  Database -> API "User data"
  API -> WebApp "Response"
  WebApp -> User "Show dashboard"
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: B) Given-When-Then structure with user actions and outcomes**

**Explanation:**

BDD (Behavior-Driven Development) is about writing requirements in plain language using a clear "Given-When-Then" structure. A good BDD scenario:
- Sets up the starting state (Given)
- Describes the user action (When)
- Describes the expected outcome (Then)

Let's analyze each option:

**A)** Incorrect. While this scenario shows the sequence of login, it doesn't follow the BDD "Given-When-Then" structure. It doesn't:
- Set up the starting state (GIVEN: User has valid credentials)
- Explicitly mark the user action (WHEN: User clicks login)
- Explicitly mark the outcome (THEN: User is logged in)

It just shows a sequence of technical steps, which is more like a traditional flow than a BDD-style scenario.

**B)** Correct! This is a perfect BDD-style scenario because:
- **GIVEN** (`// GIVEN: User has valid credentials`) — Sets up the starting state clearly
- **WHEN** (`// WHEN: User clicks login`) — Describes the specific user action
- **THEN** (`// THEN: User is logged in`) — Describes the expected outcome

The scenario also shows:
- **User's perspective** — User enters credentials, clicks login, sees dashboard
- **System responses** — API authenticates, database verifies, token generated
- **Complete experience** — From entering credentials to seeing dashboard

This scenario serves multiple purposes:
- **Requirements** — Unambiguous what "successful login" means
- **Tests** — Can be directly turned into a test case
- **Documentation** — Anyone understands the login flow
- **Communication** — Product managers, developers, and testers all agree

**C)** Incorrect. This is far too abstract. `Start -> End "Login completes"` tells you nothing about what happens. What does the user do? What does the system do? What's the experience? It's so vague it provides no value.

**D)** Incorrect. This scenario has a few issues:
- It doesn't include the user action—starts with `WebApp -> API "Login request"` instead of `User -> WebApp`
- It doesn't follow BDD "Given-When-Then" structure
- It doesn't set up the starting state (given valid credentials)
- It doesn't clearly mark the expected outcome (then logged in)
- It's more like a technical data flow than a user journey

**Key Takeaway:** BDD-style scenarios focus on the user's experience and use a clear "Given-When-Then" structure. Given sets up the starting state. When describes the user action. Then describes the expected outcome. This makes requirements unambiguous and scenarios that can serve as both documentation and test cases.

</details>

---

## Question 5: User Journeys and Error Paths

You're modeling a checkout flow and want to document an error path for when payment fails. Which scenario best models a helpful error handling experience from the user's perspective?

> "When a customer's payment fails during checkout, they should see a clear, helpful error message explaining what went wrong and what they can do next. The system should save their cart so they can try again later."

**A)**
```sruja
// partial
PaymentError = scenario "Payment Error" {
  Customer -> WebApp "Checkout"
  WebApp -> API "Process order"
  API -> PaymentGateway "Process payment"
  PaymentGateway -> API "Payment failed"
  API -> WebApp "Error"
  WebApp -> Customer "Error"
}
```

**B)**
```sruja
// partial
PaymentError = scenario "Payment Error" {
  Customer -> WebApp "Checkout"
  WebApp -> API "Process order"
  API -> PaymentGateway "Process payment"
  PaymentGateway -> API "Payment declined: insufficient funds"
  API -> WebApp "Payment failed: ERR_PAYMENT_DECLINED"
  WebApp -> Customer "Error"
}
```

**C)**
```sruja
// partial
PaymentError = scenario "Checkout with Payment Error" {
  Customer -> WebApp "Adds items to cart and clicks checkout"
  WebApp -> API "Initiates checkout"
  API -> PaymentGateway "Processes payment: $50"
  PaymentGateway -> API "Payment declined: insufficient funds"
  
  // Helpful error handling
  API -> Database "Saves cart for later"
  API -> WebApp "Returns error with details"
  WebApp -> Customer "Shows 'Payment declined. Your card has insufficient funds. Please try a different payment method or contact your bank. We've saved your cart so you can try again later.'"
}
```

**D)**
```sruja
// partial
PaymentError = scenario "Payment Failed" {
  API -> PaymentGateway "Payment"
  PaymentGateway -> API "Failed"
  API -> WebApp "Error"
  WebApp -> Customer "Can't checkout"
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: C) Helpful error message with cart preservation and clear next steps**

**Explanation:**

The key requirements from the scenario are:
1. User sees a **clear, helpful error message**
2. Error explains **what went wrong** and **what to do next**
3. System **saves their cart** so they can try again

Let's analyze each option:

**A)** Incorrect. This scenario shows the error occurring, but it provides no useful information about:
- What went wrong? (insufficient funds? expired card? declined?)
- What should the user do? (try again? use different card? contact support?)
- What error message do they see? (just "Error"?)
- Is the cart saved?

The error labels are generic ("Error", "Payment failed"). The user experience would be confusing—what went wrong? What should I do? Can I try my cart again or is it lost?

**B)** Incorrect. This scenario is better than A, but still has issues:
- It shows the specific error ("Payment declined: insufficient funds") which is good
- But the error message to the user is still cryptic: "Payment failed: ERR_PAYMENT_DECLINED"
- "ERR_PAYMENT_DECLINED" is a technical error code, not a helpful message
- The user has no idea what to do next—try a different card? Check their bank balance?
- There's no indication that the cart is saved
- It starts with `Customer -> WebApp "Checkout"` but doesn't show the complete checkout journey (adding items, reviewing cart, etc.)

**C)** Correct! This scenario models an excellent error handling experience because:
- **User action is specific** — "Adds items to cart and clicks checkout" shows the complete context
- **System response is clear** — "Payment declined: insufficient funds" tells you exactly what went wrong
- **Cart is saved** — `API -> Database "Saves cart for later"` shows the cart isn't lost
- **Error message is helpful** — "Payment declined. Your card has insufficient funds. Please try a different payment method or contact your bank. We've saved your cart so you can try again later." tells the user:
  - What went wrong (insufficient funds)
  - What they can do (try different method, contact bank)
  - Reassurance (cart is saved, can try again)

This error message follows best practices:
- **Specific** — Tells you exactly what's wrong (insufficient funds)
- **Actionable** — Tells you what to do (try different method, contact bank)
- **Reassuring** — Cart is saved, not lost
- **Human** — Written in plain language, not technical codes

**D)** Incorrect. This scenario is a mess:
- It doesn't show the user action—starts with `API -> PaymentGateway` instead of showing the user checking out
- It doesn't capture the user's perspective at all
- The error is generic ("Failed") and doesn't tell you what went wrong
- "Can't checkout" tells the user nothing—why can't they checkout? What happened? What can they do?
- There's no cart preservation mentioned
- It's from the system's perspective, not the user's

**Key Takeaway:** Error paths should be just as well-designed as happy paths. When modeling errors:
- Be specific about what went wrong (not just "error occurred")
- Write helpful error messages in plain language (not technical codes like "ERR_PAYMENT_DECLINED")
- Guide the user on next steps (try again? use different payment? contact support?)
- Provide reassurance when appropriate (cart saved, data safe, try again later)
- Think from the user's perspective (what do they see? what do they understand? what do they do next?)

A well-designed error path turns a frustrating experience into a helpful one. I've seen error messages drop support tickets by 70% just by being clear and actionable.

</details>

---

## How Did You Do?

Count your correct answers:

- **5 correct:** Excellent! You have a solid understanding of flows, data flows, and user journeys. You're ready to apply these concepts in real projects.
- **4 correct:** Great work! You understand most concepts well. Review the explanation for the question you missed to solidify your understanding.
- **3 correct:** Good effort! You understand the basics but need to practice on some concepts. Re-read the relevant lessons and try the quiz again.
- **1-2 correct:** Keep learning! You're on the right track, but need to review the lessons more carefully. Focus on understanding the "why" behind each concept, not just the "how."

---

## What's Next?

Ready to move on? In [Module 5: Feedback Loops](../module-5-feedback-loops/module-overview.md), you'll learn about how systems regulate themselves through circular cause-and-effect relationships. This is crucial for:

- Understanding how systems maintain stability
- Recognizing positive and negative feedback loops
- Modeling self-regulating systems like thermostats
- Understanding how amplifying loops can lead to runaway growth or collapse
- Designing systems that balance stability and growth

See you there!

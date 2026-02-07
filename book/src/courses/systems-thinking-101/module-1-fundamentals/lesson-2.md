# Lesson 2: The Five Core Concepts

## Learning Goals

- Learn the five core systems thinking concepts
- Understand how they apply to software architecture
- See how Sruja implements each concept

## The Five Core Concepts

In this course, you'll master five fundamental systems thinking concepts:

```
┌─────────────────────────────────────────────┐
│           SYSTEMS THINKING 101              │
├─────────────────────────────────────────────┤
│  1. Parts & Relationships                   │
│  2. Boundaries                              │
│  3. Flows                                   │
│  4. Feedback Loops                          │
│  5. Context                                 │
└─────────────────────────────────────────────┘
```

## 1. Parts and Relationships

**Definition**: What the system contains and how those pieces connect.

### In Sruja

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "End User"
Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  DB = database "Database"
}

// Relationships = how parts connect
Customer -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.API "Calls"
Shop.API -> Shop.DB "Reads"
```

### Why It Matters

- **Structure**: Defines what exists in your system
- **Interactions**: Shows how components communicate
- **Coupling**: Reveals dependencies between parts

## 2. Boundaries

**Definition**: What's inside the system vs. what's outside (the environment).

### In Sruja

```sruja
// Inside: Your system
Shop = system "Shop" {
  WebApp = container "Web App"
  API = container "API"
}

// Outside: External systems
PaymentGateway = system "Payment Service" {
  metadata { tags ["external"] }
}

// Crossing the boundary
Shop.API -> PaymentGateway "Processes payments"
```

### Why It Matters

- **Scope**: What you're responsible for vs. dependencies
- **Integration**: How you connect with external systems
- **Security**: What needs to be protected
- **Testing**: What's in scope for your tests

## 3. Flows

**Definition**: How information, data, and actions move through the system.

### In Sruja (Data Flow)

```sruja
OrderFlow = scenario "Order Processing" {
  Customer -> Shop.WebApp "Submits order"
  Shop.WebApp -> Shop.API "Sends order data"
  Shop.API -> Shop.DB "Saves order"
  Shop.API -> PaymentGateway "Charges payment"
  Shop.API -> Shop.WebApp "Returns result"
  Shop.WebApp -> Customer "Shows confirmation"
}
```

### Why It Matters

- **Data lineage**: Where does information come from and go?
- **Process understanding**: What's the sequence of actions?
- **Bottlenecks**: Where do things slow down?
- **Error paths**: What happens when something fails?

## 4. Feedback Loops

**Definition**: How actions create reactions that affect future actions.

### In Sruja (Cycles)

```sruja
// User feedback loop
User -> App.WebApp "Submits form"
App.WebApp -> App.API "Validates"
App.API -> App.WebApp "Returns result"
App.WebApp -> User "Shows feedback"

// System feedback loop
App.API -> App.DB "Updates inventory"
App.DB -> App.API "Notifies low stock"
App.API -> Admin "Sends alert"
Admin -> App.API "Adjusts inventory"
```

### Why It Matters

- **Reactive behavior**: How systems respond to changes
- **Stability**: Does the system self-correct or spiral out of control?
- **Adaptation**: How the system learns and evolves
- **Monitoring**: What metrics indicate system health?

## 5. Context

**Definition**: The environment the system operates in - stakeholders, dependencies, constraints.

### In Sruja

```sruja
// Your system
Shop = system "Shop"

// Context: Stakeholders
Customer = person "End User"
Admin = person "Administrator"
Support = person "Customer Support"

// Context: Dependencies
PaymentGateway = system "Payment Service"
EmailService = system "Email Service"
Analytics = system "Analytics"

// Context relationships
Customer -> Shop "Uses"
Shop -> PaymentGateway "Depends on"
Shop -> EmailService "Sends notifications"
Admin -> Shop "Manages"
```

### Why It Matters

- **Stakeholders**: Who cares about this system?
- **Dependencies**: What external systems do you rely on?
- **Constraints**: What limits your design choices?
- **Success criteria**: What does "good" look like?

## How They Work Together

A complete systems thinking view combines all five:

```
┌──────────────────────────────────────────┐
│              CONTEXT                     │
│  Stakeholders: Users, Admins, Support    │
│  Dependencies: External APIs, Services   │
│                                          │
│    ┌────────── BOUNDARY ──────────┐     │
│    │                               │     │
│    │   PARTS ↔ RELATIONSHIPS       │     │
│    │   [User] → [App] → [DB]       │     │
│    │         ↑       ↓             │     │
│    │         └── FEEDBACK ─────┘  │     │
│    │                               │     │
│    │   FLOWS: Order, Payment      │     │
│    └───────────────────────────────┘     │
└──────────────────────────────────────────┘
```

## Sruja Mapping Summary

| Systems Thinking Concept | Sruja Feature                                                       |
| ------------------------ | ------------------------------------------------------------------- |
| Parts & Relationships    | Elements (`person`, `system`, `container`) and relationships (`->`) |
| Boundaries               | `system` definitions, `metadata { tags ["external"] }`              |
| Flows                    | `scenario` and `flow` blocks                                        |
| Feedback Loops           | Cycles in relationships (explicitly allowed)                        |
| Context                  | External elements, `overview` block, `requirements`                 |

## Key Takeaways

1. **Five concepts form the foundation**: Parts/relationships, boundaries, flows, feedback loops, context
2. **They interconnect**: None exists in isolation
3. **Sruja supports all five**: Built into the language
4. **Use them together**: Complete architecture requires all perspectives
5. **Practice makes perfect**: You'll work with each in depth

## Exercise

Pick a system you know well. For each concept, write one observation:

- **Parts/Relationships**: Name 3 components and how they connect
- **Boundaries**: What's inside vs. outside the system?
- **Flows**: What's a common data path through the system?
- **Feedback Loops**: Any automatic responses to user actions?
- **Context**: Who are the main stakeholders?

## Next Lesson

In [Lesson 3](lesson-3.md), we'll explore the practical benefits of systems thinking for software architecture.

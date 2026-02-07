---
title: "Lesson 5: User Scenarios"
weight: 5
summary: "Modeling user flows and interactions."
---

# Lesson 5: User Scenarios

## Understanding User Journeys

A **User Scenario** describes the series of steps a user takes to achieve a specific goal within your system. While static architecture diagrams show _structure_, user scenarios show _behavior_.

### Why Model Scenarios?

1.  **Validation:** Ensures that all components required for a feature actually exist and are connected.
2.  **Clarity:** Helps stakeholders understand how the system works from a user's perspective.
3.  **Testing:** Serves as a blueprint for integration and end-to-end tests.

### Example Scenario: Buying a Ticket

1.  User searches for events.
2.  User selects a ticket.
3.  User enters payment details.
4.  System processes payment.
5.  System sends confirmation email.

---

## 🛠️ Sruja Perspective: Modeling Scenarios

Sruja provides a dedicated `scenario` keyword to model these interactions explicitly. This allows you to visualize the flow of data across your defined architecture.

```sruja
import { * } from 'sruja.ai/stdlib'


R1 = requirement functional "User can buy a ticket"
R2 = requirement performance "Process payment in < 2s"

// Define the actors and systems first
User = person "Ticket Buyer"

TicketingApp = system "Ticketing Platform" {
    WebApp = container "Web Frontend"
    PaymentService = container "Payment Processor"
    EmailService = container "Notification Service"

    WebApp -> PaymentService "Process payment"
    PaymentService -> EmailService "Trigger confirmation"
}

// Define the scenario
BuyTicket = scenario "User purchases a concert ticket" {
    User -> TicketingApp.WebApp "Selects ticket"
    TicketingApp.WebApp -> TicketingApp.PaymentService "Process payment"
    TicketingApp.PaymentService -> TicketingApp.EmailService "Trigger confirmation"
    TicketingApp.EmailService -> User "Send email"
}

view index {
include *
}
```

By defining scenarios, you can automatically generate sequence diagrams or flowcharts that map directly to your code.

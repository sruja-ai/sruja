---
title: "Lesson 5: User Journeys"
weight: 5
summary: "Modeling user flows and interactions."
---

# Lesson 5: User Journeys

## Understanding User Journeys

A **User Journey** describes the series of steps a user takes to achieve a specific goal within your system. While static architecture diagrams show *structure*, user journeys show *behavior*.

### Why Model Journeys?
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

## 🛠️ Sruja Perspective: Modeling Journeys

Sruja provides a dedicated `journey` keyword to model these interactions explicitly. This allows you to visualize the flow of data across your defined architecture.

```sruja
architecture "Ticketing System" {
    requirement R1 functional "User can buy a ticket"
    requirement R2 performance "Process payment in < 2s"

    // Define the actors and systems first
    person User "Ticket Buyer"
    
    system TicketingApp "Ticketing Platform" {
        container WebApp "Web Frontend"
        container PaymentService "Payment Processor"
        container EmailService "Notification Service"

        WebApp -> PaymentService "Process payment"
        PaymentService -> EmailService "Trigger confirmation"
    }

    // Define the journey
    journey BuyTicket {
        title "User purchases a concert ticket"
        
        steps {
            User -> WebApp "Selects ticket"
            WebApp -> PaymentService "Process payment"
            PaymentService -> EmailService "Trigger confirmation"
            EmailService -> User "Send email"
        }
    }
}
```

By defining journeys, you can automatically generate sequence diagrams or flowcharts that map directly to your code.

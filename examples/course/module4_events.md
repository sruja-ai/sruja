# Module 4: Domain Events

## Domain Events

A **Domain Event** is something that happened in the domain that you want other parts of the same domain (in-process) or other domains (cross-process) to be aware of.

### Characteristics
- **Immutable**: Once it happened, it cannot be changed.
- **Past Tense**: Named in the past tense (e.g., `OrderCreated`).
- **Relevant**: Contains data relevant to the event.

## Event Storming

Event Storming is a workshop format for quickly exploring complex business domains by focusing on domain events.

## Sruja Syntax

```sruja
context Sales {
    event OrderCreated {
        description "Emitted when a new order is placed"
        orderId string
        customerId string
        totalAmount float
    }
}
```

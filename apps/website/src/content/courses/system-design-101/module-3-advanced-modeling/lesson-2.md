---
title: "Lesson 2: Event-Driven Architecture"
weight: 2
summary: "Pub/Sub, Message Queues, Event Sourcing."
---

# Lesson 2: Event-Driven Architecture

## Synchronous vs. Asynchronous

*   **Synchronous (Request/Response):** Client waits for the server to respond (e.g., HTTP REST).
*   **Asynchronous (Event-Driven):** Client sends a message and continues work. The receiver processes it later.

## Core Concepts

### Message Queues (Point-to-Point)
A message is sent to a queue and processed by exactly one consumer.
*   *Use Case:* Background jobs (e.g., image resizing).

### Pub/Sub (Publish/Subscribe)
A message (event) is published to a topic. Multiple subscribers can receive a copy.
*   *Use Case:* notifying multiple services (e.g., "UserSignedUp" -> EmailService, AnalyticsService).

---

## 🛠️ Sruja Perspective: Modeling Events

Sruja supports `queue` as a first-class citizen to model asynchronous communication.

```sruja
architecture "Notification System" {
    system Notifications "Notification System" {
        container AuthService "Auth Service" {
            technology "Node.js"
        }

        // Define a queue or topic
        queue UserEvents "User Events Topic" {
            technology "Kafka"
            description "Events related to user lifecycle (signup, login)."
        }

        container EmailService "Email Service" {
            technology "Python"
        }

        container AnalyticsService "Analytics Service" {
            technology "Spark"
        }

        // Pub/Sub flow
        AuthService -> UserEvents "Publishes 'UserSignedUp'"
        UserEvents -> EmailService "Consumes"
        UserEvents -> AnalyticsService "Consumes"
    }
}
```

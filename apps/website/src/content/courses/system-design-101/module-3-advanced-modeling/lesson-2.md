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
specification {
  element person
  element system
  element container
  element datastore
  element queue
}

model {
    User = person "End User"
    
    Notifications = system "Notification System" {
        AuthService = container "Auth Service" {
            technology "Node.js"
            description "Handles user authentication and publishes events"
        }

        // Define a queue or topic
        UserEvents = queue "User Events Topic" {
            technology "Kafka"
            description "Events related to user lifecycle (signup, login, profile updates)."
        }

        EmailService = container "Email Service" {
            technology "Python"
            description "Sends transactional emails"
        }

        AnalyticsService = container "Analytics Service" {
            technology "Spark"
            description "Processes user events for analytics"
        }
        
        NotificationDB = datastore "Notification Database" {
            technology "PostgreSQL"
            description "Stores notification preferences and history"
        }

        // Pub/Sub flow
        User -> AuthService "Signs up"
        AuthService -> UserEvents "Publishes 'UserSignedUp' event"
        UserEvents -> EmailService "Consumes - sends welcome email"
        UserEvents -> AnalyticsService "Consumes - tracks signup"
        EmailService -> NotificationDB "Logs email sent"
    }
    
    // Model the event flow as a scenario
    scenario UserSignupFlow "User Signup Event Flow" {
        User -> AuthService "Submits registration"
        AuthService -> UserEvents "Publishes UserSignedUp"
        UserEvents -> EmailService "Triggers welcome email"
        UserEvents -> AnalyticsService "Tracks signup event"
        EmailService -> User "Sends welcome email"
    }
    
    // Data flow for analytics processing
    flow AnalyticsPipeline "Analytics Data Pipeline" {
        UserEvents -> AnalyticsService "Streams events"
        AnalyticsService -> AnalyticsService "Processes batch"
        AnalyticsService -> AnalyticsService "Generates reports"
    }
}

views {
  view index {
    title "Notification System Overview"
    include *
  }
  
  // Event flow view: Focus on async communication
  view eventflow {
    title "Event Flow View"
    include Notifications.AuthService Notifications.UserEvents
    include Notifications.EmailService Notifications.AnalyticsService
    exclude User Notifications.NotificationDB
  }
  
  // Data view: Focus on data storage
  view data {
    title "Data Storage View"
    include Notifications.EmailService Notifications.NotificationDB
    include Notifications.AnalyticsService
    exclude Notifications.AuthService Notifications.UserEvents
  }
}
```

### Key Concepts

1. **Scenarios** model behavioral flows (user journeys, use cases)
2. **Flows** model data pipelines (ETL, streaming, batch processing)
3. **Views** let you focus on different aspects (events vs data vs user experience)

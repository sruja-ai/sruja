title: "Lesson 2: Event-Driven Architecture"
weight: 2
summary: "How asynchronous events saved Netflix—and the synchronous chains that broke other systems"
time: "14 min"
---

# The Chain Reaction: When Synchronous Dependencies Kill Systems

It started with a single database query taking 2 seconds instead of 200ms.

The Analytics Service called the User Service to enrich data. The User Service called the Subscription Service to check plan details. The Subscription Service called the Payment Service to verify active status. The Payment Service called... you get the picture.

When that one database query slowed down, everything slowed down. Within minutes, the entire platform was timing out. Every service was waiting for every other service. We had a beautiful microservices architecture with synchronous calls everywhere—and it took 8 hours to recover.

The post-mortem was brutal: "We built a distributed monolith with synchronous chains." The fix? Event-driven architecture. Services that could operate independently, react to events asynchronously, and fail gracefully when dependencies were slow.

This lesson is about avoiding that mistake. You'll learn when event-driven architecture helps (and when it adds complexity), the three main patterns (queues, pub/sub, and event sourcing), and how to model events in Sruja.

## Learning Goals

By the end of this lesson, you'll be able to:

- Understand why synchronous dependencies create fragility
- Recognize when event-driven architecture solves real problems
- Choose between message queues, pub/sub, and event sourcing
- Model event-driven systems in Sruja with queues and scenarios
- Avoid common event-driven mistakes (eventual consistency isn't magic)

## Why Events Matter: The Synchronous Problem

Here's a pattern I've seen destroy systems: **Synchronous call chains.**

Service A calls Service B, which calls Service C, which calls Service D. When D is slow, C is slow. When C is slow, B is slow. When B is slow, A is slow. The user sees timeouts. The system appears down.

**The synchronous trap:**
- **Latency adds up:** 50ms + 100ms + 200ms + 150ms = 500ms total response time
- **Failures cascade:** One slow service slows everything
- **Coupling is hidden:** You think services are independent, but they're not
- **Hard to debug:** Which service caused the timeout? Good luck tracing it.

**The event-driven alternative:**

Service A publishes an event. Service B, C, and D subscribe and process independently. Service A doesn't wait. If Service D is slow, Services B and C still work. If Service D crashes, the event stays in the queue until it recovers.

**What you gain:**
- **Decoupling:** Services don't need to know about each other
- **Resilience:** One service failure doesn't cascade
- **Scalability:** Add more consumers when load increases
- **Flexibility:** Add new subscribers without touching existing services

**What you pay:**
- **Complexity:** Debugging distributed events is harder
- **Eventual consistency:** Data isn't immediately consistent across services
- **Operational overhead:** Message brokers, queues, retry logic
- **Learning curve:** Thinking in events is different from thinking in requests

## Synchronous vs. Asynchronous: A Real Comparison

Let me show you the difference with a real example: **User Registration.**

### Synchronous Approach (Request/Response)

```
User → API: POST /register
API → EmailService: Send welcome email (wait for response)
API → AnalyticsService: Track signup (wait for response)
API → CRMService: Create lead (wait for response)
API → User: "Registration complete"

Total time: 200ms + 300ms + 400ms + 250ms = 1150ms
```

**Problem:** If EmailService is slow, the user waits. If AnalyticsService is down, registration fails.

### Asynchronous Approach (Event-Driven)

```
User → API: POST /register
API → Database: Save user
API → EventQueue: Publish "UserRegistered" event
API → User: "Registration complete" (immediate response)

Meanwhile, asynchronously:
EventQueue → EmailService: Send welcome email
EventQueue → AnalyticsService: Track signup
EventQueue → CRMService: Create lead

Total response time: 200ms (just database save)
```

**Benefit:** User gets immediate response. Services process independently. If EmailService is slow, it doesn't affect the user experience.

**The trade-off:** User doesn't immediately get the email. Analytics might be a few seconds behind. But for most use cases, this is acceptable.

## When to Use Event-Driven Architecture (And When NOT To)

After years of building both synchronous and asynchronous systems, I've developed a simple decision framework.

**Use event-driven when:**

1. **Services don't need immediate response** (e.g., sending emails, analytics, notifications)
2. **Multiple services need the same data** (e.g., "UserRegistered" → Email, Analytics, CRM all need it)
3. **Work can happen in the background** (e.g., image processing, report generation)
4. **Resilience matters more than immediate consistency** (e.g., "we'll retry until it works")
5. **You need to handle traffic spikes** (e.g., queue requests during peak, process during off-peak)

**Don't use event-driven when:**

1. **User needs immediate feedback** (e.g., "Is this username available?")
2. **Strong consistency is required** (e.g., banking transactions, inventory checks)
3. **Debugging simplicity matters more than scalability** (synchronous is easier to debug)
4. **You don't have operational maturity** (monitoring queues, handling failures)
5. **The added complexity isn't worth it** (simple CRUD apps don't need events)

The biggest mistake I see? Using events for everything because they're "more scalable." Architecture should solve specific problems, not create unnecessary complexity.

## The Three Event-Driven Patterns

Event-driven architecture isn't one thing—it's three different patterns for different use cases.

### Pattern 1: Message Queues (Point-to-Point)

**What it is:** A message is sent to a queue and processed by exactly one consumer.

**How it works:**
```
Producer → Queue → Consumer (only one)
```

**Use cases:**
- Background jobs (image resizing, video transcoding)
- Task distribution (send to whichever worker is free)
- Load leveling (queue requests during spikes, process gradually)

**Real example:** When you upload a video to YouTube, it goes into a queue. One worker picks it up and transcodes it. If 1000 people upload videos simultaneously, the queue holds them until workers are available.

**Technologies:** RabbitMQ, AWS SQS, Redis queues

### Pattern 2: Pub/Sub (Publish/Subscribe)

**What it is:** A message (event) is published to a topic. Multiple subscribers can receive a copy.

**How it works:**
```
Publisher → Topic → Subscriber 1
                 → Subscriber 2
                 → Subscriber 3
```

**Use cases:**
- Broadcasting events (e.g., "UserSignedUp" → Email, Analytics, CRM all get it)
- Event notification (multiple services react to the same event)
- Real-time updates (e.g., stock prices, sports scores)

**Real example:** When you sign up for Netflix, a "UserSignedUp" event is published. The email service sends a welcome email, the analytics service tracks the signup, and the recommendation service initializes your profile—all simultaneously, all independently.

**Technologies:** Apache Kafka, Google Pub/Sub, AWS SNS/SQS

### Pattern 3: Event Sourcing

**What it is:** Store all changes as a sequence of events, not just current state.

**How it works:**
```
Instead of: User { name: "John", email: "john@example.com" }

Store: [
  { type: "UserCreated", data: { id: 1, name: "John" } },
  { type: "EmailUpdated", data: { email: "john@example.com" } }
]
```

**Use cases:**
- Audit trails (every change is recorded)
- Time-travel debugging (replay events to see what happened)
- Event replay (rebuild state from events if database corrupts)

**Real example:** Financial systems. Instead of just storing "Account balance: $1000", you store "Deposit $500", "Withdraw $200", "Deposit $700". You can replay these events to verify the balance or undo transactions.

**Technologies:** EventStore, Apache Kafka (with compacted topics)

**Which pattern to choose?**
- **Queues** for one consumer, background jobs
- **Pub/Sub** for multiple consumers, event broadcasting
- **Event Sourcing** for audit trails, complex state changes

## Real-World Case Studies

### Netflix: Events for Resilience

Netflix's event-driven architecture is legendary. Here's how they use events:

**The Challenge:** When you play a video, dozens of things need to happen: stream initialization, quality selection, analytics tracking, recommendation updates, etc.

**The Synchronous Problem:** If analytics tracking is slow, video playback shouldn't suffer. If recommendations are down, you should still be able to watch.

**The Event-Driven Solution:**
1. "VideoPlaybackStarted" event is published
2. Multiple services subscribe independently:
   - Analytics Service: Track viewing habits
   - Recommendation Service: Update "continue watching"
   - Billing Service: Track usage for account sharing
   - Quality Service: Monitor stream health

**The Result:** If Analytics Service crashes, video playback continues. Each service operates independently. Netflix achieves 99.99%+ uptime.

### LinkedIn: The Migration to Events

LinkedIn's journey to event-driven architecture is instructive:

**The Problem (2010):** Synchronous calls everywhere. The "social graph" service was called by dozens of other services. When it was slow, LinkedIn was slow.

**The Solution:**
1. Identified the most-called services
2. Gradually migrated to event-driven architecture using Kafka
3. Services now react to events instead of calling each other

**The Result:**
- 10x improvement in response times
- Ability to handle 4x more traffic with same infrastructure
- Services can fail independently without taking down the site

**The Lesson:** Migrate gradually, not all at once. Start with the most painful synchronous dependencies.

### The Startup That Over-Engineered Events

Not every story is a success. A startup I advised went all-in on events from day one:

**What They Did:**
- Kafka cluster with 10 brokers
- Event sourcing for everything (even simple CRUD)
- 50+ event types for an MVP

**The Result:**
- Spent 6 months building infrastructure before shipping features
- Debugging was nightmare (which event caused this bug?)
- Operational overhead crushed a small team

**The Lesson:** Events are powerful, but don't start with them. Evolve into event-driven architecture when you feel the pain of not having it.

## Common Event-Driven Mistakes

After years of working with event-driven systems, I've seen these patterns repeat:

**Mistake 1: Eventual Consistency Confusion**

"We'll just use events and everything will be consistent eventually!"

**Reality:** Eventual consistency means your data is inconsistent for some period. Users might not see their changes immediately. If you need strong consistency (e.g., banking), events aren't the answer.

**Mistake 2: Event Spaghetti**

"We'll publish events for everything!"

**Reality:** 200 event types create chaos. Which events do I subscribe to? What happens when event schemas change? Keep events minimal and well-documented.

**Mistake 3: No Retry Logic**

"If an event fails, we'll just retry forever!"

**Reality:** Some events can't succeed (e.g., email to invalid address). You need dead letter queues, exponential backoff, and failure handling.

**Mistake 4: Synchronous Events**

"We'll use events, but wait for the response!"

**Reality:** That's not event-driven, that's synchronous calls with extra steps. Either commit to async or use synchronous calls.

**Mistake 5: Ignoring Event Ordering**

"Order doesn't matter!"

**Reality:** "UserCreated" must come before "EmailUpdated". Use partitioning or sequencing to maintain order when it matters.

## Modeling Events in Sruja

Now that you understand the concepts, let's see how to model event-driven architecture in Sruja. The key is using `queue` for asynchronous communication and `scenario` for event flows.

### Example: User Registration with Events

```sruja
import { * } from 'sruja.ai/stdlib'

User = person "End User"

Notifications = system "Notification System" {
    AuthService = container "Auth Service" {
        technology "Node.js"
        description "Handles user authentication and publishes events"
    }

    // Define the event queue/topic
    UserEvents = queue "User Events Topic" {
        technology "Kafka"
        description "Events: UserSignedUp, UserLoggedIn, ProfileUpdated"
    }

    EmailService = container "Email Service" {
        technology "Python"
        description "Sends transactional emails asynchronously"
    }

    AnalyticsService = container "Analytics Service" {
        technology "Spark"
        description "Processes user events for analytics"
    }

    NotificationDB = database "Notification Database" {
        technology "PostgreSQL"
        description "Stores notification preferences and history"
    }

    // Pub/Sub flow: One event, multiple consumers
    User -> AuthService "Signs up (synchronous)"
    AuthService -> UserEvents "Publishes 'UserSignedUp' (async)"
    UserEvents -> EmailService "Consumes event - sends welcome email"
    UserEvents -> AnalyticsService "Consumes event - tracks signup"
    EmailService -> NotificationDB "Logs email sent"
}

// Model the complete event flow as a scenario
UserSignupFlow = scenario "User Signup Event Flow" {
    User -> AuthService "Submits registration (synchronous)"
    AuthService -> UserEvents "Publishes UserSignedUp (async)"
    UserEvents -> EmailService "Triggers welcome email (async)"
    UserEvents -> AnalyticsService "Tracks signup event (async)"
    EmailService -> User "Sends welcome email (async)"
}

// Model data pipeline for analytics
flow AnalyticsPipeline "Analytics Data Pipeline" {
    UserEvents -> AnalyticsService "Streams events continuously"
    AnalyticsService -> AnalyticsService "Processes in batches"
    AnalyticsService -> AnalyticsService "Generates reports"
}

view index {
    title "Notification System Overview"
    include *
}

// Event flow view: Focus on async communication
view eventflow {
    title "Event Flow View - Async Communication"
    include Notifications.AuthService
    include Notifications.UserEvents
    include Notifications.EmailService
    include Notifications.AnalyticsService
    exclude User Notifications.NotificationDB
}

// Data view: Focus on data storage
view data {
    title "Data Storage View"
    include Notifications.EmailService
    include Notifications.NotificationDB
    include Notifications.AnalyticsService
    exclude Notifications.AuthService Notifications.UserEvents
}
```

### Key Sruja Concepts for Events

1. **`queue`** - Models message queues and pub/sub topics
2. **`scenario`** - Models behavioral flows (user journeys, event sequences)
3. **`flow`** - Models data pipelines (streaming, batch processing)
4. **`views`** - Different perspectives for different audiences

**Notice the separation:**
- Synchronous calls: `User -> AuthService` (user waits)
- Asynchronous events: `AuthService -> UserEvents` (fire and forget)

## What to Remember

**Synchronous chains create fragility.** When every service calls every other service synchronously, one slow service slows everything. Event-driven architecture breaks these chains.

**Events trade consistency for resilience.** You gain independence and fault tolerance, but data isn't immediately consistent across services. This is acceptable for many use cases, but not all.

**Three patterns for three problems:**
- **Queues** for background jobs, one consumer
- **Pub/Sub** for broadcasting events, multiple consumers
- **Event Sourcing** for audit trails, state reconstruction

**Events aren't always the answer.** Use them when you need decoupling, resilience, or async processing. Don't use them when you need immediate feedback or strong consistency.

**Start synchronous, evolve to async.** Most successful companies (Netflix, LinkedIn, Uber) started with synchronous calls and migrated to events when they felt the pain. Don't over-engineer from day one.

**Model events explicitly in Sruja.** Use `queue` for topics, `scenario` for flows, and `views` to show different perspectives. Make the async boundaries clear.

Event-driven architecture isn't a silver bullet. It's a powerful tool for specific problems. Use it when the trade-offs make sense.

## What's Next

Now that you understand event-driven architecture, [Lesson 3](lesson-3.md) covers Advanced Scenarios—how to model complex user journeys and technical sequences that span multiple services. You'll learn when scenarios help clarify behavior and when they're just extra documentation.
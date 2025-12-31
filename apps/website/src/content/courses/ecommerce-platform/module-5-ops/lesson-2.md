---
title: "Lesson 2: Debugging Performance (Structural Analysis)"
weight: 2
summary: "Using your architecture diagram to find bottlenecks."
---

# Lesson 2: Debugging Performance

**Scenario**: It's Black Friday. The "Checkout" page is loading in 5 seconds. Why?

## The Wrong Way

Start reading random logs or guessing which database query is slow.

## The Structural Way

Look at your Sruja **User Journey** for "Purchase".

```sruja
element person
element system
element container
element queue

Customer = person "Customer"

Platform = system "E-Commerce Platform" {
Checkout = container "Checkout Service"
PaymentWorker = container "Payment Worker"
PaymentQueue = queue "Payment Jobs"
}

PaymentGateway = system "Payment Gateway" {
external true
}

// Original synchronous flow (problematic)
story Purchase "User Purchase Flow" {
Customer -> Platform.Checkout "Initiates checkout"
Platform.Checkout -> PaymentGateway "Process Payment" {
  latency "2s"
}
PaymentGateway -> Customer "Returns confirmation"
}
```

Wait, the `PaymentGateway` call is synchronous and takes 2 seconds? And it's in the critical path of the user request?

**Root Cause**: We are blocking the user while waiting for the bank.

## The Fix: Asynchronous Processing

We need to decouple the user request from the payment processing.

1.  **Introduce a Queue**: The Checkout service puts a message on a queue.
2.  **Worker**: A background worker processes the payment.
3.  **Update**: The frontend polls for status or uses WebSockets.

Let's update the architecture:

```sruja
element person
element system
element container
element queue

Customer = person "Customer"

Platform = system "E-Commerce Platform" {
Checkout = container "Checkout Service"
PaymentWorker = container "Payment Worker"
PaymentQueue = queue "Payment Jobs"
}

PaymentGateway = system "Payment Gateway" {
external true
}

// Updated asynchronous flow
Customer -> Platform.Checkout "Initiates checkout"
Platform.Checkout -> Platform.PaymentQueue "Enqueues payment job" {
latency "10ms"
}
Platform.PaymentQueue -> Platform.PaymentWorker "Processes async"
Platform.PaymentWorker -> PaymentGateway "Processes payment"
PaymentGateway -> Customer "Sends confirmation email"

// Updated scenario
story PurchaseAsync "Asynchronous Purchase Flow" {
Customer -> Platform.Checkout "Initiates checkout"
Platform.Checkout -> Platform.PaymentQueue "Enqueues job" {
  latency "10ms"
}
Platform.PaymentQueue -> Platform.PaymentWorker "Processes async"
Platform.PaymentWorker -> PaymentGateway "Processes payment"
PaymentGateway -> Customer "Sends confirmation"
}

view index {
title "Payment Processing Architecture"
include *
}

// Performance view: Focus on async flow
view performance {
title "Performance View - Async Processing"
include Platform.Checkout Platform.PaymentQueue Platform.PaymentWorker PaymentGateway
exclude Customer
}
```

By visualizing the flow, the bottleneck (and the fix) becomes obvious.

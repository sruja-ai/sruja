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
story Purchase {
    // ...
    Checkout -> PaymentGateway "Process Payment" {
        latency "2s"
    }
    // ...
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
system Platform {
    queue PaymentQueue "Payment Jobs"
    
    // Updated Flow
    Checkout -> PaymentQueue "Enqueue Job" { latency "10ms" }
    PaymentQueue -> PaymentWorker "Process Async"
}
```

By visualizing the flow, the bottleneck (and the fix) becomes obvious.

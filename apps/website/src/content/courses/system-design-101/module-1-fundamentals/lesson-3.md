---
title: "Lesson 3: Availability & Reliability"
weight: 3
summary: "Redundancy, Failover, and SLAs."
---

# Lesson 3: Availability & Reliability

## Reliability vs. Availability

- **Reliability:** The probability that a system will function correctly without failure for a specified period. It's about _correctness_.
- **Availability:** The percentage of time a system is operational and accessible. It's about _uptime_.

A system can be available but not reliable (e.g., it returns 500 errors but is "up").

## Measuring Availability

Availability is often measured in "nines":

| Availability         | Downtime per Year |
| :------------------- | :---------------- |
| 99% (Two nines)      | 3.65 days         |
| 99.9% (Three nines)  | 8.76 hours        |
| 99.99% (Four nines)  | 52.6 minutes      |
| 99.999% (Five nines) | 5.26 minutes      |

## Achieving High Availability

### Redundancy

The key to availability is eliminating Single Points of Failure (SPOF). This is done via redundancy.

- **Active-Passive:** One server handles traffic; the other is on standby.
- **Active-Active:** Both servers handle traffic. If one fails, the other takes over the full load.

### Failover

The process of switching to a redundant system upon failure. This can be manual or automatic.

---

## 🛠️ Sruja Perspective: Modeling Redundancy

You can explicitly model redundant components in Sruja to visualize your high-availability strategy.

```sruja
element person
element system
element container
element component
element datastore
element queue

Payments = system "Payment System" {
    PaymentService = container "Payment Service" {
        technology "Java"
    }

    // Modeling a primary and standby database
    PrimaryDB = container "Primary Database" {
        technology "MySQL"
        tags ["primary"]
    }

    StandbyDB = container "Standby Database" {
        technology "MySQL"
        tags ["standby"]
        description "Replicates from PrimaryDB. Promoted to primary if PrimaryDB fails."
    }

    PaymentService -> PrimaryDB "Reads/Writes"
    PrimaryDB -> StandbyDB "Replicates data"
}

view index {
include *
}
```

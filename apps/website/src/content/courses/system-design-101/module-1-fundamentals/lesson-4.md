---
title: "Lesson 4: CAP Theorem & Consistency"
weight: 4
summary: "Consistency, Availability, and Partition Tolerance."
---

# Lesson 4: CAP Theorem & Consistency

## The CAP Theorem

Proposed by Eric Brewer, the CAP theorem states that a distributed data store can only provide **two** of the following three guarantees:

1.  **Consistency (C):** Every read receives the most recent write or an error. All nodes see the same data at the same time.
2.  **Availability (A):** Every request receives a (non-error) response, without the guarantee that it contains the most recent write.
3.  **Partition Tolerance (P):** The system continues to operate despite an arbitrary number of messages being dropped or delayed by the network between nodes.

### The Reality: P is Mandatory
In a distributed system, network partitions (P) are inevitable. Therefore, you must choose between **Consistency (CP)** and **Availability (AP)** when a partition occurs.

*   **CP (Consistency + Partition Tolerance):** Wait for data to sync. If a node is unreachable, return an error. (e.g., Banking systems).
*   **AP (Availability + Partition Tolerance):** Return the most recent version of data available, even if it might be stale. (e.g., Social media feeds).

## Consistency Models

*   **Strong Consistency:** Once a write is confirmed, all subsequent reads see that value.
*   **Eventual Consistency:** If no new updates are made, eventually all accesses will return the last updated value. (Common in AP systems).

---

## 🛠️ Sruja Perspective: Documenting Guarantees

When defining data stores in Sruja, it is helpful to document their consistency guarantees, especially for distributed databases.

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
    DataLayer = system "Data Layer" {
        UserDB = container "User Database" {
            technology "Cassandra"
            // Explicitly stating the consistency model
            description "configured with replication factor 3. Uses eventual consistency for high availability."
            
            // You could also use custom tags
            tags ["AP-System", "Eventual-Consistency"]
        }
        
        BillingDB = container "Billing Database" {
            technology "PostgreSQL"
            description "Single primary with synchronous replication to ensure strong consistency."
            tags ["CP-System", "Strong-Consistency"]
        }
    }
}

views {
  view index {
    include *
  }
}
```

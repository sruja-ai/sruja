---
title: "Lesson 1: Design a Distributed Counter"
weight: 1
summary: "Sharding, Write-Behind, Eventual Consistency."
---

# Lesson 1: Design a Distributed Counter

**Goal:** Design a system to count events (e.g., YouTube views, Facebook likes) at a massive scale (e.g., 1 million writes/sec).

## The Problem with a Single Database
A standard SQL database (like PostgreSQL) can handle ~2k-5k writes/sec. If we try to update a single row (`UPDATE videos SET views = views + 1 WHERE id = 123`) for every view, the database will lock the row and become a bottleneck.

## Solutions

### 1. Sharding (Write Splitting)
Instead of one counter, have $N$ counters for the same video.
*   Randomly pick a counter from $1$ to $N$ and increment it.
*   **Total Views** = Sum of all $N$ counters.

### 2. Write-Behind (Batching)
Don't write to the DB immediately.
*   Store counts in memory (Redis) or a log (Kafka).
*   A background worker aggregates them and updates the DB every few seconds.
*   *Trade-off:* If the server crashes before flushing, you lose a few seconds of data (Eventual Consistency).

---

## 🛠️ Sruja Perspective: Modeling Write Flows

We can use Sruja to model the "Write-Behind" architecture.

```sruja
architecture "Distributed Counter" {
    system CounterService "View Counter" {
        container API "Ingestion API" {
            technology "Go"
            description "Receives 'view' events"
        }
        
        queue EventLog "Kafka" {
            description "Buffers raw view events"
        }
        
        container Worker "Aggregator" {
            technology "Python"
            description "Reads batch of events, sums them, updates DB"
            scale { min 5 }
        }
        
        datastore DB "Counter DB" {
            technology "Cassandra"
            description "Stores final counts (Counter Columns)"
        }
        
        container Cache "Read Cache" {
            technology "Redis"
            description "Caches total counts for fast reads"
        }

        API -> EventLog "Produces events"
        Worker -> EventLog "Consumes events"
        Worker -> DB "Updates counts"
        Worker -> Cache "Updates cache"
    }

    person User "Viewer"

    // Write Path (Eventual Consistency)
    scenario TrackView "User watches a video" {
        User -> API "POST /view"
        API -> EventLog "Produce Event"
        API -> User "202 Accepted"
        
        // Async processing
        EventLog -> Worker "Consume Batch"
        Worker -> DB "UPDATE views += batch_size"
        Worker -> Cache "Invalidate/Update"
    }
}
```

---
title: "Lesson 2: Databases"
weight: 2
summary: "SQL vs NoSQL, Replication, Sharding, Real-World Case Studies"
learning_objectives:
  - Understand the difference between SQL and NoSQL databases
  - Learn replication strategies and when to use each
  - Master sharding patterns for massive scale
  - See real-world examples from Netflix, Airbnb, and Uber
  - Practice making database trade-off decisions
estimated_time: "25 minutes"
difficulty: "beginner"
---

# Lesson 2: Databases

## SQL vs. NoSQL: The Fundamental Choice

Choosing between SQL and NoSQL is one of the most important architecture decisions. Here's how to think about it.

### SQL (Relational Databases)

**What it does:** Stores structured data in tables with predefined schemas using the relational model.

**Characteristics:**
- ✅ **Structured data** - Tables, rows, columns with defined types
- ✅ **ACID compliance** - Strong guarantees for Atomicity, Consistency, Isolation, Durability
- ✅ **Powerful querying** - Complex joins, aggregations, transactions
- ✅ **Data integrity** - Foreign keys, constraints, referential integrity
- ❌ **Vertical scaling** - Limited to single server (mostly)
- ❌ **Schema changes** - ALTER TABLE can be slow/complex

**Examples:** PostgreSQL, MySQL, Oracle, SQL Server

**Best for:**
- Financial transactions (banking, payments)
- Complex queries with joins and aggregations
- Strong consistency requirements
- Data relationships (e.g., user orders and order items)

### NoSQL (Non-Relational Databases)

**What it does:** Flexible data models without rigid schemas, designed for horizontal scaling.

**Characteristics:**
- ✅ **Flexible schemas** - Add fields on the fly
- ✅ **Horizontal scaling** - Distribute data across multiple servers
- ✅ **High performance** - Optimized for specific access patterns
- ✅ **Multiple models** - Document, Key-Value, Graph, Column-family
- ❌ **Weaker consistency** - Often eventual consistency
- ❌ **Limited querying** - Complex joins harder or impossible

**Examples:**
- **Document:** MongoDB, CouchDB
- **Key-Value:** Redis, DynamoDB
- **Column-family:** Cassandra, HBase
- **Graph:** Neo4j, JanusGraph

**Best for:**
- Rapidly evolving data structures
- Massive scale (PBs of data, millions of users)
- High throughput write workloads
- Hierarchical or graph data

### Decision Matrix: SQL vs NoSQL

| Factor | Choose SQL When... | Choose NoSQL When... |
|--------|-------------------|---------------------|
| **Data structure** | Fixed, well-defined schema | Flexible, evolving schema |
| **Consistency** | Strong consistency required | Eventual consistency OK |
| **Scale** | Moderate (up to TBs) | Massive (TBs to PBs) |
| **Complexity** | Complex queries, joins | Simple access patterns |
| **Transactions** | ACID transactions needed | Eventual consistency OK |
| **Speed** | Complex queries take time | Fast for specific patterns |

## Scaling Databases

### Replication: Copying Data

Replication copies data across multiple servers for redundancy and performance.

#### Master-Slave (Primary-Replica)

**How it works:** All writes go to the master. Reads can go to slaves (read replicas).

```
Write → Master DB (Primary)
         ↓ (Replication)
         → Slave 1 (Read Replica)
         → Slave 2 (Read Replica)
         → Slave 3 (Read Replica)

Read → Can go to any slave
```

**Pros:**
- ✅ **Read scalability** - Distribute read load across replicas
- ✅ **Data redundancy** - Multiple copies for backup/failover
- ✅ **Improved read latency** - Read from geographically closer replica
- ✅ **Simplifies backups** - Backup from replicas, not production

**Cons:**
- ❌ **Single write bottleneck** - All writes go through master
- ❌ **Replication lag** - Slaves might be slightly behind master
- ❌ **Complex failover** - Promoting a replica to master needs care
- ❌ **Write scaling** - Cannot scale writes (still single master)

**Real-world use:** Read-heavy systems (e.g., product catalog, user profiles, content delivery)

**Performance impact:**
- Read throughput: 10x improvement (1 master → 10 replicas)
- Write throughput: No improvement (still limited by master)
- Replication lag: Typically 10-100ms for local replicas

#### Master-Master (Active-Active)

**How it works:** Writes can go to any node. Changes are propagated between masters.

```
Write → Master 1 → Master 2 → Master 3
Write → Master 2 → Master 1 → Master 3
Write → Master 3 → Master 1 → Master 2

Read → Can go to any master
```

**Pros:**
- ✅ **Write scalability** - Distribute writes across masters
- ✅ **No single point of failure** - Any master can accept writes
- ✅ **Geographic distribution** - Masters in different regions for local writes

**Cons:**
- ❌ **Conflict resolution** - Two masters writing same row simultaneously
- ❌ **Complex setup** - More complex to configure and maintain
- ❌ **Eventual consistency** - Harder to maintain strong consistency

**Real-world use:** Global applications needing local writes in each region (e.g., social media feeds)

**Performance impact:**
- Write throughput: 3-5x improvement (1 master → 3-5 masters)
- Conflict rate: Depends on write patterns (0.1-5% of writes may conflict)
- Consistency window: 100-500ms for cross-region replication

### Sharding: Partitioning Data

Sharding (partitioning) splits data across multiple database servers based on a shard key.

#### Horizontal Sharding

**How it works:** Split rows of a table across servers based on a shard key.

```
Users Table (100M rows)
├── Shard 1: Users A-G (25M users) → DB Server 1
├── Shard 2: Users H-N (25M users) → DB Server 2
├── Shard 3: Users O-T (25M users) → DB Server 3
└── Shard 4: Users U-Z (25M users) → DB Server 4
```

**Pros:**
- ✅ **Write scalability** - Distribute writes across shards
- ✅ **Massive scale** - Handle petabytes of data
- ✅ **Parallel queries** - Query multiple shards in parallel
- ✅ **Independent scaling** - Scale hot shards independently

**Cons:**
- ❌ **Complex joins** - Cross-shard joins are hard/expensive
- ❌ **Rebalancing** - Moving data between shards is complex
- ❌ **Query complexity** - Application needs to route queries to right shard
- ❌ **Hot spots** - Popular shard keys can create imbalances

**Real-world use:** User data, time-series data, any massive dataset

**Performance impact:**
- Write throughput: 10x improvement (1 server → 10 shards)
- Query performance: Depends on shard key (local shard = fast, cross-shard = slow)
- Rebalancing time: Days to weeks for large datasets

#### Consistent Hashing

**How it works:** Use a hash ring to determine shard, minimizing data movement when adding/removing shards.

```
Hash Ring: [0, 1, 2, ..., 1023]
├── Shard 1: 0-255
├── Shard 2: 256-511
├── Shard 3: 512-767
└── Shard 4: 768-1023

User "alice" → hash("alice") % 1024 = 342 → Shard 2
User "bob"   → hash("bob") % 1024 = 789 → Shard 4
```

**Pros:**
- ✅ **Minimal rebalancing** - Adding a shard moves ~1/n of data
- ✅ **Even distribution** - Hash function spreads data evenly
- ✅ **Deterministic** - Same key always routes to same shard

**Cons:**
- ❌ **Range queries hard** - Finding users A-G requires querying all shards
- ❌ **Hash collisions** - Poor hash function can create imbalances
- ❌ **No hot spot handling** - Popular keys still create imbalances

**Real-world use:** Key-value stores, caching, session storage

## Real-World Case Studies

### Case Study 1: Netflix's Migration to Cassandra

**The Challenge:** Netflix grew from DVD-by-mail to streaming, moving from a single database to needing to handle billions of play events per day.

**Before (Single PostgreSQL):**
- Single master database
- 100M subscribers
- Limited by single server capacity
- SPOF (Single Point of Failure)

**The Migration Journey:**

**Phase 1: Master-Slave Replication**
```
PostgreSQL Master (writes) → 5 Read Replicas
                               ↓ (replication lag: 50-100ms)
```
- Result: 5x read scalability, writes still bottlenecked
- Problem: Still can't handle streaming scale

**Phase 2: Sharded PostgreSQL**
- Sharded by subscriber ID (100 shards)
- Each shard: 1M subscribers
- Result: Write scalability improved, but complex cross-shard queries
- Problem: Manual rebalancing, hot shard issues

**Phase 3: Migration to Cassandra (Final Solution)**
```
Cassandra Cluster (100 nodes)
├── Datacenter US-East (50 nodes)
├── Datacenter US-West (50 nodes)

Data Model:
├── Play Events Table (sharded by user_id)
├── Watch History Table (sharded by user_id)
└── Catalog Table (replicated 3x, no sharding)
```

**Key Decisions:**
1. **Cassandra choice** - NoSQL for horizontal scalability
2. **Multi-datacenter** - Geographic redundancy and local reads
3. **Consistency level** - QUORUM for reads/writes (balanced)
4. **Replication factor** - 3 (2 datacenters + 1 local replica)

**The Results:**
- **Scale:** 100M → 260M subscribers
- **Play events:** 1B+ events/day (vs 10M with PostgreSQL)
- **Availability:** 99.99% (was 99.9% with PostgreSQL)
- **Latency:** p99 latency <200ms (vs 500ms with sharded PostgreSQL)
- **Data volume:** 10+ PB of data (would be impossible with SQL)

> **💡 Key Insight:** Netflix chose Cassandra for its write-heavy workload (play events). They kept some data in PostgreSQL for complex queries (billing, analytics) because different workloads benefit from different database technologies.

### Case Study 2: Airbnb's Hybrid Database Strategy

**The Challenge:** Airbnb needed to handle rapid growth (1M → 100M+ users) while maintaining complex relationships between listings, bookings, and reviews.

**The Architecture:**
```
Airbnb Platform
├── PostgreSQL (Core Business)
│   ├── Users Table
│   ├── Listings Table
│   ├── Bookings Table
│   └── Reviews Table
│       └── Replicated (Master + 3 Read Replicas)
│
└── MongoDB (Flexible Data)
    ├── Search Index
    ├── Message Queue
    └── Activity Feed
```

**Why PostgreSQL for Core Business?**

1. **Data relationships matter**
   - Listing → Bookings → Reviews (complex joins)
   - User → Listings → Reviews (many-to-many relationships)
   - Foreign keys ensure referential integrity

2. **ACID transactions**
   - Booking = Create booking record + Update availability + Send notification
   - All-or-nothing: booking succeeds or fails completely
   - No partial bookings or data inconsistencies

3. **Complex queries**
   - "Find listings with availability in date range" (complex WHERE clauses)
   - "Get user's booking history with reviews" (joins + aggregations)
   - "Calculate host revenue" (complex analytics)

**Why MongoDB for Flexible Data?**

1. **Evolving schema**
   - Search index fields change frequently (add filters, sorting)
   - Message queue format evolves (add metadata, change structure)
   - No ALTER TABLE downtime

2. **Horizontal scaling**
   - Search index: 100M+ listings, complex queries
   - Activity feed: Millions of updates/second
   - Sharding easier than PostgreSQL

3. **Performance patterns**
   - Search: Read-heavy, simple queries
   - Activity feed: Write-heavy, append-only
   - Both benefit from NoSQL optimizations

**Performance Results:**
- **PostgreSQL:** 50K writes/sec, 500K reads/sec (complex queries)
- **MongoDB:** 200K writes/sec, 2M reads/sec (simple queries)
- **Overall latency:** p95 <100ms for core flows

> **💡 Key Insight:** Airbnb uses the **right tool for the right job**. PostgreSQL for complex relational data, MongoDB for flexible, high-throughput patterns. This hybrid approach gives them both data integrity and scalability.

### Case Study 3: Uber's Sharding Evolution

**The Challenge:** Uber grew from a single PostgreSQL database to processing millions of trips per second globally.

**Stage 1: Single PostgreSQL**
- Single database in one datacenter
- 1M trips/day
- Bottleneck: Single server capacity
- Risk: Datacenter outage = complete downtime

**Stage 2: Sharded PostgreSQL**
- Sharded by city_id (US trips in US DB, Europe trips in Europe DB)
- 100M trips/day
- Benefits: Geographic isolation, write scalability
- Problem: Complex cross-city queries (traveler rides in multiple cities)

**Stage 3: Schemaless Sharding (The Pivot)**
- Moved to MySQL with schemaless design (JSONB columns)
- Sharded by rider_id (each rider's data on one shard)
- 1B+ trips/day
- Benefits: Schema flexibility, better shard distribution
- Problem: Still hard to evolve, complex migrations

**Stage 4: Multi-Database Strategy (Current)**
```
Uber Platform
├── MySQL (Transactional Data)
│   ├── Trips Table (sharded by city)
│   ├── Users Table (sharded by user_id)
│   └── Payments Table (sharded by ride_id)
│
├── Cassandra (Time-Series Data)
│   ├── Event Stream (10M+ events/sec)
│   └── Telemetry (vehicle location updates)
│
├── Redis (Real-Time Data)
│   ├── Driver Location (updates every 2 sec)
│   └── Surge Pricing (calculations every 5 sec)
│
└── ElasticSearch (Search)
    ├── Location Search (geospatial queries)
    └── Destination Search (autocomplete)
```

**Key Decisions:**
1. **MySQL for transactions** - Trip data needs ACID guarantees
2. **Cassandra for time-series** - Event stream is write-heavy (append-only)
3. **Redis for real-time** - Driver locations need <100ms latency
4. **ElasticSearch for search** - Geospatial queries require specialized search

**Performance Results:**
- **Trips processed:** 1B+ trips/day (from 1M)
- **Event stream:** 10M+ events/sec (Cassandra handles write load)
- **Real-time data:** 10M+ driver locations/sec (Redis provides <50ms latency)
- **Search:** <100ms p95 for location search (ElasticSearch optimization)

> **💡 Key Insight:** Uber doesn't try to fit everything in one database. Different workloads (transactions, time-series, real-time, search) benefit from different database technologies. The complexity is higher, but the performance and scalability are unmatched.

## Production Metrics

### Database Performance Comparison

| Database | Write Throughput | Read Latency (p95) | Scale |
|----------|------------------|---------------------|-------|
| **PostgreSQL** | 50K writes/sec | 10-50ms | TBs |
| **MySQL** | 100K writes/sec | 10-40ms | TBs |
| **MongoDB** | 500K writes/sec | 5-20ms | PBs |
| **Cassandra** | 1M+ writes/sec | 20-100ms | PBs |
| **Redis** | 10M+ writes/sec | <5ms | GBs-TBs |

### Replication Performance

| Strategy | Write Latency Impact | Read Scalability | Consistency |
|----------|---------------------|-----------------|------------|
| **Master-Slave** | +5-10ms | 10x | Strong |
| **Master-Master** | +50-200ms | 3-5x | Eventual |
| **Multi-Master** | +100-500ms | 5-10x | Eventual |

### Sharding Performance

| Strategy | Write Throughput Improvement | Query Performance | Rebalancing |
|----------|----------------------------|-----------------|-------------|
| **Horizontal (range)** | 5-10x | Fast (local), Slow (cross-shard) | Days-Weeks |
| **Consistent Hashing** | 8-12x | Medium (range queries slow) | Minimal |
| **Geographic** | 2-5x | Very fast (local reads) | Complex |

## Trade-Off Scenarios

### Scenario 1: E-Commerce Platform

**Context:** Building Amazon-scale e-commerce platform. Need to handle Black Friday traffic spikes (10x normal). Data: product catalog, user accounts, orders, reviews.

**The Trade-Off Decisions:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Product Catalog** | PostgreSQL (complex queries) | MongoDB (flexible schema) | **PostgreSQL** - Complex filters, joins, ACID for inventory |
| **User Sessions** | PostgreSQL (sessions table) | Redis (key-value) | **Redis** - Fast access, TTL for expiration |
| **Orders** | PostgreSQL (ACID transactions) | MongoDB (eventual consistency) | **PostgreSQL** - Financial data needs strong consistency |
| **Reviews** | PostgreSQL (relational) | MongoDB (document) | **MongoDB** - Flexible schema, high write throughput |
| **Replication** | Master-Slave (read-heavy) | Master-Master (write-heavy) | **Master-Slave** - 90% reads, strong consistency for orders |
| **Sharding** | Not needed (scale vertically) | Yes (future-proof) | **Yes** - Black Friday traffic requires horizontal scale |

**Result:**
- **Pros:** Right database for each workload, strong consistency for transactions, scalability for Black Friday
- **Cons:** Complex architecture (4 databases), cross-database joins impossible
- **Performance:** 1M+ products, 100K+ concurrent users, p95 <100ms

### Scenario 2: Social Media Feed

**Context:** Building a real-time social media feed like Twitter. Users post updates, followers see them in timeline. Requirements: millions of posts/sec, low latency, global availability.

**The Trade-Off Decisions:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Posts Storage** | PostgreSQL (relational) | Cassandra (time-series) | **Cassandra** - Write-heavy (append-only), time-series access pattern |
| **Timeline Query** | PostgreSQL (complex joins) | Denormalized (pre-computed timelines) | **Denormalized** - Pre-compute user timelines, no complex joins |
| **Likes/Follows** | PostgreSQL (relational) | Redis (fast counters) | **Redis** - High frequency reads/writes, simple counters |
| **User Profiles** | PostgreSQL (ACID needed) | MongoDB (flexible) | **PostgreSQL** - Profile needs ACID (security, privacy) |
| **Replication** | Master-Slave (eventual OK) | Multi-Master (global writes) | **Multi-Master** - Global users need local writes |
| **Consistency** | Strong | Eventual | **Eventual** - OK if likes show with slight delay, but posts appear quickly |

**Result:**
- **Pros:** Massive write scalability, global availability, low latency for feed
- **Cons:** Eventual consistency (users might see stale data), complex architecture
- **Performance:** 10M+ posts/sec, p99 feed latency <200ms, 99.99% availability

### Scenario 3: Analytics Platform

**Context:** Building an analytics platform processing billions of events/day. Need to store raw events, compute aggregates in real-time, and serve ad-hoc queries.

**The Trade-Off Decisions:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Raw Events** | PostgreSQL (structured) | Cassandra (time-series) | **Cassandra** - Write-heavy (append-only), automatic TTL, time-series partitioning |
| **Aggregates** | PostgreSQL (materialized views) | Redis (pre-computed) | **Redis** - Fast access, TTL for expiration, update on new events |
| **Ad-hoc Queries** | PostgreSQL (complex SQL) | ElasticSearch (search/analytics) | **ElasticSearch** - Fast aggregations, full-text search, scalable |
| **User Metadata** | PostgreSQL (ACID needed) | MongoDB (flexible) | **PostgreSQL** - User accounts need ACID (security, permissions) |
| **Data Retention** | Manual cleanup | TTL-based automatic | **TTL-based** - Automate cleanup (billions of events/day) |
| **Consistency** | Strong | Eventual | **Eventual** - OK if aggregates are slightly stale, but raw events must be durable |

**Result:**
- **Pros:** Massive scale (PBs of data), automatic retention, fast aggregations
- **Cons:** Eventual consistency (aggregates might be behind), complex ETL pipelines
- **Performance:** 10B+ events/day, p95 query latency <500ms, 99.9% data durability

## Sruja Perspective: Modeling Databases

In Sruja, we document database choices with clear trade-offs and performance characteristics.

### Why Model Databases?

Modeling databases in your architecture provides:

1. **Technology choice clarity** - Document why SQL vs NoSQL
2. **Scaling strategy** - Show replication/sharding approach
3. **Performance visibility** - Track throughput, latency, scale
4. **Failure analysis** - Understand SPOFs and failover scenarios

### Example: E-Commerce Database Architecture

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce Platform" {
    description "Multi-database architecture for different workloads"
    
    // POSTGRESQL: Core transactional data
    PrimaryDB = database "Primary PostgreSQL" {
        technology "PostgreSQL 15"
        description "Stores users, products, orders - requires ACID"
        tags ["sql", "transactional"]
        
        tradeoff {
            decision "Use PostgreSQL for core business data"
            sacrifice "Write scalability (single master for writes)"
            reason "Strong consistency required for orders/payments, complex queries needed for product catalog"
            mitigation "Read replicas for read scalability, plan for sharding"
        }
        
        capacity {
            storage "10 TB"
            throughput_writes "50K/sec"
            throughput_reads "500K/sec"
            replication "Master + 3 Read Replicas"
        }
    }
    
    // MONGODB: Flexible, high-throughput data
    ReviewsDB = database "MongoDB Cluster" {
        technology "MongoDB 7"
        description "Stores reviews, activity feed - flexible schema, high writes"
        tags ["nosql", "document", "sharded"]
        
        tradeoff {
            decision "Use MongoDB for reviews and activity feed"
            sacrifice "Strong consistency (eventual consistency OK)"
            reason "Flexible schema for evolving data, high write throughput, easy sharding"
            mitigation "Write-ahead log for durability, read-your-writes for consistency"
        }
        
        capacity {
            storage "50 TB"
            throughput_writes "500K/sec"
            throughput_reads "2M/sec"
            replication "Replica Set 3 + Sharded by user_id"
        }
    }
    
    // REDIS: Real-time session data
    SessionStore = database "Redis Cluster" {
        technology "Redis 7"
        description "Stores user sessions, shopping carts - fast access, TTL"
        tags ["nosql", "key-value", "real-time"]
        
        tradeoff {
            decision "Use Redis for session data"
            sacrifice "Durability (in-memory, can lose data on crash)"
            reason "Sub-millisecond latency required for session checks, automatic TTL for cleanup"
            mitigation "AOF persistence, replicas for backup"
        }
        
        capacity {
            storage "1 TB"
            throughput_writes "10M/sec"
            throughput_reads "20M/sec"
            replication "Cluster mode (6 nodes)"
        }
    }
    
    // TRAFFIC FLOW
    UserService = container "User Service" {
        technology "Go"
    }
    
    ProductService = container "Product Service" {
        technology "Python"
    }
    
    OrderService = container "Order Service" {
        technology "Java"
    }
    
    UserService -> PrimaryDB "Reads user data"
    UserService -> SessionStore "Checks session (fast)"
    ProductService -> PrimaryDB "Queries product catalog"
    OrderService -> PrimaryDB "Creates order (transaction)"
    OrderService -> ReviewsDB "Posts review (high throughput)"
}

view index {
    title "E-Commerce Database Architecture"
    include *
}

view sql {
    title "SQL Databases"
    include ECommerce.PrimaryDB
}

view nosql {
    title "NoSQL Databases"
    include ECommerce.ReviewsDB ECommerce.SessionStore
}
```

### Key Trade-Offs Documented

**1. Database Technology Choice:**
- **PostgreSQL for transactions** - Strong consistency, complex queries, but write scalability limited
- **MongoDB for high throughput** - Flexible schema, horizontal scaling, but eventual consistency
- **Redis for real-time** - Sub-millisecond latency, but in-memory (durability concerns)

**2. Scaling Strategy:**
- **Read replicas** for PostgreSQL (read-heavy workload: 10:1 read:write ratio)
- **Sharding** for MongoDB (write-heavy reviews: user_id shard key)
- **Redis cluster** for session data (high throughput, automatic failover)

**3. Consistency vs Performance:**
- Sacrifice strong consistency for reviews (OK if reviews appear with slight delay)
- Maintain strong consistency for orders (financial data cannot be wrong)
- Use read-your-writes pattern for better user experience

## Knowledge Check

<details>
<summary><strong>Q: I'm building a social media feed like Twitter. Millions of posts/second. What database should I use for posts?</strong></summary>

**Cassandra (or similar time-series NoSQL)**

Social media feeds are write-heavy (append-only time-series) and require massive horizontal scale. SQL databases cannot handle millions of writes/second effectively. Cassandra is designed exactly for this workload: write-optimized, automatically partitions by time, scales horizontally.
</details>

<details>
<summary><strong>Q: Why use master-slave replication instead of master-master?</strong></summary>

**Master-slave is simpler and provides strong consistency.**

Master-slave writes go to a single master, reads can go to slaves. This is simple to implement, provides strong consistency (slaves always see master's changes), and works great for read-heavy workloads. Master-master scales writes better but adds complexity: conflict resolution, eventual consistency, and harder to reason about.
</details>

<details>
<summary><strong>Q: My app has flexible data (schema changes frequently). Should I use SQL or NoSQL?</strong></summary>

**NoSQL** - specifically document databases like MongoDB.

SQL databases have rigid schemas. ALTER TABLE can be slow and complex. NoSQL document databases allow you to add fields on the fly without migrations. Perfect for rapidly evolving data structures (e.g., user profiles with new features, product catalogs with new attributes).
</details>

<details>
<summary><strong>Q: What's the main benefit of consistent hashing over range sharding?</strong></summary>

**Minimal data movement when adding/removing shards.**

With range sharding, adding a shard might require moving 50% of data. With consistent hashing, adding a shard only moves ~1/n of data (where n is number of shards). This makes scaling operations much faster and less disruptive.
</details>

## Quiz: Test Your Knowledge

### Q1: Which database type is best for financial transactions requiring ACID guarantees?

- [ ] MongoDB (Document)
- [ ] Redis (Key-Value)
- [ ] PostgreSQL (Relational)
- [ ] Cassandra (Column-family)

<details>
<summary>Answer</summary>

**PostgreSQL (Relational)** provides ACID compliance (Atomicity, Consistency, Isolation, Durability) which is critical for financial transactions. MongoDB and Redis sacrifice strong consistency for performance. Cassandra provides tunable consistency but is optimized for write-heavy workloads, not complex transactions.
</details>

### Q2: Which replication strategy is best for a read-heavy system where reads outnumber writes 10:1?

- [ ] Master-Master
- [ ] Master-Slave
- [ ] No replication needed

<details>
<summary>Answer</summary>

**Master-Slave** is ideal for read-heavy workloads. All writes go to the master (single write path), but reads can be distributed across multiple read replicas. This gives you 10x read scalability with the simplicity of a single master. Master-Master gives write scalability but adds complexity (conflict resolution, eventual consistency).
</details>

### Q3: You're building a time-series analytics platform ingesting billions of events/day. Which database is best?

- [ ] PostgreSQL
- [ ] MongoDB
- [ ] Cassandra
- [ ] Redis

<details>
<summary>Answer</summary>

**Cassandra** is designed specifically for time-series workloads: write-heavy, append-only data that grows indefinitely. It automatically partitions data by time, scales horizontally, and provides TTL for automatic data retention. PostgreSQL would struggle with write throughput and storage. MongoDB works but Cassandra's time-series optimizations are better. Redis is in-memory and too expensive for billions of events.
</details>

### Q4: Which of these is NOT a characteristic of NoSQL databases?

- [ ] Flexible schemas
- [ ] Horizontal scalability
- [ ] Strong ACID compliance
- [ ] Eventual consistency is often acceptable

<details>
<summary>Answer</summary>

**Strong ACID compliance** is NOT a characteristic of most NoSQL databases. NoSQL databases typically prioritize scalability and flexibility over strong ACID guarantees. While some NoSQL databases (like MongoDB) provide document-level ACID guarantees, they don't provide the same level of multi-document transaction support as SQL databases.
</details>

### Q5: Uber uses multiple databases (MySQL, Cassandra, Redis, ElasticSearch). Why?

- [ ] To reduce complexity
- [ ] To choose the right database for each workload
- [ ] Because they couldn't decide on one
- [ ] To minimize costs

<details>
<summary>Answer</summary>

**To choose the right database for each workload** - this is a deliberate architectural decision. Different workloads benefit from different databases: MySQL for transactions (trips), Cassandra for time-series (event stream), Redis for real-time (driver locations), ElasticSearch for search. This polyglot persistence approach gives better performance and scalability than trying to force everything into one database.
</details>

### Q6: What's the main drawback of sharding?

- [ ] Slower writes (distributed across servers)
- [ ] Complex joins across shards
- [ ] More storage space needed
- [ ] Slower reads (distributed across servers)

<details>
<summary>Answer</summary>

**Complex joins across shards** is the main drawback. When data is split across multiple database servers, joining data that resides on different shards becomes complex and expensive. Cross-shard joins require querying multiple shards and merging results in the application, which is slow and complex. This is why you carefully choose shard keys to minimize cross-shard queries.
</details>

### Q7: Airbnb uses PostgreSQL for core business data and MongoDB for flexible data. Why this hybrid approach?

- [ ] PostgreSQL is faster
- [ ] MongoDB is cheaper
- [ ] Different workloads benefit from different technologies
- [ ] They couldn't scale PostgreSQL

<details>
<summary>Answer</summary>

**Different workloads benefit from different technologies** - this is a smart architectural decision. PostgreSQL excels at complex relational data with ACID transactions (users, listings, bookings). MongoDB excels at flexible, evolving schemas with high write throughput (search index, activity feed). Using the right database for each workload gives better performance and scalability than forcing everything into one technology.
</details>

### Q8: Which database feature allows Redis to automatically delete session data after 30 minutes?

- [ ] ACID transactions
- [ ] TTL (Time To Live)
- [ ] Sharding
- [ ] Master-Slave replication

<details>
<summary>Answer</summary>

**TTL (Time To Live)** allows Redis to automatically delete data after a specified time. This is perfect for session data, shopping carts, and other temporary data. You don't need manual cleanup jobs - Redis handles it automatically. Set a TTL of 30 minutes when storing a session, and Redis deletes it automatically.
</details>

## Next Steps

Now that we understand databases and how to choose between SQL and NoSQL, let's learn about caching to improve performance.
👉 **[Lesson 3: Caching (Strategies, Eviction Policies, Redis Case Study)](./lesson-3)**

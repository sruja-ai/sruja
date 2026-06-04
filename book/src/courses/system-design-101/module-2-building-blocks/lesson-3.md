---
title: "Lesson 3: Caching"
weight: 3
summary: "Caching strategies, eviction policies, and real-world Redis case studies"
learning_objectives:
  - Understand caching strategies and when to use each
  - Learn eviction policies for managing cache capacity
  - Master cache invalidation and consistency challenges
  - See real-world examples from Redis, Facebook, and CDN providers
  - Practice making caching trade-off decisions
estimated_time: "25 minutes"
difficulty: "beginner"
---

# Lesson 3: Caching

## Why Cache? The Performance Multiplier

Caching is the process of storing copies of data in a temporary storage location (cache) so that future requests for that data can be served faster. It's one of the most effective performance optimizations in system design.

**Without caching:**
```
User Request → Database (100-500ms latency)
               ↓
          Disk I/O, network round-trip
```

**With caching:**
```
User Request → Cache Hit (1-5ms latency) ✓
               ↓
          Return data immediately (no DB query)
```

**Cache miss scenario:**
```
User Request → Cache Miss
               ↓
          Database Query (100-500ms)
               ↓
          Store in Cache
               ↓
          Return data
```

### The Performance Impact

| System | Without Cache | With Cache | Improvement |
|--------|--------------|-----------|-------------|
| **User Profile** | 200ms (DB) | 2ms (Redis) | **100x faster** |
| **Product Catalog** | 150ms (DB) | 5ms (Redis) | **30x faster** |
| **API Response** | 500ms (DB + joins) | 10ms (Redis) | **50x faster** |
| **Session Data** | 50ms (DB) | 1ms (in-memory) | **50x faster** |

## Caching Strategies

### Cache-Aside (Lazy Loading)

**How it works:**
1. App checks cache
2. If miss, App reads from DB
3. App writes to cache
4. Next request hits cache

```
┌─────────────┐
│   Application │
└──────┬──────┘
       │
       ├───────── Check Cache
       │
       ├─ Hit? → Return (1-5ms)
       │
       └─ Miss? → Query DB (100-500ms)
                    ↓
                 Write to Cache
                    ↓
                 Return
```

**Pros:**
- ✅ Only requested data is cached (efficient cache usage)
- ✅ Simple to implement
- ✅ Works with any database
- ✅ Cache failure doesn't break app (fallback to DB)

**Cons:**
- ❌ Initial request is slow (cache miss penalty)
- ❌ Can have cache stampede (thundering herd) when multiple requests miss simultaneously
- ❌ Stale data until cache expires or is invalidated

**Real-world use:** Most applications, product catalogs, user profiles, API responses

### Write-Through

**How it works:**
1. App writes to cache and DB simultaneously
2. Reads always check cache first

```
┌─────────────┐
│   Application │
└──────┬──────┘
       │
       ├───── Write Request
       │
       ├─────────────────┬
       │                 │
       ▼                 ▼
   Write Cache       Write DB
   (synchronous)    (synchronous)
       │                 │
       └────────┬────────┘
                ▼
          Return (wait for both)
```

**Pros:**
- ✅ Data in cache is always fresh
- ✅ Strong consistency between cache and DB
- ✅ Simple read logic (always check cache)

**Cons:**
- ❌ Slower writes (synchronous DB write)
- ❌ Writes to data that's never read (wasted cache space)
- ❌ Higher latency for write operations

**Real-world use:** User sessions, configuration data that must be consistent

### Write-Back (Write-Behind)

**How it works:**
1. App writes to cache only
2. Cache writes to DB asynchronously
3. Reads check cache, fallback to DB if miss

```
┌─────────────┐
│   Application │
└──────┬──────┘
       │
       ├───── Write Request
       │
       ▼
   Write Cache (immediate)
       │
       ▼
   Return (immediate)
       
       └── Asynchronously → Write DB (background)
```

**Pros:**
- ✅ Extremely fast writes
- ✅ Can batch writes to DB (reduced DB load)
- ✅ Better write throughput

**Cons:**
- ❌ Data loss risk if cache crashes before syncing
- ❌ Complexity in handling failures
- ❌ Eventual consistency (DB might be behind)
- ❌ Data durability concerns

**Real-world use:** Write-heavy systems, analytics events, clickstream data

### Refresh-Ahead

**How it works:**
1. Background job refreshes cache before expiration
2. Ensures hot data is always in cache
3. Users never see cache misses for popular data

```
┌─────────────┐
│   Application │
└──────┬──────┘
       │
       ├───── Read Request
       │
       ├─ Hit? → Return
       │
       └─ Miss? → Query DB → Return
                    ↓
            Trigger background refresh

┌─────────────────┐
│ Background Job   │
└─────────┬───────┘
          │
          ├─ Find expiring cache entries
          ├─ Refresh from DB
          └─ Update cache (before users request)
```

**Pros:**
- ✅ Users never see cache misses for hot data
- ✅ Predictive caching for known hot keys
- ✅ Better user experience

**Cons:**
- ❌ Complexity in predicting what to refresh
- ❌ Wasted resources if predictions are wrong
- ❌ Background processing adds complexity

**Real-world use:** Product catalogs, leaderboards, trending content

## Eviction Policies

When the cache is full, what do you remove? This decision significantly impacts cache hit rates.

### LRU (Least Recently Used)

**How it works:** Remove the item that hasn't been used for the longest time.

**Example:**
```
Cache [10 items, capacity 1000 items]
├── User A profile (last accessed: 1 hour ago) ← Evict this
├── Product catalog entry (last accessed: 5 minutes ago)
├── Session data (last accessed: 2 minutes ago)
└── ...

When cache is full, remove User A profile (not accessed recently)
```

**Pros:**
- ✅ Simple to implement
- ✅ Works well for temporal locality (recently accessed likely to be accessed again)
- ✅ Good for general-purpose caching

**Cons:**
- ❌ Doesn't account for access frequency (popular items might be evicted)
- ❌ Can be suboptimal for certain access patterns
- ❌ Requires tracking access time (some overhead)

**Real-world use:** Most default cache implementations, web caches, database caches

### LFU (Least Frequently Used)

**How it works:** Remove the item that has been accessed least often overall.

**Example:**
```
Cache [10 items, capacity 1000 items]
├── User A profile (access count: 5) ← Evict this
├── Product catalog entry (access count: 10,000)
├── Session data (access count: 50)
└── ...

When cache is full, remove User A profile (least accessed)
```

**Pros:**
- ✅ Keeps most popular items in cache
- ✅ Good for workloads with clear popularity patterns
- ✅ Optimizes for cache hit rate

**Cons:**
- ❌ Requires tracking access frequency (more memory overhead)
- ❌ Can lock in old popular items (cold data problem)
- ❌ Doesn't adapt to changing access patterns quickly

**Real-world use:** Content distribution networks, recommendation systems, API response caches

### FIFO (First In, First Out)

**How it works:** Remove the oldest item regardless of access patterns.

**Example:**
```
Cache [10 items, capacity 1000 items]
├── Item 1 (added 1 hour ago) ← Evict this
├── Item 2 (added 30 minutes ago)
├── Item 3 (added 15 minutes ago)
└── ...

When cache is full, remove Item 1 (oldest)
```

**Pros:**
- ✅ Simplest to implement
- ✅ No tracking needed (just insertion order)
- ✅ Deterministic behavior

**Cons:**
- ❌ No consideration of access patterns
- ❌ Can evict hot items if they were cached early
- ❌ Poor cache hit rate in most scenarios

**Real-world use:** Round-robin load balancer caches, simple message queues

### TTL (Time To Live)

**How it works:** Remove items after a fixed time period, regardless of usage.

**Example:**
```
Cache entries with TTL:
├── User session (TTL: 30 minutes) ← Auto-remove after 30min
├── Product price (TTL: 5 minutes) ← Auto-remove after 5min
├── Configuration (TTL: 1 hour) ← Auto-remove after 1hour
└── News feed (TTL: 1 minute) ← Auto-remove after 1min
```

**Pros:**
- ✅ Guarantees data freshness
- ✅ Simple to understand and reason about
- ✅ Works great for time-sensitive data
- ✅ Automatic cleanup (no manual eviction needed)

**Cons:**
- ❌ Popular items might expire and be re-fetched
- ❌ Need to choose appropriate TTL (tuning required)
- ❌ Can cause cache stampede if popular items expire simultaneously

**Real-world use:** Session data, real-time prices, news feeds, API rate limiting

### Random Replacement

**How it works:** Randomly select an item to evict.

**Pros:**
- ✅ Simple to implement
- ✅ No tracking overhead
- ✅ Works surprisingly well in practice

**Cons:**
- ❌ Can evict hot items
- ❌ Suboptimal cache hit rate
- ❌ Unpredictable behavior

**Real-world use:** Simple caching implementations where tracking overhead is a concern

## Real-World Case Studies

### Case Study 1: Redis at Scale

**The Challenge:** Redis is used by companies like Twitter, Instagram, and Uber for ultra-low latency caching. How does Redis handle millions of requests per second with sub-millisecond latency?

**The Architecture:**
```
Clients → Redis Cluster (100+ nodes)
         ├── Node 1-20: US-East
         ├── Node 21-40: US-West
         ├── Node 41-60: EU-Central
         └── Node 61-80: AP-Southeast
         
Sharding Strategy: Consistent hashing of keys
Replication: Master-slave for each shard
Persistence: AOF (Append Only File) + RDB snapshots
```

**Key Optimizations:**
1. **In-memory data structures** - No disk I/O for reads
2. **Single-threaded** - No locking overhead, atomic operations
3. **Efficient data structures** - Hash tables, skip lists, etc.
4. **Pipelining** - Send multiple commands in one network round-trip
5. **Lua scripting** - Execute multiple operations atomically on server

**Performance Results:**
- **Throughput:** 10M+ operations/second per cluster
- **Latency:** <1ms p50, <5ms p99
- **Memory usage:** 100GB+ RAM across cluster
- **Hit rate:** 95-98% for hot keys
- **Capacity:** 1B+ keys stored

**Use Cases:**
- Session storage (10M+ active sessions)
- Rate limiting (100K+ requests/second)
- Real-time leaderboards
- Pub/Sub messaging (100K+ messages/second)

> **💡 Key Insight:** Redis achieves incredible performance by keeping everything in memory and using single-threaded architecture. No locks means no contention. The trade-off is memory-intensive and limited by RAM capacity.

### Case Study 2: Facebook's Caching Strategy

**The Challenge:** Facebook (now Meta) serves billions of users with complex social graphs, news feeds, and real-time interactions. Caching is critical for performance at this scale.

**The Caching Architecture:**
```
Facebook Platform Caching Layers:
├── Edge Caching (Akamai CDN)
│   ├── Static assets (images, CSS, JS)
│   └── HTML content (cache time: 1-5 minutes)
│
├── Application Caching (Tao + Memcached)
│   ├── User profiles
│   ├── Friend lists
│   ├── News feed entries
│   └── Permissions
│
├── Database Caching (MySQL + caching layer)
│   ├── Hot database rows
│   ├── Query results
│   └── Join results
│
└── Client Caching (Browser)
    ├── API responses
    ├── Static resources
    └── Service Worker cache
```

**Tao (The Associations and Objects):**
- Facebook's distributed edge caching system
- 100+ terabytes of cache across multiple datacenters
- Cache hit rate: 98-99% for read-heavy workloads
- Consistent hashing for key distribution
- Hot items replicated across multiple caches

**Memcached Configuration:**
```
Cluster: 10,000+ servers
Total capacity: 10+ TB of RAM
Items cached: 1+ trillion objects
Hit rate: 98% (overall)
Latency: <5ms p95
```

**News Feed Caching Strategy:**
```
User A's News Feed Generation:
├── Check cache for user's feed (TTL: 5 minutes)
├── If miss, generate from:
│   ├── Friend list (cached, TTL: 1 hour)
│   ├── Friend's posts (cached, TTL: 10 minutes)
│   ├── Friend's likes (cached, TTL: 5 minutes)
│   └── Ranking algorithm (real-time)
├── Assemble feed
└── Cache result (TTL: 5 minutes)
```

**Performance Results:**
- **News feed generation:** From 500ms (DB) to 10ms (cache)
- **User profile loads:** From 200ms (DB) to 2ms (cache)
- **Social graph queries:** From 100ms (DB) to 5ms (cache)
- **Overall hit rate:** 98% across all cache layers

> **💡 Key Insight:** Facebook uses multiple cache layers (edge, application, database, client) with different TTLs for each layer. This multi-layer approach gives them both performance (edge cache) and freshness (shorter TTLs at application layer). The complexity is high, but the performance improvement is massive.

### Case Study 3: CDN Caching (CloudFront, Cloudflare, Akamai)

**The Challenge:** Serve static assets and content to users globally with low latency. How do CDNs achieve sub-100ms latency worldwide?

**The Architecture:**
```
User Request → Edge Server (closest geographically)
             ↓ (cache hit: <10ms)
          Return cached content
             
User Request → Edge Server (cache miss)
             ↓ (fetch from origin)
          Origin Server
             ↓ (return)
          Edge Server (cache for future)
             ↓
          Return to user
```

**CloudFront Caching:**
```
AWS Global Network:
├── 600+ edge locations worldwide
├── 500+ points of presence
├── 13 regional edge caches
└── 200+ PoPs (Points of Presence)

Cache Behaviors:
├── Static assets (images, CSS, JS): 1 year TTL
├── API responses: 5-60 minute TTL
├── HTML content: 1-5 minute TTL
└── Dynamic content: no caching (bypass)
```

**Cache-Control Headers:**
```
Cache-Control: max-age=3600, public
  → Cache for 1 hour, CDNs and browsers can cache

Cache-Control: max-age=0, no-cache, no-store, must-revalidate
  → No caching, always fetch from origin

Cache-Control: max-age=86400, s-maxage=3600, public
  → Browser cache for 1 day, CDN cache for 1 hour
```

**Performance Results:**
| Scenario | Without CDN | With CDN | Improvement |
|----------|-------------|----------|-------------|
| **Static asset from US to Asia** | 500ms | 50ms | **10x faster** |
| **API response from Europe to US** | 300ms | 30ms | **10x faster** |
| **Video streaming (HD)** | 2000ms | 200ms | **10x faster** |
| **Edge locations covered** | 0 (origin only) | 600+ locations | **Global** |

**Cache Invalidation:**
- **Time-based:** Automatic expiration based on TTL
- **Manual:** Invalidate specific paths or objects
- **Purge:** Remove from all edge locations (takes 5-30 minutes)

> **💡 Key Insight:** CDNs use geographic distribution and massive scale (thousands of edge servers) to achieve low latency globally. The cache is distributed across edge locations, with each location caching content for nearby users. This gives sub-100ms latency worldwide while offloading origin servers.

## Production Metrics

### Cache Performance Comparison

| Cache System | Throughput | Latency (p95) | Hit Rate | Capacity |
|--------------|------------|----------------|----------|----------|
| **Redis** | 10M+ ops/sec | <5ms | 95-98% | 1GB-1TB |
| **Memcached** | 100M+ ops/sec | <2ms | 90-95% | 10GB-100TB |
| **Varnish** | 10K+ req/sec | <10ms | 80-90% | 10GB-100GB |
| **CDN (CloudFront)** | 1M+ req/sec | <50ms | 70-80% | Unlimited |

### Eviction Policy Performance

| Policy | Hit Rate | Complexity | Memory Overhead | Best For |
|--------|---------|------------|-----------------|----------|
| **LRU** | 70-80% | Low | Low | General purpose |
| **LFU** | 75-85% | Medium | Medium | Popularity-based |
| **TTL** | 60-90% | Low | Very Low | Time-sensitive data |
| **Random** | 65-75% | Very Low | None | Simple systems |

## Trade-Off Scenarios

### Scenario 1: E-Commerce Product Catalog

**Context:** Building an e-commerce platform with 1M+ products. 90% of traffic is browsing products (read-heavy), 10% is purchasing (write-heavy). Need to handle Black Friday spikes (10x normal traffic).

**The Trade-Off Decisions:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Cache Strategy** | Write-Through | Cache-Aside | **Cache-Aside** - Only cache hot products, not every product |
| **Eviction Policy** | LRU | TTL | **LRU** - Popular products stay in cache, cold products evicted |
| **TTL for Prices** | 1 hour | 5 minutes | **5 minutes** - Prices change frequently, must stay fresh |
| **TTL for Details** | 1 day | 1 hour | **1 hour** - Product details change less often than prices |
| **Cache Invalidation** | Time-based (TTL) | Manual purge | **TTL + Manual purge** - Auto-expire for simplicity, manual purge for price changes |
| **Cache Layer** | Single Redis cluster | Multi-tier (Redis + CDN) | **Redis + CDN** - CDN for product images, Redis for API data |

**Result:**
- **Pros:** High cache hit rate (90%+), fresh pricing data, handles Black Friday traffic
- **Cons:** Manual cache invalidation adds complexity, dual cache layer adds operational overhead
- **Performance:** 90% cache hit rate, <10ms API response for cached products, handles 10x traffic spikes

### Scenario 2: Real-Time Leaderboard

**Context:** Building a real-time game with global leaderboards. 10M+ players, millions of score updates per second. Rankings must be updated in real-time (<100ms latency).

**The Trade-Off Decisions:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Cache Strategy** | Cache-Aside | Write-Through | **Write-Through** - Leaderboard must always show latest scores |
| **Data Structure** | Sorted set (Redis ZSET) | Hash map + manual sorting | **Sorted set** - Built-in sorted structure, O(log N) updates |
| **Cache Size** | Top 1000 players | Top 1M players | **Top 1000** - Cache only hot players, DB for full list |
| **TTL** | 1 minute | 10 seconds | **10 seconds** - Scores update frequently, must stay fresh |
| **Refresh Strategy** | Client polls | Server push (WebSocket) | **Server push** - Real-time updates, no polling overhead |
| **Backup Strategy** | Snapshot every hour | AOF + RDB | **AOF + RDB** - AOF for durability, RDB for fast recovery |

**Result:**
- **Pros:** Real-time updates, efficient sorted queries, high write throughput
- **Cons:** Memory-intensive (sorted sets), complex backup strategy
- **Performance:** <50ms latency for top 1000, 1M+ score updates/second

### Scenario 3: User Session Storage

**Context:** Building a web application with 1M+ active users. Sessions need to be fast (<5ms) and secure. Sessions should expire after 30 minutes of inactivity.

**The Trade-Off Decisions:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Cache Strategy** | Cache-Aside | Write-Back | **Cache-Aside** - Simpler, strong consistency needed |
| **Storage** | Database (SQL) | Redis | **Redis** - Sub-millisecond latency, automatic TTL |
| **TTL** | 30 minutes | 1 hour | **30 minutes** - Security requirement (sessions expire) |
| **Persistence** | None | AOF snapshots | **AOF** - Prevent data loss if Redis crashes |
| **Session Data** | Minimal (user ID) | Full (user profile, cart) | **Minimal in cache** - Faster access, smaller cache, profile in DB |
| **Geo-Distribution** | Single datacenter | Multi-region | **Multi-region** - Global users, reduce latency |
| **Backup Strategy** | None | Replica set | **Replica set** - Prevent session loss on failure |

**Result:**
- **Pros:** Fast session access, automatic expiration, geo-distributed for low latency
- **Cons:** Redis is memory-intensive, complexity in multi-region setup
- **Performance:** <2ms session lookup, automatic TTL cleanup, 99.99% availability

## Sruja Perspective: Modeling Caches

In Sruja, we document caching strategies with clear trade-offs and performance characteristics.

### Why Model Caches?

Modeling caches in your architecture provides:

1. **Performance visibility** - Track cache hit rates, latency improvements
2. **Strategy clarity** - Document which caching strategy and why
3. **Failure analysis** - Understand impact of cache failures
4. **Capacity planning** - See cache size requirements and scaling needs

### Example: E-Commerce Caching Architecture

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce Platform" {
    description "Multi-tier caching strategy for e-commerce platform"
    
    // EDGE CACHE: CDN for static assets
    EdgeCache = container "CDN Cache" {
        technology "CloudFront / Cloudflare"
        description "Caches product images, CSS, JS globally"
        tags ["cache", "edge", "cdn"]
        
        tradeoff {
            decision "Use CDN for static assets"
            sacrifice "Cost (CDN services cost money)"
            reason "Reduces latency 10x globally, offloads origin servers"
            mitigation "Use cache-control headers, optimize asset sizes"
        }
        
        capacity {
            ttl_images "1 year"
            ttl_api "5-60 minutes"
            ttl_html "1-5 minutes"
        }
    }
    
    // APPLICATION CACHE: Redis for API data
    ProductCache = container "Product Cache" {
        technology "Redis Cluster"
        description "Caches product details, prices, inventory"
        tags ["cache", "application", "redis"]
        
        tradeoff {
            decision "Use Cache-Aside strategy"
            sacrifice "Initial request latency (cache miss)"
            reason "Only cache hot products, efficient memory usage"
            mitigation "Refresh-ahead for popular products, monitor hit rate"
        }
        
        tradeoff {
            decision "Use LRU eviction policy"
            sacrifice "Memory for cold products (frequent evictions)"
            reason "Popular products stay in cache, good temporal locality"
            mitigation "TTL for prices (5min), details (1hour)"
        }
        
        slo {
            latency {
                p95 "10ms"
                p99 "50ms"
                window "7 days"
            }
            hit_rate {
                target "90%"
                window "24 hours"
            }
        }
        
        capacity {
            memory "100GB"
            throughput "10M ops/sec"
            hit_rate "90%"
        }
    }
    
    // DATABASE: PostgreSQL for persistent storage
    ProductDB = database "Product Database" {
        technology "PostgreSQL"
        description "Stores all product data persistently"
        tags ["database", "sql"]
        
        tradeoff {
            decision "Use PostgreSQL for persistent storage"
            sacrifice "Write speed (disk I/O slower than memory)"
            reason "Strong consistency, ACID transactions, complex queries"
            mitigation "Read replicas, caching layer"
        }
    }
    
    // SERVICES
    ProductService = container "Product Service" {
        technology "Go"
        description "API for product catalog and search"
    }
    
    // TRAFFIC FLOW
    Client -> ProductService "Requests product"
    ProductService -> ProductCache "Check cache (Cache-Aside, LRU, TTL)"
    ProductService -> ProductDB "Query on cache miss"
    ProductService -> EdgeCache "Serve images/CSS/JS (CDN)"
}

view index {
    title "E-Commerce Caching Architecture"
    include *
}

view caching {
    title "Caching Strategy"
    include ECommerce.ProductCache
}
```

### Key Trade-Offs Documented

**1. Cache Strategy Choice:**
- **Why Cache-Aside?** Only cache hot data (efficient memory usage)
- **Why LRU eviction?** Popular products stay in cache (good temporal locality)
- **Why different TTLs?** Prices change frequently (5min), details don't (1hour)

**2. CDN Integration:**
- Use CDN for static assets (images, CSS, JS)
- Global edge locations for sub-100ms latency
- Cache-control headers for different TTLs

**3. Consistency vs Performance:**
- Sacrifice some write speed for cache consistency
- TTL ensures freshness even with Cache-Aside
- Manual cache invalidation for critical updates

**4. Capacity Planning:**
- 100GB Redis cache for 1M products
- 90% hit rate target
- 10M+ ops/sec throughput needed for Black Friday

## Knowledge Check

<details>
<summary><strong>Q: When should I use Cache-Aside vs Write-Through?</strong></summary>

**Cache-Aside** when you only want to cache frequently accessed data (hot data). This is more memory-efficient because you don't cache everything. Initial request is slower (cache miss), but subsequent requests are fast.

**Write-Through** when you need strong consistency between cache and database. Every write updates both cache and DB synchronously, ensuring cache is always fresh. This is slower for writes but simpler for consistency-critical data like user sessions.

</details>

<details>
<summary><strong>Q: My cache is filling up too fast. Which eviction policy should I use?</strong></summary>

**LRU (Least Recently Used)** is a good default choice. It removes items that haven't been accessed recently, which works well for most workloads (temporal locality - recently accessed items are likely to be accessed again).

If you know your workload has clear popularity patterns (some items accessed 100x more than others), use **LFU (Least Frequently Used)**. This keeps the most popular items in cache, improving hit rate for hot data.

</details>

<details>
<summary><strong>Q: How do I prevent cache stampede (thundering herd)?</strong></summary>

**Cache stampede** occurs when many requests miss for the same hot key simultaneously, all querying the database and overwhelming it.

**Solutions:**
1. **Locking** - First request acquires lock, others wait and use cached result
2. **Refresh-ahead** - Background job refreshes hot keys before expiration
3. **Probabilistic early expiration** - Randomly expire some items early to spread load
4. **Request coalescing** - Merge simultaneous requests for the same key

The best solution depends on your workload. Locking is simplest, refresh-ahead gives best user experience.

</details>

## Quiz: Test Your Knowledge

### Q1: Which caching strategy only caches data that's actually requested?

- [ ] Write-Through
- [ ] Write-Back
- [ ] Cache-Aside
- [ ] Refresh-Ahead

<details>
<summary>Answer</summary>

**Cache-Aside** only caches data that's actually requested. When a request comes in:
1. Check cache - if hit, return data
2. If miss, query database
3. Store in cache for next request

This is memory-efficient because you don't waste cache space on data nobody requests. Write-Through caches everything on write, regardless of whether it will be read.

</details>

### Q2: Which eviction policy removes the item that hasn't been accessed for the longest time?

- [ ] LFU (Least Frequently Used)
- [ ] FIFO (First In, First Out)
- [ ] LRU (Least Recently Used)
- [ ] TTL (Time To Live)

<details>
<summary>Answer</summary>

**LRU (Least Recently Used)** removes the item that hasn't been accessed for the longest time. This works well for most workloads because of temporal locality - items that were accessed recently are likely to be accessed again.

FIFO removes the oldest item regardless of access pattern. LFU removes the least frequently accessed item. TTL removes items after a fixed time.

</details>

### Q3: You're building a real-time leaderboard with frequent score updates. Which caching strategy should you use?

- [ ] Cache-Aside
- [ ] Write-Through
- [ ] Write-Back
- [ ] No caching needed

<details>
<summary>Answer</summary>

**Write-Through** ensures the cache always has the latest scores, which is critical for a real-time leaderboard. Every score update writes to both cache and database synchronously.

Cache-Aside would be problematic because score updates might not be immediately reflected in the cache. Write-Back is dangerous because if the cache crashes before syncing to database, you could lose score updates.

</details>

### Q4: What's the main benefit of using a CDN (Content Delivery Network) for caching?

- [ ] Reduces database load
- [ ] Improves global latency by serving content from edge locations
- [ ] Improves cache hit rate
- [ ] Simplifies cache invalidation

<details>
<summary>Answer</summary>

**Improves global latency by serving content from edge locations**. CDNs have thousands of servers distributed globally. When a user requests content, it's served from the closest edge server, which could be just a few milliseconds away vs. hundreds of milliseconds from the origin server.

While CDNs also reduce origin server load, their primary benefit is geographic distribution for low latency. Cache hit rate depends on your caching strategy and TTL configuration, not the CDN itself.

</details>

### Q5: Which of these is NOT a characteristic of Redis caching?

- [ ] In-memory storage for sub-millisecond latency
- [ ] Single-threaded architecture for atomic operations
- [ ] Automatic disk-based persistence by default
- [ ] Supports multiple data structures (strings, hashes, sets, sorted sets)

<details>
<summary>Answer</summary>

**Automatic disk-based persistence by default** is NOT a characteristic of Redis. Redis is primarily an in-memory cache. While it does support persistence options (RDB snapshots and AOF - Append Only File), it doesn't automatically persist to disk by default. Data is lost if Redis crashes without persistence configured.

Redis is in-memory (fast), single-threaded (no locking overhead), and supports multiple data structures (strings, hashes, sets, sorted sets, lists).

</details>

### Q6: Facebook uses multiple cache layers (edge, application, database, client). Why?

- [ ] Because they couldn't decide on one caching strategy
- [ ] To reduce complexity
- [ ] Each layer serves a different purpose with appropriate TTLs
- [ ] All cache layers do the same thing

<details>
<summary>Answer</summary>

**Each layer serves a different purpose with appropriate TTLs.** Facebook's caching strategy:

- **Edge cache (CDN):** Static assets with long TTLs (images, CSS, JS) - 1 day to 1 year
- **Application cache (Tao/Memcached):** Dynamic data with medium TTLs (friend lists, posts) - 5-60 minutes
- **Database cache:** Hot rows and query results with short TTLs - 1-10 minutes
- **Client cache:** Browser and Service Worker cache with controlled TTLs

This multi-layer approach gives them both performance (edge cache) and freshness (shorter TTLs at application layer) while optimizing for different access patterns at each layer.

</details>

### Q7: What's the trade-off with Write-Back (Write-Behind) caching?

- [ ] Slower reads
- [ ] Risk of data loss if cache crashes before syncing
- [ ] Complex read logic
- [ ] Higher memory usage

<details>
<summary>Answer</summary>

**Risk of data loss if cache crashes before syncing** is the main trade-off with Write-Back caching. In Write-Back, writes go to cache immediately (fast) and the cache asynchronously writes to the database. If the cache crashes before syncing, those writes are lost forever.

Write-Back is very fast for writes and can batch database operations, but the data loss risk makes it unsuitable for critical data. Use it for analytics events, clickstream data, or other data where occasional loss is acceptable.

</details>

### Q8: You're building an e-commerce platform with 1M products. 90% of traffic is browsing (read-heavy). Which caching strategy is best?

- [ ] Write-Back
- [ ] Write-Through
- [ ] Cache-Aside with LRU eviction
- [ ] No caching needed

<details>
<summary>Answer</summary>

**Cache-Aside with LRU eviction** is ideal for this scenario. Here's why:

1. **Cache-Aside** - Only cache frequently accessed products (efficient memory usage, don't waste space on cold products)
2. **LRU eviction** - Popular browsing products stay in cache (good temporal locality)
3. **Read-heavy workload** - 90% of traffic is browsing, so cache hit rate will be high (90%+)

Write-Through would cache every product on write (inefficient). Write-Back has data loss risk (not acceptable for product data). Cache-Aside is perfect: cache the hot products (20% of products get 80% of traffic) and let LRU evict cold products.

</details>

## Next Steps

Now that we understand caching strategies, eviction policies, and real-world implementations, let's put it all together with a full system design exercise.

👉 **[Complete Module 2: The Building Blocks](./module-overview)**

Up next: **Module 3: Advanced Modeling** - Learn about system boundaries, context mapping, and more complex architectural patterns!

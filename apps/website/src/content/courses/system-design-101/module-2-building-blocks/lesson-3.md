---
title: "Lesson 3: Caching"
weight: 3
summary: "Caching strategies and eviction policies."
---

# Lesson 3: Caching

## Why Cache?

Caching is the process of storing copies of data in a temporary storage location (cache) so that future requests for that data can be served faster.

- **Reduce Latency:** Memory is faster than disk.
- **Reduce Load:** Fewer queries to the database.

## Caching Strategies

### Cache-Aside (Lazy Loading)

1.  App checks cache.
2.  If miss, App reads from DB.
3.  App writes to cache.

- **Pros:** Only requested data is cached.
- **Cons:** Initial request is slow (cache miss).

### Write-Through

1.  App writes to cache and DB simultaneously.

- **Pros:** Data in cache is always fresh.
- **Cons:** Slower writes.

### Write-Back (Write-Behind)

1.  App writes to cache only.
2.  Cache writes to DB asynchronously.

- **Pros:** Fast writes.
- **Cons:** Data loss risk if cache fails before syncing.

## Eviction Policies

When the cache is full, what do you remove?

- **LRU (Least Recently Used):** Remove the item that hasn't been used for the longest time.
- **LFU (Least Frequently Used):** Remove the item used least often.
- **FIFO (First In, First Out):** Remove the oldest item.

---

## 🛠️ Sruja Perspective: Modeling Caches

In Sruja, caches are often modeled as separate containers or components.

```sruja
import { * } from 'sruja.ai/stdlib'


Catalog = system "Product Catalog System" {
    WebApp = container "Storefront" {
        technology "Node.js"
    }

    ProductCache = container "Product Cache" {
        technology "Memcached"
        description "Caches product details using LRU eviction."
    }

    ProductDB = container "Product Database" {
        technology "MongoDB"
    }

    WebApp -> ProductCache "Read (Cache-Aside)"
    WebApp -> ProductDB "Read on Miss"
}

view index {
include *
}
```

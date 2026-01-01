---
title: "Lesson 1: Design a URL Shortener"
weight: 1
summary: "TinyURL: Hashing, Key-Value Stores, Redirection."
---

# Lesson 1: Design a URL Shortener

**Goal:** Design a service like TinyURL that takes a long URL and converts it into a short alias (e.g., `http://tiny.url/xyz`).

## Requirements

### Functional

- `shorten(long_url) -> short_url`
- `redirect(short_url) -> long_url`
- Custom aliases (optional).

### Non-Functional

- **Highly Available:** If the service is down, URL redirection stops working.
- **Low Latency:** Redirection must happen in milliseconds.
- **Read-Heavy:** 100:1 read-to-write ratio.

## Core Design

### 1. Database Choice

Since we need fast lookups and the data model is simple (Key-Value), a **NoSQL Key-Value Store** (like DynamoDB or Redis) is ideal.

- **Key:** `short_alias`
- **Value:** `long_url`

### 2. Hashing Algorithm

How do we generate the alias?

- **MD5/SHA256:** Too long.
- **Base62 Encoding:** Converts a unique ID (from a counter or database ID) into a string of characters [a-z, A-Z, 0-9].

---

## 🛠️ Sruja Perspective: Modeling the Flow

We can use Sruja to model the system components and the user scenario for redirection.

```sruja
import { * } from 'sruja.ai/stdlib'


R1 = requirement functional "Shorten long URL"
R2 = requirement functional "Redirect short URL"
R3 = requirement availability "High availability for redirects"
R4 = requirement performance "Low latency (< 200ms)"

// Define the system boundary
TinyURL = system "TinyURL Service" {
  WebServer = container "API Server" {
    technology "Go"
    scale {
      min 3
      max 20
      metric "cpu > 70%"
    }
  }

  DB = database "UrlStore" {
    technology "DynamoDB"
    description "Stores mapping: short_alias -> long_url"
  }

  Cache = container "Cache" {
    technology "Redis"
    description "Caches popular redirects"
  }

  WebServer -> Cache "Reads"
  WebServer -> DB "Reads/Writes"
}

User = person "User"

// Define the redirection scenario (most common - cache hit)
RedirectFlowCacheHit = scenario "User clicks a short link (cache hit)" {
  User -> TinyURL.WebServer "GET /xyz"
  TinyURL.WebServer -> TinyURL.Cache "Check cache for 'xyz'"
  TinyURL.Cache -> TinyURL.WebServer "Hit: 'http://example.com'"
  TinyURL.WebServer -> User "301 Redirect (from cache)"
}

// Cache miss scenario
RedirectFlowCacheMiss = scenario "User clicks a short link (cache miss)" {
  User -> TinyURL.WebServer "GET /xyz"
  TinyURL.WebServer -> TinyURL.Cache "Check cache for 'xyz'"
  TinyURL.Cache -> TinyURL.WebServer "Miss"
  TinyURL.WebServer -> TinyURL.DB "Get long_url for 'xyz'"
  TinyURL.DB -> TinyURL.WebServer "Return 'http://example.com'"
  TinyURL.WebServer -> TinyURL.Cache "Cache the mapping"
  TinyURL.WebServer -> User "301 Redirect to 'http://example.com'"
}

// URL shortening scenario
ShortenURL = scenario "User creates a short URL" {
  User -> TinyURL.WebServer "POST /shorten with long_url"
  TinyURL.WebServer -> TinyURL.WebServer "Generate base62 alias"
  TinyURL.WebServer -> TinyURL.DB "Store mapping: alias -> long_url"
  TinyURL.DB -> TinyURL.WebServer "Confirm stored"
  TinyURL.WebServer -> User "Return short URL"
}

view index {
include *
}
```

---
title: "Lesson 1: Design a URL Shortener"
weight: 1
summary: "TinyURL: Hashing, Key-Value Stores, Redirection."
---

# Lesson 1: Design a URL Shortener

**Goal:** Design a service like TinyURL that takes a long URL and converts it into a short alias (e.g., `http://tiny.url/xyz`).

## Requirements

### Functional
*   `shorten(long_url) -> short_url`
*   `redirect(short_url) -> long_url`
*   Custom aliases (optional).

### Non-Functional
*   **Highly Available:** If the service is down, URL redirection stops working.
*   **Low Latency:** Redirection must happen in milliseconds.
*   **Read-Heavy:** 100:1 read-to-write ratio.

## Core Design

### 1. Database Choice
Since we need fast lookups and the data model is simple (Key-Value), a **NoSQL Key-Value Store** (like DynamoDB or Redis) is ideal.
*   **Key:** `short_alias`
*   **Value:** `long_url`

### 2. Hashing Algorithm
How do we generate the alias?
*   **MD5/SHA256:** Too long.
*   **Base62 Encoding:** Converts a unique ID (from a counter or database ID) into a string of characters [a-z, A-Z, 0-9].

---

## 🛠️ Sruja Perspective: Modeling the Flow

We can use Sruja to model the system components and the user scenario for redirection.

```sruja
architecture "URL Shortener" {
    requirement R1 functional "Shorten long URL"
    requirement R2 functional "Redirect short URL"
    requirement R3 availability "High availability for redirects"
    requirement R4 latency "Low latency (< 200ms)"

    // Define the system boundary
    system TinyURL "TinyURL Service" {
        container WebServer "API Server" {
            technology "Go"
            scale {
                min 3
                max 20
                metric "cpu > 70%"
            }
        }
        
        datastore DB "UrlStore" {
            technology "DynamoDB"
            description "Stores mapping: short_alias -> long_url"
        }
        
        container Cache "Cache" {
            technology "Redis"
            description "Caches popular redirects"
        }

        TinyURL.WebServer -> TinyURL.Cache "Reads"
        TinyURL.WebServer -> TinyURL.DB "Reads/Writes"
    }

    person User "User"

    // Define the redirection scenario
    scenario RedirectFlow "User clicks a short link" {
        User -> TinyURL.WebServer "GET /xyz"
        TinyURL.WebServer -> TinyURL.Cache "Check cache for 'xyz'"
        TinyURL.Cache -> TinyURL.WebServer "Miss"
        TinyURL.WebServer -> TinyURL.DB "Get long_url for 'xyz'"
        TinyURL.DB -> TinyURL.WebServer "Return 'http://example.com'"
        TinyURL.WebServer -> User "301 Redirect to 'http://example.com'"
    }
}
```

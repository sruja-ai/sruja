---
title: "Lesson 1: Design a Chat Application"
weight: 1
summary: "WhatsApp: WebSockets, Pub/Sub, Message Persistence."
---

# Lesson 1: Design a Chat Application

**Goal:** Design a real-time chat service like WhatsApp or Slack that supports 1-on-1 and Group messaging.

## Requirements

### Functional
*   Send/Receive messages in real-time.
*   See user status (Online/Offline).
*   Message history (persistent storage).

### Non-Functional
*   **Low Latency:** Messages must appear instantly.
*   **Consistency:** Messages must be delivered in order.
*   **Availability:** High uptime.

## Core Design

### 1. Communication Protocol
HTTP is request/response (pull). For chat, we need **push**.
*   **WebSockets:** Keeps a persistent connection open between client and server.

### 2. Message Flow
*   **User A** sends message to **Chat Server**.
*   **Chat Server** finds which server **User B** is connected to (using a Session Store like Redis).
*   **Chat Server** pushes message to **User B**.

### 3. Storage
*   **Chat History:** Write-heavy. Cassandra or HBase (Wide-column stores) are good for time-series data.
*   **User Status:** Key-Value store (Redis) with TTL.

---

## 🛠️ Sruja Perspective: Modeling Real-Time Flows

We can use Sruja to model the WebSocket connections and the async message processing.

```sruja
architecture "Chat System" {
    requirement R1 functional "Real-time messaging"
    requirement R2 functional "Message history"
    requirement R3 latency "Instant delivery"
    requirement R4 consistency "Ordered delivery"

    system ChatApp "WhatsApp Clone" {
        container ChatServer "Chat Server" {
            technology "Node.js (Socket.io)"
            description "Handles WebSocket connections"
            scale {
                min 10
                max 100
                metric "connections > 10k"
            }
        }
        
        datastore SessionStore "Session Store" {
            technology "Redis"
            description "Maps UserID -> WebSocketServerID"
        }
        
        datastore MessageDB "Message History" {
            technology "Cassandra"
            description "Stores chat logs"
        }
        
        queue MessageQueue "Message Queue" {
            technology "Kafka"
            description "Buffers messages for group chat fan-out"
        }

        ChatServer -> SessionStore "Reads/Writes"
        ChatServer -> MessageDB "Persists messages"
        ChatServer -> MessageQueue "Async processing"
    }

    person UserA "Alice"
    person UserB "Bob"

    // Scenario of 1-on-1 chat
    scenario SendMessage "Send Message Flow" {
        UserA -> ChatServer "Send 'Hello'"
        ChatServer -> MessageDB "Persist message"
        ChatServer -> SessionStore "Lookup Bob's connection"
        SessionStore -> ChatServer "Bob is on Server-2"
        ChatServer -> UserB "Push 'Hello'"
    }
}
```

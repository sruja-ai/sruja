---
title: "Lesson 1: Design a Chat Application"
weight: 1
summary: "WhatsApp: WebSockets, Pub/Sub, Message Persistence."
---

# Lesson 1: Design a Chat Application

**Goal:** Design a real-time chat service like WhatsApp or Slack that supports 1-on-1 and Group messaging.

## Requirements

### Functional

- Send/Receive messages in real-time.
- See user status (Online/Offline).
- Message history (persistent storage).

### Non-Functional

- **Low Latency:** Messages must appear instantly.
- **Consistency:** Messages must be delivered in order.
- **Availability:** High uptime.

## Core Design

### 1. Communication Protocol

HTTP is request/response (pull). For chat, we need **push**.

- **WebSockets:** Keeps a persistent connection open between client and server.

### 2. Message Flow

- **User A** sends message to **Chat Server**.
- **Chat Server** finds which server **User B** is connected to (using a Session Store like Redis).
- **Chat Server** pushes message to **User B**.

### 3. Storage

- **Chat History:** Write-heavy. Cassandra or HBase (Wide-column stores) are good for time-series data.
- **User Status:** Key-Value store (Redis) with TTL.

---

## 🛠️ Sruja Perspective: Modeling Real-Time Flows

We can use Sruja to model the WebSocket connections and the async message processing.

```sruja
import { * } from 'sruja.ai/stdlib'


requirement R1 functional "Real-time messaging"
requirement R2 functional "Message history"
requirement R3 latency "Instant delivery"
requirement R4 consistency "Ordered delivery"

ChatApp = system "WhatsApp Clone" {
    ChatServer = container "Chat Server" {
        technology "Node.js (Socket.io)"
        description "Handles WebSocket connections"
        scale {
            min 10
            max 100
            metric "connections > 10k"
        }
    }

    SessionStore = database "Session Store" {
        technology "Redis"
        description "Maps UserID -> WebSocketServerID"
    }

    MessageDB = database "Message History" {
        technology "Cassandra"
        description "Stores chat logs"
    }

    MessageQueue = queue "Message Queue" {
        technology "Kafka"
        description "Buffers messages for group chat fan-out"
    }

    ChatServer -> SessionStore "Reads/Writes"
    ChatServer -> MessageDB "Persists messages"
    ChatServer -> MessageQueue "Async processing"
}

UserA = person "Alice"
UserB = person "Bob"

// Scenario: 1-on-1 chat (user online)
scenario SendMessageOnline "Send Message - Recipient Online" {
    UserA -> ChatApp.ChatServer "Send 'Hello'"
    ChatApp.ChatServer -> ChatApp.MessageDB "Persist message"
    ChatApp.ChatServer -> ChatApp.SessionStore "Lookup Bob's connection"
    ChatApp.SessionStore -> ChatApp.ChatServer "Bob is on Server-2"
    ChatApp.ChatServer -> UserB "Push 'Hello' via WebSocket"
    UserB -> ChatApp.ChatServer "ACK received"
}

// Scenario: 1-on-1 chat (user offline)
scenario SendMessageOffline "Send Message - Recipient Offline" {
    UserA -> ChatApp.ChatServer "Send 'Hello'"
    ChatApp.ChatServer -> ChatApp.MessageDB "Persist message"
    ChatApp.ChatServer -> ChatApp.SessionStore "Lookup Bob's connection"
    ChatApp.SessionStore -> ChatApp.ChatServer "Bob is offline"
    ChatApp.ChatServer -> ChatApp.MessageDB "Mark as pending delivery"
}

// Scenario: Group chat (fan-out)
scenario SendGroupMessage "Send Group Message" {
    UserA -> ChatApp.ChatServer "Send 'Hello' to Group"
    ChatApp.ChatServer -> ChatApp.MessageDB "Persist message"
    ChatApp.ChatServer -> ChatApp.MessageQueue "Enqueue for fan-out"
    ChatApp.MessageQueue -> ChatApp.ChatServer "Process for each member"
    ChatApp.ChatServer -> ChatApp.SessionStore "Lookup each member's server"
    ChatApp.ChatServer -> UserB "Push to member 1"
    ChatApp.ChatServer -> UserC "Push to member 2"
    ChatApp.ChatServer -> UserD "Push to member 3"
}

// Scenario: Message history retrieval
scenario GetMessageHistory "Retrieve Message History" {
    UserA -> ChatApp.ChatServer "Request chat history"
    ChatApp.ChatServer -> ChatApp.MessageDB "Query messages"
    ChatApp.MessageDB -> ChatApp.ChatServer "Return messages"
    ChatApp.ChatServer -> UserA "Send history"
}

view index {
include *
}
```

---
title: "Queue"
weight: 8
summary: "A Queue represents a message queue, topic, or event stream."
---

# Queue

A **Queue** represents a message queue, topic, or event stream. It is a specialized type of Container used for asynchronous communication.

## Syntax

```sruja
queue ID "Label" {
    description "Optional description"
    technology "Technology"
}
```

## Example

```sruja
queue Events "Event Stream" {
    technology "Kafka"
    description "Handles domain events"
}
```

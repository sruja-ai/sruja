---
title: "Queue"
weight: 34
summary: "A Queue represents a message queue, topic, or event stream."
---

# Queue

A **Queue** represents a message queue, topic, or event stream. It is a specialized type of Container used for asynchronous communication.

## Syntax

```sruja
queue = kind "Queue"

ID = queue "Label" {
description "Optional description"
technology "Technology"
}
```

## Example

```sruja
queue = kind "Queue"

Events = queue "Event Stream" {
technology "Kafka"
description "Handles domain events"
}
```

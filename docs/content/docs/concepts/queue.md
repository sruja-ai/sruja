---
title: Queue
weight: 8
---

# Queue

A **Queue** represents a message queue, topic, or event stream. It is a specialized type of Container used for asynchronous communication.

## Syntax

```sruja
queue ID "Label/Name" {
    technology "Technology"
    description "Description"
    tags ["tag1", "tag2"]
}
```

## Example

```sruja
queue Events "Event Bus" {
    technology "Kafka"
    description "Handles domain events."
}
```

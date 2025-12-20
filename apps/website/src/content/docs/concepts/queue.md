---
title: "Queue"
weight: 34
summary: "A Queue represents a message queue, topic, or event stream."
---

# Queue

A **Queue** represents a message queue, topic, or event stream. It is a specialized type of Container used for asynchronous communication.

## Syntax

```sruja
specification {
  element queue
}

model {
  ID = queue "Label" {
    description "Optional description"
    technology "Technology"
  }
}
```

## Example

```sruja
specification {
  element queue
}

model {
  Events = queue "Event Stream" {
    technology "Kafka"
    description "Handles domain events"
  }
}
```

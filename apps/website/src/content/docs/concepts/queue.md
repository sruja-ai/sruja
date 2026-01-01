---
title: "Queue"
weight: 34
summary: "A Queue represents a message queue, topic, or event stream."
---

# Queue

A **Queue** represents a message queue, topic, or event stream. It is a specialized type of Container used for asynchronous communication.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


ID = queue "Label" {
description "Optional description"
technology "Technology"
}
```

## Example

```sruja
import { * } from 'sruja.ai/stdlib'


Events = queue "Event Stream" {
technology "Kafka"
description "Handles domain events"
}
```

---
title: Relations
weight: 5
---

# Relations

**Relations** describe how elements interact with each other. They are the lines connecting the boxes in your diagram.

## Syntax

```sruja
Source -> Destination "Label"
```

Or with a technology/protocol:

```sruja
Source -> Destination "Label" {
    technology "HTTPS/JSON"
}
```

## Example

```sruja
User -> WebApp "Visits"
WebApp -> DB "Reads Data"
```

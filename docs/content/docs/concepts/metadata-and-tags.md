---
title: Metadata & Tags
weight: 11
---

# Metadata & Tags

Sruja allows you to attach additional information to your elements using Metadata and Tags.

## Tags

Tags are simple string labels that can be used for filtering, styling, or categorization.

### Syntax

```sruja
tags ["tag1", "tag2"]
```

## Metadata

Metadata allows you to attach key-value pairs to elements. This is useful for storing information like team ownership, cost centers, links to other docs, etc.

### Syntax

```sruja
metadata {
    "Owner" "Team A"
    "CostCenter" "1234"
    "Repo" "github.com/org/repo"
}
```

## Technology

Most elements (Container, Component, etc.) support a `technology` field to specify the tech stack.

### Syntax

```sruja
technology "Go, gRPC"
```

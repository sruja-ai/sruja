---
title: "Metadata & Tags"
weight: 11
summary: "Attach additional information to your elements using Metadata and Tags."
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
system API "API" {
    metadata {
        owner "Team A"
        tier "1"
    }
}
```

## Technology

Most elements (Container, Component, etc.) support a `technology` field to specify the tech stack.

### Syntax

```sruja
metadata {
    key "value"
}
```

## See Also

- [Validation](/docs/concepts/validation)

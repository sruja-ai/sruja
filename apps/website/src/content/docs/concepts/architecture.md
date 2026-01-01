---
title: "Architecture"
weight: 10
summary: "The architecture block is the root element of any Sruja model."
---

# Architecture

The `architecture` block is the root element of a Sruja model. It represents the entire scope of what you are modeling.

## Syntax

### Explicit Block (Recommended for large projects)

```sruja
import { * } from 'sruja.ai/stdlib'


// ... define systems, persons, etc. here

view index {
include *
}
```

### Minimal Example

For simple examples, you can use a minimal structure:

```sruja
import { * } from 'sruja.ai/stdlib'


MySystem = system "My System"
User = person "User"
```

## Purpose

- **Scope Boundary**: Everything inside is part of the model.
- **Naming**: Gives a name to the overall architecture.

---
title: Architecture
weight: 1
summary: "The architecture block is the root element of any Sruja model."
---

# Architecture

The `architecture` block is the root element of a Sruja model. It represents the entire scope of what you are modeling.

## Syntax

### Explicit Block (Recommended for large projects)

```sruja
architecture "My System Name" {
    // ... define systems, persons, etc. here
}
```

### Implicit Block (Recommended for scripts/snippets)

You can also define elements at the top level without an `architecture` wrapper. Sruja will automatically wrap them in a default architecture.

```sruja
system MySystem "My System" { ... }
person User "User"
```

## Purpose

-   **Scope Boundary**: Everything inside is part of the model.
-   **Naming**: Gives a name to the overall architecture.


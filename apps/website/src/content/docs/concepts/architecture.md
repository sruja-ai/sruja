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
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
    // ... define systems, persons, etc. here
}

views {
  view index {
    include *
  }
}
```

### Minimal Example

For simple examples, you can use a minimal structure:

```sruja
specification {
  element system
  element person
}

model {
  MySystem = system "My System"
  User = person "User"
}
```

## Purpose

-   **Scope Boundary**: Everything inside is part of the model.
-   **Naming**: Gives a name to the overall architecture.


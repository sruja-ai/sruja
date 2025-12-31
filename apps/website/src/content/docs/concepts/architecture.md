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
person = kind "Person"
system = kind "System"
container = kind "Container"
component = kind "Component"
datastore = kind "Datastore"
queue = kind "Queue"

// ... define systems, persons, etc. here

view index {
include *
}
```

### Minimal Example

For simple examples, you can use a minimal structure:

```sruja
system = kind "System"
person = kind "Person"

MySystem = system "My System"
User = person "User"
```

## Purpose

- **Scope Boundary**: Everything inside is part of the model.
- **Naming**: Gives a name to the overall architecture.

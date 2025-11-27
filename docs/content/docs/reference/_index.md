---
title: DSL Reference
weight: 3
---

# DSL Reference

This section provides a comprehensive reference for the Sruja Domain Specific Language (DSL).

## Grammar

The Sruja DSL is designed to be human-readable and concise. It uses a C-like syntax with braces `{}` for grouping.

### Basic Rules

-   **Strings**: Double-quoted strings `"value"`.
-   **Identifiers**: Alphanumeric names (e.g., `WebApp`, `API_Service`).
-   **Comments**: `//` for single line, `/* ... */` for multi-line.

## Elements

### Architecture

```sruja
architecture "Name" { ... }
```

### System

```sruja
system ID "Label" { ... }
```

### Container

```sruja
container ID "Label" { ... }
```

### Component

```sruja
component ID "Label" { ... }
```

### Person

```sruja
person ID "Label"
```

### Relations

```sruja
From -> To "Label"
```

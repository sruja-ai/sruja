---
title: Introduction
type: docs
cascade:
  BookSection: docs
summary: "Sruja is a modern, developer-friendly language for defining software architecture."
---

# Sruja: Architecture as Code

**Sruja** is a modern, developer-friendly language for defining software architecture. It allows you to describe your systems, containers, and components using a clean, C4-model-based DSL, and then generate diagrams, documentation, and more.

## Why Sruja?

### Code-First
Treat your architecture like code. Version control it, review it, and evolve it. Sruja files are plain text, making them easy to diff and merge. This allows you to integrate architectural decision-making into your existing development workflows (Pull Requests, Code Reviews).

### C4 Model
Built-in support for the C4 model (System, Container, Component) ensures a standardized way to describe software. You don't need to invent your own boxes and arrows; Sruja provides the primitives you need to describe software systems at different levels of abstraction.

### Diagrams as Code
Generate beautiful diagrams (via D2) automatically from your model. No more dragging boxes around in Visio or Lucidchart. When your code changes, your diagrams update automatically. This ensures your documentation never goes stale.

### Validation
Ensure your architecture follows best practices with built-in validation rules. Sruja can check for things like:
-   Orphaned elements (defined but not connected).
-   Circular dependencies.
-   Missing descriptions or technology tags.
-   Compliance with architectural policies.

## Key Features

### Simple DSL
The Sruja DSL is designed to be easy to read and write. It uses a familiar C-style syntax with braces for grouping.

```sruja
system "MySystem" {
    container "WebApp" {
        technology "Go"
    }
}
```

### D2 Export
Sruja exports directly to D2, a modern diagram scripting language. D2 handles the layout and routing, producing high-quality SVG diagrams that look professional and are easy to read.

### Validation Engine
Catch architectural smells early. The validation engine runs every time you build or check your model, providing immediate feedback on the health of your architecture.

### Extensible
Built for the future. Sruja is designed to be extensible, allowing for new export formats, custom validation rules, and integration with other tools in your ecosystem.

## Getting Started

Ready to dive in? Check out the [Getting Started]({{< relref "getting-started" >}}) guide to install Sruja and build your first model.

## Community

Join the community to discuss architecture, share models, and contribute to Sruja.


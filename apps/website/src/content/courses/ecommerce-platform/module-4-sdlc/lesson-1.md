---
title: "Lesson 1: The Local Loop"
weight: 1
summary: "Using Sruja for local development and testing."
---

# Lesson 1: The Local Loop

How do you use Sruja while you code?

## 1. The Blueprint
Keep `architecture/main.sruja` open in a split pane. It is your map. Before you create a new file or function, verify where it fits in the architecture.

## 2. Generating Boilerplate (Future)
Imagine running `sruja gen` to scaffold your Go microservices based on your `container` definitions. While this feature is in development, you can manually align your folder structure to your architecture.

```text
src/
  orders/      # Matches 'container OrderService'
  inventory/   # Matches 'container InventoryService'
```

## 3. Local Validation
Before you commit, run:

```bash
sruja validate .
```

This checks for:
*   **Orphans**: Components defined but never used.
*   **Broken Links**: Relations pointing to non-existent elements.
*   **Policy Violations**: Did you accidentally introduce a circular dependency?

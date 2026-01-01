---
title: "Getting Started"
weight: 1
summary: "From zero to architecture in 5 minutes. Install Sruja and deploy your first diagram."
difficulty: "beginner"
estimatedTime: "5 minutes"
---

# Your First Architecture

Welcome to the future of system design.

Sruja allows you to define your software architecture as code. No more dragging boxes around. No more outdated PNGs on a wiki. **You write code, Sruja draws the maps.**

## 1. Installation

Install the Sruja CLI to compile, validate, and export your diagrams.

### Mac / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

### From Source (Go)

```bash
go install github.com/sruja-ai/sruja/cmd/sruja@latest
```

_Verify installation:_

```bash
sruja --version
# Should output something like: sruja version v0.2.0
```

---

## 2. Hello, World!

Let's model a simple web application. Create a file named `hello.sruja`.

### The Code

Copy and paste this into your file:

```sruja
// hello.sruja

system = kind "System"
container = kind "Container"
database = kind "Database"
person = kind "Person"

// 1. Define the System
webApp = system "My Cool Startup" {
    description "The next big thing."

    frontend = container "React App"
    api = container "Go Service"
    db = database "PostgreSQL"

    // 2. Define Connections
    frontend -> api "Requests Data"
    api -> db "Reads/Writes"
}

// 3. Define Users
user = person "Early Adopter"

// 4. Connect User to System
user -> webApp.frontend "Visits Website"
```

### 3. Generate the Diagram

Run this command in your terminal:

```bash
sruja export mermaid hello.sruja > diagram.mmd
```

You have just created a **Diagram-as-Code** artifact! You can paste the content of `diagram.mmd` into [Mermaid Live Editor](https://mermaid.live) to see it, or use the VS Code extension to preview it instantly.

> [!TIP]
> **VS Code User?**
> Install the [Sruja VS Code Extension](https://marketplace.visualstudio.com/) for real-time preview, autocomplete, and syntax highlighting.

---

## 4. Understanding the Basics

Let's break down what just happened.

1.  **Flat Syntax**: Sruja uses a flat syntax where all declarations are top-level. There are no `specification`, `model`, or `views` wrapper blocks required.
2.  **`kind` Declarations**: Before using elements, you must declare their "kind". This establishes the vocabulary (e.g., `system`, `container`, `person`) and enables:
    - **Early Validation**: Catches typos in element types.
    - **Custom Types**: You can define your own kinds like `microservice` or `serverless`.
3.  **Element Instantiation**: You create instances of kinds using the `=` operator (e.g., `webApp = system "My Cool Startup"`).
4.  **`->` Relationships**: Defines how things talk to each other. Relationships can be defined anywhere in the file.
5.  **Nested Elements**: Containers and components can be defined inside a system block `{ ... }`, or referred to using dot notation (e.g., `webApp.frontend`).
6.  **Views (Implicit/Explicit)**: Sruja automatically generates C4 diagrams for you. You can optionally define `view` blocks for custom perspectives.

---

## What Now?

You have the tools. Now get the skills.

- 🎓 **Learn the Core**: Take the **[System Design 101](/courses/system-design-101)** course to move beyond "Hello World".
- 🏗 **See Real Patterns**: Copy production-ready code from **[Examples](/docs/examples)**.
- 🛠 **Master the CLI**: Learn how to validate constraints in **[CLI Basics](/tutorials/basic/cli-basics)**.

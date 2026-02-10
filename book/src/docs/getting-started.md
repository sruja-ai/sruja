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

### Option A – install script (downloads from [GitHub Releases](https://github.com/sruja-ai/sruja/releases))

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

### Option B – from Git (requires Rust)

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
```

### Option C – build from source

```bash
git clone https://github.com/sruja-ai/sruja.git && cd sruja
make build
```

Ensure the `sruja` binary is on your `PATH` (install script uses `~/.local/bin` by default).

_Verify installation:_

```bash
sruja --version
# Should output something like: sruja version v0.2.0
```

---

## 2. Hello, World!

Let's model a simple web application. Create a file named `hello.sruja`.

This page uses **stdlib import** (same style as the rest of the book). The [Quick start](../getting-started.md) uses explicit kinds (`person = kind "Person"`, etc.) with no import — both are valid; use whichever you prefer.

### The Code

Copy and paste this into your file:

```sruja
// hello.sruja
import { * } from 'sruja.ai/stdlib'

// 1. Define the System
webApp = system "My Cool Startup" {
    description "The next big thing."

    frontend = container "React App"
    api = container "Rust Service"
    db = database "PostgreSQL"

    // 2. Define Connections
    frontend -> api "Requests Data"
    api -> db "Reads/Writes"
}

// 3. Define Users
user = person "Early Adopter"

// 4. Connect User to System
user -> webApp.frontend "Visits Website"

view index {
    include *
}
```

> [!TIP]
> **Using stdlib imports**: The `import { * } from 'sruja.ai/stdlib'` line provides all standard kinds (person, system, container, database, etc.) so you don't declare them manually. For the minimal style with explicit kinds, see [Quick start](../getting-started.md).

### 3. Generate the Diagram

Run this command in your terminal:

```bash
sruja export mermaid hello.sruja > diagram.mmd
```

You have just created a **Diagram-as-Code** artifact! You can paste the content of `diagram.mmd` into [Mermaid Live Editor](https://mermaid.live) to see it, or use the VS Code extension to preview it instantly.

**What you'll get:** A beautiful C4 diagram showing:

- The user (person) on the left
- Your system with containers (Web, API, DB) in the middle
- Relationships (arrows) showing how they connect

> [!TIP]
> **Want to see it visually?**
>
> - **VS Code User?** Install the [Sruja VS Code Extension](https://marketplace.visualstudio.com/) for real-time preview, autocomplete, and syntax highlighting.
> - **Preview in editor:** Use the [VS Code extension](../vscode.md) for syntax and diagram preview.

---

## 4. Understanding the Basics

Let's break down what just happened.

1.  **Import Standard Kinds**: The `import { * } from 'sruja.ai/stdlib'` line gives you all standard element types (person, system, container, database, etc.) without needing to declare them manually.
2.  **Element Creation**: You create elements using the `=` operator (e.g., `webApp = system "My Cool Startup"`).
3.  **Nested Elements**: Containers and components can be defined inside a system block `{ ... }`, or referred to using dot notation (e.g., `webApp.frontend`).
4.  **Relationships**: The `->` operator defines how things connect. Relationships can be defined anywhere in the file.
5.  **Views**: The `view index { include * }` block tells Sruja to generate a diagram showing all elements. Sruja automatically generates C4 diagrams for you.
6.  **Flat Syntax**: Sruja uses a flat syntax where all declarations are top-level. There are no wrapper blocks required.

> [!NOTE]
> **Custom Kinds**: If you need custom element types (like `microservice` or `serverless`), you can declare them manually: `microservice = kind "Microservice"`. For most use cases, the standard kinds from stdlib are sufficient.

---

## What Now?

You have the tools. Now get the skills.

- 🎓 **Learn the Core**: Take the **[System Design 101](courses/system-design-101/course-overview.md)** course to move beyond "Hello World".
- 🏗 **See Real Patterns**: Copy production-ready code from **[Examples](docs/examples.md)**.
- 🛠 **Master the CLI**: Learn how to validate constraints in **[CLI Basics](tutorials/basic/cli-basics.md)**.

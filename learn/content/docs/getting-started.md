---
title: Getting Started
weight: 1
summary: "Install Sruja and create your first architecture model."
---

# Getting Started with Sruja

This guide will help you install Sruja and create your first architecture model.

## Installation

### Automated Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

### Manual Download

Download the latest release for your operating system from the [GitHub Releases](https://github.com/sruja-ai/sruja/releases) page.

### From Source (Go)

If you have Go installed, you can still build from source:

```bash
go install github.com/sruja-ai/sruja/cmd/sruja@latest
```

Verify the installation:

```bash
sruja --version
```

## Your First Project

1.  **Create a file**: Create a new file named `architecture.sruja`.

2.  **Write your model**:

    {{< playground >}}
    architecture "My System" {
        system App "My App" {
            container Web "Web Server"
            datastore DB "Database"
        }
        person User "User"

        User -> Web "Visits"
        Web -> DB "Reads/Writes"
    }
    {{< /playground >}}

3.  **Visualize**: Export to D2 to see the diagram.

    ```bash
    sruja export d2 architecture.sruja > architecture.d2
    ```

    You can then render `architecture.d2` using the [D2 CLI](https://d2lang.com/) or online playground.

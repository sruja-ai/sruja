---
title: "Getting Started"
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

Note for macOS (Apple Silicon): If the automated installer reports a missing
Darwin arm64 binary, use one of the alternatives below (Manual Download or Go install).

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

If `sruja` is not found in your shell, add Go's `bin` directory to your `PATH`:

```bash
export PATH="$HOME/go/bin:$PATH"
```

## Your First Project
**Create a file**: Create a new file named `architecture.sruja`.

**Write your model**:
```sruja
system App "My App" {
  container Web "Web Server"
  datastore DB "Database"
}
person User "User"

User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"
```

**Visualize**: Open in Studio for an interactive diagram, or export Markdown with Mermaid for docs.

```bash
# Interactive visualization
# Open /studio/ and paste your DSL, or use the "Open in Studio" button in Learn

# Markdown export with Mermaid
sruja export mermaid architecture.sruja > architecture.md
```

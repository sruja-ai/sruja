# Sruja DSL Quick Start

Get started with Sruja DSL in 5 minutes.

## Installation

### Option 1: Download Binary

```bash
# macOS / Linux
curl -sSL https://sruja.dev/install.sh | bash

# Or download from releases
# https://github.com/sruja-ai/sruja/releases
```

### Option 2: Package Managers

```bash
# Homebrew (macOS)
brew install sruja

# npm (Node.js)
npm install -g sruja

# Go install
go install github.com/sruja-ai/sruja/apps/cli/cmd@latest
```

### Option 3: VSCode Extension

1. Open VSCode
2. Search for "Sruja" in Extensions
3. Click Install
4. Open any `.sruja` file to get started

---

## Your First Architecture (30 seconds)

### Step 1: Initialize a Project

```bash
sruja init
```

This creates:
- `architecture.sruja` - Your architecture file
- `README.md` - Documentation
- `.vscode/settings.json` - Editor settings

### Step 2: Choose a Template

```bash
# Basic starter
sruja init --template basic

# Microservices
sruja init --template microservices

# Event-driven
sruja init --template event-driven

# Monolith
sruja init --template monolith

# API Gateway
sruja init --template api-gateway

# Service Mesh
sruja init --template service-mesh
```

### Step 3: View Your Architecture

```bash
# Generate diagram
sruja compile architecture.sruja

# Validate
sruja lint architecture.sruja

# Get explanations
sruja explain API
```

---

## Quick Examples

### Minimal Architecture

```sruja
architecture "My System" {
  system API "API Service" {
    container App "Web App"
    datastore DB "Database"
  }
}
```

### With Relations

```sruja
architecture "E-Commerce" {
  system API "API Service" {
    container WebApp "Web Application"
    datastore DB "Database"
  }
  
  person User "Customer"
  
  User -> API "uses"
  API.WebApp -> API.DB "stores data"
}
```

### With Metadata

```sruja
architecture "My System" {
  system API "API Service" {
    metadata {
      team: "Platform"
      tier: "gold"
      owner: "platform-team@company.com"
    }
    
    container App "Web Application" {
      metadata {
        rate_limit: "100/s"
        auto_scale: "true"
      }
    }
  }
}
```

---

## Next Steps

### 1. Learn the Basics

- [DSL Syntax Guide](../specs/dsl-specification.md)
- [Core Concepts](../specs/core-concepts.md)

### 2. Try Advanced Features

- [Metadata & Extensions](../specs/metadata-model.md)
- [Imports & Multi-file](../specs/imports.md)
- [Journeys](../specs/journeys.md)

### 3. Explore Patterns

- [Microservices Pattern](../patterns/microservices.md)
- [Event-Driven Pattern](../patterns/event-driven.md)
- [API Gateway Pattern](../patterns/api-gateway.md)

### 4. Integrate with Your Workflow

- [Git Integration](../guides/git-integration.md)
- [CI/CD Integration](../guides/ci-cd.md)
- [Plugin Development](../guides/plugin-development.md)

---

## Common Commands

```bash
# Initialize project
sruja init

# Compile to diagram
sruja compile architecture.sruja

# Compile to specific format
sruja compile architecture.sruja --format mermaid
sruja compile architecture.sruja --format d2

# Validate
sruja lint architecture.sruja

# Format code
sruja fmt architecture.sruja

# Explain element
sruja explain BillingAPI

# List elements
sruja list systems
sruja list containers
```

---

## Editor Support

### VSCode

1. Install the Sruja extension
2. Open any `.sruja` file
3. Get autocomplete, hover docs, and error checking

### Other Editors

- **Vim/Neovim**: LSP client configuration
- **IntelliJ**: Language plugin (coming soon)
- **Emacs**: LSP client configuration

---

## Need Help?

- 📖 [Full Documentation](../README.md)
- 💬 [Community Discord](https://discord.gg/sruja)
- 🐛 [Issue Tracker](https://github.com/sruja-ai/sruja/issues)
- 📧 [Email Support](mailto:support@sruja.dev)

---

## What's Next?

Once you're comfortable with the basics:

1. **Add metadata** to your elements
2. **Create journeys** to document user flows
3. **Write ADRs** to document decisions
4. **Export diagrams** for documentation
5. **Try plugins** for cloud mappings and more

Welcome to Sruja DSL! 🚀


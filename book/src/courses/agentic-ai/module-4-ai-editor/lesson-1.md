---
title: "Lesson 1: MCP Server Setup"
weight: 1
summary: "Configure Sruja as an MCP server for AI code editors."
---

# Lesson 1: MCP Server Setup

## What Is MCP?

The **Model Context Protocol (MCP)** is a standardized way for AI tools to connect to external data sources and services. With MCP, AI editors like Cursor, Copilot, and others can query Sruja's architecture graph directly.

## Why MCP for Sruja?

Without MCP, AI editors don't know your architecture:
- AI suggests changes without understanding dependencies
- AI doesn't know which components are critical
- AI can't tell you the blast radius of a change

With MCP, AI editors can:
- Query architecture graph in real-time
- Understand component relationships
- Provide architecture-aware suggestions
- Warn about breaking changes

## Setting Up Sruja MCP

### 1. Install Sruja CLI

```bash
# From source
cargo install --path crates/sruja-cli

# Or from release
curl -fsSL https://sruja.ai/install.sh | bash
```

### 2. Verify MCP is Available

```bash
sruja mcp --help
```

### 3. Configure for Cursor

Create or edit `~/.cursor/mcp.json` (or `.cursor/mcp.json` in your project):

```json
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp", "-r", "."],
      "env": {}
    }
  }
}
```

### 4. Configure for VS Code (Copilot)

```json
// .vscode/mcp.json in your project
{
  "sruja": {
    "command": "sruja",
    "args": ["mcp", "-r", "."]
  }
}
```

### 5. Configure for Cline/Windsurf

```json
// .clinerules or .windsurf/rules/sruja.md
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp", "-r", "."]
    }
  }
}
```

## Testing the Connection

```bash
# Start MCP server in test mode
sruja mcp -r . --test

# Or start and watch logs
sruja mcp -r . --verbose
```

In your AI editor, try asking:
- "What components does the API depend on?"
- "Show me the architecture context"
- "What would break if I change the database?"

## MCP Tools Available

Once connected, the AI editor can use these Sruja tools:

| Tool | Purpose |
|------|---------|
| `sruja_query` | Query architecture graph |
| `sruja_why` | Explain why a component exists |
| `sruja_impact` | Analyze change blast radius |
| `sruja_context` | Get architecture context |
| `sruja_lint` | Validate DSL |

## Next Steps

Lesson 2 covers building and optimizing context for AI assistance.
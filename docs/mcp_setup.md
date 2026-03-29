# Sruja MCP Server Setup Guide

Connect Sruja to your favorite AI agent (Cursor, Claude Desktop, etc.) to provide architecture-aware context.

## 1. Prerequisites

- [Rust](https://rustup.rs/) (to build Sruja)
- An AI agent that supports the Model Context Protocol (MCP).

## 2. Installation

Build and install the Sruja CLI:

```bash
# From the sruja repository root
make install
# or
cargo install --path crates/sruja-cli
```

Verify the installation:

```bash
sruja --version
```

## 3. Configuration

### Claude Desktop

Add the following to your `claude_desktop_config.json` (usually at `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp"]
    }
  }
}
```

### Cursor

1. Open Cursor Settings.
2. Go to **Features** -> **MCP**.
3. Click "Add New MCP Server".
4. Name: `Sruja`
5. Type: `command`
6. Command: `sruja mcp --root /absolute/path/to/your/repo`

If you use the VS Code / Cursor extension, you can also run **Sruja: Register MCP Server (Cursor)** and it will write the same scoped command into `.cursor/mcp.json` for the selected workspace folder.

## 4. Available Tools

Sruja provides several tools to help AI agents understand your architecture:

- `sruja_get_architecture_summary`: Get a compact high-level overview.
- `sruja_get_neighbors`: Find what depends on a component or what it depends on.
- `sruja_find_path`: Trace the flow between two components.
- `sruja_get_entrypoints`: Identify system entrypoints (APIs, Services).
- `sruja_get_data_stores`: List databases and queues.

## 5. Workflow Example

**User:** "How does the payment flow work in this repo?"

**Agent (using Sruja):**
1. Calls `sruja_get_entrypoints` to find `PaymentController`.
2. Calls `sruja_get_neighbors(id="PaymentController")` to see downstream services.
3. Calls `sruja_find_path(source="PaymentController", target="PaymentGateway")` to trace the full flow.
4. Explains the architecture with high confidence.

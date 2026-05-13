# Sruja MCP Server Setup Guide

Connect Sruja to your favorite AI agent (Cursor, Claude Desktop, etc.) to provide architecture-aware context.

**Why MCP + Sruja instead of pasting the repo into the prompt?** Large, unstructured chat context degrades signal-to-noise and cost. Sruja keeps **reviewed architecture and relationships** outside the model window; the agent **pulls** compact, task-relevant facts through tools. That matches a **tool-use / grounding** layer while your editor remains the orchestrator. For how this relates to broader multi-agent topologies (supervisor, A2A, and so on), see [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md).

## 1. Prerequisites

- [Rust](https://rustup.rs/) (to build Sruja)
- An AI agent that supports the Model Context Protocol (MCP).

## 2. Installation

Build and install the Sruja CLI:

```bash
# From the sruja repository root
just install
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

## 4. Available tools

Sruja exposes **many** MCP tools (graph navigation, drift, NL retrieval, focus briefing, federation, proposals, and more). The canonical categorized list is **[mcp_tools_reference.md](mcp_tools_reference.md)** (kept aligned with `crates/sruja-cli/src/commands/mcp.rs`).

### Quick picks for new sessions

| Tool | When to use it |
|------|----------------|
| `sruja_get_architecture_summary` | First pass overview of the repo architecture. |
| `sruja_get_focus_briefing` | You have a **file** or **element id** and need blast radius + AI instructions. |
| `sruja_hybrid_query` | Natural-language question; auto-picks graph vs semantic vs hybrid retrieval. |
| `sruja_check_drift` | Compare code vs declared `.sruja` architecture. |
| `sruja_get_neighbors` / `sruja_find_path` | Local graph navigation between known ids. |

### Operator controls (stdio server)

| Variable | Purpose |
|----------|---------|
| `SRUJA_MCP_READONLY=1` | Only **read/query** tools are advertised and callable (no proposals, sandbox, scratchpad append, gate commands, or `sruja_agent_run`). |
| `SRUJA_MCP_LOG=1` | One JSON line per tool invocation on **stderr** (`tool`, `repo`, `ms`, `ok`, `error`) for pipelines and dashboards. |

Example (Cursor `mcp.json`):

```json
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp", "--root", "/absolute/path/to/your/repo"],
      "env": {
        "SRUJA_MCP_READONLY": "1",
        "SRUJA_MCP_LOG": "1"
      }
    }
  }
}
```

## 5. Workflow example

**User:** "How does the payment flow work in this repo?"

**Agent (using Sruja):**
1. Calls `sruja_get_entrypoints` to find `PaymentController`.
2. Calls `sruja_get_neighbors(id="PaymentController")` to see downstream services.
3. Calls `sruja_find_path(source="PaymentController", target="PaymentGateway")` to trace the full flow.
4. Explains the architecture with high confidence.

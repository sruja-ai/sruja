# Jupyter Kernel Protocol Integration

This package implements the Jupyter Kernel Messaging Protocol for the Sruja Architecture Kernel, enabling notebook execution in JupyterLab, VS Code, and other Jupyter-compatible clients.

## Overview

The Jupyter protocol integration wraps the Sruja Kernel (`pkg/kernel`) with a Jupyter-compliant messaging layer. This allows notebooks to execute Sruja DSL, queries, diagrams, and validations through standard Jupyter interfaces.

## Architecture

```
Jupyter Client (JupyterLab/VSCode)
    ↓
Jupyter Kernel Messaging Protocol (JSON messages)
    ↓
Jupyter Server (pkg/kernel/jupyter)
    ↓
Sruja Kernel (pkg/kernel)
    ↓
Architecture Model (IR)
```

## Protocol Implementation

### Message Types Supported

- ✅ `kernel_info_request` - Kernel capabilities and info
- ✅ `execute_request` - Execute code cells
- ✅ `complete_request` - Autocomplete support
- ✅ `inspect_request` - Hover/inspection support
- ✅ `is_complete_request` - Syntax completeness check
- ✅ `shutdown_request` - Shutdown/restart kernel

### Transport Modes

**Current Implementation:**
- ✅ **stdio** - Standard input/output (works with VSCode, JupyterLite)

**Future:**
- ⏳ **ZeroMQ** - Classic Jupyter transport (for JupyterLab classic)
- ⏳ **WebSocket** - For web-based clients

## Usage

### Running the Kernel

The kernel can be started as a standalone process:

```bash
go run apps/kernel/main.go
```

Or installed as a Jupyter kernel:

```bash
# Install kernel spec
jupyter kernelspec install pkg/kernel/jupyter/kernel.json --name sruja
```

### Kernel Spec

The kernel spec (`kernel.json`) defines how Jupyter clients should launch the kernel:

```json
{
  "argv": ["sruja-kernel", "-f", "{connection_file}"],
  "display_name": "Sruja Architecture",
  "language": "sruja",
  "interrupt_mode": "message"
}
```

## Message Flow

### Execute Request Example

**Client → Kernel:**
```json
{
  "header": {
    "msg_type": "execute_request",
    "msg_id": "abc123"
  },
  "content": {
    "code": "system Billing { container BillingAPI {} }",
    "silent": false,
    "store_history": true
  }
}
```

**Kernel → Client (iopub):**
```json
{
  "header": {
    "msg_type": "display_data",
    "msg_id": "def456"
  },
  "parent_header": {...},
  "content": {
    "data": {
      "text/plain": "Architecture updated",
      "application/sruja-ir+json": "{...}"
    }
  }
}
```

**Kernel → Client (shell):**
```json
{
  "header": {
    "msg_type": "execute_reply",
    "msg_id": "abc123-reply"
  },
  "content": {
    "status": "ok",
    "execution_count": 1
  }
}
```

## Cell Type Detection

The server automatically detects cell types from code content:

- **Magic commands** (`%ir`, `%snapshot`, etc.) → DSL cells
- **Diagram commands** (`diagram system X`) → Diagram cells
- **Validation commands** (`validate all`) → Validation cells
- **Query commands** (`select systems...`) → Query cells
- **Default** → DSL cells

## Output Formatting

Kernel outputs are formatted as Jupyter `display_data` messages with appropriate MIME types:

- `text/plain` - Human-readable text
- `application/sruja-ir+json` - Architecture IR (JSON)
- `application/sruja-diagnostics+json` - Validation diagnostics
- `text/mermaid` - Mermaid diagrams
- `text/d2` - D2 diagrams
- `image/svg+xml` - SVG diagrams (future)

## Extending

### Adding New Message Types

1. Add message structs to `protocol.go`
2. Add handler method to `server.go`
3. Route in `handleMessage()`

### ZeroMQ Transport ✅

ZeroMQ transport is now fully implemented:

1. ✅ ZeroMQ library integrated (`github.com/go-zeromq/zmq4`)
2. ✅ Connection file parsing (`ParseConnectionFile`)
3. ✅ ZeroMQ sockets (shell, iopub, stdin, control, heartbeat)
4. ✅ Message routing across sockets
5. ✅ Automatic transport selection (stdio if no connection file, ZeroMQ if connection file provided)

**Usage:**
```bash
# ZeroMQ mode (classic JupyterLab)
go run apps/kernel/main.go -f /path/to/connection.json

# stdio mode (VSCode, JupyterLite)
go run apps/kernel/main.go
```

## Testing

See `pkg/kernel/jupyter/*_test.go` for tests (to be implemented).

## References

- [Jupyter Kernel Messaging Protocol](https://jupyter-client.readthedocs.io/en/stable/messaging.html)
- [Jupyter Kernel Spec](https://jupyter-client.readthedocs.io/en/stable/kernels.html#kernel-specs)


# Kernel Messaging Protocol

[← Back to Notebooks Index](./README.md)

## Overview

The Sruja Kernel integrates with the **Jupyter Kernel Messaging Protocol** to enable notebook execution. This document describes how Sruja operations map to Jupyter messages.

## Jupyter Kernel Messaging Protocol

The protocol operates over **ZeroMQ** (classic) or **WebSockets** (JupyterLite/VSCode) using several message types:

### Channels

- `shell` - Request/response for execution
- `iopub` - Kernel output stream
- `stdin` - User input
- `control` - Interrupts, restarts

### Message Types

- `execute_request` - Execute code
- `execute_reply` - Execution result
- `stream` - Text output
- `display_data` - Rich output (diagrams, diagnostics)
- `error` - Execution error
- `inspect_request` - Hover information
- `complete_request` - Autocomplete
- `kernel_info_request` - Kernel capabilities
- `shutdown_request` - Shutdown kernel

## Message Mapping

### DSL Cell Execution

**Jupyter → Kernel:**
```json
{
  "msg_type": "execute_request",
  "content": {
    "code": "system Billing { ... }",
    "silent": false,
    "store_history": true
  }
}
```

**Kernel Behavior:**
1. Parse DSL → AST
2. Update Architecture Model (IR)
3. Run validators
4. Generate diagnostics
5. Generate diagrams (if requested)
6. Store IR in kernel state

**Kernel → UI (iopub):**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "text/plain": "Architecture updated",
      "application/sruja-ir+json": "{... full IR ...}",
      "image/svg+xml": "<svg>...</svg>",
      "application/sruja-diagnostics+json": [...]
    }
  }
}
```

**Kernel → UI (shell):**
```json
{
  "msg_type": "execute_reply",
  "content": {
    "status": "ok"
  }
}
```

### Query Cell Execution

**Example:**
```json
{
  "code": "select systems where tags contains 'public'"
}
```

**Kernel → UI:**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "application/json": [... query results ...],
      "text/plain": "<table representation>"
    }
  }
}
```

### Diagram Cell Execution

**Example:**
```json
{
  "code": "diagram system Billing"
}
```

**Kernel → UI:**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "image/svg+xml": "<svg>...</svg>",
      "text/mermaid": "graph TD\n  A-->B"
    }
  }
}
```

### Validation Cell Execution

**Example:**
```json
{
  "code": "validate event PaymentCompleted"
}
```

**Kernel → UI:**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "application/sruja-diagnostics+json": [
        {
          "severity": "error",
          "message": "Missing required field",
          "location": {"line": 5}
        }
      ]
    }
  }
}
```

### AI Cell Execution

**Example:**
```json
{
  "code": "ai refine system Billing for reliability"
}
```

**Kernel Action:**
1. Detect AI cell type from metadata
2. Forward content to AI/MCP backend (Cursor AI)
3. Receive suggestions
4. Return to UI as diff or natural language

**Kernel → UI:**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "text/plain": "AI Suggested Changes:\n- Improve retry policies\n- Add circuit breaker",
      "application/sruja-diff+json": [
        {"patch": "..."}
      ]
    }
  }
}
```

## Special Commands

### Snapshots

**Magic command:**
```
%snapshot "iteration-12"
```

**Kernel → UI:**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "application/sruja-snapshot+json": {
        "name": "iteration-12",
        "ir": {...}
      },
      "text/plain": "Snapshot 'iteration-12' created."
    }
  }
}
```

### Variants

**Magic command:**
```
%variant create "async-payments" base="iteration-12"
```

**Kernel → UI:**
```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "application/sruja-variant+json": {...},
      "text/plain": "Variant 'async-payments' created."
    }
  }
}
```

**Note:** Variants and snapshots are stored in notebook metadata, not in cells.

## LSP-Like Features

### Autocomplete

**Jupyter → Kernel:**
```json
{
  "msg_type": "complete_request",
  "content": {
    "code": "sys",
    "cursor_pos": 3
  }
}
```

**Kernel → UI:**
```json
{
  "msg_type": "complete_reply",
  "content": {
    "matches": ["system", "system Billing", ...]
  }
}
```

### Hover / Inspect

**Jupyter → Kernel:**
```json
{
  "msg_type": "inspect_request",
  "content": {
    "code": "PaymentCompleted",
    "cursor_pos": 15
  }
}
```

**Kernel → UI:**
```json
{
  "msg_type": "inspect_reply",
  "content": {
    "status": "ok",
    "data": {
      "text/plain": "Event PaymentCompleted\nFields: ...",
      "application/sruja-ir+json": {...}
    }
  }
}
```

### Diagnostics

Diagnostics appear as `display_data` messages:

```json
{
  "msg_type": "display_data",
  "content": {
    "data": {
      "application/sruja-diagnostics+json": [...]
    }
  }
}
```

## Custom MIME Types

Sruja defines custom MIME types for rich architecture output:

- `application/sruja-ir+json` - Architecture IR
- `application/sruja-diagnostics+json` - Validation diagnostics
- `application/sruja-diff+json` - Contract/entity diffs
- `application/sruja-simulation+json` - Event simulation results
- `application/sruja-snapshot+json` - Snapshot data
- `application/sruja-variant+json` - Variant data

Standard MIME types:
- `image/svg+xml` - SVG diagrams
- `text/mermaid` - Mermaid diagrams
- `text/d2` - D2 diagrams

## Execution Flow

### End-to-End Flow

**User runs a DSL cell (`SHIFT+ENTER`):**

1. **Jupyter → Kernel**: `execute_request`
2. **Kernel**:
   - Parse DSL
   - Update IR
   - Run validators
   - Generate diagrams
   - Emit outputs
3. **Kernel → Jupyter**: `stream`, `display_data`, `execute_reply`
4. **UI renders**:
   - Diagnostics panel
   - Diagrams
   - IR explorer

Snapshots/variants update notebook metadata.

## Error Handling

**Kernel → UI:**
```json
{
  "msg_type": "error",
  "content": {
    "ename": "ParserError",
    "evalue": "Unexpected token at line 5",
    "traceback": [...]
  }
}
```

Additionally, semantic errors go through diagnostics:
```
application/sruja-diagnostics+json
```

## Interrupts / Restarts

**Kernel must handle:**
- `interrupt_request` - Stop execution
- `shutdown_request` - Clear Architecture IR

**Architecture IR resets on:**
- Restart kernel
- Load new notebook
- `%reset` magic command

## Magic Commands

Lightweight commands supported:

```
%ir                    # Show current IR
%diagram system Billing
%validate all
%snapshot iteration-3
%variant list
```

These are sent as normal `execute_request` messages and parsed by the kernel.

## VS Code Notebook API

VS Code treats `.ipynb` notebooks generically. The kernel is accessed through the **Jupyter Notebook Controller API**.

**Outputs appear as:**
- Text
- HTML
- Images
- Custom renderers for Sruja MIME types

**VS Code supports custom renderers via:**
- `notebookRenderer` extension point
- WebView-based visuals

So `application/sruja-ir+json` can open the Model Explorer UI.

## Summary

This mapping shows how the **Sruja Kernel** integrates with the **Jupyter Kernel Messaging Protocol**:

✅ DSL → `execute_request` → parse + update IR
✅ Query cells → `execute_request` → JSON tables
✅ Diagram cells → `display_data` (SVG/Mermaid)
✅ Validation cells → diagnostics
✅ AI cells → external AI → diff outputs
✅ Snapshots & variants → notebook metadata + MIME types
✅ LSP-like features → `inspect_request`, `complete_request`
✅ Rich outputs → custom MIME types
✅ Zero modifications to `.ipynb` format
✅ Full interop with JupyterLab & VS Code notebooks

## Next Steps

- [Architecture Kernel](./kernel.md) - Kernel implementation details
- [WASM Execution](./wasm-execution.md) - Browser-based execution


# Notebook Format

[← Back to Notebooks Index](./README.md)

## Overview

Sruja notebooks use the **standard Jupyter Notebook format (`.ipynb`)** - a JSON-based format that's:

- ✅ Portable across platforms
- ✅ Git-friendly
- ✅ Tool-compatible (JupyterLab, VS Code, etc.)
- ✅ Extensible via metadata

## Why `.ipynb`?

**Advantages:**

- ✅ No reinventing the notebook format
- ✅ Rich outputs supported (SVG, markdown, JSON)
- ✅ VS Code and JupyterLab can open it out of the box
- ✅ Cell execution model already exists
- ✅ Kernel protocol (Jupyter Kernel Messaging) is well-established
- ✅ Tools like nbconvert, nbviewer work
- ✅ Git diff tools for notebooks already exist
- ✅ Collaboration is easy
- ✅ Extensions exist for custom renderers

This shortens implementation by *months*.

## Basic Structure

A Sruja notebook is a JSON file:

```json
{
  "nbformat": 4,
  "nbformat_minor": 5,
  "metadata": {
    "sruja": {
      "kernel_version": "1.0",
      "config": {}
    }
  },
  "cells": [
    {
      "cell_type": "code",
      "metadata": {
        "sruja_cell_type": "dsl"
      },
      "source": "system Billing { ... }",
      "outputs": []
    }
  ]
}
```

## Metadata

### Notebook-Level Metadata

Store Sruja-specific configuration in notebook metadata:

```json
{
  "metadata": {
    "sruja": {
      "kernel_version": "1.0",
      "config": {
        "auto_validate": true,
        "auto_diagram": false
      },
      "snapshots": {
        "iteration-12": {
          "created_at": "2024-01-01T00:00:00Z",
          "ir": { /* JSON IR snapshot */ }
        }
      },
      "variants": {
        "async-payments": {
          "base": "iteration-12",
          "patches": [
            { "op": "update", "id": "Payment", ... }
          ]
        }
      }
    }
  }
}
```

### Cell-Level Metadata

Each cell can have Sruja-specific metadata:

```json
{
  "cell_type": "code",
  "metadata": {
    "sruja_cell_type": "dsl",  // or "query", "diagram", "ai", "validation"
    "name": "BillingAPI DSL",
    "tags": ["architecture", "billing"]
  },
  "source": "system Billing { ... }"
}
```

## Cell Types

### DSL Cell

```json
{
  "cell_type": "code",
  "metadata": {
    "sruja_cell_type": "dsl"
  },
  "source": "system Billing {\n  container BillingAPI { ... }\n}"
}
```

### Query Cell

```json
{
  "cell_type": "code",
  "metadata": {
    "sruja_cell_type": "query"
  },
  "source": "select components where constraints.pii == true"
}
```

### Diagram Cell

```json
{
  "cell_type": "code",
  "metadata": {
    "sruja_cell_type": "diagram"
  },
  "source": "diagram system Billing"
}
```

### Validation Cell

```json
{
  "cell_type": "code",
  "metadata": {
    "sruja_cell_type": "validation"
  },
  "source": "validate system Billing"
}
```

### AI Cell

```json
{
  "cell_type": "code",
  "metadata": {
    "sruja_cell_type": "ai"
  },
  "source": "ai refine system Billing for reliability"
}
```

### Markdown Cell

Standard markdown cells (no special metadata needed):

```json
{
  "cell_type": "markdown",
  "source": "# Architecture Overview\n\nThis notebook defines the Billing system..."
}
```

## Output Formats

### Standard Text Output

```json
{
  "output_type": "stream",
  "name": "stdout",
  "text": "Validated successfully"
}
```

### Rich Output (Architecture IR)

```json
{
  "output_type": "display_data",
  "data": {
    "text/plain": "Architecture updated",
    "application/sruja-ir+json": "{ ... full IR ... }"
  }
}
```

### Diagram Output (SVG)

```json
{
  "output_type": "display_data",
  "data": {
    "image/svg+xml": "<svg>...</svg>",
    "text/mermaid": "graph TD\n  A-->B"
  }
}
```

### Diagnostics Output

```json
{
  "output_type": "display_data",
  "data": {
    "application/sruja-diagnostics+json": [
      {
        "severity": "error",
        "message": "Missing required field",
        "location": { "file": "cell", "line": 5 }
      }
    ]
  }
}
```

## Custom MIME Types

Sruja defines custom MIME types:

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

## Snapshots and Variants

Stored in notebook metadata (not in cells):

```json
{
  "metadata": {
    "sruja": {
      "snapshots": {
        "iteration-12": {
          "created_at": "2024-01-01T00:00:00Z",
          "ir": { /* full ArchitectureStore JSON */ }
        }
      },
      "variants": {
        "async-payments": {
          "base": "iteration-12",
          "created_at": "2024-01-02T00:00:00Z",
          "patches": [
            {
              "operation": "update",
              "elementType": "entity",
              "elementId": "Payment",
              "payload": { "fields": [...] }
            }
          ]
        }
      }
    }
  }
}
```

## Example Notebook

Complete example:

```json
{
  "nbformat": 4,
  "nbformat_minor": 5,
  "metadata": {
    "kernelspec": {
      "display_name": "Sruja Architecture Kernel",
      "language": "sruja",
      "name": "sruja"
    },
    "sruja": {
      "kernel_version": "1.0"
    }
  },
  "cells": [
    {
      "cell_type": "markdown",
      "source": "# Billing System Architecture"
    },
    {
      "cell_type": "code",
      "metadata": { "sruja_cell_type": "dsl" },
      "source": "domain Payments {\n  entity Payment { ... }\n}",
      "outputs": []
    },
    {
      "cell_type": "code",
      "metadata": { "sruja_cell_type": "diagram" },
      "source": "diagram domain Payments",
      "outputs": [
        {
          "output_type": "display_data",
          "data": {
            "image/svg+xml": "<svg>...</svg>"
          }
        }
      ]
    }
  ]
}
```

## Compatibility

Sruja notebooks are compatible with:

- ✅ JupyterLab
- ✅ VS Code (Notebook API)
- ✅ Jupyter Notebook viewer
- ✅ nbconvert (export to HTML, PDF, etc.)
- ✅ Git (version control)
- ✅ GitHub (rendering)

## Customization

You **DO NOT** modify:
- `.ipynb` structure
- Jupyter client behavior

You **DO** implement:
- Sruja Kernel (Go + WASM)
- Custom JupyterLab renderer for Sruja diagrams
- VS Code Notebook Renderer for Sruja IR
- AI integration layer
- MCP architectural drift alignment module

Everything else is standard Jupyter.

## Next Steps

- [Kernel Messaging Protocol](./kernel-messaging.md) - How kernel communicates
- [Architecture Kernel](./kernel.md) - Execution engine details


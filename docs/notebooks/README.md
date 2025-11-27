# Sruja Architecture Notebooks

A notebook-based workspace for interactive architecture design, validation, and AI-assisted development.

[← Back to Documentation Index](../README.md)

## Overview

Sruja Architecture Notebooks provide a **Jupyter notebook-like environment** for architecture design, where you can:

- ✅ Write and execute DSL cells interactively
- ✅ Validate architecture incrementally
- ✅ Generate diagrams on-demand
- ✅ Run SrujaQL queries
- ✅ Create snapshots and variants
- ✅ Use AI-assisted architecture refinement
- ✅ Simulate event lifecycles
- ✅ Track architecture evolution

Think: **Jupyter Notebooks + Architecture DSL + AI + MCP Integration**

## Core Concept

A Sruja Notebook is a **stateful architecture workspace** that behaves like:

- **A notebook** (cells, incremental execution, history)
- **A compiler** (validates, builds graphs, checks constraints)
- **An architecting IDE** (LSP, AI, diagrams)
- **A workflow engine** (reviews, approvals, variants)
- **A model repository** (architecture as code)

## Documentation

### Getting Started
- **[Getting Started Tutorial](./tutorials/getting-started.md)** - Step-by-step guide to your first notebook
- **[Examples](./examples/README.md)** - Practical examples and patterns
- **[Advanced Patterns](./tutorials/advanced-patterns.md)** - Complex architecture patterns

### Core Documentation
- **[Notebook Overview](./overview.md)** - What is a Sruja Notebook?
- **[Architecture Kernel](./kernel.md)** - The execution engine behind notebooks
- **[Notebook Format](./format.md)** - Using `.ipynb` format with Sruja
- **[Git-Based Workflow](./git-workflow.md)** - Review, approval, and governance via Git
- **[Kernel Messaging Protocol](./kernel-messaging.md)** - Jupyter kernel protocol integration
- **[WASM Execution](./wasm-execution.md)** - Browser-based kernel execution
- **[Cursor AI Integration](./cursor-ai-integration.md)** - Using Cursor AI with notebooks
- **[MCP Tools Reference](./mcp-tools.md)** - Complete MCP tool definitions

### Reference
- **[Performance Guide](./performance.md)** - Optimization strategies and best practices
- **[Implementation Status](./IMPLEMENTATION-STATUS.md)** - Current implementation status
- **[Completed Features](./COMPLETED-FEATURES.md)** - Implemented features
- **[Final Status](./FINAL-STATUS.md)** - Complete feature summary

## Quick Start

### Creating a Notebook

Sruja notebooks use the standard `.ipynb` format with Sruja-specific metadata:

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
      "source": "system Billing {\n  container BillingAPI { ... }\n}"
    }
  ]
}
```

### Notebook Cell Types

1. **DSL Cells** - Architecture DSL code
2. **Query Cells** - SrujaQL queries
3. **Diagram Cells** - Generate diagrams
4. **Validation Cells** - Run validations
5. **Simulation Cells** - Event lifecycle simulation
6. **Markdown Cells** - Documentation and notes

## Benefits

- **Iterative Design** - Build architecture incrementally
- **Visual Feedback** - See diagrams and diagnostics immediately
- **AI-Powered** - Get suggestions and fixes from Cursor AI
- **Versioned** - Git-based versioning with snapshots and variants
- **Governed** - Automated review and approval workflows
- **Executable** - Architecture that validates and runs
- **Portable** - Standard `.ipynb` format works everywhere
- **Performant** - Optimized with caching and incremental operations

## Git-Based Governance

Notebooks integrate seamlessly with Git workflows:

- ✅ Architecture changes as PRs
- ✅ Automated validation in CI/CD
- ✅ Policy-based approval requirements
- ✅ Full audit trail in Git history
- ✅ Variants as branches, snapshots as commits

See [Git-Based Workflow](./git-workflow.md) for complete details.

## Status

**✅ Production Ready** - The Sruja Kernel is production-ready with all core features implemented!

### ✅ Completed Features (9/9)
- ✅ Query Engine Integration (SrujaQL)
- ✅ Diagram Generation (Mermaid & D2)
- ✅ Enhanced Validation Cells
- ✅ Magic Commands Support
- ✅ Event Simulation Engine
- ✅ Enhanced Variant Diff/Merge
- ✅ WASM Compilation
- ✅ Jupyter Protocol Integration (stdio)
- ✅ ZeroMQ Transport

**Infrastructure Complete:**
- ✅ Performance optimizations (caching)
- ✅ Comprehensive documentation
- ✅ Example notebooks and tutorials

See [Final Status](./FINAL-STATUS.md) for complete details.

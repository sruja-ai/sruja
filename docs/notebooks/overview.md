# Sruja Notebook Overview

[← Back to Notebooks Index](./README.md)

## What is a Sruja Architecture Notebook?

A Sruja Notebook is a **Jupyter notebook-like environment** for interactive architecture design. It combines:

- **Interactive DSL editing** (like code cells in Jupyter)
- **Stateful architecture kernel** (maintains architecture state between cells)
- **Live validation** (see errors and warnings immediately)
- **Diagram generation** (visual architecture views)
- **AI co-authoring** (Cursor AI integration)
- **Version control** (snapshots and variants)

## The Core Idea

Instead of editing static `.sruja` files and running CLI commands, you work in an **interactive notebook** where:

1. You write architecture DSL in cells
2. Execute cells to see results immediately
3. Build architecture incrementally
4. Get instant feedback (diagrams, validations, diagnostics)
5. Use AI to refine and improve architecture
6. Create variants for experimentation

## Notebook vs. Traditional DSL Files

| Feature | Traditional DSL Files | Sruja Notebooks |
|---------|---------------------|-----------------|
| **Editing** | Static files | Interactive cells |
| **Execution** | CLI commands | Cell execution |
| **Feedback** | After compilation | Immediate |
| **State** | File-based | In-memory kernel |
| **Iteration** | Edit → Run → Check | Live editing |
| **AI Integration** | External tools | Built-in |
| **Diagrams** | Generate separately | Inline in cells |

## Workflow Example

### Traditional Workflow

```bash
# Edit file
vim architecture.sruja

# Compile
sruja compile architecture.sruja

# Check for errors
sruja validate architecture.sruja

# Generate diagram
sruja diagram architecture.sruja

# See results, iterate...
```

### Notebook Workflow

```python
# Cell 1: Define domain
domain Payments {
  entity Payment { ... }
}

# Cell 2: Define events  
events {
  event PaymentAuthorized { ... }
}

# Cell 3: Validate (instant feedback)
validate entity Payment

# Cell 4: Generate diagram (inline)
diagram lifecycle Payment
```

All in one place, with immediate feedback.

## Key Features

### 1. Incremental Execution

The kernel maintains state between cells:

```python
# Cell 1: Define system
system Billing {
  container BillingAPI { ... }
}

# Cell 2: Add component (kernel remembers system)
modify BillingAPI add component PaymentService

# Cell 3: Query what we've built
select components where system == "Billing"
```

### 2. Live Validation

See errors as you type:

```python
# Cell with errors
entity Payment {
  amount: Float
  # Error: missing required field 'currency'
}

# Kernel immediately shows diagnostic
```

### 3. Inline Diagrams

Diagrams appear in cell outputs:

```python
# Cell
diagram system Billing

# Output: SVG diagram rendered inline
```

### 4. AI Integration

Cursor AI can help refine architecture:

```python
# Cell
ai refine system Billing for reliability

# AI suggests: retry policies, circuit breakers, etc.
```

### 5. Snapshots and Variants

Experiment safely:

```python
# Save checkpoint
snapshot "iteration-5"

# Create variant
variant "async-payments" from "iteration-5"

# Experiment...
# Merge back if successful
```

## Cell Types

### DSL Cells

Write architecture DSL:

```python
system Billing {
  container BillingAPI {
    component PaymentService {
      api POST /payments
    }
  }
}
```

### Query Cells

Use SrujaQL to query architecture:

```python
select components where constraints.pii == true
graph dependencies of Billing
diff contracts BillingAPI 1.0 vs 2.0
```

### Validation Cells

Run validations:

```python
validate system Billing
validate event PaymentCompleted
validate all
```

### Diagram Cells

Generate diagrams:

```python
diagram system Billing
diagram lifecycle Payment
diagram event-flow PaymentCompleted
```

### AI Cells

Get AI assistance:

```python
ai suggest improvements for Payment entity
ai propose variant: async refund workflow
ai fix all violations
```

### Simulation Cells

Simulate event flows:

```python
simulate PaymentLifecycle from PENDING
simulate events: PaymentAuthorized, PaymentCompleted
```

## Architecture Kernel

The **Architecture Kernel** is the execution engine:

- Parses DSL
- Maintains architecture state (IR)
- Runs validators
- Generates diagrams
- Executes queries
- Manages snapshots/variants

See [Architecture Kernel](./kernel.md) for details.

## Execution Model

When you execute a cell:

1. Kernel parses DSL → AST
2. Updates architecture model (IR)
3. Runs validators
4. Produces diagnostics
5. Generates outputs (diagrams, tables, etc.)
6. Updates kernel state

The kernel is **stateful** - it remembers everything between cells.

## File Format

Sruja notebooks use **standard `.ipynb` format** (Jupyter notebook format):

- Works with JupyterLab, VS Code, and other notebook tools
- Git-friendly JSON format
- Portable across platforms
- Supports rich outputs (SVG, Mermaid, JSON)

See [Notebook Format](./format.md) for details.

## Git-Based Governance

Sruja notebooks integrate seamlessly with Git for architecture governance:

- ✅ Architecture changes as PRs
- ✅ Automated validation in CI/CD
- ✅ Policy-based approval workflows
- ✅ Full audit trail in Git history
- ✅ Variants as branches, snapshots as commits

See [Git-Based Workflow](./git-workflow.md) for complete details on review and approval processes.

## Next Steps

- [Architecture Kernel](./kernel.md) - How the kernel works
- [Notebook Format](./format.md) - `.ipynb` format details
- [Git-Based Workflow](./git-workflow.md) - Review, approval, and governance
- [Cursor AI Integration](./cursor-ai-integration.md) - AI-powered workflows
- [MCP Tools Reference](./mcp-tools.md) - Available MCP tools


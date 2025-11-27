# Getting Started with Sruja Notebooks

[← Back to Notebooks Index](../README.md)

## Introduction

Sruja Architecture Notebooks provide an interactive environment for designing, validating, and evolving software architecture. This tutorial will guide you through creating your first architecture notebook.

## Prerequisites

- JupyterLab, VSCode with Jupyter extension, or compatible notebook environment
- Sruja kernel installed (see installation guide)
- Basic understanding of software architecture concepts

## Step 1: Create Your First Notebook

1. Open your notebook environment
2. Create a new notebook
3. Select "Sruja" as the kernel

## Step 2: Define Your First System

In a new cell, type:

```sruja
system MyApplication {
  description "My first architecture system"
  
  container WebApp {
    description "Web application"
    technology "React"
  }
  
  container Backend {
    description "Backend API"
    technology "Node.js"
  }
  
  relation WebApp -> Backend {
    type uses
    description "API calls"
  }
}
```

Execute the cell. You should see:
- ✅ Success message
- No diagnostics (or warnings if any)

## Step 3: Query Your Architecture

In a new cell, query what you've created:

```sruja
find systems
```

This returns all systems in your architecture.

Try more queries:

```sruja
// Find all containers
find containers

// Find containers in MyApplication
find containers in MyApplication

// Find relations
find relations
```

## Step 4: Generate a Diagram

Visualize your architecture:

```sruja
diagram MyApplication format mermaid
```

This generates a Mermaid diagram showing your system structure.

## Step 5: Validate Your Architecture

Check for issues:

```sruja
validate all
```

Review any diagnostics and fix issues.

## Step 6: Use Magic Commands

Try some magic commands:

```sruja
// View the internal representation
%ir

// Create a snapshot
%snapshot create initial "First version"

// List snapshots
%snapshot list
```

## Step 7: Build Incrementally

Add more components:

```sruja
system MyApplication {
  // ... existing code ...
  
  container Database {
    description "Data storage"
    technology "PostgreSQL"
  }
  
  relation Backend -> Database {
    type uses
    description "Data persistence"
  }
}
```

Notice how the model updates incrementally.

## Step 8: Explore Advanced Features

### Create a Variant

```sruja
// Create a variant for experimentation
%variant create experimental initial "Trying new approach"
```

### Simulate Events

```sruja
// Define entity with lifecycle
entity Order {
  lifecycle {
    CREATED -> PROCESSING
    PROCESSING -> COMPLETED
  }
}

// Simulate lifecycle
simulate Order from CREATED events: OrderProcessed
```

## Common Workflows

### Workflow 1: Design → Validate → Visualize

1. Design architecture in DSL cells
2. Validate with `validate all`
3. Generate diagrams
4. Iterate based on feedback

### Workflow 2: Experiment with Variants

1. Create snapshot of stable version
2. Create variant for experimentation
3. Make changes in variant
4. Compare with `%variant diff`
5. Merge if successful

### Workflow 3: Query-Driven Exploration

1. Build initial architecture
2. Use queries to explore relationships
3. Discover dependencies
4. Refine based on insights

## Tips for Success

1. **Start Simple**: Begin with high-level systems, add detail gradually
2. **Validate Often**: Run validation after each significant change
3. **Use Snapshots**: Save your work frequently with snapshots
4. **Query First**: Use queries to understand your model before making changes
5. **Document Decisions**: Use markdown cells to document architectural decisions

## Next Steps

- Explore [Example Notebooks](../examples/README.md)
- Read [Advanced Features](../overview.md#advanced-features)
- Check [Best Practices](../examples/README.md#best-practices)

---

**Congratulations!** You've created your first Sruja Architecture Notebook. Continue building and exploring!


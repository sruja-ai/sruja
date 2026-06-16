# Domain Schema & Context Graphs

Sruja has evolved from a C4-specific architecture-as-code tool into a generalized **Context Graph** engine. This allows you to define any domain-specific language (DSL) and validate it using pluggable schemas.

## See also

For how Sruja combines autonomous coding (`agent loop`) with architecture governance, see [Grounded harness and continual learning](../GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md).

## What is a Context Graph?

A Context Graph is a governed, structured representation of knowledge designed for both humans and AI agents. Unlike a generic knowledge graph, a Context Graph prioritizes:
- **Governance**: Explicit rules about what can exist and how things can relate.
- **Intent**: Distinguishes between the "reviewed truth" (what should be) and "fresh evidence" (what is).
- **Resolution**: Handles nested scopes and fully qualified names (FQN) for precise retrieval.

### Terminology: “context graph” in the wild

Some literature uses **context graph** for **organizational decision traces** (who approved what, under which policy version, citing which precedent). Sruja’s Context Graph is the **repository intent + structure** layer: it is the map agents should respect while coding. For **decision-style traces** in-repo, use **context events** (`.sruja/context_events.jsonl`), **Agentic Memory**, and run snapshots—see [CONTEXT_ENGINEERING.md](../CONTEXT_ENGINEERING.md#context-graphs-sruja-vs-industry-usage).

## The Schema DSL

You can now define custom domains using the `schema` block at the top of your `.sruja` files.

### Syntax

```sruja
schema "Compliance" {
  node_kinds ["regulation", "policy", "control"]
  edge_kinds ["mandates", "satisfies", "implements"]
  
  nesting {
    regulation -> policy
    policy -> control
  }
}

// Now you can use these kinds:
GDPR = regulation "GDPR" {
  description "General Data Protection Regulation"
  
  DataPrivacy = policy "Data Privacy Policy" {
    description "Internal policy for data privacy"
    
    EncryptRest = control "Encryption at Rest" {
      description "All databases must use AES-256"
    }
  }
}
```

### Key Components

- **`node_kinds`**: A list of strings defining valid element kinds.
- **`edge_kinds`**: A list of strings defining valid relationship labels.
- **`nesting`**: Defines hierarchical rules (Parent -> Child). If an element is nested incorrectly, Sruja will report a validation error.

## Default Architecture Schema

If no `schema` is defined, Sruja defaults to the standard C4-inspired Architecture schema:

- **Node Kinds**: `person`, `system`, `container`, `component`, `database`, `queue`, `service`
- **Edge Kinds**: `depends_on`, `calls`, `reads_from`, `writes_to`, `publishes_to`, `subscribes_to`, `owns`, `contains`, `uses`
- **Nesting Rules**:
  - `system` can contain `container`, `database`, `queue`, `component`
  - `container` can contain `component`

## Validation

When you run `sruja lint`, the engine:
1. Detects if a `schema` block exists in the file.
2. If found, it uses that schema for all subsequent elements and relations.
3. If not found, it defaults to the `architecture` schema.

This ensures that your architecture (or compliance, or organizational) graph remains consistent with your stated intent.

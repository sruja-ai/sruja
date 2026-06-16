# Documentation (docs/)

**Policy:** This directory contains only **implemented** features and implementation-aligned documentation.

Sruja is documented here as a context engineering product with a small default loop:

- capture knowledge and decisions
- retrieve task-scoped context
- verify changes against structure and intent

Optional reviewed intent (`repo.sruja`) and richer workflows build on that core. See [FEATURE_TIERS.md](FEATURE_TIERS.md) and [HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md).

---

## Start Here

```bash
# 1. Detect current structure
sruja start -r .
sruja drift -r . --structural-only --advisory

# 2. Retrieve task context
sruja focus -r . --file path/to/file.rs
sruja ai -r . --task "Refactor auth boundary"

# 3. Verify after edits
sruja verify-task --profile coding -r .
```

**Core guides:** [FEATURE_TIERS.md](FEATURE_TIERS.md), [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md), [HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md)

---

## Core Docs

| Doc | Purpose |
|-----|---------|
| [FEATURE_TIERS.md](FEATURE_TIERS.md) | Core foundations vs extensions |
| [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) | Product framing around knowledge, decisions, and retrieval |
| [HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md) | How Sruja fits beside an editor-hosted agent loop |
| [mcp_tools_reference.md](mcp_tools_reference.md) | MCP tool catalog and profiles |
| [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md) | Optional reviewed-intent authoring flow |

## Extensions And Advanced Docs

| Doc | Purpose |
|-----|---------|
| [ENTERPRISE_ADOPTION.md](ENTERPRISE_ADOPTION.md) | Enterprise adoption tiers: evaluation → CI gates → AI-DLC workflows |
| [ENTERPRISE_POLICY.md](ENTERPRISE_POLICY.md) | Baselines/exceptions: advisory → baseline → enforce |
| [OFFLINE_INSTALL.md](OFFLINE_INSTALL.md) | Offline/air-gapped install and operation |
| [PRIVACY_AND_RETENTION.md](PRIVACY_AND_RETENTION.md) | Data emitted, retention guidance, and no-telemetry posture |
| [FEDERATION.md](FEDERATION.md) | Multi-repo composition, retrieval order, and artifact shapes |
| [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) | Context engineering principles and pipeline |
| [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) | Closed-loop agent (`agent loop`), deterministic grader, continual learning, local `--enrich-cmd`, agent memory |
| [architecture/domain-schema.md](architecture/domain-schema.md) | Context Graphs, custom `schema` blocks, default architecture kinds, and `sruja lint` validation |
| [architecture/README.md](architecture/README.md) | Sruja's own architecture models: platform, context pipeline, and development workflow |
| [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) | DSL reference |
| [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) | Language and modeling principles |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Development guide |
| [CODING_GUIDELINES.md](CODING_GUIDELINES.md) | Coding standards |
| [SCOPE.md](SCOPE.md) | Product scope |
| [SECURITY.md](SECURITY.md) | Security policy |
| [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) | Known limitations of architecture analysis |
| [mcp_setup.md](mcp_setup.md) | MCP server setup for AI editors |
| [mcp_tools_reference.md](mcp_tools_reference.md) | Full MCP tool catalog, categories, and env flags |
| [adr/](adr/) | Architecture decision records |
| [internal/](internal/) | Internal docs |

---

## By Role

| You Are | Start With |
|---------|------------|
| **User** | [FEATURE_TIERS.md](FEATURE_TIERS.md) |
| **Contributor** | [CONTRIBUTING.md](CONTRIBUTING.md) |
| **Architecture reviewer** | [architecture/README.md](architecture/README.md) |

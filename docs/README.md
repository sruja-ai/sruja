# Documentation (docs/)

**Policy:** This directory contains only **implemented** features and implementation-aligned documentation.

Sruja is documented here as **architecture-as-code plus context engineering for AI-assisted SDLC workflows**. The docs explain both the DSL and the retrieval pipeline that helps editors, skills, and AI tools work from reviewed architecture truth instead of guesses.

---

## Start Here

```bash
# 1. Install the skill
npx skills add sruja-ai/sruja --skill sruja-architecture

# 2. Refresh repo context when you want fresh evidence
sruja sync -r .

# 3. In your AI editor:
# "Use sruja-architecture. Gather evidence (prefer .sruja/context.json when present),
# generate or update repo.sruja, then run sruja lint and fix."
```

**Full guide:** [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md)

---

## Docs

| Doc | Purpose |
|-----|---------|
| [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md) | **Primary entry** – Install, use, and operate the architecture/context-engineering workflow |
| [FEDERATION_SETUP_GUIDE.md](FEDERATION_SETUP_GUIDE.md) | **Multi-repo federation** – Step-by-step guide for `repo.bundle.json` and `system.index.json` |
| [FEDERATION.md](FEDERATION.md) | Retrieval order, artifact shapes, and multi-repo composition rules |
| [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) | Context engineering principles and pipeline |
| [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md) | How common agentic / multi-agent patterns map to Sruja (MCP, graph, scope) |
| [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) | Grounded harness (lint/drift), host-owned Act/Reflect/Learn, local `--enrich-cmd`, agent memory |
| [architecture/domain-schema.md](architecture/domain-schema.md) | Context Graphs, custom `schema` blocks, default architecture kinds, and `sruja lint` validation |
| [UBIQUITOUS_LANGUAGE.md](UBIQUITOUS_LANGUAGE.md) | Shared terminology and definitions |
| [architecture/README.md](architecture/README.md) | Sruja's own architecture models: platform, context pipeline, and development workflow |
| [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) | DSL reference |
| [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) | Language and modeling principles |
| [RUN_GUIDE.md](RUN_GUIDE.md) | Build, run, and demo commands |
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
| **User** | [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md) |
| **Contributor** | [CONTRIBUTING.md](CONTRIBUTING.md) |
| **Architecture reviewer** | [architecture/README.md](architecture/README.md) |

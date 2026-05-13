# Agentic orchestration patterns and Sruja

Industry writing on multi-agent systems often discusses **reflection**, **tool use**, **planning**, **multi-agent collaboration**, and topologies such as sequential pipelines, parallel fan-out, hierarchical coordinators, and evaluator–optimizer loops. This document maps those ideas to **what Sruja is today** (architecture-as-code, graph queries, MCP, drift and context engineering) so teams know where Sruja fits and what would be **out of scope** unless the product explicitly grows into an agent runtime.

## MCP: tools and grounding, not peer-to-peer agents

Sruja exposes a **Model Context Protocol (MCP)** server so AI editors can call structured tools (summaries, neighbors, paths, focus briefings, context score, and related capabilities depending on version). That aligns with the common split:

- **MCP** answers: *What tools and grounded facts can this session use?*
- The **host** (Cursor, Claude Desktop, CI, or your own stack) remains the orchestrator for multi-step reasoning, routing, and sub-agents.

Sruja does **not** need to embed a LangGraph-style in-process agent loop to deliver strong value: it supplies **deterministic, schema-driven** architecture evidence on demand. Setup: [mcp_setup.md](mcp_setup.md).

## Shared state instead of stuffing the chat window

Patterns such as **tiered memory** and **context engineering** stress moving stable facts out of the model’s volatile window. In Sruja terms:

- **Architecture graph** and `repo.sruja` (and federation artifacts where used) act as **semantic memory**: reviewed structure and relationships.
- **Drift** commands and linting align with **governance**: whether the model’s picture matches enforced truth.
- **Ingested docs** under `.sruja/context/` (with links to elements) ground policy and decisions without pasting entire wikis into every prompt.

See [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) for the product framing.

## Modeling orchestration in `.sruja` (not executing it)

You can **document** how *your* system orchestrates agents—supervisor, hierarchy, mesh, pipelines—using the DSL and relationships. The book course **Agentic AI → Agentic Patterns → Multi-Agent Orchestration** walks through examples. That is **architecture description and communication**, not Sruja executing those agents at runtime.

## Adjacent to “evaluator–optimizer” and reflection

Sruja includes workflow-oriented hooks (for example agentic memory and trajectory evaluation in `sruja-agent`, and repo guidance such as drift checks and evaluation loops in contributor workflows). These resemble **reflection and evaluation at the engineering process layer**, not necessarily a dedicated critic LLM debating a generator inside the product.

## Patterns that are optional or future-facing

The following are useful **conceptually** or if Sruja later becomes a federated agent platform; they are **not** required for core graph, MCP, or drift workflows today.

| Topic | Relation to Sruja |
|--------|-------------------|
| **A2A**, Agent Cards, long-lived remote task lifecycles | Inter-agent discovery and delegation protocols; large new surface (auth, lifecycle, networking). |
| **Blackboard / swarm runtimes** | Centralized shared-state execution models; Sruja’s graph is a static/drift-checked model, not a live blackboard scheduler. |
| **AdaptOrch-style topology routing laws** | Interesting for R&D or recommendations (“when is this subgraph mostly parallel?”); not needed for baseline value. |

## The 80 / 20 split in practice

A practical industry heuristic is **~80% deterministic orchestration**, **~20%** genuinely open-ended model judgment. Sruja intentionally leans **deterministic**: parse, validate, scan, query, score, and lint. The **agentic** slice stays in the editor or your automation layer, which calls Sruja when it needs grounded architecture answers.

## Observability as you add automation

If you add more **autonomous** steps (headless runs, hooks, or multi-bot flows), invest in **traceability**: which MCP tools ran, with what arguments, what graph version or git SHA was in scope, and what changed in `.sruja` or policy files. Non-deterministic failures often look like subtle semantic drift or loops, not process crashes—telemetry and CI evaluation suites matter more than raw uptime metrics.

## Practical checklist

1. **Explain** onboarding and reviews using MCP + graph as the **grounding layer**, not megaprompts of raw repo text.
2. **Teach** teams to draw orchestration and boundaries in `.sruja` so agent designs stay reviewable like any other architecture.
3. **Gate** risky automation with drift checks, lint, and continuous evaluation as prompts and tools evolve.

For editor wiring, start with [mcp_setup.md](mcp_setup.md) and [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md).

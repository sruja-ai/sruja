---
title: "Agentic AI with Sruja"
summary: "Model agent systems, RAG pipelines, and governance with Sruja DSL."
difficulty: "advanced"
topic: "ai"
description: "A practical course on architecting agentic AI systems using Sruja: foundations, RAG, orchestration, and production governance."
estimatedTime: "3–4 hours"
---

# Agentic AI with Sruja

Learn to design agent-based AI systems with clear boundaries, interfaces, and governance using Sruja DSL.

**Learning objectives**

- Model orchestration, tools, and memory in `.sruja` (architecture description—not runtime execution).
- Distinguish **Sruja as a grounded harness** (lint, drift, evidence, MCP) from **the editor/CI host** that runs the LLM loop.
- Operate continual learning in token space: agent memory, bounded `agent plan` / `agent apply`, and optional local inference via `--enrich-cmd`.

For how common multi-agent and MCP narratives map to Sruja’s scope (grounding vs. runtime orchestration), see [Agentic orchestration patterns and Sruja](../../../../docs/AGENTIC_ORCHESTRATION_AND_SRUJA.md). For harness + host-owned learning, see [Grounded harness and continual learning](../../../../docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md). For the **Context Graph** model, custom `schema` syntax, and default validation rules, see [Domain schema and context graphs](../../../../docs/architecture/domain-schema.md).

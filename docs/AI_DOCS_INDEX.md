# AI Integration Documentation Index

Single entry point for Sruja's AI-related docs.

## Start here

| Doc | Audience | Purpose |
|-----|----------|---------|
| [INSTALL_AS_SKILL.md](INSTALL_AS_SKILL.md) | End users | **Install in your AI editor.** One-page guide: Cursor, Copilot, one command, which skill to choose. |
| [FIRST_PR_WITH_SRUJA.md](FIRST_PR_WITH_SRUJA.md) | Teams | **Fastest “aha”.** Add a PR gate + commit a first blueprint in ~10 minutes. |
| [AI_EDITOR_INTEGRATION.md](AI_EDITOR_INTEGRATION.md) | Users, developers | **Primary guide.** Setup with Cursor, Copilot, VS Code, Continue.dev; LSP; CI validation; quick-start files and skills. |
| [AI_ASSISTANT_GUIDE.md](AI_ASSISTANT_GUIDE.md) | AI users | Using AI assistants with Sruja (prompts, patterns). |

## Integration

| Doc | Audience | Purpose |
|-----|----------|---------|
| [AI_INTEGRATION.md](AI_INTEGRATION.md) | Integrators | Full integration guide: DSL overview, prompt templates, tool schemas for AI editors. |

## Related

- [INCREMENTAL_ARCHITECTURE_CAPTURE.md](INCREMENTAL_ARCHITECTURE_CAPTURE.md) – Capture architecture in pieces and stitch into one model (single repo or multi-repo).
- [ARCHITECTURE_INTELLIGENCE.md](ARCHITECTURE_INTELLIGENCE.md) – CLI-first drift/why, zero-key deterministic mode; LLM optional. Includes **current state** (CLI vs desktop, config).
- [ARCHITECTURE_INTELLIGENCE_BEST_PRACTICES.md](ARCHITECTURE_INTELLIGENCE_BEST_PRACTICES.md) – Research summary: modelling vs diagramming, drift, evidence-based docs, ADR, AI-assisted discovery; Sruja alignment and follow-ups.
- [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md) – **Discovery from code:** Research (ArchAgent, static-analysis combination, C4+AI) and best practices for more accurate architecture capture; phased playbook and skill implementation.
- [REAL_RUNS_PROOF.md](REAL_RUNS_PROOF.md) – **Proof:** Real CLI runs on express repo (quickstart, drift, scan, why, discover, lint, run_demo.sh) with captured output; use to verify docs match behaviour.
- [SKILLS_WITHOUT_CURSOR_CLI.md](SKILLS_WITHOUT_CURSOR_CLI.md) – Use Sruja skills without Cursor CLI: `sruja generate --prompt-only` (skill + context → prompt for any LLM), or other agents.
- [REVIEWING_AI_GENERATED_CODE.md](REVIEWING_AI_GENERATED_CODE.md) – Reviewing AI-generated .sruja and PR practices.
- [.cursorrules](https://github.com/sruja-ai/sruja/blob/main/.cursorrules) (repo root) – DSL rules for AI code generation.
- [skills/sruja-architecture/](../skills/sruja-architecture/) – Architecture skill (patterns, trade-offs) for agents.

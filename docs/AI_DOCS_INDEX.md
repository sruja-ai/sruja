# AI Integration Documentation Index

Single entry point for Sruja's AI-related docs.

## Start here

| Doc | Audience | Purpose |
|-----|----------|---------|
| [INSTALL_AS_SKILL.md](INSTALL_AS_SKILL.md) | End users | **Install in your AI editor.** One-page guide: Cursor, Copilot, one command, which skill to choose. |
| [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md) | End users | **One path:** Install skill → paste one prompt → validate; optional drift/export. |
| [FIRST_PR_WITH_SRUJA.md](FIRST_PR_WITH_SRUJA.md) | Teams | **Fastest “aha”.** Add a PR gate + commit a first blueprint in ~10 minutes. |
| [AI_EDITOR_INTEGRATION.md](AI_EDITOR_INTEGRATION.md) | Users, developers | **Primary guide.** Setup with Cursor, Copilot, VS Code, Continue.dev; LSP; CI validation; quick-start files and skills. |
| [AI_ASSISTANT_GUIDE.md](AI_ASSISTANT_GUIDE.md) | AI users | Using AI assistants with Sruja (prompts, patterns). |

## Integration

| Doc | Audience | Purpose |
|-----|----------|---------|
| [AI_INTEGRATION.md](AI_INTEGRATION.md) | Integrators | Full integration guide: DSL overview, prompt templates, tool schemas for AI editors. |

## Related

- [SKILLS_WITHOUT_CURSOR_CLI.md](SKILLS_WITHOUT_CURSOR_CLI.md) – Use Sruja skills without Cursor CLI: `sruja generate --prompt-only` (skill + context → prompt for any LLM), or other agents.
- [.cursorrules](https://github.com/sruja-ai/sruja/blob/main/.cursorrules) (repo root) – DSL rules for AI code generation.
- [skills/sruja-architecture/](../skills/sruja-architecture/) – Architecture skill (patterns, trade-offs) for agents.

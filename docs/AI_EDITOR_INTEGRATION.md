# AI Editor Integration

Use Sruja with Cursor, GitHub Copilot, VS Code + Copilot, Continue.dev, or any LSP-aware editor. This doc is the single reference for AI-assisted Sruja workflows (2026).

If you want the fastest end-to-end proof in a real PR, start with [First PR with Sruja](FIRST_PR_WITH_SRUJA.md).

## Install Sruja as a skill (recommended)

**One command** – Get full architecture rules (DSL syntax, patterns, anti-patterns, trade-offs) in your AI editor:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Works with **Cursor**, **Codex**, and any editor that uses the [skills.sh](https://skills.sh) ecosystem. For editor-specific steps and Cursor-only options, see **[Install as skill](INSTALL_AS_SKILL.md)**.

## Quick setup (all methods)

| Method | What you get |
|--------|----------------|
| **Skill (recommended)** | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` → full architecture rules for agents |
| **Manual** | Copy `.cursorrules`, `.copilot-instructions.md`, or `.architecture-skill.md` from repo root or [skills/sruja-architecture](../skills/sruja-architecture/) into your project |
| **VS Code** | Extension provides LSP, syntax highlighting, and diagnostics for `.sruja` files |

## Files and what they do

| File | Used by | Purpose |
|------|---------|--------|
| `.cursorrules` | Cursor | DSL syntax and patterns for Sruja generation |
| `.copilot-instructions.md` | GitHub Copilot | Same rules in Copilot instruction format |
| `.architecture-skill.md` | Any editor | Short pointer to install the full architecture skill |
| `skills/sruja-architecture/` | Skills.sh / agents | Full rule set (principles, patterns, anti-patterns, trade-offs) |

Use **quick-start files** for correct DSL syntax. Use the **sruja-architecture skill** when you want the AI to apply architectural patterns, bounded contexts, and trade-offs (e.g. monolith vs microservices).

## LSP (language server)

The **Sruja VS Code extension** ships a WASM-based LSP. When editing `.sruja`:

- **Diagnostics** – invalid references, missing fields, circular deps
- **Completions** – component kinds, property names
- **Semantic info** – structure for AI context

Any editor that can launch the Sruja LSP (or use the VS Code extension) gets the same semantics. For AI tools: prefer feeding LSP-backed diagnostics and structure over raw text when available.

## Catching bugs before merge

- **Local:** Run `sruja lint` on any changed `.sruja`; run `make test` (or `cargo test` / `npm run test`) for the stack you touched.
- **CI (this repo):** On every push/PR, when **any** `**/*.sruja` file changes, the workflow runs `sruja lint` on **all** `.sruja` files in the repo (examples, docs, lib, test-examples, etc.). So AI-added or edited architecture is validated automatically.
- **PR template:** Includes checklists for “AI-generated code” and “Architecture / .sruja review” so authors confirm they ran lint and basic checks before requesting review.
- **Reviewers:** Treat `.sruja` as a contract: ensure `sruja lint` is green and relationship labels/technologies/descriptions are present and specific.

## Validation in pipelines

Always validate AI- or human-written DSL:

```bash
sruja lint path/to/*.sruja
```

Example CI (e.g. GitHub Actions):

```yaml
- run: find . -name '*.sruja' -exec sruja lint {} \;
```

In this repo, use the reusable action `./.github/actions/sruja-validate` with `files: "**/*.sruja"` to validate the whole tree when any .sruja changes.

## Prompt templates (practical)

**Generate from description**

```
Generate Sruja architecture DSL for: [one paragraph].

Rules: architecture block, define every component before relationships,
double-quoted strings, technology for every container, descriptive relationship labels.
Output valid .sruja only. I will run `sruja lint` to verify.
```

**Fix errors**

```
This .sruja file fails `sruja lint` with: [paste errors].
Fix only what’s needed so it passes. Keep the same architecture intent.
```

**Refactor**

```
Refactor this Sruja architecture to [goal, e.g. “split into microservices by bounded context”].
Preserve all relationships and add any new ones needed. Output valid .sruja.
```

## Editor-specific notes

- **Cursor** – Best: run `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`. Alternatively, put `.cursorrules` in project root for DSL syntax only, or point Cursor at `skills/sruja-architecture/AGENTS.md` for the full guide. See [Install as skill](INSTALL_AS_SKILL.md#cursor).
- **GitHub Copilot** – Uses `.copilot-instructions.md`. Keep it in project root.
- **Continue.dev** – Add to config: `"contextFiles": [".cursorrules", ".copilot-instructions.md"]`.
- **Other LSP editors** – Use the Sruja VS Code extension if possible, or run the same LSP; ensure quick-start files are in the project so AI has DSL rules.

## References

- **Install as skill** – [INSTALL_AS_SKILL.md](INSTALL_AS_SKILL.md) – one-page guide for Cursor, Copilot, and other editors
- **Language spec** – [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md)
- **Examples** – `book/valid-examples/` (canonical, rendered in the mdBook)
- **Docs** – https://sruja.ai/docs
- **Skill install** – `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`

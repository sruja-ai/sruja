# Install Sruja in Your AI Code Editor

Use Sruja as a **skill** (or rules) in your AI coding assistant so it generates valid Sruja DSL, follows architectural patterns, and can discover architecture from your codebase.

**Single path (install → prompt → validate → optional drift):** [Getting started with the skill](GETTING_STARTED_SKILL.md).

**What you get:**

- **Valid .sruja** – Correct syntax, define-before-use, descriptions, relationship labels.
- **Architecture patterns** – Monolith vs microservices, event-driven, CQRS, hexagonal.
- **Optional discovery** – AI can analyze your repo and produce or refine `.sruja` files.

### Recommended prompt (architecture discovery)

For the best results when discovering architecture from a codebase, use this prompt in Cursor/IDE chat (from the [sruja-architecture-agent](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture-agent) skill):

*"Analyze this repo and generate a Sruja architecture file (architecture.sruja). Be thorough: main systems, containers, technologies, descriptions for every element, and relationships with clear labels. Run sruja lint and fix until it passes. Use the sruja-architecture-agent skill."*

Install the agent skill first: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent`

---

## Quick install by editor

| Editor | What to do |
|--------|------------|
| **Cursor** | Run once: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` — or add project rules (see [Cursor](#cursor) below). |
| **GitHub Copilot** | Copy [.copilot-instructions.md](https://github.com/sruja-ai/sruja/blob/main/.copilot-instructions.md) into your repo root, or use the skill command and point Copilot at the same rules. |
| **Continue.dev** | Add to config: `"contextFiles": [".cursorrules", ".copilot-instructions.md"]` and drop those files from the Sruja repo into your project. |
| **Any (skills.sh)** | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` |

---

## Cursor

**Option A – Skills (recommended)**  
Install the Sruja architecture skill so the agent gets full rules (principles, patterns, anti-patterns, trade-offs):

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

After this, you can use the Cursor CLI (`agent`) with Sruja in **any repo, in any folder** — just `cd` to your project and run the agent; the skill is available globally.

**Option B – Project rules only**  
For DSL syntax and basic patterns without the full skill, copy into your project root:

- [.cursorrules](https://github.com/sruja-ai/sruja/blob/main/.cursorrules) – Sruja DSL rules (Cursor reads this automatically).

**Option C – Project skill directory**  
If your editor supports project-scoped skills (e.g. `.cursor/skills/`), clone or copy the skill into your repo:

```bash
mkdir -p .cursor/skills
git clone --depth 1 https://github.com/sruja-ai/sruja .cursor/skills/sruja-source
# Then point your editor at .cursor/skills/sruja-source/skills/sruja-architecture/
# or symlink: ln -s ../sruja-source/skills/sruja-architecture .cursor/skills/sruja-architecture
```

Use **Option A** for the best experience with minimal setup.

---

## Which skill to install

| Skill | When to use |
|-------|--------------|
| **sruja-architecture** | Generating or refactoring `.sruja` files; applying patterns and trade-offs. **Start here.** |
| **sruja-architecture-agent** | You want the AI to discover architecture from your codebase and produce/update `.sruja` files. |
| **sruja-architecture-collaboration** | Multi-agent architecture sessions, review workflows, pattern library. |

**Install commands:**

```bash
# Architecture rules and patterns (most users)
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture

# AI-powered architecture discovery from code
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent

# Collaborative multi-agent architecture
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-collaboration
```

---

## After installing

1. **Validate** – After the AI edits a `.sruja` file, run `sruja lint path/to/file.sruja` (or use the editor’s “Run validation” if available).
2. **CLI** – For quickstart, drift, and “why” in the terminal: [Install the Sruja CLI](https://sruja.ai) (`curl -fsSL https://sruja.ai/install.sh | bash`).
3. **Full guide** – [AI Editor Integration](AI_EDITOR_INTEGRATION.md) – LSP, CI, prompt templates.

---

## References

- **Skill catalog** – [skills/README.md](../skills/README.md)
- **Using Sruja in your project** – [USING_SRUJA_IN_YOUR_PROJECT.md](USING_SRUJA_IN_YOUR_PROJECT.md)
- **Repo pointer file** – [.architecture-skill.md](../.architecture-skill.md) (same install instructions, in repo root)

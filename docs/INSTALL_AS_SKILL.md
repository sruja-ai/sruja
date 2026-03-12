# Install Sruja in Your AI Code Editor

> **Product posture:** Integration is **skills + CLI** only. There is **no MCP server** and no `sruja mcp` command—use the skill in your editor and the CLI for scan/drift/intent/why. See [skills/README.md](../skills/README.md).

Use Sruja as a **skill** (or rules) in your AI coding assistant so it generates valid Sruja DSL, follows architectural patterns, and can discover architecture from your codebase.

**Single path (install → prompt → validate → optional drift):** [Getting started with the skill](GETTING_STARTED_SKILL.md).

**What you get:**

- **Valid .sruja** – Correct syntax, define-before-use, descriptions, relationship labels.
- **Architecture patterns** – Monolith vs microservices, event-driven, CQRS, hexagonal.
- **Optional discovery** – AI can analyze your repo and produce or refine `.sruja` files. For more accurate capture, the skill uses **discovery modes** (overview / standard / deep-dive / diff) and a **phased playbook**; see [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md).

### Recommended prompt (architecture discovery — one prompt, easy)

Install the agent skill once, then paste this in your AI chat (Cursor, Copilot, etc.). You get `architecture.sruja` plus optional requirements/ADRs/flows when the agent finds evidence in your docs.

**Install:** `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent`

**Prompt to paste:**

*"Use the sruja-architecture-agent skill. Run \`sruja discover --context -r .\`, then generate \`architecture.sruja\` with systems, containers, components, and relationships (evidence-based; no guessing). If you find requirements, ADRs, or key flows in repo docs (README, docs/, adr/, SECURITY.md, etc.), add them to the file with citations; otherwise list 'Open questions' and do not invent. Run \`sruja lint architecture.sruja\` and fix until it passes."*

Export to Markdown: `sruja export markdown architecture.sruja`

### Recommended workflow (confirm-first, richest output)

If you want **high confidence** requirements/ADRs/scenarios/flows (and you want the user to confirm intent before it becomes architecture truth), use this 3-pass workflow:

- **Pass 1 (C4 structure only)**:
  - *"Use the sruja-architecture-agent skill. Run `sruja discover --context -r .`. Generate `architecture.sruja` with C4 structure (systems/containers/components) and labeled relationships. Do not add requirements/ADRs/scenarios/flows yet. Run `sruja lint architecture.sruja` and fix until it passes. Then summarize the architecture in 5–10 bullets and list 5–10 open questions."*
- **Pass 2 (Intent Review draft, citations only)**:
  - *"Extract candidate requirements, ADRs/decisions, scenarios, and flows from repo docs/specs/configs. Output an Intent Review list with citations to file paths for each item. Do not write into `architecture.sruja` yet. End with: 'Reply CONFIRM to encode these into the DSL, or EDIT with corrections'."*
- **Pass 3 (Encode confirmed intent)**:
  - *"Update `architecture.sruja` by adding the confirmed requirements/ADRs/scenarios/flows as DSL blocks. Include citations in descriptions. Run `sruja lint architecture.sruja` and fix until it passes."*

### Prompts by discovery mode

Use these when you want a specific scope or flow (skill defines modes in [SKILL.md](../skills/sruja-architecture-agent/SKILL.md)):

| Mode | When to use | Prompt to paste |
|------|-------------|------------------|
| **High-level overview** | Quick map: systems and main containers only | *"Use the sruja-architecture-agent skill in **high-level-overview** mode. Run `sruja discover --context -r .`. Generate `architecture.sruja` with only persons, systems, top-level containers, and key externals — no component-level detail. Run `sruja lint architecture.sruja` and fix until it passes."* |
| **Standard** (default) | Full capture, 10–30 components | Same as [Recommended prompt](#recommended-prompt-architecture-discovery--one-prompt-easy) above. |
| **Subsystem deep-dive** | One area in detail (e.g. `services/auth`) | *"Use the sruja-architecture-agent skill in **subsystem-deep-dive** mode for the path `services/auth` (or [your path]). Run `sruja discover --context -r services/auth`. Generate architecture for that area only (containers + components); treat other areas as external systems. Run `sruja lint architecture.sruja` and fix until it passes."* |
| **Diff-and-refine** | Update existing file from current code | *"Use the sruja-architecture-agent skill in **diff-and-refine** mode. Compare the repo to the existing `architecture.sruja`. Propose only additions, removals, or relationship fixes; do not rewrite from scratch. Run `sruja lint` on the updated file and fix until it passes."* |

**Tip:** To build a prompt that includes repo context and drift output for diff-and-refine (so the AI sees `sruja discover --context` and `sruja drift` in one paste), use the script in the Sruja repo: `evaluation/real-world-test/run_diff_refine_prompt.sh [repo_path] [architecture.sruja]`. Paste the generated file into your AI chat.

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
